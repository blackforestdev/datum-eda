// Immediate screen-space interaction overlays (slice S4 — spec §2 HoverEngine +
// §4.2 weight table): the hover pre-highlight and the cursor crosshair.
//
// Both are strictly class-A `ScreenConstant` chrome (spec §4.1): a fixed device-
// pixel weight resolved every frame against the live camera and NEVER emitted into
// a retained world (nm) buffer — so they do not scale with zoom and preserve the
// `render == CAM` law. They are driven by the live cursor/hover, which is absent in
// the offscreen visual-test capture, so every builder here emits nothing when its
// inputs are `None` and the board frame stays byte-identical.
//
// This is a real `#[path] mod` child of the crate root (declared in `scene.rs`), a
// DESCENDANT of the module that defines `PreparedScene`/`RetainedScene` and the
// private `Projection`/`WorldHitShape`/`WorldHitRegion` types — so it reaches them
// via `use super::*` exactly as the sibling `coordinate_hit` module does.

use super::*;
use datum_gui_protocol::CrosshairStyle;
use datum_gui_viewport::WeightClass;

/// Hover pre-highlight ring weight — spec §4.2 "Hover pre-highlight | A | 1.5 px".
const HOVER_WEIGHT: WeightClass = WeightClass::ScreenConstant(1.5);
/// Cursor crosshair weight — spec §4.2 "Cursor crosshair | A | 1.0 px".
const CROSSHAIR_WEIGHT: WeightClass = WeightClass::ScreenConstant(1.0);
/// Screen-px inset so the hover ring sits just OUTSIDE the hovered object's
/// projected bbox — a subtle halo rather than a stroke that fights the object's
/// own outline. Also lets the ring stay readable when the schematic overlay rides
/// the underlay pass (drawn beneath the world geometry).
const HOVER_MARGIN_PX: f32 = 3.0;
/// Half-length of each crosshair arm (a small LOCAL crosshair at the cursor, the
/// least-intrusive option — see the design flag in the slice report).
const CROSSHAIR_ARM_PX: f32 = 14.0;

impl PreparedScene {
    pub(crate) fn interaction_viewport(&self, surface: SceneSurface) -> Option<RectPx> {
        let (x, y) = self.crosshair_cursor_screen?;
        self.surface_passes
            .iter()
            .find(|pass| pass.surface == surface && pass.scene_viewport.contains(x, y))
            .map(|pass| pass.scene_viewport)
    }

    /// Override the companion schematic pass camera while keeping its immediate
    /// grid and interaction buffers projected through the same warm camera.
    pub fn set_schematic_camera(&mut self, camera: CameraState) {
        self.schematic_camera = camera;
        if let Some(pass) = self
            .surface_passes
            .iter_mut()
            .find(|pass| pass.surface == SceneSurface::Schematic)
        {
            pass.camera = camera;
        }
        self.schematic_underlay_vertices = build_schematic_grid_vertices(
            self.schematic_scene_viewport,
            &self.schematic_bounds,
            self.schematic_camera,
        );
        self.schematic_overlay_vertices = build_schematic_interaction_vertices(
            self.schematic_scene_viewport,
            &self.schematic_bounds,
            self.schematic_camera,
            self.schematic_hover_bounds_nm,
            self.crosshair_cursor_screen,
            self.crosshair_style,
        );
    }

    /// Refresh high-frequency pointer chrome without rebuilding retained world
    /// geometry or reconstructing the prepared shell.
    pub fn refresh_interaction(
        &mut self,
        state: &ReviewWorkspaceState,
        board_retained: &RetainedScene,
    ) {
        let board_hover = state.ui.hovered_object.as_ref().and_then(|hover| {
            (hover.surface == datum_gui_protocol::PaneContent::Board)
                .then(|| board_hover_bounds(board_retained, &hover.object_id))
                .flatten()
        });
        self.schematic_hover_bounds_nm = state.ui.hovered_object.as_ref().and_then(|hover| {
            (hover.surface == datum_gui_protocol::PaneContent::Schematic)
                .then(|| {
                    state
                        .schematic_scene
                        .as_ref()
                        .and_then(|scene| schematic_symbol_bounds(scene, &hover.object_id))
                })
                .flatten()
        });
        self.crosshair_cursor_screen = state.ui.cursor_pos.map(|point| (point.x, point.y));
        self.crosshair_style = state.ui.crosshair_style;

        let mut board = Vec::new();
        let active = self.crosshair_cursor_screen.and_then(|(x, y)| {
            self.surface_passes
                .iter()
                .find(|pass| pass.scene_viewport.contains(x, y))
                .cloned()
        });
        if let Some(pass) = active.as_ref().filter(|pass| pass.surface == SceneSurface::Board) {
            let field = inset_rect(pass.scene_viewport, 10.0, 10.0, 10.0, 10.0);
            let projection = Projection::new(field, &pass.bounds, pass.camera);
            push_pane_interaction(
                &mut board,
                &projection,
                pass.scene_viewport,
                board_hover,
                self.crosshair_cursor_screen,
                self.crosshair_style,
            );
        }
        self.board_interaction_vertices = quads_to_vertices(&board);
        self.schematic_overlay_vertices = active
            .as_ref()
            .filter(|pass| pass.surface == SceneSurface::Schematic)
            .map_or_else(Vec::new, |pass| {
                build_schematic_interaction_vertices(
                    Some(pass.scene_viewport),
                    &pass.bounds,
                    pass.camera,
                    self.schematic_hover_bounds_nm,
                    self.crosshair_cursor_screen,
                    self.crosshair_style,
                )
            });
    }
}

/// Push the class-A hover pre-highlight ring for a screen-space bounding rect.
fn push_hover_ring(out: &mut Vec<Quad>, screen_rect: RectPx, scale_px_per_nm: f32) {
    let ring = RectPx {
        x: screen_rect.x - HOVER_MARGIN_PX,
        y: screen_rect.y - HOVER_MARGIN_PX,
        width: screen_rect.width + HOVER_MARGIN_PX * 2.0,
        height: screen_rect.height + HOVER_MARGIN_PX * 2.0,
    };
    push_rect_border(
        out,
        ring,
        HOVER_HIGHLIGHT,
        HOVER_WEIGHT.resolve_px(scale_px_per_nm),
    );
}

/// Push a small `Local` class-A crosshair centred on `cursor`: a short cross that
/// marks the cursor without dividing the view.
fn push_local_crosshair(out: &mut Vec<Quad>, cursor: (f32, f32), scale_px_per_nm: f32) {
    let (cx, cy) = cursor;
    let w = CROSSHAIR_WEIGHT.resolve_px(scale_px_per_nm);
    // Horizontal arm.
    out.push(Quad::from_rect(
        RectPx {
            x: cx - CROSSHAIR_ARM_PX,
            y: cy - w * 0.5,
            width: CROSSHAIR_ARM_PX * 2.0,
            height: w,
        },
        CURSOR_CROSSHAIR,
    ));
    // Vertical arm.
    out.push(Quad::from_rect(
        RectPx {
            x: cx - w * 0.5,
            y: cy - CROSSHAIR_ARM_PX,
            width: w,
            height: CROSSHAIR_ARM_PX * 2.0,
        },
        CURSOR_CROSSHAIR,
    ));
}

/// Push a `FullViewport` class-A crosshair: horizontal + vertical hairlines that
/// span the whole focused pane `viewport` through `cursor` (the CAD "full
/// crosshair"). Class-A ScreenConstant, so the 1px hairlines never scale with
/// zoom.
fn push_full_crosshair(
    out: &mut Vec<Quad>,
    cursor: (f32, f32),
    viewport: RectPx,
    scale_px_per_nm: f32,
) {
    let (cx, cy) = cursor;
    let w = CROSSHAIR_WEIGHT.resolve_px(scale_px_per_nm);
    // Horizontal hairline spanning the viewport width.
    out.push(Quad::from_rect(
        RectPx {
            x: viewport.x,
            y: cy - w * 0.5,
            width: viewport.width,
            height: w,
        },
        CURSOR_CROSSHAIR,
    ));
    // Vertical hairline spanning the viewport height.
    out.push(Quad::from_rect(
        RectPx {
            x: cx - w * 0.5,
            y: viewport.y,
            width: w,
            height: viewport.height,
        },
        CURSOR_CROSSHAIR,
    ));
}

/// Append the immediate interaction overlays for ONE pane, in that pane's own
/// screen space: a hover ring for `hover_bounds` (projected through the pane's live
/// camera) and a crosshair at `cursor` when it lies inside `viewport`. Emits
/// nothing for the `None`/off-pane inputs — the empty-in-capture guarantee.
pub(crate) fn push_pane_interaction(
    out: &mut Vec<Quad>,
    projection: &Projection,
    viewport: RectPx,
    hover_bounds: Option<datum_gui_protocol::RectNm>,
    cursor: Option<(f32, f32)>,
    crosshair_style: CrosshairStyle,
) {
    let scale = projection.scale;
    if let Some(bounds) = hover_bounds {
        push_hover_ring(out, projection.project_rect(bounds), scale);
    }
    // The cursor crosshair is a user preference (decision 023 UVT-005): the View
    // menu picks the style, `None` emits nothing. The pane containment gate keeps
    // the crosshair inside the focused pane it belongs to.
    if let Some(cursor) = cursor
        && viewport.contains(cursor.0, cursor.1)
    {
        match crosshair_style {
            CrosshairStyle::FullViewport => push_full_crosshair(out, cursor, viewport, scale),
            CrosshairStyle::Local => push_local_crosshair(out, cursor, scale),
            CrosshairStyle::None => {}
        }
    }
}

/// Build the schematic pane's immediate screen-space grid underlay.
pub(crate) fn build_schematic_grid_vertices(
    schematic_scene_viewport: Option<RectPx>,
    schematic_bounds: &datum_gui_protocol::SceneBounds,
    schematic_camera: CameraState,
) -> Vec<Vertex> {
    let mut quads = Vec::new();
    if let Some(viewport) = schematic_scene_viewport {
        let field = inset_rect(viewport, 10.0, 10.0, 10.0, 10.0);
        let projection = Projection::new(field, schematic_bounds, schematic_camera);
        push_schematic_grid(&mut quads, &projection);
    }
    quads_to_vertices(&quads)
}

/// Build schematic hover/cursor chrome separately from the grid so the renderer
/// can composite it after world geometry, exactly like the board overlay.
pub(crate) fn build_schematic_interaction_vertices(
    schematic_scene_viewport: Option<RectPx>,
    schematic_bounds: &datum_gui_protocol::SceneBounds,
    schematic_camera: CameraState,
    hover_bounds: Option<datum_gui_protocol::RectNm>,
    cursor: Option<(f32, f32)>,
    crosshair_style: CrosshairStyle,
) -> Vec<Vertex> {
    let mut quads = Vec::new();
    if let Some(viewport) = schematic_scene_viewport {
        let field = inset_rect(viewport, 10.0, 10.0, 10.0, 10.0);
        let projection = Projection::new(field, schematic_bounds, schematic_camera);
        push_pane_interaction(
            &mut quads,
            &projection,
            viewport,
            hover_bounds,
            cursor,
            crosshair_style,
        );
    }
    quads_to_vertices(&quads)
}

/// The world-nm bounding box of a hit-test shape (its exact rect for a `Rect`,
/// else the axis-aligned box of its geometry) — the hover ring's world extent.
fn shape_bounds_nm(shape: &WorldHitShape) -> Option<datum_gui_protocol::RectNm> {
    match shape {
        WorldHitShape::Rect(rect) => Some(*rect),
        WorldHitShape::Polyline { path, .. } | WorldHitShape::Polygon(path) => {
            bounds_of_points(path.iter().copied())
        }
        WorldHitShape::Circle { center, radius_nm } => {
            let r = *radius_nm as i64;
            Some(datum_gui_protocol::RectNm {
                min_x: center.x - r,
                min_y: center.y - r,
                max_x: center.x + r,
                max_y: center.y + r,
            })
        }
    }
}

/// The world bbox of the board object identified by `id`, from the resolved board
/// hit regions (the same rects the board hit-test uses), or `None` if not present.
pub(crate) fn board_hover_bounds(
    retained: &RetainedScene,
    id: &str,
) -> Option<datum_gui_protocol::RectNm> {
    retained
        .world_hit_index
        .regions()
        .iter()
        .find(|region| match &region.target {
            HitTarget::AuthoredObject(target) | HitTarget::ReviewAction(target) => target == id,
            _ => false,
        })
        .and_then(|region| shape_bounds_nm(&region.shape))
}

/// The world bbox of the schematic SYMBOL identified by `id` (a projected
/// `board_graphics` body tagged `schematic-symbol:<uuid>`) — the schematic hover
/// ring's extent. Resolved straight from the projected scene because the schematic
/// `RetainedScene` is not threaded into `PreparedScene` construction.
pub(crate) fn schematic_symbol_bounds(
    scene: &BoardReviewSceneV1,
    id: &str,
) -> Option<datum_gui_protocol::RectNm> {
    scene
        .board_graphics
        .iter()
        .find(|graphic| graphic.object_id == id)
        .and_then(|graphic| bounds_of_points(graphic.path.iter().copied()))
}

/// Axis-aligned world bbox of a point iterator, or `None` when empty.
fn bounds_of_points(
    points: impl IntoIterator<Item = PointNm>,
) -> Option<datum_gui_protocol::RectNm> {
    let mut iter = points.into_iter();
    let first = iter.next()?;
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (first.x, first.y, first.x, first.y);
    for point in iter {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    Some(datum_gui_protocol::RectNm {
        min_x,
        min_y,
        max_x,
        max_y,
    })
}

#[cfg(test)]
mod interaction_overlay_tests {
    use super::*;

    fn bounds() -> datum_gui_protocol::SceneBounds {
        datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 10_000_000,
            max_y: 10_000_000,
        }
    }

    fn viewport() -> RectPx {
        RectPx {
            x: 100.0,
            y: 100.0,
            width: 400.0,
            height: 300.0,
        }
    }

    fn projection() -> Projection {
        Projection::new(viewport(), &bounds(), CameraState::fit_to_bounds(&bounds()))
    }

    fn count_color(quads: &[Quad], color: [f32; 3]) -> usize {
        quads.iter().filter(|quad| quad.color == color).count()
    }

    fn quad_bbox(quad: &Quad) -> (f32, f32, f32, f32) {
        let xs = quad.points.iter().map(|p| p.0);
        let ys = quad.points.iter().map(|p| p.1);
        (
            xs.clone().fold(f32::MAX, f32::min),
            ys.clone().fold(f32::MAX, f32::min),
            xs.fold(f32::MIN, f32::max),
            ys.fold(f32::MIN, f32::max),
        )
    }

    /// (b, builder) The hover pre-highlight emits a class-A ring in the hover
    /// colour around the hovered object's projected bbox, and NO crosshair.
    #[test]
    fn hover_bounds_emit_a_hover_ring_only() {
        let mut quads = Vec::new();
        let hover = datum_gui_protocol::RectNm {
            min_x: 2_000_000,
            min_y: 2_000_000,
            max_x: 4_000_000,
            max_y: 4_000_000,
        };
        push_pane_interaction(
            &mut quads,
            &projection(),
            viewport(),
            Some(hover),
            None,
            CrosshairStyle::FullViewport,
        );
        assert!(
            count_color(&quads, HOVER_HIGHLIGHT) >= 4,
            "a hover ring is 4 border rects in the hover colour"
        );
        assert_eq!(
            count_color(&quads, CURSOR_CROSSHAIR),
            0,
            "no crosshair without a cursor"
        );
    }

    /// (c, builder) The default `FullViewport` crosshair emits two class-A hairlines
    /// that SPAN the pane viewport and straddle the cursor.
    #[test]
    fn full_viewport_crosshair_spans_the_pane() {
        let mut quads = Vec::new();
        let cursor = (300.0, 250.0);
        push_pane_interaction(
            &mut quads,
            &projection(),
            viewport(),
            None,
            Some(cursor),
            CrosshairStyle::FullViewport,
        );
        assert_eq!(
            count_color(&quads, HOVER_HIGHLIGHT),
            0,
            "no ring without hover"
        );
        let arms: Vec<_> = quads
            .iter()
            .filter(|quad| quad.color == CURSOR_CROSSHAIR)
            .collect();
        assert_eq!(
            arms.len(),
            2,
            "a full crosshair is one horizontal + one vertical hairline"
        );
        let vp = viewport();
        let mut spans_width = false;
        let mut spans_height = false;
        for arm in &arms {
            let (min_x, min_y, max_x, max_y) = quad_bbox(arm);
            assert!(
                min_x <= cursor.0 && cursor.0 <= max_x && min_y <= cursor.1 && cursor.1 <= max_y,
                "each hairline straddles the cursor point"
            );
            if (max_x - min_x) >= vp.width - 0.5 {
                spans_width = true;
            }
            if (max_y - min_y) >= vp.height - 0.5 {
                spans_height = true;
            }
        }
        assert!(
            spans_width && spans_height,
            "FullViewport spans the pane width (H hairline) and height (V hairline)"
        );
    }

    /// (c, builder) The `Local` crosshair emits a SMALL cross centred on the cursor
    /// that does NOT span the viewport.
    #[test]
    fn local_crosshair_is_a_small_cross_at_the_cursor() {
        let mut quads = Vec::new();
        let cursor = (300.0, 250.0);
        push_pane_interaction(
            &mut quads,
            &projection(),
            viewport(),
            None,
            Some(cursor),
            CrosshairStyle::Local,
        );
        let arms: Vec<_> = quads
            .iter()
            .filter(|quad| quad.color == CURSOR_CROSSHAIR)
            .collect();
        assert_eq!(
            arms.len(),
            2,
            "a local crosshair is one horizontal + one vertical arm"
        );
        let vp = viewport();
        for arm in &arms {
            let (min_x, min_y, max_x, max_y) = quad_bbox(arm);
            assert!(
                min_x <= cursor.0 && cursor.0 <= max_x && min_y <= cursor.1 && cursor.1 <= max_y,
                "each arm straddles the cursor point"
            );
            assert!(
                (max_x - min_x) < vp.width && (max_y - min_y) < vp.height,
                "a local cross is short — it does not span the pane"
            );
        }
    }

    /// (c, builder) The `None` style emits no crosshair even with a cursor present.
    #[test]
    fn none_style_emits_no_crosshair() {
        let mut quads = Vec::new();
        push_pane_interaction(
            &mut quads,
            &projection(),
            viewport(),
            None,
            Some((300.0, 250.0)),
            CrosshairStyle::None,
        );
        assert!(quads.is_empty(), "CrosshairStyle::None emits nothing");
    }

    /// Switching `crosshair_style` switches the emitted geometry: FullViewport arms
    /// are strictly longer than the Local ones for the same cursor.
    #[test]
    fn switching_style_switches_modes() {
        let cursor = (300.0, 250.0);
        let arm_extent = |style| {
            let mut quads = Vec::new();
            push_pane_interaction(
                &mut quads,
                &projection(),
                viewport(),
                None,
                Some(cursor),
                style,
            );
            quads
                .iter()
                .filter(|q| q.color == CURSOR_CROSSHAIR)
                .map(|q| {
                    let (min_x, min_y, max_x, max_y) = quad_bbox(q);
                    (max_x - min_x).max(max_y - min_y)
                })
                .fold(0.0_f32, f32::max)
        };
        assert!(
            arm_extent(CrosshairStyle::FullViewport) > arm_extent(CrosshairStyle::Local),
            "FullViewport hairlines outspan the Local cross"
        );
        assert_eq!(arm_extent(CrosshairStyle::None), 0.0, "None emits nothing");
    }

    /// The default is `FullViewport` (decision 023 spec §2).
    #[test]
    fn default_crosshair_style_is_full_viewport() {
        assert_eq!(CrosshairStyle::default(), CrosshairStyle::FullViewport);
    }

    /// The empty-in-capture guarantee: with neither hover nor cursor (the offscreen
    /// visual-test state), the builder emits nothing — so the board stays byte-
    /// identical, regardless of the selected style.
    #[test]
    fn no_inputs_emit_nothing() {
        for style in [
            CrosshairStyle::FullViewport,
            CrosshairStyle::Local,
            CrosshairStyle::None,
        ] {
            let mut quads = Vec::new();
            push_pane_interaction(&mut quads, &projection(), viewport(), None, None, style);
            assert!(quads.is_empty(), "no cursor + no hover => no overlay quads");
        }
    }

    /// A cursor OUTSIDE the pane viewport draws no crosshair (it belongs to another
    /// pane) — the per-focused-pane containment gate.
    #[test]
    fn cursor_outside_viewport_draws_no_crosshair() {
        let mut quads = Vec::new();
        push_pane_interaction(
            &mut quads,
            &projection(),
            viewport(),
            None,
            Some((10.0, 10.0)),
            CrosshairStyle::FullViewport,
        );
        assert!(
            quads.is_empty(),
            "a cursor outside the pane emits no crosshair"
        );
    }

    /// (c, wiring) Schematic interaction chrome is isolated from the grid buffer.
    #[test]
    fn crosshair_flows_into_the_schematic_overlay_only() {
        let baseline = build_schematic_interaction_vertices(
            Some(viewport()),
            &bounds(),
            CameraState::fit_to_bounds(&bounds()),
            None,
            None,
            CrosshairStyle::FullViewport,
        );
        let with_cursor = build_schematic_interaction_vertices(
            Some(viewport()),
            &bounds(),
            CameraState::fit_to_bounds(&bounds()),
            None,
            Some((300.0, 250.0)),
            CrosshairStyle::FullViewport,
        );
        let crosshair_verts = with_cursor
            .iter()
            .filter(|vertex| vertex.color == CURSOR_CROSSHAIR)
            .count();
        assert!(
            with_cursor.len() > baseline.len() && crosshair_verts > 0,
            "the crosshair adds class-A vertices to the schematic overlay"
        );
        let grid = build_schematic_grid_vertices(
            Some(viewport()),
            &bounds(),
            CameraState::fit_to_bounds(&bounds()),
        );
        assert_eq!(
            grid.iter()
                .filter(|vertex| vertex.color == CURSOR_CROSSHAIR)
                .count(),
            0,
            "the pre-world grid must never contain interaction chrome"
        );
    }
}
