use super::*;
use datum_gui_protocol::selection_resolution::NativeSelectionResolution;
use datum_gui_protocol::selection_view::build_selection_view_for_documents;
use datum_gui_protocol::{AuthoringViewBasis, MoveHandleFeedback, SelectionSubject};

fn workspace() -> (
    super::super::tests::Fixture,
    ReviewWorkspaceState,
    BoardReviewSceneV1,
) {
    let (fixture, mut state) = super::super::tests::selected_wire_state();
    let model = eda_engine::substrate::ProjectResolver::new(fixture.root())
        .resolve()
        .unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let SelectionTarget::Native(view) = &state.selection else {
        panic!("native fixture")
    };
    let subject = view.subject.clone();
    state.schematic_authoring =
        datum_gui_protocol::schematic_authoring::SchematicAuthoring::from_model(&model).unwrap();
    let mut documents = datum_gui_protocol::schematic_documents::SchematicDocuments::from_source(
        model.project.project_id,
        model.model_revision.clone(),
        "Wire handles".into(),
        authority.snapshot().schematic_source().unwrap().clone(),
    )
    .unwrap();
    let key = documents.rows[0].key.clone();
    documents
        .prepare_scene(&key, fixture.root(), &model)
        .unwrap();
    let pane = documents.open_beside(&mut state.ui.layout, &key).unwrap();
    state.ui.layout.focused = pane;
    let scene = documents.scene_for_pane(pane).unwrap().clone();
    state.selection = SelectionTarget::Native(Box::new(
        build_selection_view_for_documents(
            &model,
            &authority,
            subject,
            &state.scene,
            &[&scene],
            |_| true,
        )
        .unwrap(),
    ));
    state.schematic_documents = documents;
    state.schematic_scene = Some(scene.clone());
    state.library =
        datum_gui_protocol::library_browser::LibraryBrowser::capture(std::sync::Arc::new(model));
    (fixture, state, scene)
}

fn pass(
    state: &ReviewWorkspaceState,
    scene: &BoardReviewSceneV1,
    zoom: f32,
) -> PreparedSurfacePass {
    let mut camera = CameraState::fit_to_bounds(&scene.bounds);
    camera.zoom = zoom;
    PreparedSurfacePass {
        pane_id: state.ui.layout.focused,
        surface: SceneSurface::Schematic,
        schematic_document: scene.schematic_document.clone(),
        scene_viewport: RectPx {
            x: 0.,
            y: 0.,
            width: 800.,
            height: 600.,
        },
        bounds: scene.bounds.clone(),
        camera,
        grid_lod_previous: Default::default(),
        grid_lod_resolved: Default::default(),
    }
}

fn projection(pass: &PreparedSurfacePass) -> Projection {
    Projection::new(
        inset_rect(pass.scene_viewport, 10., 10., 10., 10.),
        &pass.bounds,
        pass.camera,
    )
}

#[test]
fn selected_grips_share_node_priority_and_cached_hover_without_authored_writes() {
    let (fixture, state, scene) = workspace();
    let before = fixture.source_bytes();
    let pane = state.ui.layout.focused;
    let handles = selected_handles(&state, &scene, pane);
    assert!(handles.iter().any(|h| h.kind == Kind::Endpoint));
    assert!(handles.iter().any(|h| h.kind == Kind::Junction));
    // The existing canonical fixture has a genuine Junction at its Wire's from end.
    let at_origin = handles
        .iter()
        .find(|h| h.point == PointNm { x: 0, y: 0 })
        .unwrap();
    assert_eq!(at_origin.kind, Kind::Junction);
    let catalog = Catalog::capture(&state);
    assert!(catalog.is_current(&state));
    let pass = pass(&state, &scene, 1.);
    let at = project_point(at_origin.point, &projection(&pass));
    assert_eq!(catalog.hover_kind(&pass, at), Some(Kind::Junction));
    let mut glyphs = Vec::new();
    push_selected(
        &mut glyphs,
        &scene,
        &state,
        pane,
        &projection(&pass),
        pass.scene_viewport,
    );
    assert!(!glyphs.is_empty());
    assert!(
        glyphs
            .iter()
            .flat_map(|q| q.points)
            .all(|(x, y)| pass.scene_viewport.contains(x, y))
    );
    assert_eq!(fixture.source_bytes(), before);
}

#[test]
fn stale_locked_hidden_foreign_and_nonobject_ownership_do_not_gain_grips() {
    let (fixture, state, scene) = workspace();
    let before = fixture.source_bytes();
    let pane = state.ui.layout.focused;
    for mode in 0..6 {
        let mut changed = state.clone();
        let SelectionTarget::Native(view) = &mut changed.selection else {
            unreachable!()
        };
        let SelectionSubject::Object(id) = &view.subject else {
            unreachable!()
        };
        let id = id.clone();
        match mode {
            0 => view.stale = true,
            1 => {
                view.locked.insert(id);
            }
            2 => {
                view.hidden.insert(id);
            }
            3 => {
                view.subject =
                    SelectionSubject::Object(datum_gui_protocol::AuthoredSelectionIdentity {
                        instance_path: vec![uuid::Uuid::new_v4()],
                        ..id
                    });
            }
            4 => changed.ui.filters.show_authored = false,
            _ => view.subject = SelectionSubject::Run(id.into()),
        }
        assert!(
            selected_handles(&changed, &scene, pane).is_empty(),
            "case {mode}"
        );
    }
    let mut stale_scene = scene.clone();
    stale_scene.source_revision.push_str("-stale");
    assert!(selected_handles(&state, &stale_scene, pane).is_empty());
    assert_eq!(fixture.source_bytes(), before);
}

#[test]
fn glyphs_are_screen_constant_and_node_ring_preserves_the_authored_core() {
    let mut outline = Vec::new();
    glyph(&mut outline, (20., 20.), Kind::Endpoint, false);
    let bounds = |quads: &[Quad]| {
        quads
            .iter()
            .flat_map(|q| q.points)
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), (x, _)| {
                (lo.min(x), hi.max(x))
            })
    };
    let (lo, hi) = bounds(&outline);
    assert_eq!(hi - lo, 6.);
    let mut node = Vec::new();
    glyph(&mut node, (20., 20.), Kind::Junction, true);
    assert!(!node.is_empty());
    assert!(
        node.iter()
            .all(|q| !bounds_from_projected_points(&q.points).contains(20., 20.))
    );
    let (fixture, state, scene) = workspace();
    let before = fixture.source_bytes();
    for zoom in [0.5, 4.] {
        let pass = pass(&state, &scene, zoom);
        let endpoint = selected_handles(&state, &scene, pass.pane_id)
            .into_iter()
            .find(|h| h.kind == Kind::Endpoint)
            .unwrap();
        let mut quads = Vec::new();
        glyph(
            &mut quads,
            project_point(endpoint.point, &projection(&pass)),
            Kind::Endpoint,
            false,
        );
        let (lo, hi) = bounds(&quads);
        assert!((hi - lo - 6.).abs() < 0.001);
    }
    assert_eq!(fixture.source_bytes(), before);
}

#[test]
fn visible_grip_corners_remain_capturable_after_zooming_out() {
    let (fixture, state, scene) = workspace();
    let before = fixture.source_bytes();
    let pass = pass(&state, &scene, 0.01);
    let endpoint = selected_handles(&state, &scene, pass.pane_id)
        .into_iter()
        .find(|h| h.kind == Kind::Endpoint)
        .unwrap();
    let at = project_point(endpoint.point, &projection(&pass));
    let corner = (at.0 + 2.5, at.1 + 2.5);
    let catalog = Catalog::capture(&state);
    let (key, identity, handle) = catalog.pick(&pass, corner).unwrap();
    assert_eq!(identity, endpoint.identity);
    assert_eq!(key, endpoint.key);
    assert_eq!(handle, endpoint.handle);
    let world =
        crate::coordinate_hit::editor_viewport(pass.pane_id, pass.surface, &projection(&pass))
            .screen_to_world(datum_gui_protocol::ScreenPointPx {
                x: corner.0,
                y: corner.1,
            })
            .unwrap();
    assert!(
        !crate::schematic_anchor_contains(endpoint.width, endpoint.point, world),
        "this corner must exercise screen-constant grip capture, not the old world envelope"
    );
    assert_eq!(fixture.source_bytes(), before);
}

#[test]
fn armed_feedback_stays_in_the_qualified_source_and_clears_after_cancel() {
    let (_fixture, mut state, scene) = workspace();
    let pane = state.ui.layout.focused;
    let basis = AuthoringViewBasis::capture_qualified_schematic(&state, pane).unwrap();
    state.authoring.gesture = datum_gui_protocol::AuthoringGestureState {
        preset: None,
        tool: datum_gui_protocol::WorkspaceTool::Move,
        anchor: Some(PointNm { x: 0, y: 0 }),
        preview: Some(PointNm {
            x: 500000,
            y: 300001,
        }),
        target_object_id: Some("schematic-wiring-fixture".into()),
        view_basis: Some(basis.clone()),
    };
    state.authoring.handle_feedback = Some(MoveHandleFeedback {
        basis,
        kind: Kind::Endpoint,
        source: PointNm { x: 0, y: 0 },
        destination: PointNm {
            x: 500000,
            y: 300001,
        },
        snap_target: Some(PointNm {
            x: 500000,
            y: 300001,
        }),
    });
    let pass = pass(&state, &scene, 1.);
    let mut quads = Vec::new();
    push_selected(
        &mut quads,
        &scene,
        &state,
        pane,
        &projection(&pass),
        pass.scene_viewport,
    );
    assert!(!quads.is_empty());
    let mut foreign = scene.clone();
    foreign.schematic_document.as_mut().unwrap().source_sheet = uuid::Uuid::new_v4();
    let mut absent = Vec::new();
    push_selected(
        &mut absent,
        &foreign,
        &state,
        pane,
        &projection(&pass),
        pass.scene_viewport,
    );
    assert!(absent.is_empty());
    state.cancel_authoring_gesture();
    assert!(state.authoring.handle_feedback.is_none());
}
