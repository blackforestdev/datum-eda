use super::{
    board_surface_color, BoardSurfaceRole, Projection, Quad, RectPx, SCHEMATIC_GRID_MAJOR,
    SCHEMATIC_GRID_MINOR,
};
use datum_gui_viewport::{
    grid::{GridLodState, GridViewport},
    profile::GridMark,
    AxisProjection, GridConfig, GridEngine, GridMode, GridTier, ViewportProfile, WeightClass,
};
use std::sync::LazyLock;

static BOARD_GRID_TIERS: [GridTier; 3] = [
    GridTier {
        major_pitch_nm: (10_000_000, 10_000_000),
        minor_pitch_nm: None,
    },
    GridTier {
        major_pitch_nm: (5_000_000, 5_000_000),
        minor_pitch_nm: Some((2_500_000, 2_500_000)),
    },
    GridTier {
        major_pitch_nm: (2_500_000, 2_500_000),
        minor_pitch_nm: Some((1_250_000, 1_250_000)),
    },
];
static SCHEMATIC_GRID_TIERS: [GridTier; 3] = [
    GridTier {
        major_pitch_nm: (5_080_000, 5_080_000),
        minor_pitch_nm: None,
    },
    GridTier {
        major_pitch_nm: (5_080_000, 5_080_000),
        minor_pitch_nm: Some((2_540_000, 2_540_000)),
    },
    GridTier {
        major_pitch_nm: (2_540_000, 2_540_000),
        minor_pitch_nm: Some((1_270_000, 1_270_000)),
    },
];

static BOARD_PROFILE: LazyLock<ViewportProfile> = LazyLock::new(|| ViewportProfile {
    grid: GridConfig {
        mode: GridMode::Square,
        mark: GridMark::Lines,
        weight: WeightClass::ScreenConstant(1.0),
        minor_color: board_surface_color(BoardSurfaceRole::GridMinor),
        major_color: board_surface_color(BoardSurfaceRole::GridMajor),
        tiers: &BOARD_GRID_TIERS,
        origin_nm: Some((0, 0)),
    },
    ..ViewportProfile::default()
});

static SCHEMATIC_PROFILE: LazyLock<ViewportProfile> = LazyLock::new(|| ViewportProfile {
    grid: GridConfig {
        mode: GridMode::Square,
        mark: GridMark::Lines,
        weight: WeightClass::ScreenConstant(1.0),
        minor_color: SCHEMATIC_GRID_MINOR,
        major_color: SCHEMATIC_GRID_MAJOR,
        tiers: &SCHEMATIC_GRID_TIERS,
        origin_nm: Some((0, 0)),
    },
    ..ViewportProfile::default()
});

/// Resolve governed grid LOD for one concrete surface projection. Runtime owns
/// `previous_lod` by pane identity; the renderer owns profile and projection
/// details, so neither layer duplicates the other's policy.
pub fn resolve_surface_grid_lod(
    surface: crate::SceneSurface,
    viewport: crate::RectPx,
    bounds: &datum_gui_protocol::SceneBounds,
    camera: crate::CameraState,
    previous_lod: GridLodState,
) -> GridLodState {
    let field = crate::inset_rect(viewport, 10.0, 10.0, 10.0, 10.0);
    let projection = crate::Projection::new(field, bounds, camera);
    let profile = match surface {
        crate::SceneSurface::Board => &*BOARD_PROFILE,
        crate::SceneSurface::Schematic => &*SCHEMATIC_PROFILE,
    };
    GridEngine::resolve_lod(&profile.grid, projection.scale, previous_lod)
}

fn emit_immediate_grid(
    out: &mut Vec<Quad>,
    projection: &Projection,
    profile: &ViewportProfile,
    previous_lod: GridLodState,
) -> GridLodState {
    let viewport = GridViewport {
        x: projection.viewport.x,
        y: projection.viewport.y,
        width: projection.viewport.width,
        height: projection.viewport.height,
    };
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
    let lod = GridEngine::resolve_lod(&profile.grid, projection.scale, previous_lod);
    let Some(tier) = lod.tier else {
        return lod;
    };
    out.extend(
        GridEngine::compute(&profile.grid, tier, viewport, x_axis, y_axis)
            .into_iter()
            .map(|line| {
                Quad::from_rect(
                    RectPx {
                        x: line.x,
                        y: line.y,
                        width: line.width,
                        height: line.height,
                    },
                    line.color,
                )
            }),
    );
    lod
}

pub(crate) fn push_scene_grid_with_lod(
    out: &mut Vec<Quad>,
    projection: &Projection,
    previous_lod: GridLodState,
) -> GridLodState {
    emit_immediate_grid(out, projection, &BOARD_PROFILE, previous_lod)
}

pub(crate) fn push_schematic_grid_with_lod(
    out: &mut Vec<Quad>,
    projection: &Projection,
    previous_lod: GridLodState,
) -> GridLodState {
    emit_immediate_grid(out, projection, &SCHEMATIC_PROFILE, previous_lod)
}

pub(crate) fn push_scene_grid(out: &mut Vec<Quad>, projection: &Projection) {
    let _ = push_scene_grid_with_lod(out, projection, GridLodState::default());
}

pub(crate) fn push_schematic_grid(out: &mut Vec<Quad>, projection: &Projection) {
    let _ = push_schematic_grid_with_lod(out, projection, GridLodState::default());
}
