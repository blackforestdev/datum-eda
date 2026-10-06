use super::*;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, CompoundSelection,
    selection_resolution::NativeSelectionResolution, selection_view::build_selection_view,
};

fn viewport() -> RectPx {
    RectPx {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 300.0,
    }
}
fn extent(quads: &[Quad]) -> (f32, f32, f32, f32) {
    quads.iter().flat_map(|q| q.points).fold(
        (f32::MAX, f32::MAX, f32::MIN, f32::MIN),
        |(a, b, c, d), (x, y)| (a.min(x), b.min(y), c.max(x), d.max(y)),
    )
}

#[test]
fn asset_matches_code_geometry() {
    // The drawing above is this exact keyline asset; a redesign must change both.
    for required in [
        r#"viewBox="0 0 24 24""#,
        r#"stroke="currentColor""#,
        r#"stroke-width="1.7""#,
        r#"stroke-linecap="round""#,
        r#"stroke-linejoin="round""#,
        r#"<rect x="5" y="11" width="14" height="9" rx="1.5"/>"#,
        r#"<path d="M8 11V8a4 4 0 0 1 8 0v3"/>"#,
        r#"<circle cx="12" cy="15" r="1.2"/>"#,
    ] {
        assert!(ASSET.contains(required), "asset lacks {required}");
    }
    assert_eq!(ASSET.matches('<').count(), 5, "no undeclared asset element");
    let icons = datum_gui_protocol::load_default_gui_icon_set().unwrap();
    let entry = &icons.icons["selection-lock-anchor"];
    assert_eq!(
        (entry.source.as_str(), entry.status.as_str()),
        ("eda", "exists")
    );
    assert_eq!(entry.asset.as_deref(), Some("selection-lock-anchor.svg"));
}

#[test]
fn glyph_is_whole_neutral_and_centred_or_absent_at_a_pane_edge() {
    let mut quads = Vec::new();
    push_at(&mut quads, (200.0, 150.0), viewport());
    assert!(!quads.is_empty());
    assert!(quads.iter().all(|q| q.color == COLOR));
    let (x0, y0, x1, y1) = extent(&quads);
    // Body spans x 5..19, shackle top y 4, body bottom y 20, plus half strokes.
    let half_stroke = STROKE_UNITS * PX_PER_UNIT * 0.5;
    let near = |a: f32, b: f32| (a - b).abs() < 0.5;
    assert!(near(x0, 200.0 - 7.0 * PX_PER_UNIT - half_stroke));
    assert!(near(x1, 200.0 + 7.0 * PX_PER_UNIT + half_stroke));
    assert!(near(y0, 150.0 - 8.0 * PX_PER_UNIT - half_stroke));
    assert!(near(y1, 150.0 + 8.0 * PX_PER_UNIT + half_stroke));
    let mut clipped = Vec::new();
    push_at(&mut clipped, (3.0, 150.0), viewport());
    assert!(clipped.is_empty(), "no partial padlock at a pane edge");
}

#[test]
fn hover_anchors_cover_locked_parent_and_owned_pads_only() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    for c in &mut state.scene.components {
        c.locked = false;
    }
    assert_eq!(HoverAnchors::build(&state.scene), HoverAnchors::default());
    let locked = state.scene.components[0].clone();
    state.scene.components[0].locked = true;
    let anchors = HoverAnchors::build(&state.scene);
    assert_eq!(anchors.get(&locked.object_id), Some(locked.position));
    for pad in &state.scene.pads {
        let expected = (pad.component_uuid == locked.component_uuid).then_some(locked.position);
        assert_eq!(anchors.get(&pad.object_id), expected);
    }
    for other in &state.scene.components[1..] {
        assert_eq!(anchors.get(&other.object_id), None);
    }
}

#[test]
#[ignore = "requires frozen library-parents native recipe (S5A_NATIVE_PARENT_FIXTURE)"]
fn actual_locked_package_shows_padlock_for_object_focus_and_hover_only() {
    let root = std::path::PathBuf::from(
        std::env::var_os("S5A_NATIVE_PARENT_FIXTURE").expect("explicit frozen recipe"),
    );
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let before: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let state = datum_gui_protocol::load_board_editor_workspace_state(
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
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    // The frozen recipe's Package lock bits reach this scene (native_parent_recipe).
    let package = |locked: bool| {
        let c = state
            .scene
            .components
            .iter()
            .find(|c| c.locked == locked)
            .unwrap();
        Id {
            class: Class::Footprint,
            id: c.source_object_uuid.parse().unwrap(),
            instance_path: vec![],
        }
    };
    let (locked, unlocked) = (package(true), package(false));
    let projected = state
        .scene
        .components
        .iter()
        .find(|c| c.source_object_uuid == locked.id.to_string())
        .unwrap();
    assert!(projected.locked);
    let projection = Projection::new(
        viewport(),
        &state.scene.bounds,
        CameraState::fit_to_bounds(&state.scene.bounds),
    );
    let glyphs = |subject| {
        let view = build_selection_view(
            &model,
            &authority,
            subject,
            &state.scene,
            state.schematic_scene.as_ref(),
            |_| true,
        )
        .unwrap();
        let mut quads = Vec::new();
        push_selected(&mut quads, &view, &projection, viewport());
        (view, quads)
    };
    let (view, single) = glyphs(SelectionSubject::Object(locked.clone()));
    assert!(view.locked.contains(&locked));
    assert_eq!(view.authored_anchor(&locked), Some(projected.position));
    let (x0, y0, x1, y1) = extent(&single);
    let at = projection.project_point(projected.position);
    assert!(((x0 + x1) * 0.5 - at.0).abs() < 0.5);
    assert!(((y0 + y1) * 0.5 - at.1).abs() < 0.5);
    assert!(
        glyphs(SelectionSubject::Object(unlocked.clone()))
            .1
            .is_empty()
    );
    let members = std::collections::BTreeSet::from([locked.clone(), unlocked.clone()]);
    let compound =
        |focus| SelectionSubject::Compound(CompoundSelection::new(members.clone(), focus).unwrap());
    assert!(glyphs(compound(None)).1.is_empty(), "no glyph field");
    assert!(glyphs(compound(Some(unlocked.clone()))).1.is_empty());
    assert_eq!(glyphs(compound(Some(locked.clone()))).1, single);
    let anchors = HoverAnchors::build(&state.scene);
    assert_eq!(anchors.get(&projected.object_id), Some(projected.position));
    let mut hover = Vec::new();
    push_hover(
        &mut hover,
        anchors.get(&projected.object_id),
        &projection,
        viewport(),
    );
    assert_eq!(hover, single);
    for (path, bytes) in before {
        assert_eq!(std::fs::read(&path).unwrap(), bytes, "read-only cue");
    }
}

/// Production-renderer captures for same-size HUMAN review; never a golden.
#[cfg(feature = "visual")]
#[test]
#[ignore = "requires S5A_NATIVE_PARENT_FIXTURE and S5A_LOCK_CUE_CAPTURE_DIR"]
fn actual_locked_package_padlock_capture_for_review() {
    let root = std::path::PathBuf::from(std::env::var_os("S5A_NATIVE_PARENT_FIXTURE").unwrap());
    let out = std::path::PathBuf::from(std::env::var_os("S5A_LOCK_CUE_CAPTURE_DIR").unwrap());
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
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
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let locked = state.scene.components.iter().find(|c| c.locked).unwrap();
    let locked_key = locked.object_id.clone();
    let id = Id {
        class: Class::Footprint,
        id: locked.source_object_uuid.parse().unwrap(),
        instance_path: vec![],
    };
    let view = build_selection_view(
        &model,
        &authority,
        SelectionSubject::Object(id),
        &state.scene,
        state.schematic_scene.as_ref(),
        |_| true,
    )
    .unwrap();
    let mut renderer = crate::visual_capture::OffscreenRenderer::new(1280, 800).unwrap();
    renderer
        .render_workspace(&state, None)
        .unwrap()
        .save(out.join("lock-cue-none.png"))
        .unwrap();
    state.selection = crate::SelectionTarget::Native(Box::new(view));
    renderer
        .render_workspace(&state, None)
        .unwrap()
        .save(out.join("lock-cue-selected.png"))
        .unwrap();
    state.selection = crate::SelectionTarget::None;
    state.ui.hovered_object = Some(datum_gui_protocol::HoverTarget {
        object_id: locked_key,
        surface: datum_gui_protocol::PaneContent::Board,
    });
    renderer
        .render_workspace(&state, None)
        .unwrap()
        .save(out.join("lock-cue-hover.png"))
        .unwrap();
}
