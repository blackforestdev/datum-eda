use eda_engine::ir::geometry::Point;
use eda_engine::schematic::SchematicPrimitive;

use super::common::*;
use crate::*;

pub(super) fn push_drawing_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    drawing: &SchematicPrimitive,
) {
    // Free graphic drawings are symbol-body geometry: colour them `--sym` grey.
    match drawing {
        SchematicPrimitive::Line { uuid, from, to } => push_line_graphic(
            graphics,
            points,
            format!("schematic-drawing-line:{uuid}"),
            *uuid,
            *from,
            *to,
            SCHEMATIC_STROKE_NM,
            SCHEMATIC_SYMBOL_LAYER,
        ),
        SchematicPrimitive::Rect { uuid, min, max } => {
            let center = Point {
                x: (min.x + max.x) / 2,
                y: (min.y + max.y) / 2,
            };
            push_rect_graphic(
                graphics,
                points,
                text,
                format!("schematic-drawing-rect:{uuid}"),
                *uuid,
                center,
                (max.x - min.x).abs() / 2,
                (max.y - min.y).abs() / 2,
                None,
                SCHEMATIC_SYMBOL_LAYER,
                SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
            );
        }
        SchematicPrimitive::Circle {
            uuid,
            center,
            radius,
        } => push_circle_graphic(
            graphics,
            points,
            format!("schematic-drawing-circle:{uuid}"),
            *uuid,
            *center,
            (*radius).max(SCHEMATIC_STROKE_NM),
            SCHEMATIC_SYMBOL_LAYER,
        ),
        SchematicPrimitive::Arc { uuid, arc } => {
            let path = arc_path_points(*arc);
            points.extend(path.iter().copied());
            graphics.push(BoardGraphicPrimitive {
                object_id: format!("schematic-drawing-arc:{uuid}"),
                object_kind: "schematic_graphic".to_string(),
                primitive_kind: "polyline".to_string(),
                source_object_uuid: uuid.to_string(),
                layer_id: SCHEMATIC_SYMBOL_LAYER.to_string(),
                path,
                holes: Vec::new(),
                width_nm: Some(SCHEMATIC_STROKE_NM),
            });
        }
    }
}
