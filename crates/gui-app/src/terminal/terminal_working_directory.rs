//! Safe local working-directory inheritance for newly opened terminal tabs.
//!
//! The shell reports its directory through OSC 7 as a file URI. Datum accepts
//! only absolute local paths, decodes URI path bytes without Unicode loss, and
//! falls back to the project root when the report is remote, malformed, stale,
//! or does not name a directory.

use crate::terminal_session::TerminalLaunchContext;
use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

pub(super) fn context_for_new_terminal(
    base: &TerminalLaunchContext,
    reported_working_directory: Option<&str>,
) -> TerminalLaunchContext {
    let mut context = base.clone();
    context.launch_working_directory = reported_working_directory
        .and_then(local_working_directory)
        .filter(|path| path.is_dir())
        .unwrap_or_else(|| {
            base.project_root
                .clone()
                .unwrap_or_else(|| base.launch_working_directory.clone())
        });
    context
}

/// A new consumer captures the current shared projection, not the startup one.
pub(super) fn context_for_new_terminal_from_state(
    base: &TerminalLaunchContext,
    state: &datum_gui_protocol::ReviewWorkspaceState,
    cursor: Option<(f32, f32)>,
    reported_working_directory: Option<&str>,
) -> TerminalLaunchContext {
    let Some(root) = base.project_root.as_deref() else {
        return context_for_new_terminal(base, reported_working_directory);
    };
    let mut current = crate::terminal_session::terminal_launch_context_from_state_with_cursor(
        root, state, cursor,
    );
    current.terminal_profile = base.terminal_profile.clone();
    context_for_new_terminal(&current, reported_working_directory)
}

impl crate::Runtime {
    pub(super) fn context_for_active_terminal_launch(
        &self,
    ) -> anyhow::Result<TerminalLaunchContext> {
        let current = self.terminal_launch_context.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Open a Project before starting its terminal session")
        })?;
        let origin = self
            .terminal_sessions
            .active_origin_context()
            .unwrap_or(current);
        let cwd = self
            .workspace()
            .ui
            .terminal
            .current_working_directory
            .as_deref();
        Ok(if origin.project_root != current.project_root {
            // Preserved shells never combine their origin root with the new
            // Project's selection, source revision or projection context.
            context_for_new_terminal(origin, cwd)
        } else {
            context_for_new_terminal_from_state(
                current,
                self.workspace(),
                self.last_cursor_pos,
                cwd,
            )
        })
    }
}

fn local_working_directory(value: &str) -> Option<PathBuf> {
    let encoded = if let Some(uri) = value.strip_prefix("file://") {
        if let Some(path) = uri.strip_prefix("localhost/") {
            format!("/{path}")
        } else if uri.starts_with('/') {
            uri.to_owned()
        } else {
            return None;
        }
    } else if value.starts_with('/') {
        value.to_owned()
    } else {
        return None;
    };
    if encoded.contains(['?', '#']) {
        return None;
    }
    let bytes = percent_decode_path(encoded.as_bytes())?;
    let path = PathBuf::from(OsString::from_vec(bytes));
    path.is_absolute().then_some(path)
}

fn percent_decode_path(encoded: &[u8]) -> Option<Vec<u8>> {
    let mut decoded = Vec::with_capacity(encoded.len());
    let mut index = 0;
    while index < encoded.len() {
        if encoded[index] != b'%' {
            if encoded[index] == 0 {
                return None;
            }
            decoded.push(encoded[index]);
            index += 1;
            continue;
        }
        let high = *encoded.get(index + 1)?;
        let low = *encoded.get(index + 2)?;
        let byte = hex_value(high)?
            .checked_mul(16)?
            .checked_add(hex_value(low)?)?;
        if byte == 0 {
            return None;
        }
        decoded.push(byte);
        index += 3;
    }
    Some(decoded)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn new_consumer_captures_current_selection_without_reusing_startup_context() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let base = crate::terminal_session::terminal_launch_context_from_state(
            std::path::Path::new("/project"),
            &state,
        );
        state.selection = datum_gui_protocol::SelectionTarget::AuthoredObject("current".into());
        let current = context_for_new_terminal_from_state(&base, &state, Some((12.0, 34.0)), None);
        assert_eq!(
            current.selection_context,
            datum_gui_protocol::DatumSelectionContext::from_selection(&state.selection)
        );
        assert_ne!(current.selection_context, base.selection_context);
        assert_eq!(current.terminal_profile, base.terminal_profile);
        assert_eq!(current.project_root, base.project_root);
        assert_eq!(Some(current.launch_working_directory), base.project_root);
        assert_eq!(current.cursor_context.screen_px, Some([12, 34]));
    }

    #[test]
    fn new_terminal_inherits_local_osc7_directory_without_changing_project_identity() {
        let root =
            std::env::temp_dir().join(format!("datum-new-terminal-cwd-{}", std::process::id()));
        let project = root.join("project");
        let child = root.join("agent work");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&child).unwrap();
        let base = TerminalLaunchContext::for_project_root(&project);
        let report = format!("file://{}", child.display().to_string().replace(' ', "%20"));

        let context = context_for_new_terminal(&base, Some(&report));

        assert_eq!(context.project_root, Some(project));
        assert_eq!(context.launch_working_directory, child);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn local_plain_and_localhost_paths_are_accepted() {
        assert_eq!(
            local_working_directory("/tmp/datum"),
            Some(PathBuf::from("/tmp/datum"))
        );
        assert_eq!(
            local_working_directory("file://localhost/tmp/datum"),
            Some(PathBuf::from("/tmp/datum"))
        );
    }

    #[test]
    fn remote_malformed_and_stale_reports_fall_back_to_project_root() {
        let root = std::env::temp_dir().join(format!(
            "datum-new-terminal-cwd-fallback-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let base = TerminalLaunchContext::for_project_root(&root);
        for report in [
            "file://remote.example/tmp/project",
            "relative/project",
            "file:///tmp/bad%GGpath",
            "file:///tmp/missing-datum-directory",
            "file:///tmp/path?query",
            "file:///tmp/zero%00byte",
        ] {
            let context = context_for_new_terminal(&base, Some(report));
            assert_eq!(context.launch_working_directory, root, "report={report}");
        }
        let _ = fs::remove_dir_all(root);
    }
}
