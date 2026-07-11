use eda_engine::ir::geometry::Point;
use eda_engine::schematic::{HierarchicalPort, LabelKind, NetLabel, PortDirection};
use eda_engine::text::{TextHAlign, TextVAlign};
use uuid::Uuid;

use super::common::*;
use crate::*;

/// P2.2e: a net label by kind. `Global`/`Hierarchical` become the pointed pentagon
/// tag on `Schematic.GlobalLabel` (`--info` blue); `Local`/`Power` keep the chip.
pub(super) fn push_net_label_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    label: &NetLabel,
) {
    match label.kind {
        LabelKind::Global | LabelKind::Hierarchical => push_pentagon_tag(
            graphics,
            points,
            text,
            format!("schematic-label:{}", label.uuid),
            label.uuid,
            label.position,
            &label.name,
            1,
        ),
        LabelKind::Local | LabelKind::Power => push_rect_graphic(
            graphics,
            points,
            text,
            format!("schematic-label:{}", label.uuid),
            label.uuid,
            label.position,
            LABEL_HALF_WIDTH_NM,
            LABEL_HALF_HEIGHT_NM,
            Some(label.name.clone()),
            SCHEMATIC_ANNOTATION_LAYER,
            SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
        ),
    }
}

/// P2.2e: a hierarchical sheet port as a DIRECTIONAL pentagon tag on
/// `Schematic.GlobalLabel` (`--info`). The tag points away from the sheet by
/// direction (Output/Bidirectional extend +x; Input/Passive mirror to -x).
pub(super) fn push_hierarchical_port_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    port: &HierarchicalPort,
) {
    let dir_sign = match port.direction {
        PortDirection::Input | PortDirection::Passive => -1,
        PortDirection::Output | PortDirection::Bidirectional => 1,
    };
    push_pentagon_tag(
        graphics,
        points,
        text,
        format!("schematic-port:{}", port.uuid),
        port.uuid,
        port.position,
        &port.name,
        dir_sign,
    );
}

/// A pointed pentagon net-label / port tag: a flat body with one pointed end at the
/// net attach point (`position`). `dir_sign` = +1 extends the body +x (tip on the
/// left), -1 mirrors it. Rendered as a closed polyline on `Schematic.GlobalLabel`
/// with the name centred in the body.
#[allow(clippy::too_many_arguments)]
fn push_pentagon_tag(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    object_id: String,
    source_uuid: Uuid,
    position: Point,
    name: &str,
    dir_sign: i64,
) {
    let anchor = point_nm(position);
    let half_h = GLOBAL_LABEL_HALF_HEIGHT_NM;
    let char_count = name.trim().chars().count().max(1) as i64;
    let body_w = GLOBAL_LABEL_NOTCH_NM + char_count * GLOBAL_LABEL_CHAR_NM + GLOBAL_LABEL_PAD_NM;
    let notch_x = anchor.x + dir_sign * GLOBAL_LABEL_NOTCH_NM;
    let body_x = anchor.x + dir_sign * body_w;
    let path = vec![
        anchor,
        PointNm {
            x: notch_x,
            y: anchor.y - half_h,
        },
        PointNm {
            x: body_x,
            y: anchor.y - half_h,
        },
        PointNm {
            x: body_x,
            y: anchor.y + half_h,
        },
        PointNm {
            x: notch_x,
            y: anchor.y + half_h,
        },
        anchor,
    ];
    points.extend(path.iter().copied());
    graphics.push(BoardGraphicPrimitive {
        object_id,
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "polyline".to_string(),
        source_object_uuid: source_uuid.to_string(),
        layer_id: SCHEMATIC_GLOBAL_LABEL_LAYER.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(GLOBAL_LABEL_STROKE_NM),
    });
    text.push(
        points,
        source_uuid,
        "label-name",
        name,
        PointNm {
            x: (notch_x + body_x) / 2,
            y: anchor.y,
        },
        LABEL_TEXT_HEIGHT_NM,
        TextHAlign::Center,
        TextVAlign::Center,
        SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
    );
}
