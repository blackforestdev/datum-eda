//! Typed local authoring transport. Used on the application worker, never the
//! native input thread. The owned companion is distinct from imported sessions.
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::{fs::DirBuilderExt, net::UnixStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum NativeEditAction {
    Move {
        component: Uuid,
        x_nm: i64,
        y_nm: i64,
    },
    Undo {
        expected_tip: Uuid,
    },
    Redo {
        expected_tip: Uuid,
    },
    Save,
}
#[derive(Clone, Debug)]
pub struct NativeEditIntent {
    pub project_root: PathBuf,
    pub project_id: Uuid,
    pub expected_revision: String,
    pub expected_tip: Option<Uuid>,
    pub action: NativeEditAction,
    pub request_id: Uuid,
}
impl NativeEditIntent {
    pub fn new(
        project_root: PathBuf,
        project_id: Uuid,
        expected_revision: String,
        expected_tip: Option<Uuid>,
        action: NativeEditAction,
    ) -> Self {
        Self {
            project_root,
            project_id,
            expected_revision,
            expected_tip,
            action,
            request_id: Uuid::new_v4(),
        }
    }
}

#[derive(Debug)]
pub struct NativeEditReceipt {
    pub applied: bool,
    pub reconciled: bool,
    pub description: Value,
}

pub struct NativeProjectClient {
    child: Child,
    directory: PathBuf,
    socket: PathBuf,
}
impl NativeProjectClient {
    pub fn spawn() -> Result<Self> {
        let binary = crate::cli_bridge::resolve_daemon_companion()?;
        let parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let directory = parent.join(format!("datum-native-edit-{}", Uuid::new_v4()));
        std::fs::DirBuilder::new().mode(0o700).create(&directory)?;
        let socket = directory.join("engine.sock");
        let spawned = Command::new(&binary)
            .arg("--socket")
            .arg(&socket)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn();
        let child = match spawned {
            Ok(child) => child,
            Err(error) => {
                let _ = std::fs::remove_dir(&directory);
                return Err(error).with_context(|| {
                    format!("start native writer companion {}", binary.display())
                });
            }
        };
        let mut client = Self {
            child,
            directory,
            socket,
        };
        let until = Instant::now() + Duration::from_secs(5);
        loop {
            if client.socket.exists() {
                break;
            }
            if let Some(status) = client.child.try_wait()? {
                bail!("native writer companion exited: {status}");
            }
            if Instant::now() >= until {
                bail!("native writer companion did not become available");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(client)
    }
    fn call(&self, method: &str, params: Value, id: Uuid) -> Result<Value> {
        let started = Instant::now();
        let mut stream = UnixStream::connect(&self.socket)?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;
        serde_json::to_writer(
            &mut stream,
            &json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}),
        )?;
        stream.write_all(b"\n")?;
        stream.flush()?;
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line)?;
        let response: Value =
            serde_json::from_str(&line).context("native writer receipt unavailable")?;
        if std::env::var_os("DATUM_TRACE_TIMING").is_some() {
            crate::trace_protocol_timing(format!(
                "native RPC {method} {}ms",
                started.elapsed().as_millis()
            ));
        }
        if response.get("id") != Some(&json!(id)) {
            bail!("native writer receipt identity mismatch");
        }
        if let Some(error) = response.get("error").filter(|e| !e.is_null()) {
            bail!(
                "{}",
                error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("native write refused")
            );
        }
        response
            .get("result")
            .cloned()
            .context("native writer result missing")
    }
    pub fn describe(&self, root: &Path) -> Result<Value> {
        self.describe_snapshot(root, false)
    }
    fn describe_snapshot(&self, root: &Path, include_model: bool) -> Result<Value> {
        let result = self.call(
            "native.describe",
            json!({"project_root":root,"confirm_durable":true,"include_model":include_model}),
            Uuid::new_v4(),
        )?;
        if result.get("durability_confirmed") != Some(&json!(true)) {
            bail!(
                "Native writer companion did not confirm Project durability; use the matching build"
            );
        }
        Ok(result)
    }
    /// One complete existing DesignModel paired with the same fresh durable
    /// response. No persistence or mutation representation is introduced.
    pub fn read_model(
        &self,
        root: &Path,
        expected: &Value,
    ) -> Result<eda_engine::substrate::DesignModel> {
        // A complete model already paired with the operation's fresh durable
        // reconciliation is sufficient for preparation. Publication still
        // requires the worker's final fresh revision/tip/durability check.
        if let Some(snapshot) = expected.get("model") {
            anyhow::ensure!(
                expected.get("durability_confirmed") == Some(&json!(true)),
                "Snapshot durability was not confirmed"
            );
            let model =
                <eda_engine::substrate::DesignModel as serde::Deserialize>::deserialize(snapshot)?;
            validate_read_model(root, expected, expected, &model)?;
            return Ok(model);
        }
        let mut description = self.call(
            "native.describe",
            json!({"project_root":root,"confirm_durable":true,"include_model":true}),
            Uuid::new_v4(),
        )?;
        anyhow::ensure!(
            description.get("durability_confirmed") == Some(&json!(true)),
            "Snapshot durability was not confirmed"
        );
        let model: eda_engine::substrate::DesignModel = serde_json::from_value(
            description
                .get_mut("model")
                .context("Matching complete engine snapshot unavailable")?
                .take(),
        )?;
        validate_read_model(root, expected, &description, &model)?;
        Ok(model)
    }

    pub fn execute(&self, intent: &NativeEditIntent) -> Result<NativeEditReceipt> {
        let initial = self.describe(&intent.project_root)?;
        if initial.get("transaction_tip") != Some(&json!(intent.expected_tip)) {
            bail!("Project history changed since the edit began; refresh before editing");
        }
        if initial.get("project_id") != Some(&json!(intent.project_id))
            || initial.get("model_revision").and_then(Value::as_str)
                != Some(intent.expected_revision.as_str())
        {
            bail!("Project changed since the edit began; refresh before editing");
        }
        if let NativeEditAction::Save = intent.action {
            if initial
                .get("diagnostics")
                .and_then(Value::as_array)
                .is_some_and(|ds| {
                    ds.iter().any(|d| {
                        d.get("code").and_then(Value::as_str).is_some_and(|c| {
                            c.starts_with("journal_") || c == "revision_integrity_unavailable"
                        })
                    })
                })
            {
                bail!("Save not confirmed: Project integrity needs recovery");
            }
            return Ok(NativeEditReceipt {
                applied: false,
                reconciled: false,
                description: initial,
            });
        }
        let (verb, params) = match intent.action {
            NativeEditAction::Move {
                component,
                x_nm,
                y_nm,
            } => (
                "datum.pcb.move_component",
                json!({"component":component,"x_nm":x_nm,"y_nm":y_nm}),
            ),
            NativeEditAction::Undo { expected_tip } => {
                ("datum.journal.undo", json!({"expected_tip":expected_tip}))
            }
            NativeEditAction::Redo { expected_tip } => {
                ("datum.journal.redo", json!({"expected_tip":expected_tip}))
            }
            NativeEditAction::Save => unreachable!(),
        };
        let reason = format!("GUI {verb}; request {}", intent.request_id);
        let result = self.call(
            "native.write",
            json!({"project_root":intent.project_root,
            "verb":verb,"params":params,"actor":"datum-gui","source":"tool","reason":reason,
            "expected_model_revision":intent.expected_revision,"expected_transaction_tip":intent.expected_tip,"batch_id":intent.request_id}),
            intent.request_id,
        );
        // An error/timeout never means a second attempt. Query the canonical
        // journal after guarded recovery, correlating this exact request.
        let description = self.describe_snapshot(&intent.project_root, true)?;
        let found = description
            .get("transactions")
            .and_then(Value::as_array)
            .is_some_and(|ts| {
                ts.iter()
                    .any(|t| t.get("reason").and_then(Value::as_str) == Some(reason.as_str()))
            });
        match result {
            Ok(_) if found => Ok(NativeEditReceipt {
                applied: true,
                reconciled: false,
                description,
            }),
            Err(_) if found => Ok(NativeEditReceipt {
                applied: true,
                reconciled: true,
                description,
            }),
            Err(error) => Err(error.context("edit not confirmed; no retry was sent")),
            Ok(_) => bail!("edit receipt could not be reconciled against the Project journal"),
        }
    }
}
impl Drop for NativeProjectClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[cfg(test)]
#[path = "native_edit_client_tests.rs"]
mod tests;

fn validate_read_model(
    root: &Path,
    expected: &Value,
    description: &Value,
    model: &eda_engine::substrate::DesignModel,
) -> Result<()> {
    for key in ["project_id", "model_revision", "transaction_tip"] {
        anyhow::ensure!(
            description.get(key).is_some() && description.get(key) == expected.get(key),
            "Project changed during snapshot read: {key}"
        );
    }
    anyhow::ensure!(
        description.get("project_id") == Some(&json!(model.project.project_id))
            && description.get("model_revision").and_then(Value::as_str)
                == Some(model.model_revision.0.as_str())
            && description.get("transaction_tip")
                == Some(&json!(eda_engine::revision::transaction_tip(
                    &model.journal
                )))
            && description.get("journal_len") == Some(&json!(model.journal.len())),
        "Engine snapshot identity/revision/history mismatch"
    );
    let manifest = model
        .source_shards
        .iter()
        .find(|s| s.kind == eda_engine::substrate::SourceShardKind::ProjectManifest)
        .context("Engine snapshot lacks Project manifest")?;
    anyhow::ensure!(
        manifest.path.canonicalize()? == root.join("project.json").canonicalize()?,
        "Engine snapshot belongs to another Project root"
    );
    Ok(())
}
