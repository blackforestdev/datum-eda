//! Output-only Datum Console publication boundary (decision 033).

use datum_gui_protocol::{ConsoleFeedbackDraft, ConsoleFeedbackState};

pub(super) fn occurred_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

pub(super) fn publish(console: &mut ConsoleFeedbackState, draft: ConsoleFeedbackDraft) -> u64 {
    console.publish(draft)
}

#[cfg(test)]
mod tests {
    use super::{occurred_unix_ms, publish};
    use datum_gui_protocol::{
        ConsoleFeedbackCategory, ConsoleFeedbackDraft, ConsoleFeedbackSource, ConsoleFeedbackState,
        TerminalLaneState,
    };

    #[test]
    fn publication_is_typed_and_cannot_mutate_terminal_state() {
        let mut console = ConsoleFeedbackState::default();
        let terminal = TerminalLaneState::default();
        let terminal_before = terminal.clone();

        publish(
            &mut console,
            ConsoleFeedbackDraft::action_echo(
                ConsoleFeedbackSource::Viewport,
                occurred_unix_ms(),
                "fit board",
            ),
        );

        let record = console.latest().unwrap();
        assert_eq!(record.source, ConsoleFeedbackSource::Viewport);
        assert_eq!(record.category, ConsoleFeedbackCategory::ActionEcho);
        assert_eq!(record.message, "fit board");
        assert_eq!(terminal, terminal_before);
    }

    #[test]
    fn terminal_progress_and_findings_producers_stay_outside_console() {
        let terminal_owned_sources = [
            include_str!("application_terminal_shutdown.rs"),
            include_str!("runtime_terminal_clipboard.rs"),
            include_str!("runtime_terminal_context.rs"),
            include_str!("runtime_terminal_dock.rs"),
            include_str!("runtime_terminal_input.rs"),
            include_str!("runtime_terminal_links.rs"),
            include_str!("runtime_terminal_pointer.rs"),
            include_str!("terminal_accessibility_bridge.rs"),
            include_str!("terminal_session_controls.rs"),
        ];
        for source in terminal_owned_sources {
            assert!(!source.contains("log_console_"));
            assert!(!source.contains("log_review_event"));
        }

        let production_refresh = include_str!("production_status_refresh.rs");
        assert!(!production_refresh.contains("log_console_"));
        assert!(!production_refresh.contains("log_review_event"));

        let runtime = include_str!("main.rs");
        assert!(!runtime.contains("log_review_event"));
        let finding_branch = runtime
            .split("HitTarget::CheckFinding(fingerprint) =>")
            .nth(1)
            .and_then(|tail| tail.split("HitTarget::FitBoard =>").next())
            .expect("check-finding route remains present");
        assert!(!finding_branch.contains("log_console_"));
    }

    #[test]
    fn history_controls_never_take_keyboard_focus() {
        use crate::keyboard_focus::focus_after_hit_target;
        use datum_gui_protocol::{ApplicationFocus, ConsoleHistoryFilter, PaneId};
        use datum_gui_render::HitTarget;

        let editor = ApplicationFocus::Editor(PaneId(1));
        for target in [
            HitTarget::ConsoleHistoryToggle,
            HitTarget::ConsoleHistoryFilter(ConsoleHistoryFilter::Operations),
        ] {
            assert_eq!(focus_after_hit_target(editor, true, &target), editor);
        }
    }
}
