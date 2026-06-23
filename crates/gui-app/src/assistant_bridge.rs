use crate::AssistantContext;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

pub(super) struct AssistantSession {
    pub(super) _child: Child,
    pub(super) stdin: Arc<Mutex<ChildStdin>>,
    pub(super) rx: Receiver<AssistantBridgeOutput>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct AssistantBridgeConfig {
    #[serde(default)]
    pub(super) api_key: Option<String>,
    #[serde(default)]
    pub(super) model: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct AssistantBridgeInput {
    #[serde(rename = "type")]
    pub(super) kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) context: Option<AssistantContext>,
}

#[derive(Debug, Deserialize)]
pub(super) struct AssistantBridgeOutput {
    #[serde(rename = "type")]
    pub(super) kind: String,
    #[serde(default)]
    pub(super) message: String,
    #[serde(default)]
    pub(super) actions: Vec<AssistantBridgeAction>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(super) struct AssistantBridgeAction {
    #[serde(rename = "type")]
    pub(super) kind: String,
    #[serde(default)]
    pub(super) tab: Option<String>,
    #[serde(default)]
    pub(super) tool: Option<String>,
    #[serde(default)]
    pub(super) reference: Option<String>,
    #[serde(default)]
    pub(super) action_id: Option<String>,
    #[serde(default)]
    pub(super) dx_nm: Option<i64>,
    #[serde(default)]
    pub(super) dy_nm: Option<i64>,
    #[serde(default)]
    pub(super) command: Option<String>,
}

pub(super) fn spawn_assistant_session(config: &AssistantBridgeConfig) -> Result<AssistantSession> {
    let script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/datum_assistant_bridge.py");
    let mut command = Command::new("/usr/bin/python3");
    command
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(api_key) = &config.api_key {
        command.env("OPENAI_API_KEY", api_key);
    }
    if let Some(model) = &config.model {
        command.env("DATUM_ASSISTANT_MODEL", model);
    }
    let mut child = command
        .spawn()
        .with_context(|| format!("spawn assistant bridge {}", script.display()))?;
    let stdin = child.stdin.take().context("take assistant bridge stdin")?;
    let stdout = child
        .stdout
        .take()
        .context("take assistant bridge stdout")?;
    let stderr = child
        .stderr
        .take()
        .context("take assistant bridge stderr")?;
    let stdin = Arc::new(Mutex::new(stdin));
    let (tx, rx) = mpsc::channel();
    {
        let tx = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                match serde_json::from_str::<AssistantBridgeOutput>(&line) {
                    Ok(message) => {
                        let _ = tx.send(message);
                    }
                    Err(err) => {
                        let _ = tx.send(AssistantBridgeOutput {
                            kind: "error".to_string(),
                            message: format!("assistant bridge emitted invalid JSON: {err}"),
                            actions: Vec::new(),
                        });
                    }
                }
            }
        });
    }
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            let _ = tx.send(AssistantBridgeOutput {
                kind: "error".to_string(),
                message: line,
                actions: Vec::new(),
            });
        }
    });
    Ok(AssistantSession {
        _child: child,
        stdin,
        rx,
    })
}
