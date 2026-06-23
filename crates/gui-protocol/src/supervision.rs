//! Supervision-reflection scene-contract companions (Decision-013 level-1 parity).
//!
//! These are READ-ONLY projections of the resolved `DesignModel`
//! (`eda_engine::substrate`). They introduce NO design authority into the GUI:
//! every field is a projection of a named `DesignModel` field per
//! `specs/gui/GUI_SUPERVISION_REFLECTION.md` §3/§4. They are companion contracts
//! attached to the scene, not edits to the existing `BoardReviewSceneV1`
//! primitives, so they do not perturb the checked-in goldens.
//!
//! This slice lands the two projections at the heart of the supervision floor:
//! - R12 `SupervisionJournalReflectionV1` — the provenance-honest activity ledger
//!   (a projection of `journal: Vec<TransactionRecord>` + `journal_cursor`).
//! - R13 `SupervisionResolverStatusV1` — the resolver-status / recovery
//!   projection (a projection of `diagnostics: Vec<ResolveDiagnostic>` +
//!   `source_shards` + `model_revision`).
//!
//! All ids are `String`-typed to match the existing scene-contract triple
//! (`object_id`, `object_kind`, `source_object_uuid`); `Uuid`/`ModelRevision`
//! engine values are rendered to their canonical display strings at projection
//! time. The types are byte-stable: the engine model is already deterministic,
//! and these projections preserve journal order and reflect the
//! already-sorted `BTreeMap`/`Vec` engine state.

use serde::{Deserialize, Serialize};

use eda_engine::substrate::{
    CommitSource, DesignModel, ResolveDiagnostic, TransactionKind, TransactionRecord,
};

use crate::{
    ReviewWorkspaceState, SelectionTarget, SessionCommand, SessionCommandResult, SessionEvent,
};

/// Read-only supervision-reflection navigation handler (§5). Returns
/// `Some(result)` for the supervision command arms and `None` otherwise so the
/// caller can fall through to other handlers. Every arm mutates ONLY consumer
/// state (hover / selection / active-variant view state); none construct an
/// `OperationBatch`, touch the `DesignModel`, or reach `commit()`. The supervision
/// `SessionCommand` arms carry no commit-bearing payload, so the absence of a
/// write path here is structural, not merely runtime discipline. The runtime
/// half of this invariant is gated by PS-SR-2.
pub(crate) fn apply_supervision_command(
    workspace: &mut ReviewWorkspaceState,
    command: &SessionCommand,
) -> Option<SessionCommandResult> {
    let result = match command {
        SessionCommand::HoverObject(object_id) => {
            frame_result(workspace.hover_object(object_id.as_deref()))
        }
        SessionCommand::SelectFinding(finding_id) => {
            selection_result(workspace, |ws| ws.select_finding(finding_id))
        }
        SessionCommand::SelectJournalEntry(transaction_id) => {
            selection_result(workspace, |ws| ws.select_journal_entry(transaction_id))
        }
        SessionCommand::SelectRelationship(relationship_id) => {
            selection_result(workspace, |ws| ws.select_relationship(relationship_id))
        }
        SessionCommand::SetActiveVariant(variant) => {
            scene_result(workspace.set_active_variant(variant.as_deref()))
        }
        SessionCommand::RefreshModel => SessionCommandResult {
            handled: true,
            events: vec![SessionEvent::SceneChanged],
        },
        _ => return None,
    };
    Some(result)
}

fn frame_result(changed: bool) -> SessionCommandResult {
    single_event_result(changed, SessionEvent::FrameChanged)
}

fn scene_result(changed: bool) -> SessionCommandResult {
    single_event_result(changed, SessionEvent::SceneChanged)
}

fn single_event_result(changed: bool, event: SessionEvent) -> SessionCommandResult {
    if changed {
        SessionCommandResult {
            handled: true,
            events: vec![event],
        }
    } else {
        SessionCommandResult {
            handled: false,
            events: Vec::new(),
        }
    }
}

/// Selection commands emit `SelectionChanged` + `FrameChanged` when they take.
fn selection_result(
    workspace: &mut ReviewWorkspaceState,
    mutate: impl FnOnce(&mut ReviewWorkspaceState) -> bool,
) -> SessionCommandResult {
    if mutate(workspace) {
        SessionCommandResult {
            handled: true,
            events: vec![
                SessionEvent::SelectionChanged(workspace.selection.clone()),
                SessionEvent::FrameChanged,
            ],
        }
    } else {
        SessionCommandResult {
            handled: false,
            events: Vec::new(),
        }
    }
}

impl ReviewWorkspaceState {
    /// Supervision-reflection hover (consumer state only). Returns true when the
    /// hovered object id actually changed. Read-only: touches no `DesignModel`.
    pub fn hover_object(&mut self, object_id: Option<&str>) -> bool {
        let next = object_id.map(|id| id.to_string());
        if self.ui.hovered_object_id == next {
            return false;
        }
        self.ui.hovered_object_id = next;
        true
    }

    /// Supervision-reflection finding selection (consumer state only).
    pub fn select_finding(&mut self, finding_id: &str) -> bool {
        if finding_id.is_empty() {
            return false;
        }
        self.selection = SelectionTarget::Finding(finding_id.to_string());
        true
    }

    /// Supervision-reflection journal/transaction selection (consumer state only).
    pub fn select_journal_entry(&mut self, transaction_id: &str) -> bool {
        if transaction_id.is_empty() {
            return false;
        }
        self.selection = SelectionTarget::JournalEntry(transaction_id.to_string());
        true
    }

    /// Supervision-reflection relationship selection (consumer state only).
    pub fn select_relationship(&mut self, relationship_id: &str) -> bool {
        if relationship_id.is_empty() {
            return false;
        }
        self.selection = SelectionTarget::Relationship(relationship_id.to_string());
        true
    }

    /// Supervision-reflection active-variant view state (consumer state only,
    /// never journaled, never a commit). Returns true when the choice changed.
    pub fn set_active_variant(&mut self, variant: Option<&str>) -> bool {
        let next = variant.map(|id| id.to_string());
        if self.ui.filters.active_variant == next {
            return false;
        }
        self.ui.filters.active_variant = next;
        true
    }
}

/// Read-only supervision-reflection projection bundle attached to the workspace
/// so the GUI renderer can DISPLAY committed engine state (Decision-013 level-1).
///
/// This is a pure projection carrier: it holds the already-projected R12 journal
/// reflection and R13 resolver status. It introduces NO design authority — there
/// is no field from which a renderer or handler could construct an
/// `OperationBatch` or reach `commit()`. The renderer reads these to draw the
/// supervision panel; selection/hover remain consumer state on the workspace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SupervisionReflectionState {
    /// R12 — present once a resolved model has been projected.
    pub journal: Option<SupervisionJournalReflectionV1>,
    /// R13 — present once a resolved model has been projected.
    pub resolver_status: Option<SupervisionResolverStatusV1>,
}

impl SupervisionReflectionState {
    /// Project both supervision surfaces (R12 + R13) from a resolved
    /// `DesignModel`. Read-only: borrows the model, never mutates or re-solves.
    pub fn from_design_model(model: &DesignModel) -> Self {
        Self {
            journal: Some(SupervisionJournalReflectionV1::from_design_model(model)),
            resolver_status: Some(SupervisionResolverStatusV1::from_design_model(model)),
        }
    }

    /// True when the resolver projected into recovery mode (§4.8): the canvas is
    /// suppressed and the diagnostics list is promoted to primary.
    pub fn is_recovery(&self) -> bool {
        self.resolver_status
            .as_ref()
            .map(|status| status.mode == SUPERVISION_RESOLVER_MODE_RECOVERY)
            .unwrap_or(false)
    }
}

/// R12 — the journal / activity ledger.
///
/// A read-only reflection of `DesignModel.journal` and `journal_cursor`. No new
/// authority: it cannot mutate the journal, only project it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupervisionJournalReflectionV1 {
    pub contract: String,
    pub version: u32,
    pub model_revision: String,
    pub applied_transaction_count: usize,
    /// Newest-last, in committed journal order.
    pub entries: Vec<SupervisionJournalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupervisionJournalEntry {
    pub transaction_id: String,
    pub batch_id: String,
    pub transaction_kind: String,
    pub undo_of: Option<String>,
    pub redo_of: Option<String>,
    pub actor: String,
    pub source: String,
    pub reason: String,
    pub before_model_revision: String,
    pub after_model_revision: String,
    pub created_object_ids: Vec<String>,
    pub modified_object_ids: Vec<String>,
    pub deleted_object_ids: Vec<String>,
    pub applied: bool,
}

pub const SUPERVISION_JOURNAL_REFLECTION_V1_CONTRACT: &str = "supervision_journal_reflection_v1";

impl SupervisionJournalReflectionV1 {
    /// Project the committed journal activity from a resolved `DesignModel`.
    /// Read-only: borrows the model, never mutates it.
    pub fn from_design_model(model: &DesignModel) -> Self {
        let applied_transaction_count = model.journal_cursor.applied_transaction_count;
        let entries = model
            .journal
            .iter()
            .enumerate()
            .map(|(index, record)| {
                SupervisionJournalEntry::from_record(record, index < applied_transaction_count)
            })
            .collect();
        Self {
            contract: SUPERVISION_JOURNAL_REFLECTION_V1_CONTRACT.to_string(),
            version: 1,
            model_revision: model.model_revision.0.clone(),
            applied_transaction_count,
            entries,
        }
    }
}

impl SupervisionJournalEntry {
    fn from_record(record: &TransactionRecord, applied: bool) -> Self {
        Self {
            transaction_id: record.transaction_id.to_string(),
            batch_id: record.batch_id.to_string(),
            transaction_kind: transaction_kind_str(record.transaction_kind).to_string(),
            undo_of: record.undo_of.map(|id| id.to_string()),
            redo_of: record.redo_of.map(|id| id.to_string()),
            actor: record.provenance.actor.clone(),
            source: commit_source_str(record.provenance.source).to_string(),
            reason: record.provenance.reason.clone(),
            before_model_revision: record.before_model_revision.0.clone(),
            after_model_revision: record.after_model_revision.0.clone(),
            created_object_ids: record.diff.created.iter().map(|id| id.to_string()).collect(),
            modified_object_ids: record.diff.modified.iter().map(|id| id.to_string()).collect(),
            deleted_object_ids: record.diff.deleted.iter().map(|id| id.to_string()).collect(),
            applied,
        }
    }
}

/// Exhaustive over `TransactionKind` (snake_case), mirroring the engine serde.
fn transaction_kind_str(kind: TransactionKind) -> &'static str {
    match kind {
        TransactionKind::Normal => "normal",
        TransactionKind::Undo => "undo",
        TransactionKind::Redo => "redo",
    }
}

/// Exhaustive over `CommitSource` (snake_case), mirroring the engine serde.
/// A new engine arm forces a compile error here rather than silently bucketing.
fn commit_source_str(source: CommitSource) -> &'static str {
    match source {
        CommitSource::Manual => "manual",
        CommitSource::Cli => "cli",
        CommitSource::Test => "test",
        CommitSource::Tool => "tool",
        CommitSource::Assistant => "assistant",
    }
}

/// R13 — resolver diagnostics / recovery state.
///
/// A read-only reflection of `DesignModel.diagnostics` (`ResolveDiagnostic`),
/// `source_shards`, and `model_revision`. `mode`/`coherent`/`severity` are GUI
/// classifications of the already-resolved model, never new engine fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupervisionResolverStatusV1 {
    pub contract: String,
    pub version: u32,
    pub mode: String,
    /// `Some(live)` in resolved mode, `None` in recovery.
    pub model_revision: Option<String>,
    pub shard_count: usize,
    pub coherent: bool,
    pub diagnostics: Vec<SupervisionDiagnosticEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupervisionDiagnosticEntry {
    pub code: String,
    pub message: String,
    pub path: Option<String>,
    /// GUI CLASSIFICATION, not an engine field. `ResolveDiagnostic` carries no
    /// severity today (§4.8/OQ9); classified by the code-prefix table below.
    pub severity: String,
}

pub const SUPERVISION_RESOLVER_STATUS_V1_CONTRACT: &str = "supervision_resolver_status_v1";

pub const SUPERVISION_RESOLVER_MODE_RESOLVED: &str = "resolved";
pub const SUPERVISION_RESOLVER_MODE_RECOVERY: &str = "recovery";
pub const SUPERVISION_DIAGNOSTIC_SEVERITY_ERROR: &str = "error";
pub const SUPERVISION_DIAGNOSTIC_SEVERITY_WARNING: &str = "warning";

/// Golden-stable severity classification by `ResolveDiagnostic.code` prefix
/// (§4.8). The table is total via the fail-safe default: an unknown / unmatched
/// code classifies as `error`, which promotes the project into `recovery` rather
/// than silently downgrading to a warning the supervisor might ignore. New
/// diagnostic codes that must be warnings are added here as a reviewable diff.
const SUPERVISION_WARNING_CODE_PREFIXES: &[&str] = &[
    // Non-fatal advisories the resolver can continue past. Prefixed so families
    // of related codes classify uniformly and the table stays golden-stable.
    "WARN_",
    "ADVISORY_",
    "STALE_",
];

/// Classify a diagnostic code to `{error, warning}`. Fail-safe: unknown ⇒ error.
pub fn classify_diagnostic_severity(code: &str) -> &'static str {
    if SUPERVISION_WARNING_CODE_PREFIXES
        .iter()
        .any(|prefix| code.starts_with(prefix))
    {
        SUPERVISION_DIAGNOSTIC_SEVERITY_WARNING
    } else {
        SUPERVISION_DIAGNOSTIC_SEVERITY_ERROR
    }
}

impl SupervisionResolverStatusV1 {
    /// Project the resolver status / recovery state from a resolved
    /// `DesignModel`. Read-only: borrows the model, never re-solves or mutates.
    pub fn from_design_model(model: &DesignModel) -> Self {
        let diagnostics: Vec<SupervisionDiagnosticEntry> = model
            .diagnostics
            .iter()
            .map(SupervisionDiagnosticEntry::from_diagnostic)
            .collect();
        let has_error = diagnostics
            .iter()
            .any(|entry| entry.severity == SUPERVISION_DIAGNOSTIC_SEVERITY_ERROR);
        let mode = if has_error {
            SUPERVISION_RESOLVER_MODE_RECOVERY
        } else {
            SUPERVISION_RESOLVER_MODE_RESOLVED
        };
        let coherent = !has_error;
        let model_revision = if coherent {
            Some(model.model_revision.0.clone())
        } else {
            None
        };
        Self {
            contract: SUPERVISION_RESOLVER_STATUS_V1_CONTRACT.to_string(),
            version: 1,
            mode: mode.to_string(),
            model_revision,
            shard_count: model.source_shards.len(),
            coherent,
            diagnostics,
        }
    }
}

impl SupervisionDiagnosticEntry {
    fn from_diagnostic(diagnostic: &ResolveDiagnostic) -> Self {
        Self {
            code: diagnostic.code.clone(),
            message: diagnostic.message.clone(),
            path: diagnostic
                .path
                .as_ref()
                .map(|path| path.display().to_string()),
            severity: classify_diagnostic_severity(&diagnostic.code).to_string(),
        }
    }
}

use eda_engine::substrate::{
    CommitDiff, CommitProvenance, JournalCursor, ModelRevision, ProjectManifestSummary,
};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Build a deterministic `DesignModel` carrying one journal entry per
/// `CommitSource` arm. Used by the projection tests, the PS-SR-2 read-only
/// invariant test, and the supervision visual goldens (PS-SR-4). It is a fixture
/// builder, not design authority: the model is in-memory and read-only.
pub fn fixture_design_model_with_full_provenance() -> DesignModel {
    {
        let project_id =
            Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("uuid parses");
        let sources = [
            CommitSource::Manual,
            CommitSource::Cli,
            CommitSource::Test,
            CommitSource::Tool,
            CommitSource::Assistant,
        ];
        let journal: Vec<TransactionRecord> = sources
            .iter()
            .enumerate()
            .map(|(index, source)| {
                let txid = Uuid::from_u128(0xa000 + index as u128);
                let created = Uuid::from_u128(0xb000 + index as u128);
                TransactionRecord {
                    transaction_id: txid,
                    batch_id: Uuid::from_u128(0xc000 + index as u128),
                    transaction_kind: TransactionKind::Normal,
                    undo_of: None,
                    redo_of: None,
                    before_model_revision: ModelRevision(format!("rev-{index}")),
                    after_model_revision: ModelRevision(format!("rev-{}", index + 1)),
                    provenance: CommitProvenance {
                        actor: format!("actor-{index}"),
                        source: *source,
                        reason: format!("reason-{index}"),
                    },
                    diff: CommitDiff {
                        created: vec![created],
                        modified: Vec::new(),
                        deleted: Vec::new(),
                    },
                    operations: Vec::new(),
                    inverse_operations: Vec::new(),
                }
            })
            .collect();
        DesignModel {
            project: ProjectManifestSummary {
                project_id,
                name: "supervision-fixture".to_string(),
                schema_version: Some(1),
            },
            model_revision: ModelRevision("rev-5".to_string()),
            source_shards: Vec::new(),
            objects: BTreeMap::new(),
            component_instances: BTreeMap::new(),
            relationships: BTreeMap::new(),
            relationship_statuses: BTreeMap::new(),
            variants: BTreeMap::new(),
            variant_populations: BTreeMap::new(),
            import_map: BTreeMap::new(),
            zone_fills: BTreeMap::new(),
            manufacturing_plans: BTreeMap::new(),
            panel_projections: BTreeMap::new(),
            output_jobs: BTreeMap::new(),
            output_job_runs: BTreeMap::new(),
            artifact_runs: BTreeMap::new(),
            check_runs: BTreeMap::new(),
            artifact_metadata: BTreeMap::new(),
            proposals: BTreeMap::new(),
            journal,
            journal_cursor: JournalCursor {
                applied_transaction_count: 5,
            },
            diagnostics: Vec::new(),
        }
    }
}

/// Attach diagnostics to a fixture model (test/golden helper).
pub fn with_diagnostics(mut model: DesignModel, diagnostics: Vec<ResolveDiagnostic>) -> DesignModel {
    model.diagnostics = diagnostics;
    model
}

/// Fixture model that resolves into RECOVERY mode (§4.8 / PS-SR-6): the
/// full-provenance model carrying an unclassified (error-severity) diagnostic so
/// the resolver-status projection promotes to recovery. Drives the
/// `supervision_resolver_recovery` golden.
pub fn fixture_design_model_with_split_project_diagnostic() -> DesignModel {
    with_diagnostics(
        fixture_design_model_with_full_provenance(),
        vec![ResolveDiagnostic {
            code: "SPLIT_PROJECT".to_string(),
            message: "two project roots resolved; project is incoherent".to_string(),
            path: Some("/tmp/split-a.kicad_pro".into()),
        }],
    )
}

/// Deterministic supervision-reflection golden fixture: the fixture workspace
/// scene with the Supervision dock tab active and the read-only R12/R13
/// projections of `model` attached. Read-only: projections are built from an
/// in-memory `DesignModel` and carry no edit/commit affordance. The dock is sized
/// tall enough to show the resolver banner plus all five provenance rows.
fn supervision_fixture_workspace_state(model: &DesignModel) -> ReviewWorkspaceState {
    let mut state = crate::load_fixture_workspace_state().with_supervision_from_model(model);
    state.ui.active_dock_tab = Some(crate::DockTab::Supervision);
    state.ui.dock_height_px = 320;
    state.selection = SelectionTarget::None;
    state
}

/// PS-SR-4 provenance fixture: a clean (resolved) resolver status and a
/// provenance-honest journal with one commit per `CommitSource` arm
/// (`manual`/`cli`/`test`/`tool`/`assistant`). Drives `supervision_activity_provenance`.
pub fn supervision_fixture_workspace_state_resolved() -> ReviewWorkspaceState {
    supervision_fixture_workspace_state(&fixture_design_model_with_full_provenance())
}

/// PS-SR-6 recovery fixture: the resolved model carries an unclassified
/// (error-severity) diagnostic, so the panel renders the recovery banner with
/// diagnostics primary (`QG-RESOLVER-RECOVERY`). Drives `supervision_resolver_recovery`.
pub fn supervision_fixture_workspace_state_recovery() -> ReviewWorkspaceState {
    supervision_fixture_workspace_state(&fixture_design_model_with_split_project_diagnostic())
}

#[cfg(test)]
mod tests {
    use super::*;
    use eda_engine::substrate::ResolveDiagnostic;

    #[test]
    fn journal_reflection_projects_every_commit_source_verbatim() {
        let model = fixture_design_model_with_full_provenance();
        let reflection = SupervisionJournalReflectionV1::from_design_model(&model);

        assert_eq!(reflection.contract, SUPERVISION_JOURNAL_REFLECTION_V1_CONTRACT);
        assert_eq!(reflection.version, 1);
        assert_eq!(reflection.model_revision, "rev-5");
        assert_eq!(reflection.applied_transaction_count, 5);
        assert_eq!(reflection.entries.len(), 5);

        let sources: Vec<&str> = reflection
            .entries
            .iter()
            .map(|entry| entry.source.as_str())
            .collect();
        assert_eq!(sources, vec!["manual", "cli", "test", "tool", "assistant"]);

        for entry in &reflection.entries {
            assert!(entry.applied, "all fixture entries are within the cursor");
            assert_eq!(entry.transaction_kind, "normal");
            assert_eq!(entry.created_object_ids.len(), 1);
            assert_ne!(entry.before_model_revision, entry.after_model_revision);
        }
    }

    #[test]
    fn journal_reflection_marks_beyond_cursor_entries_unapplied() {
        let mut model = fixture_design_model_with_full_provenance();
        model.journal_cursor.applied_transaction_count = 3;
        let reflection = SupervisionJournalReflectionV1::from_design_model(&model);

        let applied: Vec<bool> = reflection
            .entries
            .iter()
            .map(|entry| entry.applied)
            .collect();
        assert_eq!(applied, vec![true, true, true, false, false]);
    }

    #[test]
    fn resolver_status_is_resolved_when_no_error_diagnostics() {
        let model = fixture_design_model_with_full_provenance();
        let status = SupervisionResolverStatusV1::from_design_model(&model);

        assert_eq!(status.contract, SUPERVISION_RESOLVER_STATUS_V1_CONTRACT);
        assert_eq!(status.mode, SUPERVISION_RESOLVER_MODE_RESOLVED);
        assert!(status.coherent);
        assert_eq!(status.model_revision.as_deref(), Some("rev-5"));
        assert!(status.diagnostics.is_empty());
    }

    #[test]
    fn resolver_status_enters_recovery_on_unknown_error_diagnostic() {
        let model = with_diagnostics(
            fixture_design_model_with_full_provenance(),
            vec![ResolveDiagnostic {
                code: "SPLIT_PROJECT".to_string(),
                message: "two project roots".to_string(),
                path: Some("/tmp/a.kicad_pro".into()),
            }],
        );
        let status = SupervisionResolverStatusV1::from_design_model(&model);

        assert_eq!(status.mode, SUPERVISION_RESOLVER_MODE_RECOVERY);
        assert!(!status.coherent);
        assert!(status.model_revision.is_none(), "recovery hides the revision");
        assert_eq!(status.diagnostics.len(), 1);
        assert_eq!(
            status.diagnostics[0].severity,
            SUPERVISION_DIAGNOSTIC_SEVERITY_ERROR
        );
        assert_eq!(
            status.diagnostics[0].path.as_deref(),
            Some("/tmp/a.kicad_pro")
        );
    }

    #[test]
    fn warning_prefixed_diagnostic_stays_resolved() {
        let model = with_diagnostics(
            fixture_design_model_with_full_provenance(),
            vec![ResolveDiagnostic {
                code: "WARN_ORPHAN_NET".to_string(),
                message: "net has no pads".to_string(),
                path: None,
            }],
        );
        let status = SupervisionResolverStatusV1::from_design_model(&model);

        assert_eq!(status.mode, SUPERVISION_RESOLVER_MODE_RESOLVED);
        assert!(status.coherent);
        assert_eq!(
            status.diagnostics[0].severity,
            SUPERVISION_DIAGNOSTIC_SEVERITY_WARNING
        );
    }

    #[test]
    fn unknown_diagnostic_code_classifies_error_failsafe() {
        assert_eq!(
            classify_diagnostic_severity("TOTALLY_NEW_CODE"),
            SUPERVISION_DIAGNOSTIC_SEVERITY_ERROR
        );
        assert_eq!(
            classify_diagnostic_severity("WARN_X"),
            SUPERVISION_DIAGNOSTIC_SEVERITY_WARNING
        );
    }
}
