//! Shared screen-space grid engine (spec §5 "Grid Engine").
//!
//! The single grid mechanism every `EditorViewport` surface consumes. It
//! computes an ordered list of screen-space (device-pixel) grid-line rects for
//! the current camera. It is deliberately abstract: it never sees gui-render's
//! `Projection` or gui-protocol's scene types — a surface hands it a pixel
//! viewport, a per-axis world-nm→screen-px affine, the world extent to span, and
//! its per-surface [`GridConfig`]. This keeps the crate on std only (UVT-002)
//! and lets the board and (S1b) the schematic share one weight-stable engine.
//!
//! ## Slice S1a
//!
//! First real mechanism + first consumer wiring: gui-render's board grid
//! (`push_scene_grid`) repoints onto this engine and must stay byte-identical.
//! The schematic grid switches in S1b.

use crate::profile::GridConfig;

/// The device-pixel rect the grid spans (the surface's viewport).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridViewport {
    /// Left edge, device px.
    pub x: f32,
    /// Top edge, device px.
    pub y: f32,
    /// Width, device px.
    pub width: f32,
    /// Height, device px.
    pub height: f32,
}

/// One axis's affine world-nm → screen-px projection.
///
/// Reproduces the board renderer's per-axis map exactly:
/// `px = offset + (nm - origin_nm) as f32 * scale`. The `nm - origin_nm`
/// subtraction is done in i64 and only then cast to f32 — matching the renderer
/// bit-for-bit, so the shared engine is byte-identical on the board surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisProjection {
    /// Device-pixels per nanometre for this axis (the live camera scale).
    pub scale: f32,
    /// Screen-px offset at `origin_nm`.
    pub offset: f32,
    /// World-nm coordinate the offset is anchored to (the axis bounds origin).
    pub origin_nm: i64,
}

impl AxisProjection {
    /// Project a world-nm coordinate to a device-pixel position on this axis.
    #[inline]
    pub fn project(&self, nm: i64) -> f32 {
        self.offset + (nm - self.origin_nm) as f32 * self.scale
    }
}

/// The world-nm extent the grid iterates across (typically the scene bounds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridBounds {
    /// Minimum X, nm.
    pub min_x: i64,
    /// Minimum Y, nm.
    pub min_y: i64,
    /// Maximum X, nm.
    pub max_x: i64,
    /// Maximum Y, nm.
    pub max_y: i64,
}

/// One computed grid line as an abstract device-pixel rect + rgb colour. The
/// surface turns each into its own quad/vertex primitive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridLine {
    /// Left edge, device px.
    pub x: f32,
    /// Top edge, device px.
    pub y: f32,
    /// Width, device px.
    pub width: f32,
    /// Height, device px.
    pub height: f32,
    /// Resolved rgb.
    pub color: [f32; 3],
}

/// The shared grid mechanism.
pub struct GridEngine;

impl GridEngine {
    /// Compute the ordered grid lines for the current camera.
    ///
    /// `tier` indexes `config.tiers` (the consumer's resolved LOD; the board
    /// keeps its `detail_tier` selection on its own side). Emission order is
    /// minor-then-major, each vertical-then-horizontal — the exact order the
    /// board previously pushed its quads, so the retained output is
    /// byte-identical. An out-of-range `tier` yields no lines.
    pub fn compute(
        config: &GridConfig,
        tier: usize,
        viewport: GridViewport,
        bounds: GridBounds,
        x_axis: AxisProjection,
        y_axis: AxisProjection,
    ) -> Vec<GridLine> {
        let mut lines = Vec::new();
        let Some(tier) = config.tiers.get(tier) else {
            return lines;
        };
        // Class-A screen-constant weight: 1 device px on the board, resolved
        // against the live scale (zoom-invariant for `ScreenConstant`).
        let line_px = config.weight.resolve_px(x_axis.scale);

        // Minor first (if present), then major — each pass vertical then
        // horizontal, matching the board's historical quad order exactly.
        let passes = [
            tier.minor_pitch_nm.map(|p| (p, config.minor_color)),
            Some((tier.major_pitch_nm, config.major_color)),
        ];
        for (pitch_nm, color) in passes.into_iter().flatten() {
            if pitch_nm <= 0 {
                continue;
            }
            // Vertical lines: a full-height rect at each pitch step across X.
            let mut x = floor_multiple(bounds.min_x, pitch_nm);
            let end_x = ceil_multiple(bounds.max_x, pitch_nm);
            while x <= end_x {
                lines.push(GridLine {
                    x: x_axis.project(x),
                    y: viewport.y,
                    width: line_px,
                    height: viewport.height,
                    color,
                });
                x += pitch_nm;
            }
            // Horizontal lines: a full-width rect at each pitch step across Y.
            let mut y = floor_multiple(bounds.min_y, pitch_nm);
            let end_y = ceil_multiple(bounds.max_y, pitch_nm);
            while y <= end_y {
                lines.push(GridLine {
                    x: viewport.x,
                    y: y_axis.project(y),
                    width: viewport.width,
                    height: line_px,
                    color,
                });
                y += pitch_nm;
            }
        }
        lines
    }
}

/// Largest multiple of `pitch` at or below `value` (euclidean floor).
fn floor_multiple(value: i64, pitch: i64) -> i64 {
    value.div_euclid(pitch) * pitch
}

/// Smallest multiple of `pitch` at or above `value` (euclidean ceil).
fn ceil_multiple(value: i64, pitch: i64) -> i64 {
    if value.rem_euclid(pitch) == 0 {
        value
    } else {
        value.div_euclid(pitch) * pitch + pitch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{GridMode, GridTier};
    use crate::stroke::WeightClass;

    fn board_like_config() -> GridConfig {
        GridConfig {
            mode: GridMode::Square,
            weight: WeightClass::ScreenConstant(1.0),
            minor_color: [0.1, 0.1, 0.1],
            major_color: [0.2, 0.2, 0.2],
            tiers: vec![
                GridTier {
                    major_pitch_nm: 10_000_000,
                    minor_pitch_nm: None,
                },
                GridTier {
                    major_pitch_nm: 5_000_000,
                    minor_pitch_nm: Some(2_500_000),
                },
            ],
            origin_nm: None,
        }
    }

    fn identity_axis(origin_nm: i64) -> AxisProjection {
        // 1 px per 1e6 nm (1 px/mm), zero offset.
        AxisProjection {
            scale: 1e-6,
            offset: 0.0,
            origin_nm,
        }
    }

    /// The coarse tier (no minor pitch) emits only major lines, vertical then
    /// horizontal, spanning the bounds inclusive of the ceil endpoint.
    #[test]
    fn coarse_tier_emits_major_only() {
        let cfg = board_like_config();
        let vp = GridViewport {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let bounds = GridBounds {
            min_x: 0,
            min_y: 0,
            max_x: 20_000_000,
            max_y: 20_000_000,
        };
        let lines = GridEngine::compute(
            &cfg,
            0,
            vp,
            bounds,
            identity_axis(bounds.min_x),
            identity_axis(bounds.min_y),
        );
        // 0,10,20 M nm → 3 vertical + 3 horizontal.
        assert_eq!(lines.len(), 6);
        assert!(lines.iter().all(|l| l.color == cfg.major_color));
        // First three are vertical (1 px wide, full height).
        assert!(lines[..3]
            .iter()
            .all(|l| l.width == 1.0 && l.height == 100.0));
        // Last three are horizontal (full width, 1 px tall).
        assert!(lines[3..]
            .iter()
            .all(|l| l.width == 100.0 && l.height == 1.0));
    }

    /// A tier with a minor pitch emits minor lines first, then major.
    #[test]
    fn minor_lines_precede_major() {
        let cfg = board_like_config();
        let vp = GridViewport {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
        };
        let bounds = GridBounds {
            min_x: 0,
            min_y: 0,
            max_x: 5_000_000,
            max_y: 5_000_000,
        };
        let lines = GridEngine::compute(
            &cfg,
            1,
            vp,
            bounds,
            identity_axis(bounds.min_x),
            identity_axis(bounds.min_y),
        );
        // Minor 2.5M: 0,2.5,5 → 3 v + 3 h = 6. Major 5M: 0,5 → 2 v + 2 h = 4.
        assert_eq!(lines.len(), 10);
        assert!(lines[..6].iter().all(|l| l.color == cfg.minor_color));
        assert!(lines[6..].iter().all(|l| l.color == cfg.major_color));
    }

    /// An out-of-range tier index yields no lines.
    #[test]
    fn out_of_range_tier_is_empty() {
        let cfg = board_like_config();
        let vp = GridViewport {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        let bounds = GridBounds {
            min_x: 0,
            min_y: 0,
            max_x: 10,
            max_y: 10,
        };
        assert!(GridEngine::compute(&cfg, 9, vp, bounds, identity_axis(0), identity_axis(0)).is_empty());
    }
}
