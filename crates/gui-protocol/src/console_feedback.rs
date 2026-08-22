//! Typed consumer state for the output-only Datum Console (decision 033).
//!
//! The Console reports GUI action consequences. It is not a text-input model,
//! verb dispatcher, operation author, journal, terminal source, or PTY writer.

use std::collections::VecDeque;

/// The deterministic in-memory history bound inherited from the former sink.
pub const CONSOLE_FEEDBACK_CAPACITY: usize = 240;
pub const CONSOLE_JOURNAL_PROJECTION_CAPACITY: usize = 240;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConsoleFeedbackDuration {
    FourSeconds,
    #[default]
    SixSeconds,
    TenSeconds,
    Never,
}

impl ConsoleFeedbackDuration {
    pub fn milliseconds(self) -> Option<u64> {
        match self {
            Self::FourSeconds => Some(4_000),
            Self::SixSeconds => Some(6_000),
            Self::TenSeconds => Some(10_000),
            Self::Never => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConsoleHistoryFilter {
    #[default]
    All,
    Operations,
    Errors,
}

/// Read-only summary of one applied canonical journal transaction. Journal
/// order is authoritative; no wall-clock timestamp is fabricated here.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConsoleJournalProjectionRecord {
    pub journal_ordinal: usize,
    pub transaction_id: String,
    pub transaction_kind: String,
    pub commit_source: String,
    pub reason: String,
    pub operation_count: usize,
    pub created_count: usize,
    pub modified_count: usize,
    pub deleted_count: usize,
}

/// Session-local observation of resolver-journal truth. This is deliberately a
/// sibling of consumer feedback, never content inside its deque.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConsoleJournalHistoryState {
    session_baseline_applied_count: usize,
    records: VecDeque<ConsoleJournalProjectionRecord>,
    omitted_session_record_count: usize,
}

impl ConsoleJournalHistoryState {
    pub fn begin_session(&mut self, applied_transaction_count: usize) {
        self.session_baseline_applied_count = applied_transaction_count;
        self.records.clear();
        self.omitted_session_record_count = 0;
    }

    pub fn reconcile(
        &mut self,
        projection: &[ConsoleJournalProjectionRecord],
        applied_transaction_count: usize,
    ) {
        self.records = projection
            .iter()
            .filter(|record| record.journal_ordinal > self.session_baseline_applied_count)
            .cloned()
            .collect();
        while self.records.len() > CONSOLE_JOURNAL_PROJECTION_CAPACITY {
            self.records.pop_front();
        }
        let observed_count =
            applied_transaction_count.saturating_sub(self.session_baseline_applied_count);
        self.omitted_session_record_count = observed_count.saturating_sub(self.records.len());
    }

    pub fn records(&self) -> impl DoubleEndedIterator<Item = &ConsoleJournalProjectionRecord> {
        self.records.iter()
    }

    pub fn omitted_session_record_count(&self) -> usize {
        self.omitted_session_record_count
    }
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
    history_filter: ConsoleHistoryFilter,
    history_scroll_offset: usize,
    duration_preference: ConsoleFeedbackDuration,
    visibility_elapsed_ms: u64,
    last_visibility_tick_ms: Option<u64>,
    inspected: bool,
}

impl Default for ConsoleFeedbackState {
    fn default() -> Self {
        Self {
            records: VecDeque::with_capacity(CONSOLE_FEEDBACK_CAPACITY),
            next_sequence: 1,
            dropped_count: 0,
            history_expanded: false,
            history_filter: ConsoleHistoryFilter::All,
            history_scroll_offset: 0,
            duration_preference: ConsoleFeedbackDuration::default(),
            visibility_elapsed_ms: 0,
            last_visibility_tick_ms: None,
            inspected: false,
        }
    }
}

impl ConsoleFeedbackState {
    pub fn publish(&mut self, draft: ConsoleFeedbackDraft) -> u64 {
        let sequence = self.next_sequence;
        let occurred_unix_ms = draft.occurred_unix_ms;
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
        self.visibility_elapsed_ms = 0;
        self.last_visibility_tick_ms = Some(occurred_unix_ms);
        self.inspected = false;
        sequence
    }

    pub fn latest(&self) -> Option<&ConsoleFeedbackRecord> {
        self.records.back()
    }

    pub fn visible_latest(&self) -> Option<&ConsoleFeedbackRecord> {
        let latest = self.latest()?;
        if latest.category != ConsoleFeedbackCategory::ActionEcho {
            return Some(latest);
        }
        match self.duration_preference.milliseconds() {
            Some(limit) if self.visibility_elapsed_ms >= limit => None,
            _ => Some(latest),
        }
    }

    /// Advance deterministic presentation time. Inspection and expanded
    /// history pause elapsed time without removing the underlying record.
    /// Returns true only when visible-overlay state changes.
    pub fn advance_visibility(&mut self, now_unix_ms: u64, inspected: bool) -> bool {
        let was_visible = self.visible_latest().is_some();
        let previous = self.last_visibility_tick_ms.replace(now_unix_ms);
        if let Some(previous) = previous
            && !self.inspected
            && !self.history_expanded
        {
            self.visibility_elapsed_ms = self
                .visibility_elapsed_ms
                .saturating_add(now_unix_ms.saturating_sub(previous));
        }
        self.inspected = inspected;
        was_visible != self.visible_latest().is_some()
    }

    pub fn remaining_auto_hide_ms(&self) -> Option<u64> {
        let latest = self.visible_latest()?;
        if latest.category != ConsoleFeedbackCategory::ActionEcho
            || self.inspected
            || self.history_expanded
        {
            return None;
        }
        self.duration_preference
            .milliseconds()
            .map(|limit| limit.saturating_sub(self.visibility_elapsed_ms))
    }

    pub fn inspected(&self) -> bool {
        self.inspected
    }

    pub fn duration_preference(&self) -> ConsoleFeedbackDuration {
        self.duration_preference
    }

    pub fn set_duration_preference(&mut self, duration: ConsoleFeedbackDuration) {
        self.duration_preference = duration;
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

    pub fn history_filter(&self) -> ConsoleHistoryFilter {
        self.history_filter
    }

    pub fn set_history_filter(&mut self, filter: ConsoleHistoryFilter) {
        self.history_filter = filter;
        self.history_scroll_offset = 0;
    }

    pub fn history_scroll_offset(&self) -> usize {
        self.history_scroll_offset
    }

    pub fn set_history_scroll_offset(&mut self, offset: usize) {
        self.history_scroll_offset = offset;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gui_action_feedback_cannot_mutate_terminal_projection() {
        let mut state = crate::load_fixture_workspace_state();
        let terminal_before = state.ui.terminal.clone();

        state
            .ui
            .publish_console_feedback(ConsoleFeedbackDraft::action_echo(
                ConsoleFeedbackSource::Workspace,
                42,
                "fit board",
            ));

        assert_eq!(state.ui.console.latest().unwrap().message, "fit board");
        assert_eq!(
            state.ui.terminal, terminal_before,
            "GUI action feedback must not mutate terminal session projection"
        );
    }

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

    #[test]
    fn journal_history_excludes_launch_baseline_and_reports_exact_omission() {
        let mut history = ConsoleJournalHistoryState::default();
        history.begin_session(10);
        let projection = (12..=260)
            .map(|ordinal| ConsoleJournalProjectionRecord {
                journal_ordinal: ordinal,
                transaction_id: format!("tx-{ordinal}"),
                transaction_kind: "normal".to_string(),
                commit_source: "manual".to_string(),
                reason: format!("operation {ordinal}"),
                operation_count: 1,
                created_count: 0,
                modified_count: 1,
                deleted_count: 0,
            })
            .collect::<Vec<_>>();

        history.reconcile(&projection, 260);

        assert_eq!(history.records().count(), 240);
        assert_eq!(history.records().next().unwrap().journal_ordinal, 21);
        assert_eq!(history.records().next_back().unwrap().journal_ordinal, 260);
        assert_eq!(history.omitted_session_record_count(), 10);
    }

    #[test]
    fn history_filters_and_scroll_are_consumer_state_only() {
        let mut feedback = ConsoleFeedbackState::default();
        feedback.publish(ConsoleFeedbackDraft::action_echo(
            ConsoleFeedbackSource::Viewport,
            42,
            "fit board",
        ));
        let sequence = feedback.latest().unwrap().sequence;

        feedback.set_history_expanded(true);
        feedback.set_history_filter(ConsoleHistoryFilter::Errors);
        feedback.set_history_scroll_offset(7);

        assert!(feedback.history_expanded());
        assert_eq!(feedback.history_filter(), ConsoleHistoryFilter::Errors);
        assert_eq!(feedback.history_scroll_offset(), 7);
        assert_eq!(feedback.latest().unwrap().sequence, sequence);
        assert_eq!(feedback.dropped_count(), 0);
    }

    #[test]
    fn transient_echo_fades_but_remains_recoverable_in_history() {
        let mut feedback = ConsoleFeedbackState::default();
        feedback.publish(ConsoleFeedbackDraft::action_echo(
            ConsoleFeedbackSource::Viewport,
            1_000,
            "fit board",
        ));
        assert!(feedback.visible_latest().is_some());

        assert!(feedback.advance_visibility(7_000, false));
        assert!(feedback.visible_latest().is_none());
        assert_eq!(feedback.latest().unwrap().message, "fit board");
        assert_eq!(feedback.records().count(), 1);
    }

    #[test]
    fn inspection_pauses_and_resumes_remaining_echo_lifetime() {
        let mut feedback = ConsoleFeedbackState::default();
        feedback.publish(ConsoleFeedbackDraft::action_echo(
            ConsoleFeedbackSource::Viewport,
            0,
            "zoom in",
        ));
        feedback.advance_visibility(3_000, true);
        feedback.advance_visibility(20_000, true);
        feedback.advance_visibility(20_000, false);
        feedback.advance_visibility(22_999, false);
        assert!(feedback.visible_latest().is_some());
        assert!(feedback.advance_visibility(23_000, false));
        assert!(feedback.visible_latest().is_none());
    }

    #[test]
    fn prompts_refusals_and_never_preference_ignore_echo_deadline() {
        for draft in [
            ConsoleFeedbackDraft::tool_prompt(ConsoleFeedbackSource::Tool, 0, "select a component"),
            ConsoleFeedbackDraft::action_refusal(
                ConsoleFeedbackSource::Tool,
                0,
                "no component selected",
            ),
        ] {
            let mut feedback = ConsoleFeedbackState::default();
            feedback.publish(draft);
            feedback.advance_visibility(60_000, false);
            assert!(feedback.visible_latest().is_some());
        }

        let mut feedback = ConsoleFeedbackState::default();
        feedback.set_duration_preference(ConsoleFeedbackDuration::Never);
        feedback.publish(ConsoleFeedbackDraft::action_echo(
            ConsoleFeedbackSource::Viewport,
            0,
            "fit board",
        ));
        feedback.advance_visibility(u64::MAX, false);
        assert!(feedback.visible_latest().is_some());
        assert_eq!(feedback.remaining_auto_hide_ms(), None);
    }
}
