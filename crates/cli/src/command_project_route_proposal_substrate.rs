use super::*;
use crate::NativeProjectRouteAppliedTrackReportView;
use eda_engine::board::Track;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver, Proposal,
    ProposalSource, ProposalStatus, apply_accepted_proposal, commit_proposal_metadata_journaled,
};

pub(crate) struct BuiltRouteProposal {
    pub(crate) proposal: Option<Proposal>,
    pub(crate) tracks: Vec<BuiltRouteTrack>,
}

pub(crate) struct BuiltRouteTrack {
    pub(crate) track: Track,
    pub(crate) reused_via_uuid: Option<Uuid>,
    pub(crate) reused_via_uuids: Vec<Uuid>,
}

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
    let project = load_native_project(root)?;
    let model = ProjectResolver::new(root).resolve()?;
    let expected_model_revision = model.model_revision.clone();
    let mut tracks = Vec::new();
    let mut operations = Vec::new();
    for action in actions {
        if action.proposal_action == "draw_track" {
            if !project
                .board
                .nets
                .contains_key(&action.net_uuid.to_string())
            {
                bail!("board net not found in native project: {}", action.net_uuid);
            }
            let track_uuid = Uuid::new_v5(
                &model.project.project_id,
                format!(
                    "datum-eda:route-apply-track:{}:{}",
                    expected_model_revision.0, action.action_id
                )
                .as_bytes(),
            );
            let track = Track {
                uuid: track_uuid,
                net: action.net_uuid,
                from: action.from,
                to: action.to,
                width: action.width_nm,
                layer: action.layer,
            };
            operations.push(Operation::CreateBoardTrack {
                track_id: track.uuid,
                track: serde_json::to_value(&track)
                    .expect("native board track serialization must succeed"),
            });
            tracks.push(BuiltRouteTrack {
                track,
                reused_via_uuid: action.reused_via_uuid,
                reused_via_uuids: action.reused_via_uuids.clone(),
            });
        }
    }
    if operations.is_empty() {
        return Ok(BuiltRouteProposal {
            proposal: None,
            tracks,
        });
    }

    let proposal_id = Uuid::new_v5(
        &model.project.project_id,
        format!(
            "datum-eda:route-apply-proposal:{}:{}",
            expected_model_revision.0,
            actions
                .iter()
                .map(|action| action.action_id.as_str())
                .collect::<Vec<_>>()
                .join("|")
        )
        .as_bytes(),
    );
    Ok(BuiltRouteProposal {
        proposal: Some(Proposal {
            schema_version: 1,
            proposal_id,
            project_id: model.project.project_id,
            prepared_against: expected_model_revision.clone(),
            batch: OperationBatch {
                batch_id: Uuid::new_v5(&model.project.project_id, proposal_id.as_bytes()),
                expected_model_revision: Some(expected_model_revision),
                provenance: CommitProvenance {
                    actor: "datum-eda-cli".to_string(),
                    source: CommitSource::Cli,
                    reason: "route apply accepted proposal".to_string(),
                },
                operations,
            },
            rationale: "route proposal apply draw tracks".to_string(),
            affected_objects: route_proposal_affected_objects(&tracks),
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
            source: ProposalSource::Cli,
            status: ProposalStatus::Accepted,
            applied_transaction_id: None,
        }),
        tracks,
    })
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

    let project = load_native_project(root)?;
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

fn route_proposal_affected_objects(tracks: &[BuiltRouteTrack]) -> Vec<Uuid> {
    let mut affected = Vec::new();
    for track in tracks {
        affected.push(track.track.uuid);
        for via_uuid in &track.reused_via_uuids {
            if !affected.contains(via_uuid) {
                affected.push(*via_uuid);
            }
        }
    }
    affected
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
