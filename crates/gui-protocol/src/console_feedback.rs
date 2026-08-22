//! Typed consumer state for the output-only Datum Console (decision 033).
//!
//! The Console reports GUI action consequences. It is not a text-input model,
//! verb dispatcher, operation author, journal, terminal source, or PTY writer.

use std::collections::VecDeque;

/// The deterministic in-memory history bound inherited from the former sink.
pub const CONSOLE_FEEDBACK_CAPACITY: usize = 240;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleFeedbackSeverity {
    Informational,
    Success,
    Warning,
    Error,
}

/// Only GUI-owned producers can publish Console feedback. Terminal lifecycle,
/// notifications, progress, and findings deliberately have no variant here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleFeedbackSource {
    Editor,
    Menu,
    Tool,
    Viewport,
    Selection,
    Production,
    Workspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleFeedbackCategory {
    ActionEcho,
    ToolPrompt,
    ActionRefusal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleFeedbackLifetime {
    Transient,
    PersistentUntilNextAction,
}

/// A publisher-owned fact before the Console assigns its local sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleFeedbackDraft {
    pub occurred_unix_ms: u64,
    pub severity: ConsoleFeedbackSeverity,
    pub source: ConsoleFeedbackSource,
    pub category: ConsoleFeedbackCategory,
    pub lifetime: ConsoleFeedbackLifetime,
    pub message: String,
    pub action_id: Option<String>,
    pub target_id: Option<String>,
}

impl ConsoleFeedbackDraft {
    pub fn action_echo(
        source: ConsoleFeedbackSource,
        occurred_unix_ms: u64,
        message: impl Into<String>,
    ) -> Self {
        Self {
            occurred_unix_ms,
            severity: ConsoleFeedbackSeverity::Informational,
            source,
            category: ConsoleFeedbackCategory::ActionEcho,
            lifetime: ConsoleFeedbackLifetime::Transient,
            message: message.into(),
            action_id: None,
            target_id: None,
        }
    }

    pub fn tool_prompt(
        source: ConsoleFeedbackSource,
        occurred_unix_ms: u64,
        message: impl Into<String>,
    ) -> Self {
        Self {
            occurred_unix_ms,
            severity: ConsoleFeedbackSeverity::Informational,
            source,
            category: ConsoleFeedbackCategory::ToolPrompt,
            lifetime: ConsoleFeedbackLifetime::PersistentUntilNextAction,
            message: message.into(),
            action_id: None,
            target_id: None,
        }
    }

    pub fn action_refusal(
        source: ConsoleFeedbackSource,
        occurred_unix_ms: u64,
        message: impl Into<String>,
    ) -> Self {
        Self {
            occurred_unix_ms,
            severity: ConsoleFeedbackSeverity::Error,
            source,
            category: ConsoleFeedbackCategory::ActionRefusal,
            lifetime: ConsoleFeedbackLifetime::PersistentUntilNextAction,
            message: message.into(),
            action_id: None,
            target_id: None,
        }
    }

    pub fn with_action_id(mut self, action_id: impl Into<String>) -> Self {
        self.action_id = Some(action_id.into());
        self
    }

    pub fn with_target_id(mut self, target_id: impl Into<String>) -> Self {
        self.target_id = Some(target_id.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleFeedbackRecord {
    pub sequence: u64,
    pub occurred_unix_ms: u64,
    pub severity: ConsoleFeedbackSeverity,
    pub source: ConsoleFeedbackSource,
    pub category: ConsoleFeedbackCategory,
    pub lifetime: ConsoleFeedbackLifetime,
    pub message: String,
    pub action_id: Option<String>,
    pub target_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleFeedbackState {
    records: VecDeque<ConsoleFeedbackRecord>,
    next_sequence: u64,
    dropped_count: u64,
    history_expanded: bool,
}

impl Default for ConsoleFeedbackState {
    fn default() -> Self {
        Self {
            records: VecDeque::with_capacity(CONSOLE_FEEDBACK_CAPACITY),
            next_sequence: 1,
            dropped_count: 0,
            history_expanded: false,
        }
    }
}

impl ConsoleFeedbackState {
    pub fn publish(&mut self, draft: ConsoleFeedbackDraft) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.records.push_back(ConsoleFeedbackRecord {
            sequence,
            occurred_unix_ms: draft.occurred_unix_ms,
            severity: draft.severity,
            source: draft.source,
            category: draft.category,
            lifetime: draft.lifetime,
            message: draft.message,
            action_id: draft.action_id,
            target_id: draft.target_id,
        });
        while self.records.len() > CONSOLE_FEEDBACK_CAPACITY {
            self.records.pop_front();
            self.dropped_count = self.dropped_count.saturating_add(1);
        }
        sequence
    }

    pub fn latest(&self) -> Option<&ConsoleFeedbackRecord> {
        self.records.back()
    }

    pub fn records(&self) -> impl DoubleEndedIterator<Item = &ConsoleFeedbackRecord> {
        self.records.iter()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn dropped_count(&self) -> u64 {
        self.dropped_count
    }

    pub fn history_expanded(&self) -> bool {
        self.history_expanded
    }

    pub fn set_history_expanded(&mut self, expanded: bool) {
        self.history_expanded = expanded;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_are_typed_sequenced_and_bounded() {
        let mut state = ConsoleFeedbackState::default();
        for index in 0..CONSOLE_FEEDBACK_CAPACITY + 3 {
            state.publish(ConsoleFeedbackDraft::action_echo(
                ConsoleFeedbackSource::Workspace,
                index as u64,
                format!("action {index}"),
            ));
        }

        assert_eq!(state.len(), CONSOLE_FEEDBACK_CAPACITY);
        assert_eq!(state.dropped_count(), 3);
        assert_eq!(state.records().next().unwrap().sequence, 4);
        assert_eq!(state.latest().unwrap().sequence, 243);
        assert_eq!(
            state.latest().unwrap().category,
            ConsoleFeedbackCategory::ActionEcho
        );
    }

    #[test]
    fn refusal_is_persistent_typed_output() {
        let mut state = ConsoleFeedbackState::default();
        state.publish(
            ConsoleFeedbackDraft::action_refusal(
                ConsoleFeedbackSource::Tool,
                42,
                "select a board text object first",
            )
            .with_action_id("datum.board_text.edit")
            .with_target_id("board:main"),
        );

        let record = state.latest().unwrap();
        assert_eq!(record.severity, ConsoleFeedbackSeverity::Error);
        assert_eq!(record.category, ConsoleFeedbackCategory::ActionRefusal);
        assert_eq!(
            record.lifetime,
            ConsoleFeedbackLifetime::PersistentUntilNextAction
        );
        assert_eq!(record.action_id.as_deref(), Some("datum.board_text.edit"));
        assert_eq!(record.target_id.as_deref(), Some("board:main"));
    }
}
