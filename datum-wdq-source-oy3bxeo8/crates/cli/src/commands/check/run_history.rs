use crate::*;
use anyhow::Result;
use eda_engine::substrate::ProjectResolver;
use serde::Serialize;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectCheckRunListView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) check_run_count: usize,
    pub(crate) latest_check_run_id: Option<Uuid>,
    pub(crate) latest_profile_id: Option<String>,
    pub(crate) profile_latest_check_runs: Vec<NativeProjectCheckRunProfileLatestView>,
    pub(crate) check_runs: Vec<NativeProjectCheckRunSummaryView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectCheckRunProfileLatestView {
    pub(crate) profile_id: String,
    pub(crate) check_run_id: Uuid,
    pub(crate) model_revision: String,
    pub(crate) status: String,
    pub(crate) finding_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectCheckRunSummaryView {
    pub(crate) check_run_id: Uuid,
    pub(crate) project_id: Uuid,
    pub(crate) model_revision: String,
    pub(crate) profile_id: String,
    pub(crate) status: String,
    pub(crate) finding_count: usize,
    pub(crate) coverage_count: usize,
    pub(crate) latest_for_profile: bool,
    pub(crate) proposal_refs: Vec<String>,
}

pub(crate) fn query_native_project_check_run_list(
    root: &Path,
) -> Result<NativeProjectCheckRunListView> {
    let model = ProjectResolver::new(root).resolve()?;
    let mut check_runs = model
        .check_runs
        .values()
        .map(|run| NativeProjectCheckRunSummaryView {
            check_run_id: run.check_run_id,
            project_id: run.project_id,
            model_revision: run.model_revision.0.clone(),
            profile_id: run.profile_id.clone(),
            status: run.status.clone(),
            finding_count: run.finding_count,
            coverage_count: run.coverage.len(),
            latest_for_profile: false,
            proposal_refs: run.proposal_refs.clone(),
        })
        .collect::<Vec<_>>();
    check_runs.sort_by(|left, right| {
        left.model_revision
            .cmp(&right.model_revision)
            .then_with(|| left.profile_id.cmp(&right.profile_id))
            .then_with(|| left.check_run_id.cmp(&right.check_run_id))
    });
    let mut latest_by_profile = std::collections::BTreeMap::<String, Uuid>::new();
    for run in &check_runs {
        latest_by_profile.insert(run.profile_id.clone(), run.check_run_id);
    }
    for run in &mut check_runs {
        run.latest_for_profile = latest_by_profile.get(&run.profile_id) == Some(&run.check_run_id);
    }
    let profile_latest_check_runs = latest_by_profile
        .iter()
        .filter_map(|(profile_id, check_run_id)| {
            check_runs
                .iter()
                .find(|run| &run.check_run_id == check_run_id)
                .map(|run| NativeProjectCheckRunProfileLatestView {
                    profile_id: profile_id.clone(),
                    check_run_id: run.check_run_id,
                    model_revision: run.model_revision.clone(),
                    status: run.status.clone(),
                    finding_count: run.finding_count,
                })
        })
        .collect::<Vec<_>>();
    let latest = check_runs.last();
    Ok(NativeProjectCheckRunListView {
        contract: "check_run_list_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        check_run_count: check_runs.len(),
        latest_check_run_id: latest.map(|run| run.check_run_id),
        latest_profile_id: latest.map(|run| run.profile_id.clone()),
        profile_latest_check_runs,
        check_runs,
    })
}

// Phase 5: exec-layer dissolution — variant run() impls (the former
// command_exec destructure-and-forward glue, now inherent methods on the
// clap args structs).

impl CheckRunArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, profile } = self;
        Ok((
            render_output(
                format,
                &run_native_project_check_with_profile(&path, profile.as_deref())?,
            ),
            0,
        ))
    }
}

impl CheckListArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path } = self;
        Ok((
            render_output(format, &query_native_project_check_run_list(&path)?),
            0,
        ))
    }
}

impl CheckShowArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, check_run } = self;
        Ok((
            render_output(
                format,
                &query_native_project_check_run_show(&path, check_run)?,
            ),
            0,
        ))
    }
}

impl CheckProfilesArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path } = self;
        Ok((
            render_output(format, &query_native_project_check_profiles(&path)?),
            0,
        ))
    }
}

impl CheckFillZonesArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            zone_uuid,
            net_uuid,
        } = self;
        Ok((
            render_output(
                format,
                &fill_native_project_zones(&path, zone_uuid, net_uuid)?,
            ),
            0,
        ))
    }
}

impl CheckRepairStandardsArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path } = self;
        Ok((
            render_output(
                format,
                &generate_native_project_standards_repair_proposals(&path)?,
            ),
            0,
        ))
    }
}

impl CheckWaiveArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            fingerprint,
            rationale,
            created_by,
        } = self;
        Ok((
            render_output(
                format,
                &waive_native_project_finding(&path, &fingerprint, &rationale, created_by)?,
            ),
            0,
        ))
    }
}

impl CheckAcceptDeviationArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            fingerprint,
            rationale,
            accepted_by,
        } = self;
        Ok((
            render_output(
                format,
                &accept_native_project_deviation(&path, &fingerprint, &rationale, accepted_by)?,
            ),
            0,
        ))
    }
}

impl CheckImportedArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, fail_on } = self;
        let report = run_check(&path)?;
        let output = render_report(format, &report, render_check_report_text);
        Ok((output, check_exit_code(&report, fail_on)))
    }
}
