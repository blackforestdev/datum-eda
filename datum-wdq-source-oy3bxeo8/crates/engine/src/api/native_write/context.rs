//! Shared core of the native write facade: provenance, batch building, and
//! the build/commit split.
//!
//! Builders across the `native_write` families are build-only — they return a
//! [`PreparedWrite`] and never commit. Committing is a separate, explicit
//! step ([`commit_prepared`]) so proposal and dry-run flows can share the
//! same builders without forking the authoring logic.

use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EngineError;
use crate::substrate::{
    CommitProvenance, CommitReport, CommitSource, DesignModel, ObjectId, Operation, OperationBatch,
};

use super::guards::guarded_operation_batch;

/// Who is writing, through which surface, and why.
///
/// Thin authoring-facade mirror of the substrate's [`CommitProvenance`]
/// (actor + [`CommitSource`] + reason); converts losslessly via `From`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteProvenance {
    /// Acting identity, e.g. `datum-eda-cli` or a user/agent name.
    pub actor: String,
    /// Which surface authored the write.
    pub source: CommitSource,
    /// Human-readable intent recorded in the journal.
    pub reason: String,
}

impl WriteProvenance {
    pub fn new(actor: impl Into<String>, source: CommitSource, reason: impl Into<String>) -> Self {
        Self {
            actor: actor.into(),
            source,
            reason: reason.into(),
        }
    }
}

impl From<WriteProvenance> for CommitProvenance {
    fn from(provenance: WriteProvenance) -> Self {
        Self {
            actor: provenance.actor,
            source: provenance.source,
            reason: provenance.reason,
        }
    }
}

/// A fully built, guard-stamped operation batch that has not been committed.
///
/// `primary_object_id` names the object the write is "about" (the created or
/// principally mutated object) so callers can report on it without re-parsing
/// the batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedWrite {
    pub batch: OperationBatch,
    pub primary_object_id: Option<ObjectId>,
}

/// Build a guarded, revision-stamped operation batch against `model`.
///
/// Mirrors the CLI's authoring path exactly: a fresh `batch_id`,
/// `expected_model_revision` stamped from the resolved model, provenance
/// threaded onto the batch, and object-revision guards inserted immediately
/// before the first mutation of each existing object
/// (see [`super::guards::guarded_operation_batch`]).
pub fn build_batch(
    model: &DesignModel,
    provenance: WriteProvenance,
    operations: Vec<Operation>,
) -> Result<OperationBatch, EngineError> {
    guarded_operation_batch(
        model,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: provenance.into(),
            operations,
        },
    )
}

/// Commit a [`PreparedWrite`] through the single journaled commit path.
///
/// Thin wrapper over [`DesignModel::commit_journaled`]; every native write
/// surface lands here — there is no other write path.
pub fn commit_prepared(
    model: &mut DesignModel,
    project_root: &Path,
    prepared: PreparedWrite,
) -> Result<CommitReport, EngineError> {
    model.commit_journaled(project_root, prepared.batch)
}

/// Composer for multi-operation atomic writes.
///
/// `BatchComposer::compose(model, provenance)` → `push_ops(..)` (repeatable)
/// → `finish()` yields a [`PreparedWrite`] whose guards and
/// `expected_model_revision` are stamped identically to [`build_batch`].
pub struct BatchComposer<'a> {
    model: &'a DesignModel,
    provenance: WriteProvenance,
    operations: Vec<Operation>,
    primary_object_id: Option<ObjectId>,
}

impl<'a> BatchComposer<'a> {
    pub fn compose(model: &'a DesignModel, provenance: WriteProvenance) -> Self {
        Self {
            model,
            provenance,
            operations: Vec::new(),
            primary_object_id: None,
        }
    }

    /// Append a single operation.
    pub fn push_op(mut self, operation: Operation) -> Self {
        self.operations.push(operation);
        self
    }

    /// Append a sequence of operations.
    pub fn push_ops(mut self, operations: impl IntoIterator<Item = Operation>) -> Self {
        self.operations.extend(operations);
        self
    }

    /// Name the object this write is primarily about.
    pub fn primary_object(mut self, object_id: ObjectId) -> Self {
        self.primary_object_id = Some(object_id);
        self
    }

    /// Build the guarded batch; does not commit.
    pub fn finish(self) -> Result<PreparedWrite, EngineError> {
        let batch = build_batch(self.model, self.provenance, self.operations)?;
        Ok(PreparedWrite {
            batch,
            primary_object_id: self.primary_object_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::substrate::ObjectRevision;

    fn test_provenance() -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, "native write facade test")
    }

    #[test]
    fn build_batch_stamps_revision_provenance_and_guards() {
        let (_root, model, _board_id, package_id) =
            resolved_model_with_board_package("context_build");

        let batch = build_batch(
            &model,
            test_provenance(),
            vec![Operation::SetBoardPackageValue {
                package_id,
                value: "NEW".to_string(),
            }],
        )
        .expect("build_batch should succeed");

        assert_eq!(
            batch.expected_model_revision,
            Some(model.model_revision.clone())
        );
        assert_eq!(batch.provenance.actor, "unit-test");
        assert_eq!(batch.provenance.source, CommitSource::Test);
        assert_eq!(batch.provenance.reason, "native write facade test");
        assert_eq!(
            batch.operations,
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
    fn build_batch_matches_cli_guard_helper_output() {
        // The engine facade and the (now shimmed) CLI helper must produce the
        // same guarded operation sequence for the same input.
        let (_root, model, _board_id, package_id) =
            resolved_model_with_board_package("context_parity");
        let operations = vec![
            Operation::SetBoardPackageValue {
                package_id,
                value: "NEW".to_string(),
            },
            Operation::SetBoardPackageReference {
                package_id,
                reference: "U7".to_string(),
            },
        ];

        let built = build_batch(&model, test_provenance(), operations.clone())
            .expect("build_batch should succeed");
        let guarded_directly = super::super::guards::guarded_operation_batch(
            &model,
            OperationBatch {
                batch_id: built.batch_id,
                expected_model_revision: built.expected_model_revision.clone(),
                provenance: built.provenance.clone(),
                operations,
            },
        )
        .expect("direct guard helper should succeed");

        assert_eq!(built.operations, guarded_directly.operations);
    }

    #[test]
    fn commit_prepared_commits_through_journaled_path() {
        let (root, mut model, _board_id, package_id) =
            resolved_model_with_board_package("context_commit");
        let before = model.model_revision.clone();

        let prepared = BatchComposer::compose(&model, test_provenance())
            .push_ops(vec![Operation::SetBoardPackageValue {
                package_id,
                value: "NEW".to_string(),
            }])
            .primary_object(package_id)
            .finish()
            .expect("composer should finish");
        assert_eq!(prepared.primary_object_id, Some(package_id));

        let report =
            commit_prepared(&mut model, &root, prepared).expect("commit_prepared should succeed");

        assert_ne!(model.model_revision, before);
        assert_eq!(report.transaction.before_model_revision, before);
        assert_eq!(
            report.transaction.after_model_revision,
            model.model_revision
        );
        assert_eq!(
            model.objects[&package_id].object_revision,
            ObjectRevision(1)
        );
        assert!(
            crate::substrate::transaction_journal_path(&root).exists(),
            "journaled commit should append the transaction journal"
        );
    }

    #[test]
    fn commit_prepared_rejects_stale_prepared_write() {
        let (root, mut model, board_id, package_id) =
            resolved_model_with_board_package("context_stale");

        // Prepare against the current revision, then move the model forward.
        let stale = BatchComposer::compose(&model, test_provenance())
            .push_op(Operation::SetBoardPackageValue {
                package_id,
                value: "STALE".to_string(),
            })
            .finish()
            .expect("composer should finish");
        let advance = build_batch(
            &model,
            test_provenance(),
            vec![Operation::BumpObjectRevision {
                object_id: board_id,
            }],
        )
        .expect("advance batch should build");
        model
            .commit_journaled(&root, advance)
            .expect("advance commit should succeed");

        let error = commit_prepared(&mut model, &root, stale)
            .expect_err("stale prepared write should be rejected");
        assert!(error.to_string().contains("model revision mismatch"));
    }
}
