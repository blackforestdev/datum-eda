//! Session-owned identity/cwd projection; shell titles never confer authority.
use super::*;
use datum_gui_protocol::{TerminalSessionBinding, TerminalSessionContext};

fn session_context(
    origin: &TerminalLaunchContext,
    launch_directory: &Path,
    reported: Option<&str>,
) -> TerminalSessionContext {
    let cwd = reported.and_then(crate::local_path::decode_local_path);
    let binding = match &origin.project_root {
        Some(root) => TerminalSessionBinding::Project {
            name: origin
                .project_name
                .clone()
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| {
                    root.file_name()
                        .unwrap_or(root.as_os_str())
                        .to_string_lossy()
                        .into_owned()
                }),
            root: root.display().to_string(),
        },
        None => TerminalSessionBinding::Independent,
    };
    let default_tab_label = match &binding {
        TerminalSessionBinding::Project { name, .. } => name.clone(),
        TerminalSessionBinding::Independent => {
            if std::env::var_os("HOME").is_some_and(|home| PathBuf::from(home) == launch_directory)
            {
                "Home".into()
            } else {
                "Independent shell".into()
            }
        }
    };
    TerminalSessionContext {
        binding,
        default_tab_label,
        directory_reported: cwd.is_some(),
        directory: cwd
            .as_deref()
            .unwrap_or(launch_directory)
            .display()
            .to_string(),
    }
}

pub(super) fn session_tab_label(
    fallback: &str,
    explicit: bool,
    context: Option<&TerminalSessionContext>,
    title: Option<&str>,
    progress: datum_gui_protocol::TerminalProgressState,
    notification: bool,
) -> String {
    let fallback = context
        .filter(|_| !explicit)
        .map(|c| c.default_tab_label.as_str())
        .unwrap_or(fallback);
    terminal_tab_label(fallback, explicit, title, progress, notification)
}

impl TerminalSessionRegistry {
    pub(super) fn sync_session_contexts(&mut self, state: &mut TerminalLaneState) {
        // A pending launch never displays the previous session's context.
        if self.active_pending_id.is_some() || self.sessions.is_empty() {
            state.session_context = None;
        }
        for (index, slot) in self.sessions.iter_mut().enumerate() {
            let lane = if self.active_pending_id.is_none() && index == self.active_index {
                &mut *state
            } else {
                &mut slot.parked_lane
            };
            if slot.hidden_after_close {
                lane.session_context = None;
                continue;
            }
            lane.session_context =
                self.origin_contexts
                    .get(slot.session.session_id())
                    .map(|origin| {
                        session_context(
                            origin,
                            &slot.session.launch_directory,
                            lane.current_working_directory.as_deref(),
                        )
                    });
        }
    }
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod tests;
