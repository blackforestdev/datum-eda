use super::{
    board_surface_color, ceil_multiple, detail_tier, floor_multiple, push_world_polyline_segments,
    world_stroke_nm, BoardSurfaceRole, DetailTier, PointNm, Projection, Quad, RectPx,
    SCHEMATIC_GRID_MAJOR, SCHEMATIC_GRID_MINOR,
};
use datum_gui_viewport::{
    AxisProjection, GridBounds, GridConfig, GridEngine, GridMode, GridTier, GridViewport,
    WeightClass,
};

/// The board's metric grid, now computed by the shared [`GridEngine`] (spec §5,
/// slice S1a). `push_scene_grid` builds the abstract engine inputs from the live
/// [`Projection`] — a per-surface [`GridConfig`] (the exact metric pitch tiers,
/// 1 device-px class-A weight, and board grid colour tokens) plus the per-axis
/// world-nm→screen-px affine — and re-emits the returned line specs as `Quad`s.
/// The engine reproduces the previous immediate-mode pixel-axis-rect output
/// byte-for-byte; only the mechanism moved.
///
/// The board grid config maps 1:1 onto `detail_tier`, whose `Coarse`/`Normal`/
/// `Fine` discriminants (0/1/2) index `tiers`:
///
/// | tier   | idx | major   | minor   |
/// |--------|-----|---------|---------|
/// | Coarse | 0   | 10 mm   | (none)  |
/// | Normal | 1   |  5 mm   | 2.5 mm  |
/// | Fine   | 2   | 2.5 mm  | 1.25 mm |
pub(crate) fn push_scene_grid(out: &mut Vec<Quad>, projection: &Projection) {
    let config = GridConfig {
        mode: GridMode::Square,
        weight: WeightClass::ScreenConstant(1.0),
        minor_color: board_surface_color(BoardSurfaceRole::GridMinor),
        major_color: board_surface_color(BoardSurfaceRole::GridMajor),
        tiers: vec![
            GridTier {
                major_pitch_nm: 10_000_000,
                minor_pitch_nm: None,
            },
            GridTier {
                major_pitch_nm: 5_000_000,
                minor_pitch_nm: Some(2_500_000),
            },
            GridTier {
                major_pitch_nm: 2_500_000,
                minor_pitch_nm: Some(1_250_000),
            },
        ],
        origin_nm: None,
    };
    let tier = detail_tier(projection) as usize;
    let viewport = GridViewport {
        x: projection.viewport.x,
        y: projection.viewport.y,
        width: projection.viewport.width,
        height: projection.viewport.height,
    };
    let bounds = GridBounds {
        min_x: projection.bounds.min_x,
        min_y: projection.bounds.min_y,
        max_x: projection.bounds.max_x,
        max_y: projection.bounds.max_y,
    };
    // Per-axis affine reproducing `Projection::project_point` exactly: the board
    // shares one scale but a distinct offset/origin per axis.
    let x_axis = AxisProjection {
        scale: projection.scale,
        offset: projection.offset_x,
        origin_nm: projection.bounds.min_x,
    };
    let y_axis = AxisProjection {
        scale: projection.scale,
        offset: projection.offset_y,
        origin_nm: projection.bounds.min_y,
    };
    for line in GridEngine::compute(&config, tier, viewport, bounds, x_axis, y_axis) {
        out.push(Quad::from_rect(
            RectPx {
                x: line.x,
                y: line.y,
                width: line.width,
                height: line.height,
            },
            line.color,
        ));
    }
}

/// Subtle SQUARE grid underlay for the companion schematic pane (P2.2f). Unlike
/// the board grid (`push_scene_grid`, drawn immediate-mode in PIXEL space with the
/// live camera), the companion schematic is a STATIC retained WORLD buffer whose
/// vertices are nm coordinates the GPU projects — so the grid is emitted as
/// world-space lines spanning the schematic bounds, not pixel axis rects. Pitch is
/// a schematic pitch (1.27mm minor / 2.54mm major at the base tier), zoom-tiered
/// off the same `detail_tier` the board grid uses, and coloured with the schematic
/// `#sgrid` whisper so it never competes with the green wires. The board grid path
/// is untouched.
pub(crate) fn push_schematic_grid(out: &mut Vec<Quad>, projection: &Projection) {
    let detail = detail_tier(projection);
    let major_pitch_nm = match detail {
        DetailTier::Fine => 2_540_000,   // 2.54mm
        DetailTier::Normal => 5_080_000, // 5.08mm
        DetailTier::Coarse => 5_080_000,
    };
    let minor_pitch_nm = match detail {
        DetailTier::Fine => Some(1_270_000),   // 1.27mm
        DetailTier::Normal => Some(2_540_000), // 2.54mm
        DetailTier::Coarse => None,
    };
    // One-device-pixel line width, expressed in nm so the GPU projection renders it
    // as a hairline at the schematic's fit scale.
    let line_width_nm = world_stroke_nm(1.0, projection);
    if let Some(minor_pitch) = minor_pitch_nm {
        push_schematic_grid_world_lines(
            out,
            projection,
            minor_pitch,
            line_width_nm,
            SCHEMATIC_GRID_MINOR,
        );
    }
    push_schematic_grid_world_lines(
        out,
        projection,
        major_pitch_nm,
        line_width_nm,
        SCHEMATIC_GRID_MAJOR,
    );
}

/// Emits world-space vertical + horizontal grid lines at `pitch_nm` across the
/// schematic projection's nm bounds. Each line is a thin world quad the GPU
/// transforms with the schematic camera (unlike the board's pixel-space axis
/// rects).
fn push_schematic_grid_world_lines(
    out: &mut Vec<Quad>,
    projection: &Projection,
    pitch_nm: i64,
    width_nm: f32,
    color: [f32; 3],
) {
    if pitch_nm <= 0 {
        return;
    }
    let bounds = &projection.bounds;
    let mut x = floor_multiple(bounds.min_x, pitch_nm);
    let end_x = ceil_multiple(bounds.max_x, pitch_nm);
    while x <= end_x {
        push_world_polyline_segments(
            out,
            &[
                PointNm { x, y: bounds.min_y },
                PointNm { x, y: bounds.max_y },
            ],
            width_nm,
            color,
        );
        x += pitch_nm;
    }
    let mut y = floor_multiple(bounds.min_y, pitch_nm);
    let end_y = ceil_multiple(bounds.max_y, pitch_nm);
    while y <= end_y {
        push_world_polyline_segments(
            out,
            &[
                PointNm { x: bounds.min_x, y },
                PointNm { x: bounds.max_x, y },
            ],
            width_nm,
            color,
        );
        y += pitch_nm;
    }
}

