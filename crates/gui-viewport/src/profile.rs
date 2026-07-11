//! Per-surface viewport configuration scaffold (spec §1.3, §5).
//!
//! SCAFFOLD ONLY (slice S0). These structs name the fields the spec calls for so
//! that later slices have a stable shape to grow into; they carry no behaviour
//! yet. A surface is one [`ViewportProfile`] bundling small `…Config` structs —
//! never new mechanism. Slices S1+ populate the grid, camera, snap, hover,
//! selection, tool, menu, readout, and layer engines that read this config.

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

/// Grid configuration scaffold (spec §5).
///
/// SCAFFOLD: the pitch tiers and origin are placeholders for the S1 GridEngine
/// (adaptive-LOD, screen-space class-A rects). Pitches are in nanometres; the
/// tier list is ordered coarse↔fine and consumed by the §4.4 LOD threshold.
#[derive(Debug, Clone, Default)]
pub struct GridConfig {
    /// Ordered grid pitch tiers, in nanometres (e.g. board 2.5/5/10 mm,
    /// schematic 1.27/2.54 mm). Empty until an S1 profile populates it.
    pub pitch_tiers_nm: Vec<i64>,
    /// Square vs rectangular layout.
    pub mode: Option<GridMode>,
    /// Grid origin, in nanometres `(x, y)`. Placeholder for §5 origin marker.
    pub origin_nm: Option<(i64, i64)>,
}

/// Per-surface profile scaffold (spec §1.3).
///
/// SCAFFOLD: only the grid config and the stroke primitive→weight-class map are
/// stubbed here; the camera / snap / hover / selection / tool / menu / readout /
/// layer configs land with their engines in slices S1+. Kept intentionally
/// minimal so no field is speculative mechanism.
#[derive(Debug, Clone, Default)]
pub struct ViewportProfile {
    /// Grid pitch table, mode, and origin (spec §5).
    pub grid: GridConfig,
    /// Stroke primitive → weight-class map placeholder (spec §4.2). Populated by
    /// later slices as surfaces repoint onto the shared [`WeightClass`] model;
    /// each entry pairs a stringly-named primitive with its resolved class until
    /// the typed primitive enum lands.
    pub stroke: Vec<(String, WeightClass)>,
}
