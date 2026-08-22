//! Output-only Datum Console publication boundary (decision 033).

use datum_gui_protocol::{ConsoleFeedbackDraft, ConsoleFeedbackSource, ConsoleFeedbackState};

pub(super) fn occurred_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

pub(super) fn route_gui_action_echo(
    console: &mut ConsoleFeedbackState,
    message: impl Into<String>,
) -> u64 {
    console.publish(ConsoleFeedbackDraft::action_echo(
        ConsoleFeedbackSource::Editor,
        occurred_unix_ms(),
        message,
    ))
}

#[cfg(test)]
mod tests {
    use super::route_gui_action_echo;
    use datum_gui_protocol::{
        ConsoleFeedbackCategory, ConsoleFeedbackSource, ConsoleFeedbackState, TerminalLaneState,
    };

    #[test]
    fn publication_is_typed_and_cannot_mutate_terminal_state() {
        let mut console = ConsoleFeedbackState::default();
        let terminal = TerminalLaneState::default();
        let terminal_before = terminal.clone();

        route_gui_action_echo(&mut console, "fit board");

        let record = console.latest().unwrap();
        assert_eq!(record.source, ConsoleFeedbackSource::Editor);
        assert_eq!(record.category, ConsoleFeedbackCategory::ActionEcho);
        assert_eq!(record.message, "fit board");
        assert_eq!(terminal, terminal_before);
    }
}
