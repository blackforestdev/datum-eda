mod copper;
mod excellon;
mod export_error;
mod export_surface;
mod formatting;
mod gerber_mechanical;
mod mask;
mod outline;
mod silkscreen;

pub use copper::render_rs274x_copper_layer;
pub use excellon::render_excellon_drill;
pub use export_error::ExportError;
use formatting::{format_coord, format_mm_6, parse_mm_6_to_nm, render_polygon_points};
pub use gerber_mechanical::{MechanicalStroke, render_rs274x_mechanical_layer};
pub use mask::{render_rs274x_paste_layer, render_rs274x_soldermask_layer};
use outline::DEFAULT_OUTLINE_APERTURE_MM;
pub use export_surface::{
    render_rs274x_outline, render_rs274x_outline_default, render_rs274x_silkscreen_layer,
};
pub use silkscreen::{SilkscreenStroke, render_silkscreen_text_strokes};

#[cfg(test)]
mod tests;
