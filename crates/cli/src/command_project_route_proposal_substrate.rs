use super::*;
use crate::NativeProjectRouteAppliedTrackReportView;
use eda_engine::api::native_write::WriteProvenance;
use eda_engine::board::route_proposal;
use eda_engine::substrate::{
    CommitSource, ProjectResolver, Proposal, apply_accepted_proposal,
    commit_proposal_metadata_journaled,
};

pub(crate) use eda_engine::board::route_proposal::{BuiltRouteProposal, BuiltRouteTrack};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RouteProposalArtifact {
    pub(crate) kind: String,
    pub(crate) version: u32,
    pub(crate) project_uuid: Uuid,
    pub(crate) project_name: String,
    pub(crate) contract: String,
    pub(crate) actions: Vec<NativeProjectRouteProposalActionView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) proposal: Option<Proposal>,
}

pub(crate) struct LoadedRouteProposalArtifact {
    pub(crate) artifact_path: PathBuf,
    pub(crate) source_version: u32,
    pub(crate) artifact: RouteProposalArtifact,
}

pub(crate) fn build_accepted_route_proposal(
    root: &Path,
    actions: &[NativeProjectRouteProposalActionView],
) -> Result<BuiltRouteProposal> {
    let project = load_native_project_with_resolved_board(root)?;
    let model = ProjectResolver::new(root).resolve()?;
    route_proposal::build_accepted_route_proposal(
        &model,
        WriteProvenance::new(
            "datum-eda-cli",
            CommitSource::Cli,
            "route apply accepted proposal",
        ),
        actions,
        |net_uuid| project.board.nets.contains_key(&net_uuid.to_string()),
    )
    .map_err(anyhow::Error::msg)
}

pub(crate) fn apply_built_route_proposal(
    root: &Path,
    built: BuiltRouteProposal,
) -> Result<Vec<NativeProjectRouteAppliedTrackReportView>> {
    let Some(proposal) = built.proposal else {
        return Ok(Vec::new());
    };
    let mut model = ProjectResolver::new(root).resolve()?;
    let proposal_id = proposal.proposal_id;
    commit_proposal_metadata_journaled(&mut model, root, proposal)?;
    apply_accepted_proposal(&mut model, root, proposal_id)?;

    let project = load_native_project_with_resolved_board(root)?;
    Ok(built
        .tracks
        .into_iter()
        .map(|built_track| native_project_route_applied_track_report(&project, built_track))
        .collect())
}

pub(crate) fn apply_route_proposal(
    root: &Path,
    actions: &[NativeProjectRouteProposalActionView],
    proposal: Proposal,
) -> Result<Vec<NativeProjectRouteAppliedTrackReportView>> {
    let mut built = build_accepted_route_proposal(root, actions)?;
    built.proposal = Some(proposal);
    apply_built_route_proposal(root, built)
}

fn native_project_route_applied_track_report(
    project: &LoadedNativeProject,
    built_track: BuiltRouteTrack,
) -> NativeProjectRouteAppliedTrackReportView {
    NativeProjectRouteAppliedTrackReportView {
        action: "draw_board_track".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        track_uuid: built_track.track.uuid.to_string(),
        net_uuid: built_track.track.net.to_string(),
        from_x_nm: built_track.track.from.x,
        from_y_nm: built_track.track.from.y,
        to_x_nm: built_track.track.to.x,
        to_y_nm: built_track.track.to.y,
        width_nm: built_track.track.width,
        layer: built_track.track.layer,
        reused_via_uuid: built_track.reused_via_uuid.map(|uuid| uuid.to_string()),
        reused_via_uuids: built_track
            .reused_via_uuids
            .into_iter()
            .map(|uuid| uuid.to_string())
            .collect(),
    }
}
