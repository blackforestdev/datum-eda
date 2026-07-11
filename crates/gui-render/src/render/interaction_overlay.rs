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
/// The projected-id namespace prefix that marks a schematic symbol body (mirrors
/// `push_schematic_symbol_hit_regions`); the hovered surface is inferred from it,
/// so the hover pre-highlight is routed to the schematic camera without a
/// redundant stored surface field.
pub(crate) const SCHEMATIC_SYMBOL_PREFIX: &str = "schematic-symbol:";

/// Push the class-A hover pre-highlight ring for a screen-space bounding rect.
fn push_hover_ring(out: &mut Vec<Quad>, screen_rect: RectPx, scale_px_per_nm: f32) {
    let ring = RectPx {
        x: screen_rect.x - HOVER_MARGIN_PX,
        y: screen_rect.y - HOVER_MARGIN_PX,
        width: screen_rect.width + HOVER_MARGIN_PX * 2.0,
        height: screen_rect.height + HOVER_MARGIN_PX * 2.0,
    };
    push_rect_border(out, ring, HOVER_HIGHLIGHT, HOVER_WEIGHT.resolve_px(scale_px_per_nm));
}

/// Push a small local class-A crosshair centred on `cursor`.
fn push_crosshair(out: &mut Vec<Quad>, cursor: (f32, f32), scale_px_per_nm: f32) {
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
) {
    let scale = projection.scale;
    if let Some(bounds) = hover_bounds {
        push_hover_ring(out, projection.project_rect(bounds), scale);
    }
    if let Some(cursor) = cursor
        && viewport.contains(cursor.0, cursor.1)
    {
        push_crosshair(out, cursor, scale);
    }
}

/// Build the schematic pane's immediate screen-space underlay: the S1b grid plus
/// the S4 interaction overlays (hover ring + crosshair), all class-A
/// `ScreenConstant`. Returns empty vertices when there is no Schematic pane.
pub(crate) fn build_schematic_underlay_vertices(
    schematic_scene_viewport: Option<RectPx>,
    schematic_bounds: &datum_gui_protocol::SceneBounds,
    schematic_camera: CameraState,
    hover_bounds: Option<datum_gui_protocol::RectNm>,
    cursor: Option<(f32, f32)>,
) -> Vec<Vertex> {
    let mut quads = Vec::new();
    if let Some(viewport) = schematic_scene_viewport {
        let field = inset_rect(viewport, 10.0, 10.0, 10.0, 10.0);
        let projection = Projection::new(field, schematic_bounds, schematic_camera);
        push_schematic_grid(&mut quads, &projection);
        push_pane_interaction(&mut quads, &projection, viewport, hover_bounds, cursor);
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
        .world_hit_regions
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
        push_pane_interaction(&mut quads, &projection(), viewport(), Some(hover), None);
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

    /// (c, builder) The crosshair emits class-A arms centred ON the cursor, and no
    /// hover ring. Proves the crosshair overlay is emitted at the cursor position.
    #[test]
    fn cursor_emits_a_crosshair_at_the_cursor() {
        let mut quads = Vec::new();
        let cursor = (300.0, 250.0);
        push_pane_interaction(&mut quads, &projection(), viewport(), None, Some(cursor));
        assert_eq!(count_color(&quads, HOVER_HIGHLIGHT), 0, "no ring without hover");
        let arms: Vec<_> = quads
            .iter()
            .filter(|quad| quad.color == CURSOR_CROSSHAIR)
            .collect();
        assert_eq!(arms.len(), 2, "a crosshair is one horizontal + one vertical arm");
        for arm in arms {
            let (min_x, min_y, max_x, max_y) = quad_bbox(arm);
            assert!(
                min_x <= cursor.0 && cursor.0 <= max_x && min_y <= cursor.1 && cursor.1 <= max_y,
                "each crosshair arm straddles the cursor point"
            );
        }
    }

    /// The empty-in-capture guarantee: with neither hover nor cursor (the offscreen
    /// visual-test state), the builder emits nothing — so the board stays byte-
    /// identical.
    #[test]
    fn no_inputs_emit_nothing() {
        let mut quads = Vec::new();
        push_pane_interaction(&mut quads, &projection(), viewport(), None, None);
        assert!(quads.is_empty(), "no cursor + no hover => no overlay quads");
    }

    /// A cursor OUTSIDE the pane viewport draws no crosshair (it belongs to another
    /// pane) — the per-focused-pane containment gate.
    #[test]
    fn cursor_outside_viewport_draws_no_crosshair() {
        let mut quads = Vec::new();
        push_pane_interaction(&mut quads, &projection(), viewport(), None, Some((10.0, 10.0)));
        assert!(quads.is_empty(), "a cursor outside the pane emits no crosshair");
    }

    /// (c, wiring) The crosshair flows into the schematic pane's shared underlay
    /// buffer (grid + interaction) when a cursor is supplied.
    #[test]
    fn crosshair_flows_into_the_schematic_underlay() {
        let baseline =
            build_schematic_underlay_vertices(Some(viewport()), &bounds(), CameraState::fit_to_bounds(&bounds()), None, None);
        let with_cursor = build_schematic_underlay_vertices(
            Some(viewport()),
            &bounds(),
            CameraState::fit_to_bounds(&bounds()),
            None,
            Some((300.0, 250.0)),
        );
        let crosshair_verts = with_cursor
            .iter()
            .filter(|vertex| vertex.color == CURSOR_CROSSHAIR)
            .count();
        assert!(
            with_cursor.len() > baseline.len() && crosshair_verts > 0,
            "the crosshair adds class-A vertices to the schematic underlay"
        );
    }
}
