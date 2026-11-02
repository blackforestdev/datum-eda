//! Project-qualified capability discovery on the existing companion warmup.
use super::*;
use std::{collections::BTreeSet, path::Path};

/// Extract the native-write verb ids from a `native.describe` response's `native_write_verbs`
/// catalog (W3 capability discovery). Used to populate truthful native-edit menu enablement.
pub(super) fn native_write_verb_ids(
    describe: &serde_json::Value,
) -> std::collections::BTreeSet<String> {
    describe
        .get("native_write_verbs")
        .and_then(serde_json::Value::as_array)
        .map(|verbs| {
            verbs
                .iter()
                .filter_map(|verb| verb.get("id").and_then(serde_json::Value::as_str))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) struct CatalogReply {
    root: PathBuf,
    project_id: String,
    generation: u64,
    result: Result<BTreeSet<String>, String>,
}

impl CatalogReply {
    fn matches(&self, root: &Path, project_id: &str, generation: u64) -> bool {
        self.root == root && self.project_id == project_id && self.generation == generation
    }
}

impl EditWork {
    /// Warm the companion and return discovery through its existing wake route.
    /// Discovery never supplies a write receipt or replaces the live workspace.
    pub(crate) fn prepare_read(
        &mut self,
        root: PathBuf,
        project_id: String,
        generation: u64,
    ) -> Option<std::thread::JoinHandle<()>> {
        self.catalog_pending = None;
        let client = self.client.clone();
        let wake = self.wake.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        let spawn = std::thread::Builder::new()
            .name("datum-native-prepare".into())
            .spawn(move || {
                let started = std::time::Instant::now();
                let result = (|| -> Result<BTreeSet<String>> {
                    let mut owner = client
                        .lock()
                        .map_err(|_| anyhow::anyhow!("Native editor worker unavailable"))?;
                    if owner.is_none() {
                        *owner = Some(NativeProjectClient::spawn()?);
                    }
                    let describe = owner.as_ref().unwrap().prepare_read(&root)?;
                    anyhow::ensure!(
                        describe
                            .get("project_id")
                            .and_then(serde_json::Value::as_str)
                            == Some(project_id.as_str()),
                        "Capability discovery returned another Project"
                    );
                    Ok(native_write_verb_ids(&describe))
                })()
                .map_err(|error| format!("{error:#}"));
                trace_startup_timing(format!(
                    "native companion preparation {}ms verbs={:?}",
                    started.elapsed().as_millis(),
                    result.as_ref().map(BTreeSet::len)
                ));
                // Release worker ownership before publishing completion, including errors.
                drop(client);
                let _ = sender.send(CatalogReply {
                    root,
                    project_id,
                    generation,
                    result,
                });
                let _ = wake.send_event(());
            });
        match spawn {
            Ok(handle) => {
                self.catalog_pending = Some(receiver);
                Some(handle)
            }
            Err(error) => {
                trace_startup_timing(format!("native companion preparation unavailable: {error}"));
                None
            }
        }
    }
}

impl Runtime {
    pub(crate) fn poll_native_catalog(&mut self) -> bool {
        let Some(receiver) = &self.native_edit.catalog_pending else {
            return false;
        };
        let reply = match receiver.try_recv() {
            Ok(reply) => reply,
            Err(mpsc::TryRecvError::Empty) => return false,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.native_edit.catalog_pending = None;
                return false;
            }
        };
        self.native_edit.catalog_pending = None;
        let Some(backing) = self.workspace().backing.as_ref() else {
            return false;
        };
        if !reply.matches(
            &backing.request.project_root,
            &self.workspace().scene.project_uuid,
            self.project_open.session_generation,
        ) {
            return false;
        }
        match reply.result {
            Ok(verbs) => {
                // An explicitly empty catalog stays empty; never restore prior capabilities.
                self.session
                    .workspace_mut()
                    .backing
                    .as_mut()
                    .unwrap()
                    .native_write_verbs = verbs;
            }
            Err(reason) => self.log_console_refusal(
                ConsoleFeedbackSource::Menu,
                format!("Native editing capability discovery failed: {reason}"),
            ),
        }
        self.invalidate_frame();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_reply_cannot_cross_a_project_switch_or_reopen() {
        let reply = CatalogReply {
            root: PathBuf::from("project-a"),
            project_id: "id-a".into(),
            generation: 7,
            result: Ok(BTreeSet::new()),
        };
        assert!(reply.matches(Path::new("project-a"), "id-a", 7));
        assert!(!reply.matches(Path::new("copied-project-a"), "id-a", 7));
        assert!(!reply.matches(Path::new("project-a"), "replacement-id", 7));
        assert!(!reply.matches(Path::new("project-a"), "id-a", 8));
    }
}
