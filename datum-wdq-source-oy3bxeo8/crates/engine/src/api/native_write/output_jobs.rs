//! Output-job and output-job-run write builders.
//!
//! Migrated from `crates/cli/src/command_project_output_jobs.rs`,
//! `crates/cli/src/command_project_output_job_runs.rs`, and the builder halves
//! of `crates/cli/src/command_project_output_job_proposals.rs`. Builders are
//! build-only ([`PreparedWrite`] out, never committed here) so the CLI's
//! direct-commit and draft-proposal paths share the exact same authoring.

use uuid::Uuid;

use crate::error::EngineError;
use crate::substrate::{DesignModel, Operation, OutputJob, OutputJobRun};

use super::context::{BatchComposer, PreparedWrite, WriteProvenance};
use super::ids::derive_object_id;

/// Deterministic output-job-run id, namespaced by the project id (v5 seed
/// `datum-eda:output-job-run:<seed_parts joined by ':'>`).
///
/// The CLI's run-id materials (`lifecycle:...`, `failed:...`, `aggregate:...`)
/// are expressed as `seed_parts`; the joined seed is byte-identical to the
/// pre-migration `format!` materials.
pub fn derive_output_job_run_id(project_id: &Uuid, seed_parts: &[String]) -> Uuid {
    derive_object_id(project_id, "output-job-run", seed_parts)
}

/// Build a `CreateOutputJob` write for a fully formed output job.
pub fn build_create_output_job(
    model: &DesignModel,
    provenance: WriteProvenance,
    output_job: &OutputJob,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateOutputJob {
            output_job_id: output_job.id,
            output_job: serde_json::to_value(output_job)?,
        })
        .primary_object(output_job.id)
        .finish()
}

/// Build a `SetOutputJob` write replacing `previous_output_job` with
/// `output_job` (revision guard stamped automatically).
pub fn build_set_output_job(
    model: &DesignModel,
    provenance: WriteProvenance,
    previous_output_job: &OutputJob,
    output_job: &OutputJob,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetOutputJob {
            output_job_id: output_job.id,
            previous_output_job: serde_json::to_value(previous_output_job)?,
            output_job: serde_json::to_value(output_job)?,
        })
        .primary_object(output_job.id)
        .finish()
}

/// Build a `DeleteOutputJob` write for the job currently in the model
/// (revision guard stamped automatically).
pub fn build_delete_output_job(
    model: &DesignModel,
    provenance: WriteProvenance,
    output_job: &OutputJob,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteOutputJob {
            output_job_id: output_job.id,
            output_job: serde_json::to_value(output_job)?,
        })
        .primary_object(output_job.id)
        .finish()
}

/// Build a `SetOutputJobRun` evidence write for `output_job_run`.
///
/// The previous run payload is looked up from the resolved model (matching
/// the pre-migration CLI behavior), so replays and re-runs thread the prior
/// evidence into the operation.
pub fn build_set_output_job_run(
    model: &DesignModel,
    provenance: WriteProvenance,
    output_job_run: &OutputJobRun,
) -> Result<PreparedWrite, EngineError> {
    let previous_output_job_run = model
        .output_job_runs
        .get(&output_job_run.run_id)
        .map(serde_json::to_value)
        .transpose()?;
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetOutputJobRun {
            run_id: output_job_run.run_id,
            previous_output_job_run,
            output_job_run: serde_json::to_value(output_job_run)?,
        })
        .primary_object(output_job_run.run_id)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::super::context::commit_prepared;
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::substrate::{
        ArtifactKind, CommitSource, OUTPUT_JOB_RUN_SCHEMA_VERSION, ObjectRevision,
        OutputJobLogEntry, OutputJobLogLevel, OutputJobRunStatus, PRODUCTION_RECORD_SCHEMA_VERSION,
    };

    fn test_provenance(reason: &str) -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, reason)
    }

    fn test_job(project_id: &Uuid, board_id: Uuid) -> OutputJob {
        OutputJob {
            schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
            id: Uuid::new_v5(project_id, b"test-output-job"),
            name: "Gerber set fab".to_string(),
            include: vec![ArtifactKind::GerberSet],
            prefix: "fab".to_string(),
            output_dir: None,
            board_or_panel: board_id,
            variant: None,
            manufacturing_plan: None,
            object_revision: ObjectRevision(0),
        }
    }

    fn test_run(model: &DesignModel, output_job: Uuid, seed: &str) -> OutputJobRun {
        OutputJobRun {
            schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
            run_id: derive_output_job_run_id(
                &model.project.project_id,
                &[
                    "lifecycle".to_string(),
                    output_job.to_string(),
                    seed.to_string(),
                ],
            ),
            output_job,
            run_sequence: 1,
            project_id: model.project.project_id,
            model_revision: model.model_revision.clone(),
            status: OutputJobRunStatus::Running,
            artifact_id: None,
            exit_code: None,
            provenance: None,
            log: vec![OutputJobLogEntry {
                sequence: 1,
                level: OutputJobLogLevel::Info,
                message: "output job run started".to_string(),
            }],
        }
    }

    #[test]
    fn derive_run_id_matches_cli_material_layout() {
        let project_id = Uuid::new_v4();
        let output_job = Uuid::new_v4();
        // Pre-migration CLI material, reproduced verbatim as the parity oracle.
        let material = format!("datum-eda:output-job-run:lifecycle:{output_job}:rev-1:3:0");
        assert_eq!(
            derive_output_job_run_id(
                &project_id,
                &[
                    "lifecycle".to_string(),
                    output_job.to_string(),
                    "rev-1".to_string(),
                    "3".to_string(),
                    "0".to_string(),
                ],
            ),
            Uuid::new_v5(&project_id, material.as_bytes()),
        );
    }

    #[test]
    fn output_job_create_set_delete_round_trip_through_commit() {
        let (root, mut model, board_id, _package_id) =
            resolved_model_with_board_package("output_job_round_trip");
        let job = test_job(&model.project.project_id, board_id);

        let prepared = build_create_output_job(&model, test_provenance("create output job"), &job)
            .expect("create job should build");
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreateOutputJob {
                output_job_id: job.id,
                output_job: serde_json::to_value(&job).unwrap(),
            }]
        );
        commit_prepared(&mut model, &root, prepared).expect("create job should commit");

        let mut updated = job.clone();
        updated.name = "Renamed job".to_string();
        updated.object_revision = ObjectRevision(job.object_revision.0 + 1);
        let prepared =
            build_set_output_job(&model, test_provenance("update output job"), &job, &updated)
                .expect("set job should build");
        assert_eq!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision {
                object_id: job.id,
                expected_object_revision: model.objects[&job.id].object_revision,
            }
        );
        commit_prepared(&mut model, &root, prepared).expect("set job should commit");
        assert_eq!(model.output_jobs[&job.id].name, "Renamed job");

        let live = model.output_jobs[&job.id].clone();
        let prepared = build_delete_output_job(&model, test_provenance("delete output job"), &live)
            .expect("delete job should build");
        assert!(matches!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == job.id
        ));
        commit_prepared(&mut model, &root, prepared).expect("delete job should commit");
        assert!(!model.output_jobs.contains_key(&job.id));
    }

    #[test]
    fn set_output_job_run_threads_previous_run_from_model() {
        let (root, mut model, board_id, _package_id) =
            resolved_model_with_board_package("output_job_run_previous");
        let job = test_job(&model.project.project_id, board_id);
        let prepared = build_create_output_job(&model, test_provenance("create output job"), &job)
            .expect("create job should build");
        commit_prepared(&mut model, &root, prepared).expect("create job should commit");

        let run = test_run(&model, job.id, "first");
        let prepared = build_set_output_job_run(
            &model,
            test_provenance("record output job run generated evidence"),
            &run,
        )
        .expect("first run should build");
        // Evidence records are not guard targets: single unguarded operation
        // with no previous payload on first write.
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::SetOutputJobRun {
                run_id: run.run_id,
                previous_output_job_run: None,
                output_job_run: serde_json::to_value(&run).unwrap(),
            }]
        );
        commit_prepared(&mut model, &root, prepared).expect("first run should commit");

        let mut rerun = model.output_job_runs[&run.run_id].clone();
        rerun.status = OutputJobRunStatus::Succeeded;
        rerun.exit_code = Some(0);
        let prepared = build_set_output_job_run(
            &model,
            test_provenance("record output job run generated evidence"),
            &rerun,
        )
        .expect("rerun should build");
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::SetOutputJobRun {
                run_id: run.run_id,
                previous_output_job_run: Some(
                    serde_json::to_value(&model.output_job_runs[&run.run_id]).unwrap()
                ),
                output_job_run: serde_json::to_value(&rerun).unwrap(),
            }]
        );
    }
}
