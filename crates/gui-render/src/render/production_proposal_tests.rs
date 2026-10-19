use super::*;
#[test]
fn proposal_preview_affected_ids_match_scene_source_ids() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let component = state
        .scene
        .components
        .first()
        .expect("fixture component should exist")
        .clone();
    state.production.proposals = vec![datum_gui_protocol::ProductionProposalSummary {
        action_identity: None,
        proposal_id: "proposal-a".to_string(),
        status: "draft".to_string(),
        source: "check".to_string(),
        rationale: "highlight modified component".to_string(),
        operation_count: 1,
        can_apply: Some(false),
        blocker_codes: Vec::new(),
        preview: Some(datum_gui_protocol::ProductionProposalPreviewSummary {
            prepared_against: "rev-before".to_string(),
            preview_after_model_revision: "rev-after".to_string(),
            created_count: 0,
            modified_count: 1,
            deleted_count: 0,
            affected_object_count: 1,
            affected_objects: vec![component.source_object_uuid.clone()],
            render_deltas: Vec::new(),
        }),
    }];

    let affected = proposal_preview_affected_ids(&state);
    assert!(source_object_matches_preview(
        &affected,
        &component.object_id,
        &component.source_object_uuid
    ));
    assert!(component_matches_preview(
        &component.component_uuid,
        &state.scene,
        &affected
    ));
}

#[test]
fn canonical_producer_tuple_passes_through_preview_without_geometry_identity() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let identity = datum_gui_protocol::selection_artifacts::ProductionProposalActionIdentity {
        project_id: uuid::Uuid::from_u128(1),
        proposal_id: uuid::Uuid::from_u128(2),
        batch_id: uuid::Uuid::from_u128(3),
        prepared_against: eda_engine::substrate::ModelRevision("rev-before".into()),
    };
    let track = &state.scene.tracks[0];
    state.production.proposals = vec![datum_gui_protocol::ProductionProposalSummary {
        proposal_id: identity.proposal_id.to_string(),
        action_identity: Some(identity.clone()),
        status: "draft".into(),
        source: "manual".into(),
        rationale: "existing preview projection".into(),
        operation_count: 1,
        can_apply: Some(true),
        blocker_codes: vec![],
        preview: Some(datum_gui_protocol::ProductionProposalPreviewSummary {
            prepared_against: identity.prepared_against.0.clone(),
            preview_after_model_revision: "rev-after".into(),
            created_count: 1,
            modified_count: 0,
            deleted_count: 0,
            affected_object_count: 1,
            affected_objects: vec![track.source_object_uuid.clone()],
            render_deltas: vec![datum_gui_protocol::ProductionProposalRenderDeltaSummary {
                delta_kind: "create".into(),
                object_id: track.source_object_uuid.clone(),
                primitive_kind: "track_path".into(),
                layer_id: track.layer_id.clone(),
                end_layer_id: None,
                width_nm: track.width_nm,
                drill_nm: None,
                diameter_nm: None,
                path: track.path.clone(),
            }],
        }),
    }];
    assert_eq!(
        production_proposal_overlay_primitives(&state)[0].proposal_action_id,
        identity.selection_key()
    );
    state.production.proposals[0]
        .preview
        .as_mut()
        .unwrap()
        .render_deltas[0]
        .path
        .reverse();
    assert_eq!(
        production_proposal_overlay_primitives(&state)[0].proposal_action_id,
        identity.selection_key()
    );
}

#[test]
fn native_production_selection_adds_existing_canvas_cue_and_preserves_source_bytes() {
    use datum_gui_protocol::{
        SelectionSubject, SelectionTarget, selection_artifacts::build_workspace_selection_view,
        selection_resolution::NativeSelectionResolution,
    };
    use eda_engine::substrate::{
        self, CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
        ProposalCreateRequest, ProposalSource,
    };
    let (fixture, mut state) = crate::native_selection_overlay::tests::selected_state();
    let mut model = ProjectResolver::new(fixture.root()).resolve().unwrap();
    let track_id = state.scene.tracks[0].source_object_uuid.parse().unwrap();
    let board = model
        .materialized_source_shard_value(substrate::SourceShardKind::BoardRoot)
        .unwrap();
    let track = board["tracks"][state.scene.tracks[0].source_object_uuid.clone()].clone();
    let expected = model.model_revision.clone();
    let proposal = substrate::create_draft_proposal_from_batch(
        &mut model,
        fixture.root(),
        ProposalCreateRequest {
            proposal_id: None,
            batch: OperationBatch {
                batch_id: uuid::Uuid::new_v4(),
                expected_model_revision: Some(expected),
                provenance: CommitProvenance {
                    actor: "native-proof".into(),
                    source: CommitSource::Test,
                    reason: "production canvas selection".into(),
                },
                operations: vec![Operation::SetBoardTrack { track_id, track }],
            },
            rationale: "existing native track proposal".into(),
            source: ProposalSource::Manual,
            checks_run: vec![],
            finding_fingerprints: vec![],
        },
    )
    .unwrap();
    let canonical = std::fs::read_to_string(
        fixture
            .root()
            .join(format!(".datum/proposals/{}.json", proposal.proposal_id)),
    )
    .unwrap();
    let payload = format!(
        r#"{{"proposal_count":1,"proposals":{{"{}":{}}}}}"#,
        proposal.proposal_id, canonical
    );
    state.production = datum_gui_protocol::production_status_from_proposals_json(&payload).unwrap();
    let track = &state.scene.tracks[0];
    state.production.proposals[0].preview =
        Some(datum_gui_protocol::ProductionProposalPreviewSummary {
            prepared_against: proposal.prepared_against.0.clone(),
            preview_after_model_revision: model.model_revision.0.clone(),
            created_count: 0,
            modified_count: 1,
            deleted_count: 0,
            affected_object_count: 1,
            affected_objects: vec![track.source_object_uuid.clone()],
            render_deltas: vec![datum_gui_protocol::ProductionProposalRenderDeltaSummary {
                delta_kind: "modify".into(),
                object_id: track.source_object_uuid.clone(),
                primitive_kind: "track_path".into(),
                layer_id: track.layer_id.clone(),
                end_layer_id: None,
                width_nm: track.width_nm,
                drill_nm: None,
                diameter_nm: None,
                path: track.path.clone(),
            }],
        });
    state.scene.proposal_overlay_primitives.clear();
    state.scene.review_primitives.clear();
    state.review.proposal_actions.clear();
    state.selection = SelectionTarget::None;
    let retained = RetainedScene::from_workspace(&state, 1600, 1000);
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let base = PreparedScene::from_workspace(&state, 1600, 1000, camera, &retained).unwrap();
    let identity = state.production.proposals[0]
        .action_identity
        .as_ref()
        .unwrap()
        .clone();
    let key = identity.selection_key();
    let native = NativeSelectionResolution::capture(&model).unwrap();
    let view = build_workspace_selection_view(
        &model,
        &native,
        SelectionSubject::Proposal(key.clone()),
        &state,
    )
    .unwrap();
    assert!(view.members.is_empty());
    let current_view = view.clone();
    state.active_review_target_id = key.clone();
    let remembered_none =
        PreparedScene::from_workspace(&state, 1600, 1000, camera, &retained).unwrap();
    assert_eq!(
        base.viewport_overlay_vertices, remembered_none.viewport_overlay_vertices,
        "remembered producer context cannot select a restored preview while selection is None"
    );
    state.selection = SelectionTarget::Native(Box::new(view));
    let before = fixture.source_bytes();
    let proposal_bytes = std::fs::read(
        fixture
            .root()
            .join(format!(".datum/proposals/{}.json", proposal.proposal_id)),
    )
    .unwrap();
    let selected = PreparedScene::from_workspace(&state, 1600, 1000, camera, &retained).unwrap();
    assert!(
        selected
            .hit_regions
            .iter()
            .any(|h| matches!(&h.target,HitTarget::ProposalAction(id) if id==&key))
    );
    assert_ne!(
        base.viewport_overlay_vertices,
        selected.viewport_overlay_vertices
    );
    let focused_vertices = |frame: &PreparedScene| {
        frame
            .viewport_overlay_vertices
            .iter()
            .filter(|vertex| vertex.color[..3] == PROPOSAL_FOCUS)
            .count()
    };
    assert!(
        focused_vertices(&selected) > 0,
        "matching current Proposal emits selected paint"
    );
    let empty =
        build_workspace_selection_view(&model, &native, SelectionSubject::None, &state).unwrap();
    state.selection = SelectionTarget::Native(Box::new(empty));
    let native_none = PreparedScene::from_workspace(&state, 1600, 1000, camera, &retained).unwrap();
    assert_eq!(
        base.viewport_overlay_vertices,
        native_none.viewport_overlay_vertices
    );
    let mut stale = current_view.clone();
    stale.stale = true;
    let mut mismatched = current_view.clone();
    let mut other = identity.clone();
    other.proposal_id = uuid::Uuid::from_u128(999);
    mismatched.subject = SelectionSubject::Proposal(other.selection_key());
    let mut unrelated = current_view.clone();
    unrelated.subject = SelectionSubject::Diagnostic(key.clone());
    for (label, selection) in [
        ("stale", SelectionTarget::Native(Box::new(stale))),
        (
            "other producer",
            SelectionTarget::Native(Box::new(mismatched)),
        ),
        (
            "unrelated native kind",
            SelectionTarget::Native(Box::new(unrelated)),
        ),
        (
            "legacy authored object",
            SelectionTarget::AuthoredObject(key.clone()),
        ),
        (
            "legacy check finding",
            SelectionTarget::CheckFinding(key.clone()),
        ),
        (
            "other legacy action",
            SelectionTarget::ReviewAction("other-action".into()),
        ),
    ] {
        state.selection = selection;
        let frame = PreparedScene::from_workspace(&state, 1600, 1000, camera, &retained).unwrap();
        assert_eq!(
            focused_vertices(&frame),
            0,
            "{label} cannot emit Proposal selected paint"
        );
    }
    state.selection = SelectionTarget::ReviewAction(key.clone());
    let legacy = PreparedScene::from_workspace(&state, 1600, 1000, camera, &retained).unwrap();
    assert!(
        focused_vertices(&legacy) > 0,
        "explicit matching legacy action preserves selected paint"
    );
    assert_eq!(fixture.source_bytes(), before);
    assert_eq!(
        std::fs::read(
            fixture
                .root()
                .join(format!(".datum/proposals/{}.json", proposal.proposal_id))
        )
        .unwrap(),
        proposal_bytes
    );
}
