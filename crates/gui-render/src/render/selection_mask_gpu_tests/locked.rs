//! Canonical locked-placement material proof; local renderer, not native WSI acceptance.
use super::*;
use crate::SceneSurface;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, CompoundSelection,
    SelectionSubject, selection_resolution::NativeSelectionResolution,
    selection_view::build_selection_view,
};

#[test]
#[ignore = "requires S5A_NATIVE_PARENT_FIXTURE; production GPU capture, no hardware claim"]
fn actual_locked_package_hatch_survives_suppressed_anchor_and_preserves_source() {
    let root = std::path::PathBuf::from(std::env::var_os("S5A_NATIVE_PARENT_FIXTURE").unwrap());
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let before: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let mut state = datum_gui_protocol::load_board_editor_workspace_state(
        &datum_gui_protocol::LiveReviewRequest {
            project_root: root.clone(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
    )
    .unwrap();
    let locked = state
        .scene
        .components
        .iter()
        .find(|c| c.locked)
        .unwrap()
        .clone();
    let id = |component: &datum_gui_protocol::ComponentBounds| Id {
        class: Class::Footprint,
        id: component.source_object_uuid.parse().unwrap(),
        instance_path: vec![],
    };
    let locked_id = id(&locked);
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let subject = SelectionSubject::Compound(
        CompoundSelection::new(std::collections::BTreeSet::from([locked_id.clone()]), None)
            .unwrap(),
    );
    let view = build_selection_view(
        &model,
        &authority,
        subject,
        &state.scene,
        state.schematic_scene.as_ref(),
        |_| true,
    )
    .unwrap();
    assert!(view.locked.contains(&locked_id));
    assert!(view.focus().is_none());
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let mut camera = crate::CameraState::fit_to_bounds(&state.scene.bounds);
    let initial = PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
    let field = crate::inset_rect(initial.scene_viewport, 10.0, 10.0, 10.0, 10.0);
    let scale = crate::Projection::new(field, &state.scene.bounds, camera).scale;
    camera.zoom *= 0.000035 / scale;
    camera.center_x_nm = locked.position.x as f32;
    camera.center_y_nm = locked.position.y as f32;
    let projection = crate::Projection::new(field, &state.scene.bounds, camera);
    let mut anchor = Vec::new();
    crate::native_selection_overlay::lock_cue::push_selected(
        &mut anchor,
        &view,
        &projection,
        initial.scene_viewport,
    );
    assert!(
        anchor.is_empty(),
        "compound without focus suppresses repeated padlocks"
    );
    state.selection = SelectionTarget::Native(Box::new(view));
    let selection = state.selection.clone();
    let mut host = OffscreenRenderer::new(1280, 800).unwrap();
    let mut frames = Vec::new();
    for hc in [false, true] {
        state.ui.global_preferences.high_contrast_noncolor = hc;
        let mut prepared =
            PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
        prepared.append_native_selection(&state).unwrap();
        let request = &prepared
            .selection_masks
            .iter()
            .find(|(_, s, _)| *s == SceneSurface::Board)
            .unwrap()
            .2;
        let SelectionTarget::Native(view) = &state.selection else {
            unreachable!()
        };
        let mut locked_material = 0;
        for span in retained
            .presentation
            .quads
            .iter()
            .chain(&retained.presentation.strokes)
            .chain(&retained.presentation.glyphs)
        {
            let Some(key) = span.key.as_deref() else {
                continue;
            };
            let Some(owner) =
                view.presentation_owner(&state.scene, datum_gui_protocol::PaneContent::Board, key)
            else {
                continue;
            };
            if span.role != crate::presentation_ranges::Role::Material {
                continue;
            }
            if view.locked.contains(owner) {
                assert_eq!(request.material_role(span), 5);
                locked_material += 1;
            } else {
                assert_eq!(request.material_role(span), 1);
            }
        }
        assert!(locked_material > 0);
        assert_eq!(request.high_contrast(), hc);
        frames.push(capture_retained(&mut host, &prepared, &retained));
    }
    let near_hatch = |p: &image::Rgba<u8>| {
        p.0[..3]
            .iter()
            .zip([154i16, 160, 168])
            .all(|(a, b)| (i16::from(*a) - b).abs() <= 2)
    };
    let pane = initial.scene_viewport;
    let hatch_count = |image: &RgbaImage| {
        image
            .enumerate_pixels()
            .filter(|(x, y, p)| pane.contains(*x as f32, *y as f32) && near_hatch(p))
            .count()
    };
    assert!(
        hatch_count(&frames[1]) > hatch_count(&frames[0]),
        "HC material hatch must survive glyph suppression"
    );
    assert_ne!(frames[0], frames[1]);
    if let Some(out) = std::env::var_os("S5A_LOCK_HC_CAPTURE_DIR") {
        let out = std::path::PathBuf::from(out);
        frames[0]
            .save(out.join("locked-compound-normal.png"))
            .unwrap();
        frames[1].save(out.join("locked-compound-hc.png")).unwrap();
    }
    assert_eq!(state.selection, selection);
    for (path, bytes) in before {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
