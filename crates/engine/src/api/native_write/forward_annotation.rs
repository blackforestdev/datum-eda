//! Proposal-batch and schematic→board forward-annotation builders for the
//! native write facade.
//!
//! Family E of the native-write migration: all operation authoring for
//! proposal batches and forward-annotation state lives here. The CLI callers
//! in `crates/cli/src/command_project_proposals.rs`,
//! `crates/cli/src/command_project_forward_annotation_review_state.rs`, and
//! `crates/cli/src/command_project_forward_annotation_substrate.rs` are thin
//! argument-parsers over this module. Guarded board-package edits are
//! composed from the family-D builders in [`super::board_components`] — the
//! only directly authored board-package operations are the intentionally
//! unguarded ones inside the accepted forward-annotation proposal batch,
//! whose historical contract has no per-object guards.
//!
//! Builders are build-only; they never touch disk and never commit:
//! - proposal-batch builders return the guarded [`OperationBatch`] that feeds
//!   [`crate::substrate::create_draft_proposal_from_batch`];
//! - the accepted forward-annotation [`Proposal`] built here is committed and
//!   applied through the substrate-owned `commit_proposal_metadata_journaled`
//!   + `apply_accepted_proposal` path;
//! - review-state builders return a [`PreparedWrite`] committed via
//!   [`super::commit_prepared`].
//!
//! Guard-insertion ordering (each guard immediately precedes the first
//! mutation of its object), batch stamping, the deterministic v5 proposal-id
//! derivation, and payload shapes are byte-for-byte the CLI's historical
//! behavior — proposal ids and batches land in journal records and exported
//! artifacts and must not drift.

use uuid::Uuid;

use crate::board::{Track, Via};
use crate::error::EngineError;
use crate::substrate::{
    DesignModel, ObjectId, Operation, OperationBatch, Proposal, ProposalSource, ProposalStatus,
};

use super::board_components::{BoardPackageEdit, build_edit_board_package};
use super::context::{PreparedWrite, WriteProvenance, build_batch};
use super::guards::guarded_operation_batch;
use super::ids;

/// Canonical relative path of the forward-annotation review sidecar shard.
///
/// Persistence-visible contract: `SetForwardAnnotationReview` /
/// `DeleteForwardAnnotationReview` journal records carry this path and the
/// substrate rejects any other (see
/// `substrate/forward_annotation_review_journal_ops.rs`).
pub const FORWARD_ANNOTATION_REVIEW_PATH: &str = ".datum/forward_annotation_review/review.json";

/// Insert object-revision guards into an externally authored proposal batch
/// (e.g. one read from a `--batch` file) without restamping its `batch_id`,
/// `expected_model_revision`, or provenance.
///
/// This is the exact guard pass every facade-built batch receives; external
/// batches go through it so a draft proposal is always revision-guarded.
pub fn guard_proposal_batch(
    model: &DesignModel,
    batch: OperationBatch,
) -> Result<OperationBatch, EngineError> {
    guarded_operation_batch(model, batch)
}

/// Build the guarded, revision-stamped batch for a board-component
/// replacement proposal: one [`BoardPackageEdit`] per entry, applied in
/// order, guards inserted immediately before the first edit of each
/// component.
///
/// Each edit's operation is composed from
/// [`super::board_components::build_edit_board_package`] (family D owns the
/// edit→operation mapping); the per-edit guards are dropped and the combined
/// batch is re-guarded as one unit, reproducing the historical CLI sequence
/// (author all operations, then guard the whole batch) byte-for-byte.
///
/// The returned batch is uncommitted — it feeds
/// [`crate::substrate::create_draft_proposal_from_batch`].
pub fn build_board_component_replacement_proposal_batch(
    model: &DesignModel,
    provenance: WriteProvenance,
    edits: Vec<(ObjectId, BoardPackageEdit)>,
) -> Result<OperationBatch, EngineError> {
    let mut operations = Vec::with_capacity(edits.len());
    for (package_id, edit) in edits {
        let prepared = build_edit_board_package(model, provenance.clone(), package_id, edit)?;
        operations.extend(unguarded_operations(prepared));
    }
    build_batch(model, provenance, operations)
}

/// Build the batch that rewrites the forward-annotation review sidecar.
///
/// `previous_review` is the currently materialized sidecar value (None when
/// the sidecar does not exist yet); `review` is the new sidecar payload.
/// Neither operation targets an existing model object, so the batch carries
/// no object-revision guards — exactly the historical CLI batch.
pub fn build_set_forward_annotation_review(
    model: &DesignModel,
    provenance: WriteProvenance,
    previous_review: Option<serde_json::Value>,
    review: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let batch = build_batch(
        model,
        provenance,
        vec![Operation::SetForwardAnnotationReview {
            relative_path: FORWARD_ANNOTATION_REVIEW_PATH.to_string(),
            previous_review,
            review,
        }],
    )?;
    Ok(PreparedWrite {
        batch,
        primary_object_id: None,
    })
}

/// Build the batch that deletes the forward-annotation review sidecar.
/// `review` is the currently materialized sidecar value (journaled so the
/// delete is replayable).
pub fn build_clear_forward_annotation_review(
    model: &DesignModel,
    provenance: WriteProvenance,
    review: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let batch = build_batch(
        model,
        provenance,
        vec![Operation::DeleteForwardAnnotationReview {
            relative_path: FORWARD_ANNOTATION_REVIEW_PATH.to_string(),
            review,
        }],
    )?;
    Ok(PreparedWrite {
        batch,
        primary_object_id: None,
    })
}

/// One accepted forward-annotation action, already resolved by the caller to
/// its target board package (and, for removals, its stored payloads).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForwardAnnotationProposalAction {
    /// `update_component` / `value_mismatch`: set the board package value to
    /// the schematic value.
    SetComponentValue { package_id: ObjectId, value: String },
    /// `remove_component` / `board_component_missing_in_schematic`: delete
    /// the board package. `package` and `materialized` are the stored raw
    /// payloads (threaded verbatim for byte-exact journal parity).
    RemoveComponent {
        package_id: ObjectId,
        package: serde_json::Value,
        materialized: serde_json::Value,
    },
}

/// Build the self-sufficient accepted forward-annotation [`Proposal`].
///
/// Deterministic identity (byte-for-byte the historical CLI derivation):
/// - `proposal_id` is the v5 id over
///   `datum-eda:forward-annotation-proposal:<model revision>:<action ids
///   joined by '|'>` namespaced by the project id — `action_ids` are ALL
///   proposal action ids (including reviewed-out ones), matching the CLI;
/// - `batch_id` is the v5 id over the raw `proposal_id` bytes.
///
/// The batch is intentionally unguarded (its authority is the
/// `prepared_against` model-revision guard) and the proposal is born
/// `Accepted` with [`ProposalSource::Cli`]; committing its metadata and
/// applying it are the substrate-owned
/// `commit_proposal_metadata_journaled` + `apply_accepted_proposal` steps.
///
/// Returns `None` when no action survived review — no proposal exists then.
pub fn build_forward_annotation_accepted_proposal(
    model: &DesignModel,
    provenance: WriteProvenance,
    rationale: String,
    action_ids: &[&str],
    actions: Vec<ForwardAnnotationProposalAction>,
) -> Result<Option<Proposal>, EngineError> {
    if actions.is_empty() {
        return Ok(None);
    }
    let mut operations = Vec::with_capacity(actions.len());
    let mut affected_objects = Vec::with_capacity(actions.len());
    for action in actions {
        // Authored directly (not through the guarded family-D single-edit
        // builders): the historical CLI authored this batch without any
        // object-revision guard or model lookup — its only authority is the
        // `prepared_against` model-revision guard — and that contract must
        // not drift.
        let (package_id, operation) = match action {
            ForwardAnnotationProposalAction::SetComponentValue { package_id, value } => (
                package_id,
                Operation::SetBoardPackageValue { package_id, value },
            ),
            ForwardAnnotationProposalAction::RemoveComponent {
                package_id,
                package,
                materialized,
            } => (
                package_id,
                Operation::DeleteBoardPackage {
                    package_id,
                    package,
                    materialized,
                },
            ),
        };
        operations.push(operation);
        affected_objects.push(package_id);
    }

    let prepared_against = model.model_revision.clone();
    let proposal_id = ids::derive_object_id(
        &model.project.project_id,
        "forward-annotation-proposal",
        &[prepared_against.0.clone(), action_ids.join("|")],
    );
    Ok(Some(Proposal {
        schema_version: 1,
        proposal_id,
        project_id: model.project.project_id,
        prepared_against: prepared_against.clone(),
        batch: OperationBatch {
            batch_id: Uuid::new_v5(&model.project.project_id, proposal_id.as_bytes()),
            expected_model_revision: Some(prepared_against),
            provenance: provenance.into(),
            operations,
        },
        rationale,
        affected_objects,
        checks_run: Vec::new(),
        finding_fingerprints: Vec::new(),
        source: ProposalSource::Cli,
        status: ProposalStatus::Accepted,
        applied_transaction_id: None,
    }))
}

/// A renderable geometry delta extracted from a proposal batch (created/set
/// tracks and vias), for GUI/CLI proposal previews.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalRenderDelta {
    pub delta_kind: &'static str,
    pub object_id: String,
    pub primitive_kind: &'static str,
    pub layer_id: String,
    pub end_layer_id: Option<String>,
    pub width_nm: i64,
    pub drill_nm: Option<i64>,
    pub diameter_nm: Option<i64>,
    pub path: Vec<ProposalRenderPoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProposalRenderPoint {
    pub x: i64,
    pub y: i64,
}

/// Extract the renderable track/via deltas from a proposal's operation batch.
/// Read-only companion to the proposal builders: it interprets the same
/// operation payloads the builders author, so the mapping lives beside them.
pub fn proposal_render_deltas(
    batch: &OperationBatch,
) -> Result<Vec<ProposalRenderDelta>, EngineError> {
    batch
        .operations
        .iter()
        .filter_map(|operation| match operation {
            Operation::CreateBoardTrack { track_id, track } => {
                Some(track_render_delta("create", track_id.to_string(), track))
            }
            Operation::SetBoardTrack { track_id, track } => {
                Some(track_render_delta("set", track_id.to_string(), track))
            }
            Operation::CreateBoardVia { via_id, via } => {
                Some(via_render_delta("create", via_id.to_string(), via))
            }
            Operation::SetBoardVia { via_id, via } => {
                Some(via_render_delta("set", via_id.to_string(), via))
            }
            _ => None,
        })
        .collect()
}

fn track_render_delta(
    delta_kind: &'static str,
    object_id: String,
    track: &serde_json::Value,
) -> Result<ProposalRenderDelta, EngineError> {
    let track: Track = serde_json::from_value(track.clone())?;
    Ok(ProposalRenderDelta {
        delta_kind,
        object_id,
        primitive_kind: "track_path",
        layer_id: format!("L{}", track.layer),
        end_layer_id: None,
        width_nm: track.width,
        drill_nm: None,
        diameter_nm: None,
        path: vec![
            ProposalRenderPoint {
                x: track.from.x,
                y: track.from.y,
            },
            ProposalRenderPoint {
                x: track.to.x,
                y: track.to.y,
            },
        ],
    })
}

fn via_render_delta(
    delta_kind: &'static str,
    object_id: String,
    via: &serde_json::Value,
) -> Result<ProposalRenderDelta, EngineError> {
    let via: Via = serde_json::from_value(via.clone())?;
    Ok(ProposalRenderDelta {
        delta_kind,
        object_id,
        primitive_kind: "via",
        layer_id: format!("L{}", via.from_layer),
        end_layer_id: Some(format!("L{}", via.to_layer)),
        width_nm: via.diameter,
        drill_nm: Some(via.drill),
        diameter_nm: Some(via.diameter),
        path: vec![ProposalRenderPoint {
            x: via.position.x,
            y: via.position.y,
        }],
    })
}

/// Strip the object-revision guards a single-edit family-D builder inserted,
/// leaving only the authored mutation operations. Used when family-D single
/// writes are composed into one multi-operation batch that is re-guarded (or
/// intentionally left unguarded) as a whole.
fn unguarded_operations(prepared: PreparedWrite) -> impl Iterator<Item = Operation> {
    prepared
        .batch
        .operations
        .into_iter()
        .filter(|operation| !matches!(operation, Operation::GuardObjectRevision { .. }))
}

#[cfg(test)]
mod tests {
    use super::super::context::commit_prepared;
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::board::PlacedPackage;
    use crate::ir::geometry::Point;
    use crate::substrate::{
        CommitProvenance, CommitSource, ObjectRevision, ProjectResolver, ProposalCreateRequest,
        apply_accepted_proposal, create_draft_proposal_from_batch, review_proposal_status,
    };

    fn test_provenance() -> WriteProvenance {
        WriteProvenance::new(
            "unit-test",
            CommitSource::Test,
            "forward annotation facade test",
        )
    }

    fn replacement_edits(package_id: Uuid, part_id: Uuid) -> Vec<(ObjectId, BoardPackageEdit)> {
        vec![
            (package_id, BoardPackageEdit::Part { part_id }),
            (
                package_id,
                BoardPackageEdit::Value {
                    value: "NEW".to_string(),
                },
            ),
        ]
    }

    /// The exact pre-migration CLI sequence: author raw operations, then
    /// guard the whole batch — reproduced verbatim as the parity oracle.
    fn cli_sequence_batch(
        model: &DesignModel,
        operations: Vec<Operation>,
    ) -> Result<OperationBatch, EngineError> {
        guarded_operation_batch(
            model,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "forward annotation facade test".to_string(),
                },
                operations,
            },
        )
    }

    #[test]
    fn guard_proposal_batch_preserves_external_stamps_and_inserts_guards() {
        let (_root, model, _board_id, package_id) =
            resolved_model_with_board_package("fa_guard_external");
        let external_batch_id = Uuid::new_v5(&Uuid::nil(), b"external-batch");
        let external = OperationBatch {
            batch_id: external_batch_id,
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "external".to_string(),
                source: CommitSource::Assistant,
                reason: "external proposal batch".to_string(),
            },
            operations: vec![Operation::SetBoardPackageValue {
                package_id,
                value: "NEW".to_string(),
            }],
        };

        let guarded = guard_proposal_batch(&model, external).expect("guarding should succeed");

        assert_eq!(guarded.batch_id, external_batch_id);
        assert_eq!(guarded.provenance.actor, "external");
        assert_eq!(guarded.provenance.reason, "external proposal batch");
        assert_eq!(
            guarded.expected_model_revision,
            Some(model.model_revision.clone())
        );
        assert_eq!(
            guarded.operations,
            vec![
                Operation::GuardObjectRevision {
                    object_id: package_id,
                    expected_object_revision: ObjectRevision(0),
                },
                Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                },
            ]
        );
    }

    #[test]
    fn replacement_batch_matches_historical_cli_guard_ordering() {
        let (root, mut model, _board_id, first_package) =
            resolved_model_with_board_package("fa_replacement_ordering");
        // Second placed package so the parity check covers interleaved
        // per-object guard ordering across components.
        let second = PlacedPackage {
            uuid: Uuid::new_v4(),
            part: Uuid::new_v4(),
            package: Uuid::new_v4(),
            reference: "U2".to_string(),
            value: "OLD2".to_string(),
            position: Point { x: 5, y: 5 },
            rotation: 0,
            layer: 1,
            locked: false,
        };
        let prepared = super::super::board_components::build_place_board_package(
            &model,
            test_provenance(),
            &super::super::board_components::BoardPackagePlacement {
                package: second.clone(),
                materialized: serde_json::json!({}),
            },
        )
        .expect("second package should build");
        commit_prepared(&mut model, &root, prepared).expect("second package should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("fixture project should re-resolve");

        let part_id = Uuid::new_v4();
        let mut edits = replacement_edits(first_package, part_id);
        edits.push((
            second.uuid,
            BoardPackageEdit::Value {
                value: "NEW2".to_string(),
            },
        ));

        let facade =
            build_board_component_replacement_proposal_batch(&model, test_provenance(), edits)
                .expect("facade batch should build");
        let oracle = cli_sequence_batch(
            &model,
            vec![
                Operation::SetBoardPackagePart {
                    package_id: first_package,
                    part_id,
                },
                Operation::SetBoardPackageValue {
                    package_id: first_package,
                    value: "NEW".to_string(),
                },
                Operation::SetBoardPackageValue {
                    package_id: second.uuid,
                    value: "NEW2".to_string(),
                },
            ],
        )
        .expect("oracle batch should build");

        assert_eq!(facade.operations, oracle.operations);
        assert_eq!(
            facade.expected_model_revision,
            oracle.expected_model_revision
        );
        // Guard precedes the first mutation of each object, once per object.
        assert!(matches!(
            facade.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == first_package
        ));
        assert!(matches!(
            facade.operations[3],
            Operation::GuardObjectRevision { object_id, .. } if object_id == second.uuid
        ));
        assert_eq!(facade.operations.len(), 5);
    }

    #[test]
    fn replacement_batch_predicts_and_survives_journaled_proposal_apply() {
        let (root, mut model, _board_id, package_id) =
            resolved_model_with_board_package("fa_replacement_apply");
        let batch = build_board_component_replacement_proposal_batch(
            &model,
            test_provenance(),
            replacement_edits(package_id, Uuid::new_v4()),
        )
        .expect("facade batch should build");

        let proposal = create_draft_proposal_from_batch(
            &mut model,
            &root,
            ProposalCreateRequest {
                proposal_id: None,
                batch,
                rationale: "facade parity".to_string(),
                source: ProposalSource::Cli,
                checks_run: Vec::new(),
                finding_fingerprints: Vec::new(),
            },
        )
        .expect("draft proposal should create");
        review_proposal_status(
            &mut model,
            &root,
            proposal.proposal_id,
            ProposalStatus::Accepted,
        )
        .expect("proposal should accept");
        // Transaction-id prediction parity: the substrate-owned apply first
        // predicts the transaction id from the facade batch, then hard-fails
        // if the journaled commit produces a different id — so a successful
        // apply proves the facade batch (guard ordering included) predicts
        // exactly what it commits.
        let report = apply_accepted_proposal(&mut model, &root, proposal.proposal_id)
            .expect("facade-built proposal should apply through the journaled path");
        assert_eq!(
            model.proposals[&proposal.proposal_id].applied_transaction_id,
            Some(report.transaction.transaction_id),
            "applied proposal metadata must record the predicted transaction id"
        );
        assert_eq!(
            model.objects[&package_id].object_revision,
            ObjectRevision(2),
            "both replacement edits must have landed on the package"
        );
    }

    #[test]
    fn review_state_builders_author_the_historical_unguarded_batches() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("fa_review_state");
        let previous = serde_json::json!({ "schema_version": 1, "reviews": {} });
        let review = serde_json::json!({
            "schema_version": 1,
            "reviews": { "action-1": { "decision": "accepted" } }
        });

        let set = build_set_forward_annotation_review(
            &model,
            test_provenance(),
            Some(previous.clone()),
            review.clone(),
        )
        .expect("set builder should build");
        assert_eq!(set.primary_object_id, None);
        assert_eq!(
            set.batch.expected_model_revision,
            Some(model.model_revision.clone())
        );
        assert_eq!(
            set.batch.operations,
            vec![Operation::SetForwardAnnotationReview {
                relative_path: FORWARD_ANNOTATION_REVIEW_PATH.to_string(),
                previous_review: Some(previous),
                review: review.clone(),
            }]
        );

        let clear =
            build_clear_forward_annotation_review(&model, test_provenance(), review.clone())
                .expect("clear builder should build");
        assert_eq!(
            clear.batch.operations,
            vec![Operation::DeleteForwardAnnotationReview {
                relative_path: FORWARD_ANNOTATION_REVIEW_PATH.to_string(),
                review,
            }]
        );
    }

    #[test]
    fn review_state_set_commits_and_resolves_through_the_journal() {
        let (root, mut model, _board_id, _package_id) =
            resolved_model_with_board_package("fa_review_commit");
        let review = serde_json::json!({
            "schema_version": 1,
            "reviews": { "action-1": { "decision": "accepted" } }
        });
        let prepared =
            build_set_forward_annotation_review(&model, test_provenance(), None, review.clone())
                .expect("set builder should build");
        commit_prepared(&mut model, &root, prepared).expect("review write should commit");

        let resolved = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");
        assert_eq!(
            resolved
                .materialized_source_shard_value(
                    crate::substrate::SourceShardKind::ForwardAnnotationReview
                )
                .expect("review sidecar should materialize"),
            review
        );
    }

    #[test]
    fn accepted_proposal_matches_historical_deterministic_identity_and_payloads() {
        let (_root, model, _board_id, package_id) =
            resolved_model_with_board_package("fa_accepted_proposal");
        let removed_package_id = Uuid::new_v4();
        let stored = serde_json::json!({ "uuid": removed_package_id, "reference": "U9" });
        let materialized = serde_json::json!({ "component_pads": [] });
        let action_ids = ["action-1", "action-2", "action-3"];

        let proposal = build_forward_annotation_accepted_proposal(
            &model,
            WriteProvenance::new(
                "datum-eda-cli",
                CommitSource::Cli,
                "forward annotation accepted proposal",
            ),
            "forward annotation self-sufficient board updates".to_string(),
            &action_ids,
            vec![
                ForwardAnnotationProposalAction::SetComponentValue {
                    package_id,
                    value: "NEW".to_string(),
                },
                ForwardAnnotationProposalAction::RemoveComponent {
                    package_id: removed_package_id,
                    package: stored.clone(),
                    materialized: materialized.clone(),
                },
            ],
        )
        .expect("proposal should build")
        .expect("proposal should exist");

        // Byte-exact historical CLI derivation
        // (command_project_forward_annotation_substrate.rs, pre-migration).
        let expected_proposal_id = Uuid::new_v5(
            &model.project.project_id,
            format!(
                "datum-eda:forward-annotation-proposal:{}:{}",
                model.model_revision.0, "action-1|action-2|action-3"
            )
            .as_bytes(),
        );
        assert_eq!(proposal.proposal_id, expected_proposal_id);
        assert_eq!(
            proposal.batch.batch_id,
            Uuid::new_v5(&model.project.project_id, expected_proposal_id.as_bytes())
        );
        assert_eq!(proposal.schema_version, 1);
        assert_eq!(proposal.project_id, model.project.project_id);
        assert_eq!(proposal.prepared_against, model.model_revision);
        assert_eq!(
            proposal.batch.expected_model_revision,
            Some(model.model_revision.clone())
        );
        assert_eq!(proposal.batch.provenance.actor, "datum-eda-cli");
        assert_eq!(
            proposal.batch.provenance.reason,
            "forward annotation accepted proposal"
        );
        // The historical batch is intentionally unguarded.
        assert_eq!(
            proposal.batch.operations,
            vec![
                Operation::SetBoardPackageValue {
                    package_id,
                    value: "NEW".to_string(),
                },
                Operation::DeleteBoardPackage {
                    package_id: removed_package_id,
                    package: stored,
                    materialized,
                },
            ]
        );
        assert_eq!(
            proposal.affected_objects,
            vec![package_id, removed_package_id]
        );
        assert_eq!(proposal.source, ProposalSource::Cli);
        assert_eq!(proposal.status, ProposalStatus::Accepted);
        assert_eq!(proposal.applied_transaction_id, None);
    }

    #[test]
    fn accepted_proposal_without_surviving_actions_is_none() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("fa_accepted_empty");
        let built = build_forward_annotation_accepted_proposal(
            &model,
            test_provenance(),
            "unused".to_string(),
            &["action-1"],
            Vec::new(),
        )
        .expect("empty build should succeed");
        assert_eq!(built, None);
    }

    #[test]
    fn render_deltas_extract_created_and_set_tracks_and_vias() {
        let track_id = Uuid::new_v4();
        let via_id = Uuid::new_v4();
        let track = serde_json::json!({
            "uuid": track_id,
            "net": Uuid::new_v4(),
            "from": { "x": 1, "y": 2 },
            "to": { "x": 3, "y": 4 },
            "width": 200_000,
            "layer": 1
        });
        let via = serde_json::json!({
            "uuid": via_id,
            "net": Uuid::new_v4(),
            "position": { "x": 7, "y": 8 },
            "diameter": 600_000,
            "drill": 300_000,
            "from_layer": 1,
            "to_layer": 31
        });
        let batch = OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: None,
            provenance: CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "render delta test".to_string(),
            },
            operations: vec![
                Operation::CreateBoardTrack {
                    track_id,
                    track: track.clone(),
                },
                Operation::SetBoardVia {
                    via_id,
                    via: via.clone(),
                },
                Operation::SetBoardPackageValue {
                    package_id: Uuid::new_v4(),
                    value: "ignored".to_string(),
                },
            ],
        };

        let deltas = proposal_render_deltas(&batch).expect("deltas should extract");

        assert_eq!(
            deltas,
            vec![
                ProposalRenderDelta {
                    delta_kind: "create",
                    object_id: track_id.to_string(),
                    primitive_kind: "track_path",
                    layer_id: "L1".to_string(),
                    end_layer_id: None,
                    width_nm: 200_000,
                    drill_nm: None,
                    diameter_nm: None,
                    path: vec![
                        ProposalRenderPoint { x: 1, y: 2 },
                        ProposalRenderPoint { x: 3, y: 4 },
                    ],
                },
                ProposalRenderDelta {
                    delta_kind: "set",
                    object_id: via_id.to_string(),
                    primitive_kind: "via",
                    layer_id: "L1".to_string(),
                    end_layer_id: Some("L31".to_string()),
                    width_nm: 600_000,
                    drill_nm: Some(300_000),
                    diameter_nm: Some(600_000),
                    path: vec![ProposalRenderPoint { x: 7, y: 8 }],
                },
            ]
        );
    }
}
