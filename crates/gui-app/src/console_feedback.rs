//! Output-only Datum Console publication and interaction boundary (decision 033).

use super::*;
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

impl Runtime {
    pub(super) fn publish_console_feedback(&mut self, draft: ConsoleFeedbackDraft) {
        self.publish_console_feedback_with_criticality(draft, false);
    }

    fn publish_console_feedback_with_criticality(
        &mut self,
        draft: ConsoleFeedbackDraft,
        critical_consequence: bool,
    ) {
        let announcement =
            console_accessibility::announcement_for_draft(&draft, critical_consequence);
        publish(&mut self.session.workspace_mut().ui.console, draft);
        self.terminal_accessibility.announce_console(announcement);
        // Visible feedback is frame state. Invalidate here rather than relying on
        // every producer to remember a separate redraw side effect.
        self.invalidate_frame();
    }

    pub(super) fn log_console_echo(
        &mut self,
        source: ConsoleFeedbackSource,
        message: impl Into<String>,
    ) {
        self.publish_console_feedback(ConsoleFeedbackDraft::action_echo(
            source,
            occurred_unix_ms(),
            message,
        ));
    }

    pub(super) fn log_console_echo_for_action(
        &mut self,
        source: ConsoleFeedbackSource,
        action_id: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.publish_console_feedback(
            ConsoleFeedbackDraft::action_echo(source, occurred_unix_ms(), message)
                .with_action_id(action_id),
        );
    }

    pub(super) fn log_console_echo_for_target(
        &mut self,
        source: ConsoleFeedbackSource,
        target_id: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.publish_console_feedback(
            ConsoleFeedbackDraft::action_echo(source, occurred_unix_ms(), message)
                .with_target_id(target_id),
        );
    }

    pub(super) fn log_console_refusal(
        &mut self,
        source: ConsoleFeedbackSource,
        message: impl Into<String>,
    ) {
        self.publish_console_feedback(ConsoleFeedbackDraft::action_refusal(
            source,
            occurred_unix_ms(),
            message,
        ));
    }

    pub(super) fn log_console_refusal_for_action(
        &mut self,
        source: ConsoleFeedbackSource,
        action_id: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.publish_console_feedback(
            ConsoleFeedbackDraft::action_refusal(source, occurred_unix_ms(), message)
                .with_action_id(action_id),
        );
    }

    pub(super) fn log_console_refusal_for_target(
        &mut self,
        source: ConsoleFeedbackSource,
        target_id: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.publish_console_feedback(
            ConsoleFeedbackDraft::action_refusal(source, occurred_unix_ms(), message)
                .with_target_id(target_id),
        );
    }

    pub(super) fn log_console_critical_refusal_for_action(
        &mut self,
        source: ConsoleFeedbackSource,
        action_id: impl Into<String>,
        message: impl Into<String>,
    ) {
        let draft = ConsoleFeedbackDraft::action_refusal(source, occurred_unix_ms(), message)
            .with_action_id(action_id);
        self.publish_console_feedback_with_criticality(draft, true);
    }

    pub(super) fn toggle_console_history(&mut self) -> bool {
        let console = &mut self.session.workspace_mut().ui.console;
        console.set_history_expanded(!console.history_expanded());
        self.invalidate_frame();
        true
    }

    pub(super) fn handle_console_history_scroll(&mut self, scroll_lines: f32) -> bool {
        if scroll_lines.abs() <= 0.01 || !self.workspace().ui.console.history_expanded() {
            return false;
        }
        let Some((x, y)) = self.last_cursor_pos else {
            return false;
        };
        let history_panel = self
            .presented_console_layout
            .and_then(|layout| layout.history_panel);
        if !history_panel.is_some_and(|panel| panel.contains(x, y)) {
            return false;
        }
        let console = &mut self.session.workspace_mut().ui.console;
        let next = if scroll_lines > 0.0 {
            console.history_scroll_offset().saturating_add(1).min(
                datum_gui_protocol::CONSOLE_FEEDBACK_CAPACITY
                    + datum_gui_protocol::CONSOLE_JOURNAL_PROJECTION_CAPACITY
                    + 2,
            )
        } else {
            console.history_scroll_offset().saturating_sub(1)
        };
        console.set_history_scroll_offset(next);
        self.invalidate_frame();
        true
    }
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
