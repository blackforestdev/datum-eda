//! Per-surface viewport configuration (spec §1.3, §5).
//!
//! A surface is one [`ViewportProfile`] bundling small `…Config` structs — never
//! new mechanism. Slice S1a populates [`GridConfig`] (the per-surface input the
//! shared [`crate::grid::GridEngine`] reads); later slices (S1b+) populate the
//! camera, snap, hover, selection, tool, menu, readout, and layer engines that
//! read the remaining config.

use crate::stroke::WeightClass;

/// Grid layout mode (spec §5): a square grid uses one pitch on both axes; a
/// rectangular grid allows an independent pitch per axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridMode {
    /// Equal pitch on X and Y.
    Square,
    /// Independent pitch per axis.
    Rectangular,
}

/// One zoom LOD tier of a [`GridConfig`]: the major pitch and its optional
/// finer minor pitch, both in nanometres. A tier with `minor_pitch_nm == None`
/// draws only the major grid (the coarse board tier drops the minor lines).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridTier {
    /// Major grid pitch, in nanometres.
    pub major_pitch_nm: i64,
    /// Minor grid pitch, in nanometres; `None` suppresses the minor grid.
    pub minor_pitch_nm: Option<i64>,
}

/// Per-surface grid configuration (spec §5).
///
/// The static input the shared [`crate::grid::GridEngine`] reads each frame: the
/// zoom-indexed pitch tiers, the line weight class (screen-constant for the
/// class-A grid chrome), and the minor/major colours. Colours arrive as resolved
/// rgb so this crate stays token-agnostic (the surface owns its design tokens).
#[derive(Debug, Clone)]
pub struct GridConfig {
    /// Square vs rectangular layout.
    pub mode: GridMode,
    /// Grid-line stroke weight; the grid is class-A [`WeightClass::ScreenConstant`].
    pub weight: WeightClass,
    /// Resolved rgb for the minor grid lines.
    pub minor_color: [f32; 3],
    /// Resolved rgb for the major grid lines.
    pub major_color: [f32; 3],
    /// Zoom LOD tiers, indexed by the surface's resolved detail level
    /// (coarse→fine). Empty until a profile populates it.
    pub tiers: Vec<GridTier>,
    /// Grid origin, in nanometres `(x, y)`. Placeholder for the §5 origin marker.
    pub origin_nm: Option<(i64, i64)>,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            mode: GridMode::Square,
            weight: WeightClass::ScreenConstant(1.0),
            minor_color: [0.0, 0.0, 0.0],
            major_color: [0.0, 0.0, 0.0],
            tiers: Vec::new(),
            origin_nm: None,
        }
    }
}

/// Per-surface participation in the shared hover mechanism.
///
/// Object geometry remains surface-owned, but whether that surface contributes
/// hit regions and receives pre-highlight is profile policy rather than a
/// renderer branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoverConfig {
    /// Whether pointer hit-testing and hover pre-highlight are active.
    pub enabled: bool,
}

impl Default for HoverConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Per-surface participation in the shared cursor-overlay mechanism.
///
/// The user's selected presentation (full viewport, local, or hidden) is
/// session state. This profile flag only states whether the surface supports
/// that shared mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorConfig {
    /// Whether the shared cursor overlay may be drawn on this surface.
    pub enabled: bool,
}

impl Default for CursorConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Per-surface profile scaffold (spec §1.3).
///
/// SCAFFOLD: grid, stroke, hover, and cursor policy are wired here; camera, snap,
/// selection, tool, menu, readout, and layer configs land with their engines in
/// later slices. Kept intentionally
/// minimal so no field is speculative mechanism.
#[derive(Debug, Clone, Default)]
pub struct ViewportProfile {
    /// Grid pitch tiers, mode, weight, and colours (spec §5).
    pub grid: GridConfig,
    /// Stroke primitive → weight-class map placeholder (spec §4.2). Populated by
    /// later slices as surfaces repoint onto the shared [`WeightClass`] model;
    /// each entry pairs a stringly-named primitive with its resolved class until
    /// the typed primitive enum lands.
    pub stroke: Vec<(String, WeightClass)>,
    /// Participation in shared pointer hit-testing and pre-highlight.
    pub hover: HoverConfig,
    /// Participation in the shared cursor overlay.
    pub cursor: CursorConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interaction_mechanisms_are_enabled_by_default() {
        let profile = ViewportProfile::default();
        assert!(profile.hover.enabled);
        assert!(profile.cursor.enabled);
    }
}
