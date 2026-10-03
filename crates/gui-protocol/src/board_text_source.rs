use super::*;

pub(crate) fn board_text_primitive(text: &BoardText) -> BoardTextPrimitive {
    BoardTextPrimitive {
        object_id: format!("board-text:{}", text.uuid),
        object_kind: "board_text".to_string(),
        text_uuid: text.uuid.to_string(),
        source_object_uuid: None,
        text: text.text.clone(),
        layer_id: layer_id(text.layer),
        position: PointNm {
            x: text.position.x,
            y: text.position.y,
        },
        rotation_degrees: text.rotation,
        height_nm: text.height_nm,
        stroke_width_nm: text.stroke_width_nm,
        render_intent: render_intent_to_string(text.render_intent).to_string(),
        family: text.family.0.clone(),
        style: text.style.0.clone(),
        style_class: text.style_class.clone(),
        h_align: h_align_to_string(text.h_align).to_string(),
        v_align: v_align_to_string(text.v_align).to_string(),
        mirrored: text.mirrored,
        keep_upright: text.keep_upright,
        line_spacing_ratio_ppm: text.line_spacing_ratio_ppm,
        bold: text.bold,
        italic: text.italic,
    }
}

pub(crate) fn render_intent_to_string(intent: TextRenderIntent) -> &'static str {
    match intent {
        TextRenderIntent::Manufacturing => "manufacturing",
        TextRenderIntent::Annotation => "annotation",
        TextRenderIntent::Branding => "branding",
        TextRenderIntent::Documentation => "documentation",
        TextRenderIntent::UiPreview => "ui_preview",
    }
}

pub(crate) fn h_align_to_string(align: TextHAlign) -> &'static str {
    match align {
        TextHAlign::Left => "left",
        TextHAlign::Center => "center",
        TextHAlign::Right => "right",
    }
}

pub(crate) fn v_align_to_string(align: TextVAlign) -> &'static str {
    match align {
        TextVAlign::Top => "top",
        TextVAlign::Center => "center",
        TextVAlign::Bottom => "bottom",
    }
}
