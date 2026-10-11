//! Native canonical process painting and retained legacy process preview.
use super::*;

/// Geometry remains exact in the protocol; conversion here is presentation only.
pub(crate) fn paint_native_process(
    out: &mut impl Output<Quad>,
    process: &datum_gui_protocol::PlacedProcessGeometry,
    kind: PadProcessLayerKind,
    color: [f32; 3],
) {
    use datum_gui_protocol::EffectiveProcessAperture as A;
    let aperture = match kind {
        PadProcessLayerKind::Mask => &process.effective.mask,
        PadProcessLayerKind::Paste => &process.effective.paste,
    };
    let presentation = |q: datum_gui_protocol::Quadratic3| {
        (q.a.numerator as f64 / q.a.denominator as f64
            + q.b.numerator as f64 / q.b.denominator as f64 * 3.0_f64.sqrt()) as f32
    };
    let center = (
        presentation(process.center_nm[0]),
        presentation(process.center_nm[1]),
    );
    let (width, height, circle) = match aperture {
        A::Suppressed => return,
        A::Circle { diameter_nm } => (presentation(*diameter_nm), presentation(*diameter_nm), true),
        A::Rect {
            width_nm,
            height_nm,
        } => (presentation(*width_nm), presentation(*height_nm), false),
    };
    if !world_primitives::admit_outline_points(out, 64) {
        return;
    }
    let outline = if circle {
        ellipse_points(center, width, height, process.rotation as f32, 64)
    } else {
        rounded_rect_points(center, width, height, process.rotation as f32, 0.0)
    };
    world_primitives::push_convex_polygon_fill(out, &outline, color);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PadProcessLayerKind {
    Mask,
    Paste,
}

/// A native process face comes from its captured association, never a layer label.
pub(super) fn native_process_stage(
    process: &datum_gui_protocol::PlacedProcessGeometry,
    layer: &datum_gui_protocol::SceneLayer,
) -> Option<RenderStage> {
    let capability = process.source_capability.as_ref()?;
    let id: i32 = layer.layer_id.strip_prefix('L')?.parse().ok()?;
    match layer.kind.as_str() {
        "mask" if capability.front.mask_layer == Some(id) => Some(RenderStage::TopMask),
        "mask" if capability.back.mask_layer == Some(id) => Some(RenderStage::BottomMask),
        "paste" if capability.front.paste_layer == Some(id) => Some(RenderStage::TopPaste),
        "paste" if capability.back.paste_layer == Some(id) => Some(RenderStage::BottomPaste),
        _ => None,
    }
}

pub(super) fn native_process_color(stage: RenderStage) -> [f32; 3] {
    match stage {
        RenderStage::TopMask => TOP_MASK_OPENING,
        RenderStage::BottomMask => BOTTOM_MASK_OPENING,
        RenderStage::TopPaste => TOP_PASTE_OPENING,
        RenderStage::BottomPaste => BOTTOM_PASTE_OPENING,
        _ => unreachable!("process painter requires a typed process stage"),
    }
}

pub(crate) fn process_pad_bounds(
    pad: &datum_gui_protocol::PadPrimitive,
    kind: PadProcessLayerKind,
) -> datum_gui_protocol::RectNm {
    let (width_nm, height_nm) = pad_dimensions_nm(pad);
    let (expanded_width_nm, expanded_height_nm) = match kind {
        PadProcessLayerKind::Mask => {
            let clearance = pad.solder_mask_margin_nm as f32;
            (
                (width_nm + clearance * 2.0).max(1.0),
                (height_nm + clearance * 2.0).max(1.0),
            )
        }
        PadProcessLayerKind::Paste => {
            let clearance = pad.solder_paste_margin_nm as f32;
            let ratio = pad.solder_paste_margin_ratio_ppm as f32 / 1_000_000.0;
            (
                (width_nm + width_nm * ratio + clearance * 2.0).max(1.0),
                (height_nm + height_nm * ratio + clearance * 2.0).max(1.0),
            )
        }
    };
    let half_w = expanded_width_nm * 0.5;
    let half_h = expanded_height_nm * 0.5;
    let (center_x, center_y) = paint_pad_center(pad.center);
    datum_gui_protocol::RectNm {
        min_x: (center_x - half_w).round() as i64,
        min_y: (center_y - half_h).round() as i64,
        max_x: (center_x + half_w).round() as i64,
        max_y: (center_y + half_h).round() as i64,
    }
}

// Retain the original owned adapter only for the existing aperture reference tests.
#[cfg(test)]
pub(crate) fn derived_process_pad(
    pad: &datum_gui_protocol::PadPrimitive,
    process_layer_id: &str,
    kind: PadProcessLayerKind,
    _setup: &datum_gui_protocol::ScenePadExpansionSetup,
) -> datum_gui_protocol::PadPrimitive {
    let mut derived = pad.clone();
    derived.layer_id = process_layer_id.to_string();
    derived.bounds = process_pad_bounds(pad, kind);
    derived.drill_nm = None;
    derived
}

#[cfg(all(test, feature = "visual"))]
#[path = "process_pad_tests.rs"]
mod tests;
