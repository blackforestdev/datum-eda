use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteProposalExportReportView {
    pub(crate) action: String,
    pub(crate) artifact_path: String,
    pub(crate) kind: String,
    pub(crate) version: u32,
    pub(crate) project_uuid: String,
    pub(crate) contract: String,
    pub(crate) actions: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteProposalArtifactInspectionView {
    pub(crate) artifact_path: String,
    pub(crate) kind: String,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) project_uuid: String,
    pub(crate) project_name: String,
    pub(crate) contract: String,
    pub(crate) actions: usize,
    pub(crate) draw_track_actions: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteProposalArtifactApplyView {
    pub(crate) action: String,
    pub(crate) artifact_path: String,
    pub(crate) project_root: String,
    pub(crate) artifact_actions: usize,
    pub(crate) applied_actions: usize,
    pub(crate) applied: Vec<NativeProjectBoardTrackMutationReportView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRouteApplyView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) contract: String,
    pub(crate) proposal_actions: usize,
    pub(crate) applied_actions: usize,
    pub(crate) applied: Vec<NativeProjectBoardTrackMutationReportView>,
}

pub(super) fn render_native_route_proposal_export_text(
    report: &NativeProjectRouteProposalExportReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("kind: {}", report.kind),
        format!("version: {}", report.version),
        format!("project_uuid: {}", report.project_uuid),
        format!("contract: {}", report.contract),
        format!("actions: {}", report.actions),
    ];
    if is_legacy_route_export_action(&report.action) {
        lines.push(
            "note: deprecated compatibility wrapper; prefer `project export-route-path-proposal --candidate ...`"
                .to_string(),
        );
    }
    lines.join("\n")
}

fn is_legacy_route_export_action(action: &str) -> bool {
    action.starts_with("export_route_path_candidate_")
}

pub(super) fn render_native_route_proposal_artifact_inspection_text(
    report: &NativeProjectRouteProposalArtifactInspectionView,
) -> String {
    [
        format!("artifact_path: {}", report.artifact_path),
        format!("kind: {}", report.kind),
        format!("source_version: {}", report.source_version),
        format!("version: {}", report.version),
        format!("migration_applied: {}", report.migration_applied),
        format!("project_uuid: {}", report.project_uuid),
        format!("project_name: {}", report.project_name),
        format!("contract: {}", report.contract),
        format!("actions: {}", report.actions),
        format!("draw_track_actions: {}", report.draw_track_actions),
    ]
    .join("\n")
}

pub(super) fn render_native_route_proposal_artifact_apply_text(
    report: &NativeProjectRouteProposalArtifactApplyView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("project_root: {}", report.project_root),
        format!("artifact_actions: {}", report.artifact_actions),
        format!("applied_actions: {}", report.applied_actions),
    ];
    for applied in &report.applied {
        lines.push(String::new());
        lines.push(format!("track_uuid: {}", applied.track_uuid));
        lines.push(format!("net_uuid: {}", applied.net_uuid));
        lines.push(format!("from_x_nm: {}", applied.from_x_nm));
        lines.push(format!("from_y_nm: {}", applied.from_y_nm));
        lines.push(format!("to_x_nm: {}", applied.to_x_nm));
        lines.push(format!("to_y_nm: {}", applied.to_y_nm));
        lines.push(format!("width_nm: {}", applied.width_nm));
        lines.push(format!("layer: {}", applied.layer));
    }
    lines.join("\n")
}

pub(super) fn render_native_route_apply_text(report: &NativeProjectRouteApplyView) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("contract: {}", report.contract),
        format!("proposal_actions: {}", report.proposal_actions),
        format!("applied_actions: {}", report.applied_actions),
    ];
    for applied in &report.applied {
        lines.push(String::new());
        lines.push(format!("track_uuid: {}", applied.track_uuid));
        lines.push(format!("net_uuid: {}", applied.net_uuid));
        lines.push(format!("from_x_nm: {}", applied.from_x_nm));
        lines.push(format!("from_y_nm: {}", applied.from_y_nm));
        lines.push(format!("to_x_nm: {}", applied.to_x_nm));
        lines.push(format!("to_y_nm: {}", applied.to_y_nm));
        lines.push(format!("width_nm: {}", applied.width_nm));
        lines.push(format!("layer: {}", applied.layer));
    }
    lines.join("\n")
}
