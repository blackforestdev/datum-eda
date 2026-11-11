//! Production retained/prepared regression for a one-millimeter drawing line.
use super::*;
use datum_gui_protocol::{PaneContent, SceneBounds, SceneLayer};

fn drawing_state(from: PointNm, to: PointNm) -> ReviewWorkspaceState {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let mut scene = state.scene.clone();
    scene.scene_id = "edge-stroke-display-regression".into();
    scene.bounds = SceneBounds {
        min_x: from.x.min(to.x).saturating_sub(5_000_000),
        min_y: from.y.min(to.y).saturating_sub(5_000_000),
        max_x: from.x.max(to.x).saturating_add(5_000_000),
        max_y: from.y.max(to.y).saturating_add(5_000_000),
    };
    scene.layers = vec![SceneLayer {
        layer_id: "L201".into(),
        name: "Schematic.Symbol".into(),
        kind: "schematic".into(),
        render_order: 2,
        visible_by_default: true,
    }];
    scene.outline.clear();
    scene.components.clear();
    scene.component_graphics.clear();
    scene.component_texts.clear();
    scene.pads.clear();
    scene.tracks.clear();
    scene.vias.clear();
    scene.zones.clear();
    scene.board_texts.clear();
    scene.board_text_geometries.clear();
    scene.glyph_mesh_assets.clear();
    scene.unrouted_primitives.clear();
    scene.net_display.clear();
    scene.proposal_overlay_primitives.clear();
    scene.review_primitives.clear();
    scene.board_graphics = vec![BoardGraphicPrimitive {
        object_id: "schematic-drawing-line:edge-regression".into(),
        source_object_uuid: "edge-regression".into(),
        object_kind: "schematic_drawing".into(),
        primitive_kind: "line".into(),
        layer_id: "L201".into(),
        path: vec![from, to],
        holes: vec![],
        width_nm: Some(120_000),
    }];
    state.schematic_scene = Some(scene);
    state.ui.layout.set_focused_content(PaneContent::Schematic);
    state
}

#[test]
fn edge_stroke_visibility_retained_admission_preserves_source_and_camera_reuse() {
    for (from, to) in [
        (PointNm { x: 0, y: 0 }, PointNm { x: 1_000_000, y: 0 }),
        (
            PointNm {
                x: -10_000_000,
                y: -5_000_000,
            },
            PointNm {
                x: -9_000_000,
                y: -5_000_000,
            },
        ),
        (
            PointNm {
                x: i64::MAX - 1_000_000,
                y: i64::MAX,
            },
            PointNm {
                x: i64::MAX,
                y: i64::MAX,
            },
        ),
        (
            PointNm {
                x: i64::MIN,
                y: i64::MIN,
            },
            PointNm {
                x: i64::MIN + 1_000_000,
                y: i64::MIN,
            },
        ),
    ] {
        let state = drawing_state(from, to);
        let source = state.schematic_scene.clone().unwrap();
        let board = RetainedScene::from_workspace(&state, 1280, 800);
        let retained =
            RetainedScene::from_workspace_schematic_for_surface(&state, 1280, 800, 1.0).unwrap();
        let mut prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &board,
        )
        .unwrap();
        prepared.schematic_sources = crate::renderer_state::SchematicSources::capture(
            &state,
            Some(&retained),
            &Default::default(),
            1280,
            800,
            1.0,
        )
        .unwrap();
        let pass = prepared
            .surface_passes()
            .iter()
            .find(|p| p.surface == SceneSurface::Schematic)
            .unwrap()
            .clone();
        let counts = prepared
            .geometry_admission(&board, Some(&retained))
            .find(|p| p.pane_id == pass.pane_id)
            .unwrap()
            .counts()
            .unwrap();
        assert_eq!(counts.retained_stroke_instances, 1);
        assert_eq!(counts.prepared_stroke_instances, 1);
        let strokes = retained.world_strokes();
        let before = strokes[0];
        let delta = [
            (before.to[0] - before.from[0]) + (before.to_low[0] - before.from_low[0]),
            (before.to[1] - before.from[1]) + (before.to_low[1] - before.from_low[1]),
        ];
        assert_eq!(
            delta,
            [1_000_000.0, 0.0],
            "actual retained segment must not collapse"
        );
        let field = inset_rect(pass.scene_viewport, 10.0, 10.0, 10.0, 10.0);
        let projection = Projection::new(field, &source.bounds, pass.camera);
        let a = projection.project_point(from);
        let b = projection.project_point(to);
        let expected_length = 1_000_000.0 * projection.scale;
        assert!((b.0 - a.0 - expected_length).abs() <= f32::EPSILON * field.width * 2.0);
        assert_eq!(a.1, b.1);
        // The world pass clips to the pane viewport, while Fit uses a 10px
        // inset. Camera ULP rounding can put a boundary endpoint just beyond
        // that inset; it must remain inside the actual production clip.
        assert!(pass.scene_viewport.contains(a.0, a.1));
        assert!(pass.scene_viewport.contains(b.0, b.1));
        // Compare CPU placement with the camera-relative retained attributes
        // consumed by the GPU. Error allowance is the camera f64 ULP plus
        // screen f32 rounding, not a new product pixel threshold.
        for (point, high, low) in [
            (a, before.from, before.from_low),
            (b, before.to, before.to_low),
        ] {
            for (axis, center) in [pass.camera.center_x_nm, pass.camera.center_y_nm]
                .into_iter()
                .enumerate()
            {
                let camera_high = center as f32;
                let camera_low = (center - f64::from(camera_high)) as f32;
                let relative = (high[axis] - camera_high) + (low[axis] - camera_low);
                let (screen_center, actual) = if axis == 0 {
                    (field.x + field.width * 0.5, point.0)
                } else {
                    (field.y + field.height * 0.5, point.1)
                };
                let gpu = screen_center + relative * projection.scale;
                let allowance = (f64::EPSILON * center.abs().max(1.0) * 2.0) as f32
                    * projection.scale
                    + f32::EPSILON * (field.x + field.y + field.width + field.height) * 2.0;
                assert!(
                    (gpu - actual).abs() <= allowance,
                    "CPU/GPU placement disagree: {gpu} {actual}"
                );
            }
        }
        let address = strokes.as_ptr();
        let mut camera = pass.camera;
        camera.zoom = 2.0;
        camera.pan_pixels(pass.scene_viewport, &source.bounds, 20.0, -10.0);
        prepared.set_surface_camera(pass.pane_id, camera);
        assert_eq!(retained.world_strokes().as_ptr(), address);
        assert_eq!(retained.world_strokes()[0], before);
        assert_eq!(state.schematic_scene.as_ref().unwrap(), &source);
        assert_eq!(before.resolved_width_px(0.0001), 12.0);
        assert_eq!(before.resolved_width_px(0.000001), 1.0);
        let mut hidden = state.clone();
        hidden
            .ui
            .filters
            .layer_visibility
            .insert("L201".into(), false);
        assert!(retained.visible_draw_commands(&hidden).is_empty());
    }
}
