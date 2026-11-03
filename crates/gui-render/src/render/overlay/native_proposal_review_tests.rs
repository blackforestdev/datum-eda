use super::*;
use datum_gui_protocol::schematic_authoring::delete_review::DeleteReview;
use eda_engine::{
    api::native_write::schematic_delete_proposal::{
        DeleteClass, DeleteTarget, DeletedObject, NativeProposalResponse,
    },
    schematic::{Schematic, Sheet},
    substrate::ProposalStatus,
};
fn review_fixture() -> (
    ReviewWorkspaceState,
    BoardReviewSceneV1,
    uuid::Uuid,
    uuid::Uuid,
) {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.selection = SelectionTarget::None;
    let project = uuid::Uuid::new_v4();
    state.scene.project_uuid = project.to_string();
    let sheet_id = uuid::Uuid::new_v4();
    let source = Sheet {
        uuid: sheet_id,
        name: "Main".into(),
        frame: None,
        symbols: Default::default(),
        wires: Default::default(),
        junctions: Default::default(),
        labels: Default::default(),
        buses: Default::default(),
        bus_entries: Default::default(),
        ports: Default::default(),
        noconnects: Default::default(),
        texts: Default::default(),
        drawings: Default::default(),
    };
    let wire = uuid::Uuid::new_v4();
    let symbol = uuid::Uuid::new_v4();
    let proposal_id = uuid::Uuid::new_v4();
    let mut scene = state.scene.clone();
    scene.schematic_document = None;
    scene
        .board_graphics
        .push(datum_gui_protocol::BoardGraphicPrimitive {
            object_id: "wire-test".into(),
            object_kind: "schematic_wire".into(),
            primitive_kind: "line".into(),
            source_object_uuid: wire.to_string(),
            layer_id: "L0".into(),
            path: vec![PointNm { x: 0, y: 0 }, PointNm { x: 2_000_000, y: 0 }],
            holes: vec![],
            width_nm: Some(120_000),
        });
    state.schematic_documents.panes.clear();
    state.ui.layout.focused = datum_gui_protocol::PaneId(1);
    state.supervision.journal.accepted_transaction_tip = None;
    state.schematic_authoring.source = Some(Schematic {
        uuid: sheet_id,
        sheets: [(sheet_id, source.clone())].into_iter().collect(),
        sheet_definitions: Default::default(),
        sheet_instances: Default::default(),
        variants: Default::default(),
        waivers: vec![],
    });
    state.schematic_authoring.delete_review = Some(DeleteReview {
        assignment: None,
        disposition: None,
        basis: datum_gui_protocol::AuthoringViewBasis {
            project,
            revision: scene.source_revision.clone(),
            pane: datum_gui_protocol::PaneId(1),
            content: datum_gui_protocol::PaneContent::Schematic,
            source: sheet_id,
            document: None,
        },
        source,
        scene: scene.clone(),
        targets: vec![
            DeleteTarget {
                class: DeleteClass::Wire,
                id: wire,
            },
            DeleteTarget {
                class: DeleteClass::Symbol,
                id: symbol,
            },
        ],
        proposal_id,
        revision: scene.source_revision.clone(),
        tip: None,
        pending: None,
        refusal: None,
        response: Some(NativeProposalResponse {
            assignment: None,
            disposition: None,
            proposal_id,
            status: ProposalStatus::Draft,
            prepared_against: scene.source_revision.clone(),
            current_model_revision: scene.source_revision.clone(),
            transaction_tip: None,
            deleted: vec![
                DeletedObject {
                    id: wire,
                    class: "wire".into(),
                    name: String::new(),
                },
                DeletedObject {
                    id: symbol,
                    class: "symbol".into(),
                    name: "R1".into(),
                },
            ],
            selected_count: Some(2),
            owned_count: Some(0),
            transaction_id: None,
        }),
    });
    (state, scene, wire, symbol)
}
#[test]
fn batch_delete_inline_review_uses_names_counts_and_exact_source_without_modal_or_cancel_write() {
    let (mut state, scene, wire, symbol) = review_fixture();
    let source = state.schematic_authoring.source.clone();
    let viewport = RectPx {
        x: 0.,
        y: 0.,
        width: 400.,
        height: 500.,
    };
    let camera = CameraState::fit_to_bounds(&scene.bounds);
    let projection = Projection::new(
        inset_rect(viewport, 10., 10., 10., 10.),
        &scene.bounds,
        camera,
    );
    let mut plan =
        native_selection_overlay::cue_plan::PanePlan::new(&state, viewport, camera).unwrap();
    let mut quads = vec![];
    let mut text = vec![];
    render(
        &mut quads,
        &mut text,
        &scene,
        viewport,
        &projection,
        &state,
        &mut plan,
    );
    assert!(!quads.is_empty());
    assert!(text.iter().any(|t| t.text == "R1"));
    assert!(text.iter().any(|t| t.text.contains("Tab accept deletion")));
    assert!(text.iter().any(|t| t.text.contains("2 selected")));
    assert!(
        !text
            .iter()
            .any(|t| t.text.contains(&wire.to_string()) || t.text.contains(&symbol.to_string()))
    );
    // Exercise the actual frame route: there is no Board pane to render
    // these commands, and the camera is installed after frame construction.
    let pane = state.ui.layout.open_beside_root(
        datum_gui_protocol::PaneContent::Schematic,
        datum_gui_protocol::SplitOrientation::Vertical,
        0.5,
        true,
    );
    state.ui.layout.zoomed = Some(pane);
    state.schematic_documents.project = None;
    state
        .schematic_authoring
        .delete_review
        .as_mut()
        .unwrap()
        .basis
        .pane = pane;
    state.schematic_scene = Some(
        state
            .schematic_authoring
            .delete_review
            .as_ref()
            .unwrap()
            .preview_scene(),
    );
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let mut frame = PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
    frame.set_surface_camera(pane, camera);
    frame.append_native_selection(&state).unwrap();
    assert!(
        frame
            .text_runs
            .iter()
            .any(|t| t.text.contains("Tab accept deletion"))
    );
    assert!(
        frame
            .text_runs
            .iter()
            .any(|t| t.text.contains("2 selected"))
    );
    assert!(
        frame
            .viewport_overlay_vertices
            .iter()
            .any(|v| v.color == PROPOSAL_BASE)
    );
    let narrow = RectPx {
        x: 20.,
        y: 20.,
        width: 60.,
        height: 55.,
    };
    let mut narrow_plan =
        native_selection_overlay::cue_plan::PanePlan::new(&state, narrow, camera).unwrap();
    let mut narrow_quads = Vec::new();
    let mut narrow_text = Vec::new();
    render(
        &mut narrow_quads,
        &mut narrow_text,
        &scene,
        narrow,
        &projection,
        &state,
        &mut narrow_plan,
    );
    assert!(!narrow_quads.is_empty());
    assert!(
        narrow_quads
            .iter()
            .all(|q| q.points.iter().all(|p| narrow.contains(p.0, p.1)))
    );
    let mut wrong = scene.clone();
    wrong.scene_id.push_str("other");
    quads.clear();
    text.clear();
    render(
        &mut quads,
        &mut text,
        &wrong,
        viewport,
        &projection,
        &state,
        &mut plan,
    );
    assert!(quads.is_empty() && text.is_empty());
    state.schematic_authoring.delete_review = None;
    render(
        &mut quads,
        &mut text,
        &scene,
        viewport,
        &projection,
        &state,
        &mut plan,
    );
    assert!(quads.is_empty() && text.is_empty());
    assert_eq!(state.schematic_authoring.source, source);
}
#[test]
fn erc_disposition_inline_review_uses_same_chip_without_deletion_ghosts() {
    use datum_gui_protocol::schematic_authoring::disposition::DispositionEvidence;
    use eda_engine::api::native_write::schematic_delete_proposal::{
        DispositionKind, DispositionSummary,
    };
    let (mut state, scene, _, _) = review_fixture();
    let evidence = DispositionEvidence {
        check_run_id: uuid::Uuid::new_v4(),
        finding_id: uuid::Uuid::new_v4(),
        fingerprint: "erc-pin".into(),
        rule_id: "undriven_input".into(),
        message: "J1 input intentionally open".into(),
        disposition: DispositionKind::Waiver,
        rationale: "External driver".into(),
    };
    state.checks = datum_gui_protocol::CheckRunReviewState {
        check_run_id: Some(evidence.check_run_id.to_string()),
        project_id: Some(scene.project_uuid.clone()),
        model_revision: Some(scene.source_revision.clone()),
        profile_id: Some("erc".into()),
        persisted: true,
        findings: vec![datum_gui_protocol::CheckFindingSummary {
            finding_id: Some(evidence.finding_id.to_string()),
            fingerprint: evidence.fingerprint.clone(),
            rule_id: evidence.rule_id.clone(),
            message: evidence.message.clone(),
            domain: "erc".into(),
            status: "active".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let review = state.schematic_authoring.delete_review.as_mut().unwrap();
    review.targets.clear();
    let response = review.response.as_mut().unwrap();
    response.deleted.clear();
    response.disposition = Some(DispositionSummary {
        disposition: evidence.disposition,
        disposition_id: uuid::Uuid::new_v4(),
        check_run_id: evidence.check_run_id,
        finding_id: evidence.finding_id,
        fingerprint: evidence.fingerprint.clone(),
        rule_id: evidence.rule_id.clone(),
        message: evidence.message.clone(),
        rationale: evidence.rationale.clone(),
    });
    review.disposition = Some(evidence);
    assert!(review.deleted_sources().is_empty());
    let viewport = RectPx {
        x: 0.,
        y: 0.,
        width: 540.,
        height: 500.,
    };
    let camera = CameraState::fit_to_bounds(&scene.bounds);
    let projection = Projection::new(viewport, &scene.bounds, camera);
    let mut plan =
        native_selection_overlay::cue_plan::PanePlan::new(&state, viewport, camera).unwrap();
    let mut quads = vec![];
    let mut text = vec![];
    render(
        &mut quads,
        &mut text,
        &scene,
        viewport,
        &projection,
        &state,
        &mut plan,
    );
    assert!(
        text.iter()
            .any(|t| t.text.contains("Waiver") && t.text.contains("undriven_input"))
    );
    assert!(text.iter().any(|t| t.text.contains("External driver")));
    assert!(
        text.iter()
            .any(|t| t.text.contains("Tab accept disposition"))
    );
    assert!(!text.iter().any(|t| t.text.contains("accept deletion")));
    state.checks.persisted = false;
    text.clear();
    render(
        &mut quads,
        &mut text,
        &scene,
        viewport,
        &projection,
        &state,
        &mut plan,
    );
    assert!(text.iter().any(|t| t.text.contains("Proposal unavailable")));
}
#[test]
fn part_assignment_inline_review_names_binding_without_deleting_symbol_geometry() {
    use datum_gui_protocol::schematic_authoring::assignment::AssignmentEvidence;
    use eda_engine::api::native_write::{
        schematic_assignment::{AssignmentChoice, AssignmentOptions},
        schematic_delete_proposal::AssignmentSummary,
    };
    use eda_engine::{
        ir::geometry::Point,
        schematic::{HiddenPowerBehavior, PlacedSymbol, SymbolDisplayMode},
    };
    let (mut state, scene, _, symbol) = review_fixture();
    let original = PlacedSymbol {
        uuid: symbol,
        reference: "R1".into(),
        value: "22k".into(),
        part: None,
        entity: None,
        gate: None,
        lib_id: None,
        pins: vec![],
        fields: vec![],
        position: Point::new(0, 0),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![],
        hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
    };
    let choice = AssignmentChoice {
        part: uuid::Uuid::new_v4(),
        part_name: "Resistor 22k".into(),
        entity: uuid::Uuid::new_v4(),
        entity_name: "Resistor".into(),
        gate: uuid::Uuid::new_v4(),
        gate_name: "Main".into(),
    };
    let review = state.schematic_authoring.delete_review.as_mut().unwrap();
    review.targets.clear();
    review.assignment = Some(AssignmentEvidence {
        target: datum_gui_protocol::AuthoredSelectionIdentity {
            id: symbol,
            class: datum_gui_protocol::AuthoredSelectionClass::Symbol,
            instance_path: vec![],
        },
        options: AssignmentOptions {
            sheet: review.source.uuid,
            original,
            component_instance: None,
            choices: vec![choice.clone()],
        },
        selected: 0,
    });
    let receipt = review.response.as_mut().unwrap();
    receipt.deleted.clear();
    receipt.assignment = Some(AssignmentSummary {
        symbol,
        sheet: review.source.uuid,
        instance_path: vec![],
        component_instance: uuid::Uuid::new_v4(),
        reference: "R1".into(),
        previous_part: None,
        choice: choice.clone(),
    });
    assert!(review.deleted_sources().is_empty());
    assert_eq!(review.preview_scene(), review.scene);
    let viewport = RectPx {
        x: 0.,
        y: 0.,
        width: 540.,
        height: 500.,
    };
    let camera = CameraState::fit_to_bounds(&scene.bounds);
    let projection = Projection::new(viewport, &scene.bounds, camera);
    let mut plan =
        native_selection_overlay::cue_plan::PanePlan::new(&state, viewport, camera).unwrap();
    let mut quads = vec![];
    let mut text = vec![];
    render(
        &mut quads,
        &mut text,
        &scene,
        viewport,
        &projection,
        &state,
        &mut plan,
    );
    assert!(
        text.iter()
            .any(|t| t.text.contains("Assign Part") && t.text.contains("R1"))
    );
    assert!(
        text.iter()
            .any(|t| t.text.contains("Resistor 22k") && t.text.contains("Main"))
    );
    assert!(
        text.iter()
            .any(|t| t.text.contains("Tab accept assignment"))
    );
    assert!(
        !text.iter().any(
            |t| t.text.contains(&choice.part.to_string()) || t.text.contains("accept deletion")
        )
    );
}
