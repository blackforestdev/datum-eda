use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionArtifactRunSummary {
    pub run_id: String,
    pub artifact_id: String,
    pub run_source: String,
    pub output_job_id: Option<String>,
    pub run_sequence: u64,
    pub status: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct ArtifactListPayload {
    #[serde(default)]
    pub(crate) artifact_count: usize,
    #[serde(default)]
    artifact_runs: Vec<ArtifactRunPayload>,
    #[serde(default)]
    output_job_runs: Vec<OutputJobRunPayload>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ArtifactRunPayload {
    run_id: String,
    artifact_id: String,
    #[serde(default)]
    run_sequence: u64,
    status: String,
    #[serde(default)]
    exit_code: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct OutputJobRunPayload {
    run_id: String,
    output_job: String,
    #[serde(default)]
    artifact_id: Option<String>,
    #[serde(default)]
    run_sequence: u64,
    status: String,
    #[serde(default)]
    exit_code: Option<i32>,
}

pub(crate) fn artifact_run_summaries(
    payload: &ArtifactListPayload,
) -> Vec<ProductionArtifactRunSummary> {
    let mut runs = payload
        .artifact_runs
        .iter()
        .map(|run| ProductionArtifactRunSummary {
            run_id: run.run_id.clone(),
            artifact_id: run.artifact_id.clone(),
            run_source: "artifact_run".to_string(),
            output_job_id: None,
            run_sequence: run.run_sequence,
            status: run.status.clone(),
            exit_code: run.exit_code,
        })
        .collect::<Vec<_>>();
    runs.extend(payload.output_job_runs.iter().filter_map(|run| {
        let artifact_id = run.artifact_id.as_ref()?;
        Some(ProductionArtifactRunSummary {
            run_id: run.run_id.clone(),
            artifact_id: artifact_id.clone(),
            run_source: "output_job_run".to_string(),
            output_job_id: Some(run.output_job.clone()),
            run_sequence: run.run_sequence,
            status: run.status.clone(),
            exit_code: run.exit_code,
        })
    }));
    runs.sort_by(|a, b| {
        a.run_sequence
            .cmp(&b.run_sequence)
            .then_with(|| a.run_source.cmp(&b.run_source))
            .then_with(|| a.run_id.cmp(&b.run_id))
    });
    runs
}
