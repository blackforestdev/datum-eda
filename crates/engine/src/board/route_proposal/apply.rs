//! Accepted route-proposal composition: lower validated proposal actions
//! into the self-sufficient accepted [`Proposal`] the apply path commits.
//!
//! Family F sub-step 2 of the native-write migration: the CLI's
//! `command_project_route_proposal_substrate.rs` no longer hand-rolls
//! `Operation`/`OperationBatch` values — each `draw_track` action's
//! `CreateBoardTrack` operation is authored by the family-D facade builder
//! [`build_place_board_track`] (new objects receive no guards, so the
//! composed batch is byte-identical to the historical CLI batch), and the
//! deterministic identity comes from [`ids::derive_object_id`]:
//!
//! - track id: v5 over `datum-eda:route-apply-track:<model revision>:<action id>`
//! - proposal id: v5 over `datum-eda:route-apply-proposal:<model revision>:<action ids joined by '|'>`
//! - batch id: v5 over the raw proposal-id bytes
//!
//! The batch is intentionally unguarded (its authority is the
//! `prepared_against` model-revision guard) and the proposal is born
//! `Accepted`; committing its metadata and applying it stay on the
//! substrate-owned `commit_proposal_metadata_journaled` +
//! `apply_accepted_proposal` path in the CLI.

use uuid::Uuid;

use crate::api::native_write::board_routing::build_place_board_track;
use crate::api::native_write::context::WriteProvenance;
use crate::api::native_write::ids;
use crate::board::Track;
use crate::substrate::{DesignModel, OperationBatch, Proposal, ProposalSource, ProposalStatus};

use super::RouteProposalAction;

/// One composed track plus the via reuse evidence its action carried.
pub struct BuiltRouteTrack {
    pub track: Track,
    pub reused_via_uuid: Option<Uuid>,
    pub reused_via_uuids: Vec<Uuid>,
}

/// The composed accepted proposal (None when no action draws a track).
pub struct BuiltRouteProposal {
    pub proposal: Option<Proposal>,
    pub tracks: Vec<BuiltRouteTrack>,
}

/// Compose the accepted route proposal for `actions` against `model`.
///
/// `net_exists` answers against the caller's persisted board state (the
/// historical CLI checked the resolved `board.json` nets map); a missing net
/// fails with the historical error text.
pub fn build_accepted_route_proposal(
    model: &DesignModel,
    provenance: WriteProvenance,
    actions: &[RouteProposalAction],
    net_exists: impl Fn(Uuid) -> bool,
) -> Result<BuiltRouteProposal, String> {
    let expected_model_revision = model.model_revision.clone();
    let mut tracks = Vec::new();
    let mut operations = Vec::new();
    for action in actions {
        if action.proposal_action == "draw_track" {
            if !net_exists(action.net_uuid) {
                return Err(format!(
                    "board net not found in native project: {}",
                    action.net_uuid
                ));
            }
            let track_uuid = ids::derive_object_id(
                &model.project.project_id,
                "route-apply-track",
                &[
                    expected_model_revision.0.to_string(),
                    action.action_id.clone(),
                ],
            );
            let track = Track {
                uuid: track_uuid,
                net: action.net_uuid,
                from: action.from,
                to: action.to,
                width: action.width_nm,
                layer: action.layer,
            };
            let prepared = build_place_board_track(model, provenance.clone(), &track)
                .map_err(|error| error.to_string())?;
            operations.extend(prepared.batch.operations);
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

    let proposal_id = ids::derive_object_id(
        &model.project.project_id,
        "route-apply-proposal",
        &[
            expected_model_revision.0.to_string(),
            actions
                .iter()
                .map(|action| action.action_id.as_str())
                .collect::<Vec<_>>()
                .join("|"),
        ],
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
                provenance: provenance.into(),
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::api::native_write::board_routing::build_place_board_net;
    use crate::api::native_write::context::commit_prepared;
    use crate::api::native_write::genesis::{GenesisSpec, bootstrap_native_project};
    use crate::board::Net;
    use crate::ir::geometry::Point;
    use crate::substrate::{
        CommitSource, Operation, ProjectResolver, apply_accepted_proposal,
        commit_proposal_metadata_journaled,
    };

    use super::*;

    fn test_provenance() -> WriteProvenance {
        WriteProvenance::new(
            "datum-eda-cli",
            CommitSource::Cli,
            "route apply accepted proposal",
        )
    }

    fn fixture_project(label: &str) -> (PathBuf, DesignModel, Net) {
        let root = std::env::temp_dir().join(format!(
            "datum-route-proposal-apply-{label}-{}",
            Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).expect("temp project root should create");
        bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Route Proposal Apply Fixture".to_string(),
                existing_ids: None,
            },
        )
        .expect("genesis should succeed");
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("fresh project should resolve");
        let net = Net {
            uuid: Uuid::new_v4(),
            name: "SIG".to_string(),
            class: Uuid::new_v4(),
            controlled_impedance: None,
        };
        let prepared = build_place_board_net(&model, test_provenance(), &net)
            .expect("net placement should build");
        commit_prepared(&mut model, &root, prepared).expect("net placement should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");
        (root, model, net)
    }

    fn draw_track_action(net: &Net, action_id: &str, to_x: i64) -> RouteProposalAction {
        RouteProposalAction {
            action_id: action_id.to_string(),
            proposal_action: "draw_track".to_string(),
            reason: "route_path_candidate".to_string(),
            contract: "m5_route_path_candidate_v2".to_string(),
            net_uuid: net.uuid,
            net_name: net.name.clone(),
            from_anchor_pad_uuid: Uuid::new_v4(),
            to_anchor_pad_uuid: Uuid::new_v4(),
            layer: 1,
            width_nm: 200_000,
            from: Point { x: 0, y: 0 },
            to: Point { x: to_x, y: 0 },
            reused_via_uuid: None,
            reused_via_uuids: Vec::new(),
            reused_object_kind: None,
            reused_object_uuid: None,
            reused_object_from_layer: None,
            reused_object_to_layer: None,
            selected_path_bend_count: 0,
            selected_path_point_count: 2,
            selected_path_segment_index: 0,
            selected_path_segment_count: 1,
            selected_path_layer_segment_index: None,
            selected_path_layer_segment_count: None,
            selected_path_layer_segment_bend_count: None,
            selected_path_layer_segment_point_count: None,
        }
    }

    #[test]
    fn composed_proposal_matches_the_historical_cli_identity_and_batch() {
        let (root, model, net) = fixture_project("identity");
        let actions = vec![
            draw_track_action(&net, "action-1", 1_000_000),
            draw_track_action(&net, "action-2", 2_000_000),
        ];

        let built =
            build_accepted_route_proposal(&model, test_provenance(), &actions, |uuid| {
                uuid == net.uuid
            })
            .expect("proposal should build");
        let proposal = built.proposal.expect("proposal should exist");

        // Byte-exact historical CLI derivations
        // (command_project_route_proposal_substrate.rs, pre-migration).
        let expected_track_ids = actions
            .iter()
            .map(|action| {
                Uuid::new_v5(
                    &model.project.project_id,
                    format!(
                        "datum-eda:route-apply-track:{}:{}",
                        model.model_revision.0, action.action_id
                    )
                    .as_bytes(),
                )
            })
            .collect::<Vec<_>>();
        let expected_proposal_id = Uuid::new_v5(
            &model.project.project_id,
            format!(
                "datum-eda:route-apply-proposal:{}:{}",
                model.model_revision.0, "action-1|action-2"
            )
            .as_bytes(),
        );
        assert_eq!(proposal.proposal_id, expected_proposal_id);
        assert_eq!(
            proposal.batch.batch_id,
            Uuid::new_v5(&model.project.project_id, expected_proposal_id.as_bytes())
        );
        assert_eq!(proposal.schema_version, 1);
        assert_eq!(proposal.prepared_against, model.model_revision);
        assert_eq!(
            proposal.batch.expected_model_revision,
            Some(model.model_revision.clone())
        );
        assert_eq!(proposal.batch.provenance.actor, "datum-eda-cli");
        assert_eq!(
            proposal.batch.provenance.reason,
            "route apply accepted proposal"
        );
        // The historical batch is intentionally unguarded: one raw
        // CreateBoardTrack per draw_track action, serde payloads.
        let expected_operations = built
            .tracks
            .iter()
            .map(|built_track| Operation::CreateBoardTrack {
                track_id: built_track.track.uuid,
                track: serde_json::to_value(&built_track.track).unwrap(),
            })
            .collect::<Vec<_>>();
        assert_eq!(proposal.batch.operations, expected_operations);
        assert_eq!(
            built
                .tracks
                .iter()
                .map(|built_track| built_track.track.uuid)
                .collect::<Vec<_>>(),
            expected_track_ids
        );
        assert_eq!(proposal.affected_objects, expected_track_ids);
        assert_eq!(proposal.source, ProposalSource::Cli);
        assert_eq!(proposal.status, ProposalStatus::Accepted);
        assert_eq!(proposal.applied_transaction_id, None);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn composed_proposal_applies_through_the_journaled_substrate_path() {
        let (root, mut model, net) = fixture_project("apply");
        let actions = vec![draw_track_action(&net, "action-1", 1_000_000)];
        let built =
            build_accepted_route_proposal(&model, test_provenance(), &actions, |uuid| {
                uuid == net.uuid
            })
            .expect("proposal should build");
        let proposal = built.proposal.expect("proposal should exist");
        let proposal_id = proposal.proposal_id;
        let track_id = built.tracks[0].track.uuid;

        commit_proposal_metadata_journaled(&mut model, &root, proposal)
            .expect("proposal metadata should commit");
        apply_accepted_proposal(&mut model, &root, proposal_id)
            .expect("facade-composed proposal should apply");

        let resolved = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");
        assert!(
            resolved.objects.contains_key(&track_id),
            "applied track should resolve as a domain object"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_net_fails_with_the_historical_error_and_no_track_actions_yield_no_proposal() {
        let (root, model, net) = fixture_project("missing_net");
        let action = draw_track_action(&net, "action-1", 1_000_000);
        let error = match build_accepted_route_proposal(
            &model,
            test_provenance(),
            &[action.clone()],
            |_| false,
        ) {
            Err(error) => error,
            Ok(_) => panic!("missing net should fail"),
        };
        assert_eq!(
            error,
            format!("board net not found in native project: {}", net.uuid)
        );

        let mut reuse_only = action;
        reuse_only.proposal_action = "reuse_existing_copper_step".to_string();
        let built =
            build_accepted_route_proposal(&model, test_provenance(), &[reuse_only], |_| false)
                .expect("reuse-only action set should build");
        assert!(built.proposal.is_none());
        assert!(built.tracks.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }
}
