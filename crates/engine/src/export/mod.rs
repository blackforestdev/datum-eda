use crate::ir::geometry::Polygon;
use thiserror::Error;
mod copper;
mod excellon;
mod export_surface;
mod formatting;
mod gerber_mechanical;
mod mask;
mod outline;
mod silkscreen;

pub use copper::render_rs274x_copper_layer;
pub use excellon::render_excellon_drill;
use formatting::{format_coord, format_mm_6, parse_mm_6_to_nm, render_polygon_points};
pub use gerber_mechanical::{MechanicalStroke, render_rs274x_mechanical_layer};
pub use mask::{render_rs274x_paste_layer, render_rs274x_soldermask_layer};
use outline::DEFAULT_OUTLINE_APERTURE_MM;
pub use export_surface::{
    render_rs274x_outline, render_rs274x_outline_default, render_rs274x_silkscreen_layer,
};
pub use silkscreen::{SilkscreenStroke, render_silkscreen_text_strokes};

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("board outline export requires at least two vertices")]
    OutlineTooShort,
    #[error("outline aperture diameter must be positive")]
    InvalidAperture,
    #[error("copper-layer export requires positive track widths")]
    InvalidTrackWidth,
    #[error("copper-layer export requires positive via diameters")]
    InvalidViaDiameter,
    #[error("copper-layer export requires positive pad diameters")]
    InvalidPadDiameter,
    #[error("copper-layer export requires positive pad rectangle widths")]
    InvalidPadWidth,
    #[error("copper-layer export requires positive pad rectangle heights")]
    InvalidPadHeight,
    #[error("silkscreen export requires positive text heights")]
    InvalidTextHeight,
    #[error("silkscreen export requires positive text stroke widths")]
    InvalidTextStrokeWidth,
    #[error("silkscreen export encountered unsupported text character: {0}")]
    UnsupportedSilkscreenTextCharacter(char),
    #[error("drill export requires positive via drill diameters")]
    InvalidViaDrill,
}

#[cfg(test)]
mod tests;
