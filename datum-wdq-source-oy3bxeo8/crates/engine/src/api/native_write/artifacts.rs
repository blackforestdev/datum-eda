//! Artifact-evidence and check-run write builders.
//!
//! Migrated from `crates/cli/src/command_project_artifact_evidence.rs`,
//! `crates/cli/src/command_project_gerber_evidence.rs`,
//! `crates/cli/src/command_project_manufacturing_evidence.rs`, and the
//! `SetCheckRun` evidence commit in
//! `crates/cli/src/command_project_native_inspect.rs`. Those three evidence
//! files were near-identical batch composers; they share the single
//! [`build_artifact_evidence`] builder here, parameterized on which run
//! evidence accompanies the artifact metadata. Builders are build-only
//! ([`PreparedWrite`] out, never committed here).

use std::collections::BTreeSet;

use uuid::Uuid;

use crate::error::EngineError;
use crate::substrate::{
    ArtifactMetadata, ArtifactRun, CheckRun, DesignModel, Operation, OutputJobRun,
};

use super::context::{BatchComposer, PreparedWrite, WriteProvenance};

/// Build the evidence write for a generated artifact: a `SetArtifactMetadata`
/// operation, optionally followed by `SetArtifactRun` (unlinked generation
/// evidence) and/or `SetOutputJobRun` (output-job-linked evidence).
///
/// Previous payloads are looked up from the resolved model, matching the
/// pre-migration CLI behavior; evidence records are not guard targets, so the
/// batch carries exactly the pushed operations.
pub fn build_artifact_evidence(
    model: &DesignModel,
    provenance: WriteProvenance,
    artifact_metadata: &ArtifactMetadata,
    artifact_run: Option<&ArtifactRun>,
    output_job_run: Option<&OutputJobRun>,
) -> Result<PreparedWrite, EngineError> {
    let previous_artifact_metadata = model
        .artifact_metadata
        .get(&artifact_metadata.artifact_id)
        .map(serde_json::to_value)
        .transpose()?;
    let mut composer = BatchComposer::compose(model, provenance)
        .push_op(Operation::SetArtifactMetadata {
            artifact_id: artifact_metadata.artifact_id,
            previous_artifact_metadata,
            artifact_metadata: serde_json::to_value(artifact_metadata)?,
        })
        .primary_object(artifact_metadata.artifact_id);
    if let Some(run) = artifact_run {
        let previous_artifact_run = model
            .artifact_runs
            .get(&run.run_id)
            .map(serde_json::to_value)
            .transpose()?;
        composer = composer.push_op(Operation::SetArtifactRun {
            run_id: run.run_id,
            previous_artifact_run,
            artifact_run: serde_json::to_value(run)?,
        });
    }
    if let Some(run) = output_job_run {
        let previous_output_job_run = model
            .output_job_runs
            .get(&run.run_id)
            .map(serde_json::to_value)
            .transpose()?;
        composer = composer.push_op(Operation::SetOutputJobRun {
            run_id: run.run_id,
            previous_output_job_run,
            output_job_run: serde_json::to_value(run)?,
        });
    }
    composer.finish()
}

/// Build a `SetCheckRun` evidence write for a persisted check run. The
/// previous run payload is looked up from the resolved model.
pub fn build_set_check_run(
    model: &DesignModel,
    provenance: WriteProvenance,
    check_run: &CheckRun,
) -> Result<PreparedWrite, EngineError> {
    let previous_check_run = model
        .check_runs
        .get(&check_run.check_run_id)
        .map(serde_json::to_value)
        .transpose()?;
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetCheckRun {
            check_run_id: check_run.check_run_id,
            previous_check_run,
            check_run: serde_json::to_value(check_run)?,
        })
        .primary_object(check_run.check_run_id)
        .finish()
}

/// Journal-evidence query companion to the artifact builders: the artifact id
/// of the most recently journaled `SetArtifactMetadata` whose id is in
/// `existing` (later transactions win, then later operations, then the larger
/// artifact id). Returns `None` when no journaled write matches.
pub fn latest_journaled_artifact_id(
    model: &DesignModel,
    existing: &BTreeSet<Uuid>,
) -> Option<Uuid> {
    model
        .journal
        .iter()
        .enumerate()
        .flat_map(|(transaction_index, transaction)| {
            transaction.operations.iter().enumerate().filter_map(
                move |(operation_index, operation)| match operation {
                    Operation::SetArtifactMetadata { artifact_id, .. }
                        if existing.contains(artifact_id) =>
                    {
                        Some((transaction_index, operation_index, *artifact_id))
                    }
                    _ => None,
                },
            )
        })
        .max_by_key(|(transaction_index, operation_index, artifact_id)| {
            (*transaction_index, *operation_index, *artifact_id)
        })
        .map(|(_, _, artifact_id)| artifact_id)
}

#[cfg(test)]
mod tests {
    use super::super::context::commit_prepared;
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::substrate::{
        ARTIFACT_RUN_SCHEMA_VERSION, ArtifactKind, ArtifactValidationState, CommitSource,
        OUTPUT_JOB_RUN_SCHEMA_VERSION, OutputJobLogEntry, OutputJobLogLevel, OutputJobRunStatus,
    };

    fn test_provenance(reason: &str) -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, reason)
    }

    fn test_metadata(model: &DesignModel) -> ArtifactMetadata {
        ArtifactMetadata {
            schema_version: 1,
            artifact_id: Uuid::new_v5(&model.project.project_id, b"test-artifact"),
            kind: ArtifactKind::GerberSet,
            project_id: model.project.project_id,
            model_revision: model.model_revision.clone(),
            output_job: None,
            variant: None,
            generator_version: "test".to_string(),
            output_dir: None,
            files: Vec::new(),
            production_projections: Vec::new(),
            validation_state: ArtifactValidationState::NotValidated,
        }
    }

    fn test_artifact_run(model: &DesignModel, artifact_id: Uuid) -> ArtifactRun {
        ArtifactRun {
            schema_version: ARTIFACT_RUN_SCHEMA_VERSION,
            run_id: Uuid::new_v5(&model.project.project_id, b"test-artifact-run"),
            artifact_id,
            run_sequence: 1,
            project_id: model.project.project_id,
            model_revision: model.model_revision.clone(),
            status: OutputJobRunStatus::Succeeded,
            exit_code: Some(0),
            provenance: None,
            log: vec![OutputJobLogEntry {
                sequence: 1,
                level: OutputJobLogLevel::Info,
                message: "generated artifact".to_string(),
            }],
        }
    }

    fn test_output_job_run(model: &DesignModel) -> OutputJobRun {
        OutputJobRun {
            schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
            run_id: Uuid::new_v5(&model.project.project_id, b"test-output-job-run"),
            output_job: Uuid::new_v5(&model.project.project_id, b"test-output-job"),
            run_sequence: 1,
            project_id: model.project.project_id,
            model_revision: model.model_revision.clone(),
            status: OutputJobRunStatus::Succeeded,
            artifact_id: None,
            exit_code: Some(0),
            provenance: None,
            log: vec![OutputJobLogEntry {
                sequence: 1,
                level: OutputJobLogLevel::Info,
                message: "generated output job".to_string(),
            }],
        }
    }

    #[test]
    fn metadata_only_evidence_builds_single_unguarded_operation() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("artifact_metadata_only");
        let metadata = test_metadata(&model);

        let prepared = build_artifact_evidence(
            &model,
            test_provenance("record generated artifact metadata"),
            &metadata,
            None,
            None,
        )
        .expect("metadata evidence should build");

        assert_eq!(prepared.primary_object_id, Some(metadata.artifact_id));
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::SetArtifactMetadata {
                artifact_id: metadata.artifact_id,
                previous_artifact_metadata: None,
                artifact_metadata: serde_json::to_value(&metadata).unwrap(),
            }]
        );
    }

    #[test]
    fn unlinked_and_linked_evidence_order_metadata_then_runs() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("artifact_evidence_order");
        let metadata = test_metadata(&model);
        let artifact_run = test_artifact_run(&model, metadata.artifact_id);
        let output_job_run = test_output_job_run(&model);

        let unlinked = build_artifact_evidence(
            &model,
            test_provenance("generate unlinked gerber-set artifact evidence"),
            &metadata,
            Some(&artifact_run),
            None,
        )
        .expect("unlinked evidence should build");
        assert_eq!(unlinked.batch.operations.len(), 2);
        assert!(matches!(
            unlinked.batch.operations[0],
            Operation::SetArtifactMetadata { .. }
        ));
        assert!(matches!(
            &unlinked.batch.operations[1],
            Operation::SetArtifactRun { run_id, previous_artifact_run: None, .. }
                if *run_id == artifact_run.run_id
        ));

        let linked = build_artifact_evidence(
            &model,
            test_provenance("record linked artifact output-job evidence"),
            &metadata,
            None,
            Some(&output_job_run),
        )
        .expect("linked evidence should build");
        assert_eq!(linked.batch.operations.len(), 2);
        assert!(matches!(
            &linked.batch.operations[1],
            Operation::SetOutputJobRun { run_id, previous_output_job_run: None, .. }
                if *run_id == output_job_run.run_id
        ));
    }

    #[test]
    fn evidence_commit_threads_previous_metadata_on_rewrite() {
        let (root, mut model, _board_id, _package_id) =
            resolved_model_with_board_package("artifact_evidence_rewrite");
        let metadata = test_metadata(&model);

        let prepared = build_artifact_evidence(
            &model,
            test_provenance("record generated artifact metadata"),
            &metadata,
            None,
            None,
        )
        .expect("first evidence should build");
        commit_prepared(&mut model, &root, prepared).expect("first evidence should commit");
        assert!(model.artifact_metadata.contains_key(&metadata.artifact_id));

        let mut rewritten = model.artifact_metadata[&metadata.artifact_id].clone();
        rewritten.generator_version = "test-2".to_string();
        rewritten.model_revision = model.model_revision.clone();
        let prepared = build_artifact_evidence(
            &model,
            test_provenance("record generated artifact metadata"),
            &rewritten,
            None,
            None,
        )
        .expect("rewrite evidence should build");
        assert!(matches!(
            &prepared.batch.operations[0],
            Operation::SetArtifactMetadata {
                previous_artifact_metadata: Some(_),
                ..
            }
        ));
    }

    #[test]
    fn latest_journaled_artifact_id_prefers_later_journal_writes() {
        let (root, mut model, _board_id, _package_id) =
            resolved_model_with_board_package("artifact_latest_journal");
        let mut first = test_metadata(&model);
        first.artifact_id = Uuid::new_v5(&model.project.project_id, b"artifact-first");
        let mut second = test_metadata(&model);
        second.artifact_id = Uuid::new_v5(&model.project.project_id, b"artifact-second");

        for metadata in [&first, &second] {
            let prepared = build_artifact_evidence(
                &model,
                test_provenance("record generated artifact metadata"),
                metadata,
                None,
                None,
            )
            .expect("evidence should build");
            commit_prepared(&mut model, &root, prepared).expect("evidence should commit");
        }

        let existing = [first.artifact_id, second.artifact_id]
            .into_iter()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            latest_journaled_artifact_id(&model, &existing),
            Some(second.artifact_id),
        );
        assert_eq!(latest_journaled_artifact_id(&model, &BTreeSet::new()), None,);
    }

    #[test]
    fn set_check_run_builds_single_unguarded_operation() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("check_run_build");
        let check_run = CheckRun {
            schema_version: 1,
            check_run_id: Uuid::new_v5(&model.project.project_id, b"test-check-run"),
            project_id: model.project.project_id,
            model_revision: model.model_revision.clone(),
            profile_id: "release".to_string(),
            status: "pass".to_string(),
            summary: serde_json::json!({}),
            finding_count: 0,
            findings: Vec::new(),
            proposal_refs: Vec::new(),
            proposal_links: Vec::new(),
            profile_basis: Default::default(),
            coverage: Vec::new(),
            raw_report: serde_json::json!({}),
        };

        let prepared = build_set_check_run(
            &model,
            test_provenance("record release check run evidence"),
            &check_run,
        )
        .expect("check run evidence should build");

        assert_eq!(prepared.primary_object_id, Some(check_run.check_run_id));
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::SetCheckRun {
                check_run_id: check_run.check_run_id,
                previous_check_run: None,
                check_run: serde_json::to_value(&check_run).unwrap(),
            }]
        );
    }
}
