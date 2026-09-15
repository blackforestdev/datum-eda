use super::*;

#[test]
fn resolving_grid_lod_never_rebuilds_retained_geometry() {
    let bounds = datum_gui_protocol::SceneBounds {
        min_x: 0,
        min_y: 0,
        max_x: 10_000_000,
        max_y: 10_000_000,
    };
    let viewport = RectPx {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    let before = retained_scene_resolve_count();
    let first = resolve_surface_grid_lod(
        SceneSurface::Board,
        viewport,
        &bounds,
        CameraState::fit_to_bounds(&bounds),
        datum_gui_viewport::GridLodState::default(),
    );
    let _next = resolve_surface_grid_lod(
        SceneSurface::Board,
        viewport,
        &bounds,
        CameraState::fit_to_bounds(&bounds),
        first,
    );
    assert_eq!(retained_scene_resolve_count(), before);
}

// Slice S1b: the companion schematic pass draws a subtle SQUARE grid as an
// IMMEDIATE screen-space pass (shared `GridEngine`, `ScreenConstant` weight).
// This is a structural check that `push_schematic_grid` produces grid geometry
// in the schematic `#sgrid` whisper colours (major + minor at the fine tier)
// and that every emitted quad is one of those two colours — the grid must never
// borrow the board grid palette.
#[test]
fn schematic_grid_emits_square_underlay_geometry() {
    // A tight viewport-over-bounds so `detail_tier` lands on Fine (both major
    // and minor tiers active): 100px over 1mm reads well past 18px/mm.
    let projection = Projection::new(
        RectPx {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        &datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 1_000_000,
            max_y: 1_000_000,
        },
        CameraState::fit_to_bounds(&datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 1_000_000,
            max_y: 1_000_000,
        }),
    );
    let mut out = Vec::new();
    push_schematic_grid(&mut out, &projection);
    assert!(
        !out.is_empty(),
        "the schematic pass must emit a grid underlay"
    );
    let has_major = out.iter().any(|q| q.color == SCHEMATIC_GRID_MAJOR);
    let has_minor = out.iter().any(|q| q.color == SCHEMATIC_GRID_MINOR);
    assert!(
        has_major && has_minor,
        "the schematic grid must draw both major and minor lines at the fine tier"
    );
    assert!(
        out.iter()
            .all(|q| q.color == SCHEMATIC_GRID_MAJOR || q.color == SCHEMATIC_GRID_MINOR),
        "every schematic grid quad must use a schematic grid colour, never the board palette"
    );
    // The grid is square: emitted as screen-space pixel rects, it produces both
    // vertical (tall, narrow bbox) and horizontal (wide, short bbox) quads.
    let bbox = |q: &Quad| {
        let xs = q.points.iter().map(|p| p.0);
        let ys = q.points.iter().map(|p| p.1);
        let (min_x, max_x) = xs
            .clone()
            .fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(v), b.max(v)));
        let (min_y, max_y) = ys.fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(v), b.max(v)));
        (max_x - min_x, max_y - min_y)
    };
    assert!(
        out.iter().any(|q| {
            let (w, h) = bbox(q);
            h > w
        }) && out.iter().any(|q| {
            let (w, h) = bbox(q);
            w > h
        }),
        "a square grid must emit both vertical and horizontal screen-space lines"
    );
}

// Slice S1b THE FIX: the schematic grid's line weight must be SCREEN-CONSTANT —
// a fixed device pixel at any schematic zoom — because it now emits through the
// shared `GridEngine` with a `WeightClass::ScreenConstant(1.0)` weight instead
// of a world-nm width the GPU re-scaled by the camera (the old bake, which
// thickened on zoom-in). Render the grid at two very different camera zooms and
// assert every emitted line keeps a 1.0-device-px cross-section (vertical lines
// 1px wide; horizontal lines 1px tall). Before this slice the projected width
// grew with `camera.zoom`.
#[test]
fn schematic_grid_weight_is_screen_constant_across_zoom() {
    let bounds = datum_gui_protocol::SceneBounds {
        min_x: 0,
        min_y: 0,
        max_x: 1_000_000,
        max_y: 1_000_000,
    };
    let viewport = RectPx {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };
    let cross_section = |zoom: f32| -> Vec<f32> {
        let camera = CameraState {
            zoom,
            center_x_nm: 0.0,
            center_y_nm: 0.0,
        };
        let projection = Projection::new(viewport, &bounds, camera);
        let mut out = Vec::new();
        push_schematic_grid(&mut out, &projection);
        out.iter()
            .map(|q| {
                let xs = q.points.iter().map(|p| p.0);
                let ys = q.points.iter().map(|p| p.1);
                let (min_x, max_x) =
                    xs.fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(v), b.max(v)));
                let (min_y, max_y) =
                    ys.fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(v), b.max(v)));
                // The 1-device-px cross-section is the SHORTER side of each line
                // rect (the long side spans the viewport).
                (max_x - min_x).min(max_y - min_y)
            })
            .collect()
    };
    for zoom in [1.0_f32, 8.0] {
        let widths = cross_section(zoom);
        assert!(!widths.is_empty(), "grid must render at zoom {zoom}");
        assert!(
            widths.iter().all(|w| (*w - 1.0).abs() < 1e-4),
            "at zoom {zoom} every schematic grid line must stay 1.0 device px \
             (screen-constant), got {widths:?}"
        );
    }
}
