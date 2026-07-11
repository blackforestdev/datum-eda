use super::{
    board_surface_color, ceil_multiple, detail_tier, floor_multiple, push_world_polyline_segments,
    world_stroke_nm, BoardSurfaceRole, DetailTier, PointNm, Projection, Quad, RectPx,
    SCHEMATIC_GRID_MAJOR, SCHEMATIC_GRID_MINOR,
};

pub(crate) fn push_scene_grid(out: &mut Vec<Quad>, projection: &Projection) {
    let detail = detail_tier(projection);
    let major_pitch_nm = match detail {
        DetailTier::Fine => 2_500_000,
        DetailTier::Normal => 5_000_000,
        DetailTier::Coarse => 10_000_000,
    };
    let minor_pitch_nm = match detail {
        DetailTier::Fine => Some(1_250_000),
        DetailTier::Normal => Some(2_500_000),
        DetailTier::Coarse => None,
    };
    if let Some(minor_pitch) = minor_pitch_nm {
        push_grid_axis_lines(
            out,
            projection,
            minor_pitch,
            board_surface_color(BoardSurfaceRole::GridMinor),
        );
    }
    push_grid_axis_lines(
        out,
        projection,
        major_pitch_nm,
        board_surface_color(BoardSurfaceRole::GridMajor),
    );
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

fn push_grid_axis_lines(
    out: &mut Vec<Quad>,
    projection: &Projection,
    pitch_nm: i64,
    color: [f32; 3],
) {
    if pitch_nm <= 0 {
        return;
    }
    let start_x = floor_multiple(projection.bounds.min_x, pitch_nm);
    let end_x = ceil_multiple(projection.bounds.max_x, pitch_nm);
    let mut x = start_x;
    while x <= end_x {
        let x_px = projection
            .project_point(PointNm {
                x,
                y: projection.bounds.min_y,
            })
            .0;
        out.push(Quad::from_rect(
            RectPx {
                x: x_px,
                y: projection.viewport.y,
                width: 1.0,
                height: projection.viewport.height,
            },
            color,
        ));
        x += pitch_nm;
    }
    let start_y = floor_multiple(projection.bounds.min_y, pitch_nm);
    let end_y = ceil_multiple(projection.bounds.max_y, pitch_nm);
    let mut y = start_y;
    while y <= end_y {
        let y_px = projection
            .project_point(PointNm {
                x: projection.bounds.min_x,
                y,
            })
            .1;
        out.push(Quad::from_rect(
            RectPx {
                x: projection.viewport.x,
                y: y_px,
                width: projection.viewport.width,
                height: 1.0,
            },
            color,
        ));
        y += pitch_nm;
    }
}
