//! Dim/selected/material color policy applied to retained authored geometry.

use super::*;

pub(crate) fn dim_with_policy(
    color: [f32; 3],
    dimmed: bool,
    factor: f32,
    floor: f32,
    board_mix: f32,
) -> [f32; 3] {
    if !dimmed {
        return color;
    }
    let dimmed = [
        (color[0] * factor).max(floor),
        (color[1] * factor).max(floor),
        (color[2] * factor).max(floor),
    ];
    mix_color(dimmed, BOARD_INNER_FIELD, board_mix)
}

pub(crate) fn dim_authored_color(color: [f32; 3], dimmed: bool) -> [f32; 3] {
    dim_with_policy(color, dimmed, AUTHORED_DIM_FACTOR, 0.14, 0.08)
}

pub(crate) fn dim_process_color(color: [f32; 3], dimmed: bool) -> [f32; 3] {
    dim_with_policy(color, dimmed, PROCESS_DIM_FACTOR, 0.18, 0.05)
}

pub(crate) fn dim_structural_color(color: [f32; 3], dimmed: bool) -> [f32; 3] {
    dim_with_policy(color, dimmed, STRUCTURAL_DIM_FACTOR, 0.12, 0.10)
}

pub(crate) fn dim_context_color(color: [f32; 3], dimmed: bool) -> [f32; 3] {
    dim_with_policy(color, dimmed, CONTEXT_DIM_FACTOR, 0.20, 0.04)
}

pub(crate) fn selected_copper_color(base: [f32; 3]) -> [f32; 3] {
    let tinted = mix_color(base, [0.95, 0.50, 0.92], 0.52);
    mix_color(tinted, [0.98, 0.94, 1.0], 0.12)
}

pub(crate) fn selected_silk_color(base: [f32; 3]) -> [f32; 3] {
    mix_color(base, [0.98, 0.98, 0.99], 0.72)
}

pub(crate) fn selected_mechanical_color(base: [f32; 3]) -> [f32; 3] {
    mix_color(base, [0.90, 0.94, 0.98], 0.45)
}

/// How far filled-zone copper is shaded from its base material toward the
/// board field. Refinement only: large enough to read pad/zone boundaries,
/// small enough that zones still read as the same copper material.
pub(crate) const ZONE_FILL_FIELD_MIX: f32 = 0.15;

impl LayerAppearance {
    /// Material-first construction for known copper families: every authored
    /// copper primitive family (tracks, pads, zone fill, zone outline)
    /// inherits the one base material color of the owning layer. `related`,
    /// `proposal`, and `silkscreen` are bounded refinements of that material,
    /// not separate per-primitive semantic color systems
    /// (`docs/gui/M7_RENDER_LAYER_DISCIPLINE_MEMO.md` product rule).
    pub(crate) fn from_copper_material(
        base: [f32; 3],
        related: [f32; 3],
        proposal: [f32; 3],
        silkscreen: [f32; 3],
    ) -> Self {
        Self {
            authored_track: base,
            pad_copper: base,
            pad_related: related,
            // M7-REN-004: filled zone copper is a derived shade of the same
            // base material (not an independent color system) so pad/track
            // boundaries against pours and teardrop fills stay readable --
            // e.g. teardrop flanks must visually read tangent to the pad
            // circle instead of merging into one undifferentiated mass.
            zone_fill: mix_color(base, BOARD_INNER_FIELD, ZONE_FILL_FIELD_MIX),
            zone_outline: base,
            proposal,
            silkscreen,
        }
    }
}
