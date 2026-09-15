use super::{TerminalClipboardWriteRequest, TerminalDrainReport, TerminalNotificationRequest};
use crate::{
    terminal_core_adapter::TerminalCoreAdapterUpdate, terminal_session::TerminalSession,
    terminal_session_events::record_terminal_shell_metadata_event,
};
use datum_gui_protocol::{TerminalLaneState, TerminalShellPhase};

pub(super) fn consume_core_update(
    session: &TerminalSession,
    lane: &mut TerminalLaneState,
    report: &mut TerminalDrainReport,
    update: TerminalCoreAdapterUpdate,
) {
    for response in update.replies {
        if let Err(error) = session.write_bytes(&response) {
            report
                .notices
                .push(format!("terminal status response failed: {error}"));
        }
    }
    for error in update.semantic_errors {
        report
            .notices
            .push(format!("terminal core semantic limit/error: {error}"));
    }
    for event in update.events {
        match event {
            datum_terminal_core::CoreEvent::LimitReached(kind) => report
                .notices
                .push(format!("terminal core {:?} limit reached", kind).to_lowercase()),
            datum_terminal_core::CoreEvent::Notification(text) => {
                let text = text.as_str().to_owned();
                lane.latest_notification = Some(text.clone());
                report.notifications.push(TerminalNotificationRequest {
                    session_id: session.session_id().to_string(),
                    text,
                });
            }
            datum_terminal_core::CoreEvent::ClipboardRequest {
                selection,
                encoded_contents,
            } => report
                .clipboard_requests
                .push(TerminalClipboardWriteRequest {
                    session_id: session.session_id().to_string(),
                    selection,
                    encoded_contents: encoded_contents.as_slice().to_vec(),
                }),
            datum_terminal_core::CoreEvent::WorkingDirectoryChanged(directory) => {
                if let Err(error) = record_terminal_shell_metadata_event(
                    session,
                    "working_directory",
                    Some(directory.as_str()),
                    None,
                ) {
                    report
                        .notices
                        .push(format!("persist terminal cwd metadata failed: {error}"));
                }
            }
            datum_terminal_core::CoreEvent::ShellMark(mark) => {
                observe_shell_mark(session, lane, report, mark);
            }
            _ => {}
        }
    }
}

fn observe_shell_mark(
    session: &TerminalSession,
    lane: &mut TerminalLaneState,
    report: &mut TerminalDrainReport,
    mark: datum_terminal_core::ShellMark,
) {
    let (phase, kind, exit_code) = match mark {
        datum_terminal_core::ShellMark::PromptStart => {
            (TerminalShellPhase::Prompt, "prompt_start", None)
        }
        datum_terminal_core::ShellMark::CommandStart => {
            (TerminalShellPhase::CommandInput, "command_start", None)
        }
        datum_terminal_core::ShellMark::CommandExecuted => {
            (TerminalShellPhase::CommandOutput, "command_executed", None)
        }
        datum_terminal_core::ShellMark::CommandFinished { exit_code } => (
            TerminalShellPhase::CommandFinished,
            "command_finished",
            exit_code,
        ),
    };
    lane.shell_metadata.observe(phase, exit_code);
    if let Err(error) = record_terminal_shell_metadata_event(session, kind, None, exit_code) {
        report.notices.push(format!(
            "persist terminal command-boundary metadata failed: {error}"
        ));
    }
}
