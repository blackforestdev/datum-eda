//! Placed custom fields use absolute sheet anchors and remain owned Symbol text.
use super::*;

pub(super) fn push_fields(
    sheet: &Sheet,
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
) {
    for symbol in sorted_values(&sheet.symbols) {
        for field in &symbol.fields {
            let Some(at) = field.position.filter(|_| field.visible) else {
                continue;
            };
            if field.value.trim().is_empty() {
                continue;
            }
            let key = format!("custom-field:{}", field.uuid);
            text.push(
                points,
                symbol.uuid,
                &key,
                &field.value,
                point_nm(at),
                1_270_000, // Rendering Book §6: schematic field default, not a legacy label size.
                TextHAlign::Left,
                TextVAlign::Center,
                SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
            );
            if at != symbol.position {
                // Attachment metadata is immediate selection/gesture chrome only:
                // no authored paint, hit kind, electrical snap or new selection subject.
                graphics.push(BoardGraphicPrimitive {
                    object_id: format!("board-text:{}:field-tether", text_uuid(symbol.uuid, &key)),
                    object_kind: "schematic_field_tether".into(),
                    primitive_kind: "tether".into(),
                    source_object_uuid: symbol.uuid.to_string(),
                    layer_id: SCHEMATIC_ANNOTATION_LAYER.into(),
                    path: vec![point_nm(symbol.position), point_nm(at)],
                    holes: vec![],
                    width_nm: None,
                });
            }
        }
    }
}
