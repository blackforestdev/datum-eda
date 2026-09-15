use crate::ProductionStatus;

pub(crate) fn focused_artifact_id(status: &ProductionStatus) -> Option<String> {
    status
        .latest_artifact_id
        .clone()
        .or_else(|| latest_output_job_artifact_id(status))
        .or_else(|| {
            status
                .output_jobs
                .iter()
                .rev()
                .find_map(|job| job.latest_run_artifact_id.clone())
        })
        .or_else(|| {
            status
                .artifact_runs
                .last()
                .map(|run| run.artifact_id.clone())
        })
        .or_else(|| {
            status.output_jobs.iter().find_map(|job| {
                job.artifacts
                    .first()
                    .map(|artifact| artifact.artifact_id.clone())
            })
        })
}

fn latest_output_job_artifact_id(status: &ProductionStatus) -> Option<String> {
    status
        .latest_output_job_run_id
        .as_ref()
        .and_then(|run_id| {
            status
                .output_jobs
                .iter()
                .find(|job| job.latest_run_id.as_ref() == Some(run_id))
        })
        .and_then(|job| job.latest_run_artifact_id.clone())
}
