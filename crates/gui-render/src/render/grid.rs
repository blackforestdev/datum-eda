use super::{
    board_surface_color, detail_tier, BoardSurfaceRole, Projection, Quad, RectPx,
    SCHEMATIC_GRID_MAJOR, SCHEMATIC_GRID_MINOR,
};
use datum_gui_viewport::{
    AxisProjection, GridBounds, GridConfig, GridEngine, GridMode, GridTier, GridViewport,
    WeightClass,
};

/// Emit one surface's grid as an IMMEDIATE screen-space pass via the shared
/// [`GridEngine`] (spec §5). Both the board (`push_scene_grid`) and the schematic
/// (`push_schematic_grid`, slice S1b) funnel through here: the caller supplies the
/// per-surface [`GridConfig`] (pitch tiers, weight class, colours) and this derives
/// the abstract engine inputs from the live [`Projection`] — the resolved LOD tier
/// (`detail_tier`), the pixel viewport, the world-nm extent, and the per-axis
/// world-nm→screen-px affine — then re-emits the returned line specs as pixel
/// `Quad`s. Because the weight is a class-A [`WeightClass::ScreenConstant`] the
/// stroke is a fixed device-pixel width at any zoom, which is the whole point:
/// grid chrome never thickens on zoom-in (spec §4).
fn emit_immediate_grid(out: &mut Vec<Quad>, projection: &Projection, config: &GridConfig) {
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
    // Per-axis affine reproducing `Projection::project_point` exactly: one shared
    // scale but a distinct offset/origin per axis.
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
    for line in GridEngine::compute(config, tier, viewport, bounds, x_axis, y_axis) {
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

/// The board's metric grid, computed by the shared [`GridEngine`] and drawn as an
/// immediate screen-space pass. The board grid config maps 1:1 onto `detail_tier`,
/// whose `Coarse`/`Normal`/`Fine` discriminants (0/1/2) index `tiers`:
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
    emit_immediate_grid(out, projection, &config);
}

/// Subtle SQUARE grid underlay for the companion schematic pane (slice S1b, spec
/// §5 UVT-003). This is now an IMMEDIATE screen-space pass driven by the same
/// shared [`GridEngine`] and [`WeightClass::ScreenConstant`] weight as the board
/// grid — so the schematic grid line weight is a fixed device pixel at any zoom.
/// Previously it was baked into the retained schematic WORLD buffer as world-nm
/// lines, which the GPU re-scaled by the live schematic camera; that made the grid
/// thicken on zoom-in (the bug this slice fixes). Pitch is the imperial schematic
/// tier table (2.54mm / 1.27mm at the finest tier), zoom-tiered off the same
/// `detail_tier` the board grid uses, and coloured with the schematic `#sgrid`
/// whisper so it never competes with the green wires.
///
/// The tiers index `detail_tier` (Coarse=0 / Normal=1 / Fine=2), matching the old
/// world-baked pitch table exactly so the fit/default framing is visually
/// equivalent; only the weight behaviour (now zoom-invariant) changes.
pub(crate) fn push_schematic_grid(out: &mut Vec<Quad>, projection: &Projection) {
    let config = GridConfig {
        mode: GridMode::Square,
        weight: WeightClass::ScreenConstant(1.0),
        minor_color: SCHEMATIC_GRID_MINOR,
        major_color: SCHEMATIC_GRID_MAJOR,
        tiers: vec![
            // Coarse: major only, 5.08mm.
            GridTier {
                major_pitch_nm: 5_080_000,
                minor_pitch_nm: None,
            },
            // Normal: 5.08mm major / 2.54mm minor.
            GridTier {
                major_pitch_nm: 5_080_000,
                minor_pitch_nm: Some(2_540_000),
            },
            // Fine: 2.54mm major / 1.27mm minor.
            GridTier {
                major_pitch_nm: 2_540_000,
                minor_pitch_nm: Some(1_270_000),
            },
        ],
        origin_nm: None,
    };
    emit_immediate_grid(out, projection, &config);
}
