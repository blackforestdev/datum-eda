//! Shared, bounded screen-space grid generation.

use crate::profile::{GridConfig, GridMark, GridMode, GridTier};

/// Maximum number of screen-space rectangles emitted for one pane.
pub const MAX_GRID_PRIMITIVES: usize = 16_384;
/// Below this spacing the grid is visually noise and is hidden.
pub const GRID_HIDE_FLOOR_PX: f32 = 10.0;
/// A visible tier coarsens when its minor pitch falls below this spacing.
pub const GRID_COARSEN_PX: f32 = 20.0;
/// A previously coarsened tier refines only after crossing this spacing.
pub const GRID_REFINE_PX: f32 = 80.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisProjection {
    pub scale: f32,
    pub offset: f32,
    pub origin_nm: i64,
}

impl AxisProjection {
    #[inline]
    pub fn project(&self, nm: i64) -> f32 {
        self.offset + ((nm as i128 - self.origin_nm as i128) as f64 * self.scale as f64) as f32
    }

    fn visible_nm(&self, lo_px: f32, hi_px: f32) -> Option<(i64, i64)> {
        if !self.scale.is_finite() || self.scale == 0.0 || !lo_px.is_finite() || !hi_px.is_finite()
        {
            return None;
        }
        let inverse =
            |px: f32| self.origin_nm as f64 + (px as f64 - self.offset as f64) / self.scale as f64;
        let a = inverse(lo_px);
        let b = inverse(hi_px);
        if !a.is_finite() || !b.is_finite() {
            return None;
        }
        let clamp_floor = |v: f64| v.floor().clamp(i64::MIN as f64, i64::MAX as f64) as i64;
        let clamp_ceil = |v: f64| v.ceil().clamp(i64::MIN as f64, i64::MAX as f64) as i64;
        Some((clamp_floor(a.min(b)), clamp_ceil(a.max(b))))
    }
}

/// Retained by the viewport so LOD does not chatter around a threshold.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GridLodState {
    pub tier: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridLine {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 3],
}

pub struct GridEngine;

impl GridEngine {
    /// Resolve coarse-to-fine LOD using governed screen-space thresholds.
    pub fn resolve_lod(config: &GridConfig, scale: f32, previous: GridLodState) -> GridLodState {
        if config.tiers.is_empty() || !scale.is_finite() || scale <= 0.0 {
            return GridLodState::default();
        }
        let spacing = |tier: &GridTier| {
            let pitch = tier.minor_pitch_nm.unwrap_or(tier.major_pitch_nm);
            pitch.0.min(pitch.1) as f64 * scale as f64
        };
        let coarse_spacing = spacing(&config.tiers[0]);
        if coarse_spacing < GRID_HIDE_FLOOR_PX as f64 {
            return GridLodState::default();
        }

        if let Some(old) = previous.tier.filter(|&i| i < config.tiers.len()) {
            let mut tier = old;
            while tier > 0 && spacing(&config.tiers[tier]) < GRID_COARSEN_PX as f64 {
                tier -= 1;
            }
            while tier + 1 < config.tiers.len()
                && spacing(&config.tiers[tier + 1]) >= GRID_REFINE_PX as f64
            {
                tier += 1;
            }
            return GridLodState { tier: Some(tier) };
        }

        let tier = config
            .tiers
            .iter()
            .enumerate()
            .rev()
            .find(|(_, t)| spacing(t) >= GRID_COARSEN_PX as f64)
            .map_or(0, |(i, _)| i);
        GridLodState { tier: Some(tier) }
    }

    /// Generate only the inverse-projected visible extent. `tier` is explicit
    /// so renderers can retain [`GridLodState`] independently per pane.
    pub fn compute(
        config: &GridConfig,
        tier: usize,
        viewport: GridViewport,
        x_axis: AxisProjection,
        y_axis: AxisProjection,
    ) -> Vec<GridLine> {
        let mut out = Vec::new();
        let Some(tier) = config.tiers.get(tier) else {
            return out;
        };
        if viewport.width <= 0.0 || viewport.height <= 0.0 {
            return out;
        }
        let Some(x_bounds) = x_axis.visible_nm(viewport.x, viewport.x + viewport.width) else {
            return out;
        };
        let Some(y_bounds) = y_axis.visible_nm(viewport.y, viewport.y + viewport.height) else {
            return out;
        };
        let origin = config.origin_nm.unwrap_or((0, 0));
        let weight = config.weight.resolve_px(x_axis.scale).max(0.0);
        if weight == 0.0 || !weight.is_finite() {
            return out;
        }

        for (pitch, color) in [
            tier.minor_pitch_nm.map(|p| (p, config.minor_color)),
            Some((tier.major_pitch_nm, config.major_color)),
        ]
        .into_iter()
        .flatten()
        {
            let pitch = match config.mode {
                GridMode::Square => (pitch.0, pitch.0),
                GridMode::Rectangular => pitch,
            };
            if pitch.0 <= 0 || pitch.1 <= 0 {
                continue;
            }
            match config.mark {
                GridMark::Lines => {
                    emit_axis_lines(
                        &mut out, true, x_bounds, pitch.0, origin.0, viewport, x_axis, weight,
                        color,
                    );
                    emit_axis_lines(
                        &mut out, false, y_bounds, pitch.1, origin.1, viewport, y_axis, weight,
                        color,
                    );
                }
                GridMark::Crosses | GridMark::Dots => emit_marks(
                    &mut out,
                    config.mark,
                    x_bounds,
                    y_bounds,
                    pitch,
                    origin,
                    x_axis,
                    y_axis,
                    weight,
                    color,
                ),
            }
            if out.len() >= MAX_GRID_PRIMITIVES {
                break;
            }
        }
        out.truncate(MAX_GRID_PRIMITIVES);
        out
    }
}

fn first_last(bounds: (i64, i64), pitch: i64, origin: i64) -> Option<(i128, i128)> {
    let p = pitch as i128;
    let o = origin as i128;
    let lo = bounds.0 as i128;
    let hi = bounds.1 as i128;
    let first = o + (lo - o).div_euclid(p) * p;
    let first = if first < lo {
        first.checked_add(p)?
    } else {
        first
    };
    let last = o + (hi - o).div_euclid(p) * p;
    (first <= last).then_some((first, last))
}

#[allow(clippy::too_many_arguments)]
fn emit_axis_lines(
    out: &mut Vec<GridLine>,
    vertical: bool,
    bounds: (i64, i64),
    pitch: i64,
    origin: i64,
    viewport: GridViewport,
    axis: AxisProjection,
    weight: f32,
    color: [f32; 3],
) {
    let Some((mut value, last)) = first_last(bounds, pitch, origin) else {
        return;
    };
    let step = pitch as i128;
    while value <= last && out.len() < MAX_GRID_PRIMITIVES {
        let px = axis.project(value as i64);
        out.push(if vertical {
            GridLine {
                x: px,
                y: viewport.y,
                width: weight,
                height: viewport.height,
                color,
            }
        } else {
            GridLine {
                x: viewport.x,
                y: px,
                width: viewport.width,
                height: weight,
                color,
            }
        });
        let Some(next) = value.checked_add(step) else {
            break;
        };
        value = next;
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_marks(
    out: &mut Vec<GridLine>,
    mark: GridMark,
    xb: (i64, i64),
    yb: (i64, i64),
    pitch: (i64, i64),
    origin: (i64, i64),
    xa: AxisProjection,
    ya: AxisProjection,
    weight: f32,
    color: [f32; 3],
) {
    let (Some((mut x, xlast)), Some((yfirst, ylast))) = (
        first_last(xb, pitch.0, origin.0),
        first_last(yb, pitch.1, origin.1),
    ) else {
        return;
    };
    let arm = 3.0 * weight;
    while x <= xlast && out.len() < MAX_GRID_PRIMITIVES {
        let mut y = yfirst;
        while y <= ylast && out.len() < MAX_GRID_PRIMITIVES {
            let (px, py) = (xa.project(x as i64), ya.project(y as i64));
            if mark == GridMark::Dots {
                out.push(GridLine {
                    x: px,
                    y: py,
                    width: weight,
                    height: weight,
                    color,
                });
            } else {
                out.push(GridLine {
                    x: px - arm,
                    y: py,
                    width: arm * 2.0,
                    height: weight,
                    color,
                });
                if out.len() < MAX_GRID_PRIMITIVES {
                    out.push(GridLine {
                        x: px,
                        y: py - arm,
                        width: weight,
                        height: arm * 2.0,
                        color,
                    });
                }
            }
            let Some(next) = y.checked_add(pitch.1 as i128) else {
                break;
            };
            y = next;
        }
        let Some(next) = x.checked_add(pitch.0 as i128) else {
            break;
        };
        x = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GridMode, WeightClass};
    static TIERS: [GridTier; 3] = [
        GridTier {
            major_pitch_nm: (100, 100),
            minor_pitch_nm: None,
        },
        GridTier {
            major_pitch_nm: (50, 40),
            minor_pitch_nm: Some((25, 20)),
        },
        GridTier {
            major_pitch_nm: (20, 10),
            minor_pitch_nm: Some((10, 5)),
        },
    ];
    fn config(mark: GridMark) -> GridConfig {
        GridConfig {
            mode: GridMode::Rectangular,
            mark,
            weight: WeightClass::ScreenConstant(1.0),
            minor_color: [0.1; 3],
            major_color: [0.2; 3],
            tiers: &TIERS,
            origin_nm: Some((3, 7)),
        }
    }
    fn axis(scale: f32, offset: f32) -> AxisProjection {
        AxisProjection {
            scale,
            offset,
            origin_nm: 0,
        }
    }
    fn vp() -> GridViewport {
        GridViewport {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
        }
    }

    #[test]
    fn generation_is_visible_not_scene_bounded() {
        let lines = GridEngine::compute(
            &config(GridMark::Lines),
            0,
            vp(),
            axis(1.0, 0.0),
            axis(1.0, 0.0),
        );
        assert!(lines.iter().all(|l| l.x <= 100.0 && l.y <= 80.0));
        assert!(lines.len() < 10);
    }
    #[test]
    fn generation_has_hard_budget_at_extreme_zoom() {
        let lines = GridEngine::compute(
            &config(GridMark::Dots),
            2,
            vp(),
            axis(1e-9, 0.0),
            axis(1e-9, 0.0),
        );
        assert_eq!(lines.len(), MAX_GRID_PRIMITIVES);
    }
    #[test]
    fn extreme_coordinates_and_negative_scale_terminate() {
        let a = AxisProjection {
            scale: -1e-9,
            offset: 0.0,
            origin_nm: i64::MAX,
        };
        assert!(
            GridEngine::compute(&config(GridMark::Lines), 0, vp(), a, a).len()
                <= MAX_GRID_PRIMITIVES
        );
    }
    #[test]
    fn rectangular_pitch_and_origin_are_honored() {
        let lines = GridEngine::compute(
            &config(GridMark::Lines),
            1,
            vp(),
            axis(1.0, 0.0),
            axis(1.0, 0.0),
        );
        assert!(lines
            .iter()
            .any(|l| l.height == 80.0 && (l.x - 3.0).abs() < 0.01));
        assert!(lines
            .iter()
            .any(|l| l.width == 100.0 && (l.y - 7.0).abs() < 0.01));
    }
    #[test]
    fn cross_and_dot_have_distinct_representation() {
        let dots = GridEngine::compute(
            &config(GridMark::Dots),
            0,
            vp(),
            axis(1.0, 0.0),
            axis(1.0, 0.0),
        );
        let crosses = GridEngine::compute(
            &config(GridMark::Crosses),
            0,
            vp(),
            axis(1.0, 0.0),
            axis(1.0, 0.0),
        );
        assert!(!dots.is_empty() && crosses.len() == dots.len() * 2);
        assert!(dots.iter().all(|p| p.width == 1.0 && p.height == 1.0));
    }
    #[test]
    fn lod_hides_coarsens_and_refines_with_hysteresis() {
        let cfg = config(GridMark::Lines);
        assert_eq!(
            GridEngine::resolve_lod(&cfg, 0.09, GridLodState::default()).tier,
            None
        );
        assert_eq!(
            GridEngine::resolve_lod(&cfg, 1.0, GridLodState { tier: Some(2) }).tier,
            Some(1)
        );
        assert_eq!(
            GridEngine::resolve_lod(&cfg, 3.0, GridLodState { tier: Some(0) }).tier,
            Some(0)
        );
        assert_eq!(
            GridEngine::resolve_lod(&cfg, 9.0, GridLodState { tier: Some(0) }).tier,
            Some(1)
        );
    }
    #[test]
    fn invalid_inputs_emit_nothing() {
        assert!(GridEngine::compute(
            &config(GridMark::Lines),
            99,
            vp(),
            axis(1.0, 0.0),
            axis(1.0, 0.0)
        )
        .is_empty());
        assert!(GridEngine::compute(
            &config(GridMark::Lines),
            0,
            vp(),
            axis(f32::NAN, 0.0),
            axis(1.0, 0.0)
        )
        .is_empty());
    }
}
