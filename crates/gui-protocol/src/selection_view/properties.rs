//! One read-only class field vocabulary over canonical engine source values.
use super::*;
pub(super) fn fields(class: Class) -> &'static [(&'static str, &'static str)] {
    match class {
        Class::Footprint => &[
            ("placement_position_nm", "position"),
            ("placement_rotation", "rotation"),
            ("reference", "reference"),
            ("value", "value"),
            ("locked", "locked"),
        ],
        Class::Symbol => &[
            ("placement_position_nm", "position"),
            ("placement_rotation", "rotation"),
            ("reference", "reference"),
            ("value", "value"),
        ],
        Class::Track => &[
            ("copper_width_nm", "width"),
            ("board_layer", "layer"),
            ("board_net_id", "net"),
            ("section_start_nm", "from"),
            ("section_end_nm", "to"),
            ("section_midpoint_nm", "midpoint"),
        ],
        Class::Via => &[
            ("position_nm", "position"),
            ("copper_diameter_nm", "diameter"),
            ("drill_diameter_nm", "drill"),
            ("board_net_id", "net"),
            ("from_layer", "from_layer"),
            ("to_layer", "to_layer"),
        ],
        Class::Zone => &[
            ("board_layer", "layer"),
            ("board_net_id", "net"),
            ("priority", "priority"),
        ],
        Class::BoardText => &[
            ("text", "text"),
            ("position_nm", "position"),
            ("text_rotation", "rotation"),
            ("text_height_nm", "height_nm"),
            ("text_stroke_width_nm", "stroke_width_nm"),
            ("text_family", "family"),
            ("text_style", "style"),
            ("text_h_align", "h_align"),
            ("text_v_align", "v_align"),
            ("text_mirrored", "mirrored"),
            ("text_bold", "bold"),
            ("text_italic", "italic"),
            ("text_keep_upright", "keep_upright"),
            ("text_line_spacing_ppm", "line_spacing_ratio_ppm"),
            ("board_layer", "layer"),
        ],
        Class::SchematicText => &[
            ("text", "text"),
            ("position_nm", "position"),
            ("text_rotation", "rotation"),
            ("text_height_nm", "height_nm"),
            ("text_stroke_width_nm", "stroke_width_nm"),
            ("text_family", "family"),
            ("text_style", "style"),
            ("text_h_align", "h_align"),
            ("text_v_align", "v_align"),
            ("text_mirrored", "mirrored"),
            ("text_bold", "bold"),
            ("text_italic", "italic"),
            ("text_keep_upright", "keep_upright"),
            ("text_line_spacing_ppm", "line_spacing_ratio_ppm"),
        ],
        Class::Label => &[
            ("net_label_name", "name"),
            ("net_label_kind", "kind"),
            ("position_nm", "position"),
        ],
        Class::Port => &[
            ("port_name", "name"),
            ("port_direction", "direction"),
            ("position_nm", "position"),
        ],
        Class::Junction | Class::NoConnect => &[("position_nm", "position")],
        Class::Wire => &[("section_start_nm", "from"), ("section_end_nm", "to")],
        Class::Pin => &[
            ("pin_number", "number"),
            ("pin_name", "name"),
            ("pin_electrical_type", "electrical_type"),
            ("terminal_position_nm", "position"),
        ],
        // No universal property/lock infrastructure, graphic field invention or
        // definition-editor schema is introduced by read-only S5A.
        _ => &[],
    }
}
pub(super) fn values(class: Class, value: &Value) -> BTreeMap<String, Value> {
    // Engine deserialization owns existing defaulted text semantics; scene
    // defaults and rounded Inspector formatting never supply canonical values.
    let normalized = if class == Class::BoardText {
        serde_json::from_value::<eda_engine::board::BoardText>(value.clone())
            .ok()
            .and_then(|text| serde_json::to_value(text).ok())
    } else {
        None
    };
    let value = normalized.as_ref().unwrap_or(value);
    fields(class)
        .iter()
        .filter_map(|(semantic, source)| {
            value
                .get(source)
                .filter(|v| !v.is_null())
                .map(|v| ((*semantic).into(), v.clone()))
        })
        .collect()
}

/// Existing canonical point or first authored path vertex, not a scene/layout
/// center. No generated Zone component or missing graphic anchor is invented.
pub(super) fn anchor(class: Class, value: &Value) -> Option<crate::PointNm> {
    let point = match class {
        Class::Track | Class::Wire => value.get("from")?,
        Class::BusSection => value.get("segments")?.as_array()?.first()?,
        Class::BoardOutline => value.get("outline")?.get("vertices")?.as_array()?.first()?,
        Class::Zone | Class::BoardGraphic | Class::SchematicGraphic => return None,
        _ => value.get("position")?,
    };
    Some(crate::PointNm {
        x: point.get("x")?.as_i64()?,
        y: point.get("y")?.as_i64()?,
    })
}
