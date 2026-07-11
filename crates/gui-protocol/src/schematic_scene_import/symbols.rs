use eda_engine::schematic::{PlacedSymbol, SymbolPin};
use eda_engine::text::{TextHAlign, TextVAlign};
use uuid::Uuid;

use super::common::*;
use crate::*;

/// Projects one placed symbol at full fidelity: an IEC rectangular body sized
/// from the pin envelope, a pin line + terminal marker per pin, refdes/value
/// text near the body, and pin name/number text. Pins carry absolute positions
/// (rotation baked in at import), so the body is derived from where the pins sit
/// relative to the symbol origin and every pin line meets the terminal a wire
/// already connects to.
pub(super) fn push_symbol_graphics(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    symbol: &PlacedSymbol,
) {
    let center = point_nm(symbol.position);
    let (half_w, half_h) = symbol_body_half_extents(center, &symbol.pins);

    // 1. IEC rectangular body (hollow, `--sym` grey stroke).
    push_rect_graphic(
        graphics,
        points,
        text,
        format!("schematic-symbol:{}", symbol.uuid),
        symbol.uuid,
        symbol.position,
        half_w,
        half_h,
        None,
        SCHEMATIC_SYMBOL_LAYER,
        SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
    );

    // 2. Pin lines + terminal markers, 4. pin name/number text.
    for (index, pin) in symbol.pins.iter().enumerate() {
        push_symbol_pin(graphics, points, text, symbol.uuid, center, half_w, half_h, index, pin);
    }

    // 3. Refdes above the body, value below it.
    let refdes = if symbol.reference.is_empty() {
        symbol.lib_id.clone().unwrap_or_default()
    } else {
        symbol.reference.clone()
    };
    text.push(
        points,
        symbol.uuid,
        "refdes",
        &refdes,
        PointNm {
            x: center.x,
            y: center.y - half_h - SYMBOL_TEXT_GAP_NM,
        },
        REFDES_HEIGHT_NM,
        TextHAlign::Center,
        TextVAlign::Bottom,
        SCHEMATIC_REFDES_TEXT_LAYER_INT,
    );
    text.push(
        points,
        symbol.uuid,
        "value",
        &symbol.value,
        PointNm {
            x: center.x,
            y: center.y + half_h + SYMBOL_TEXT_GAP_NM,
        },
        VALUE_HEIGHT_NM,
        TextHAlign::Center,
        TextVAlign::Top,
        SCHEMATIC_VALUE_TEXT_LAYER_INT,
    );
}

/// Derives the symbol body half-extents from the pin envelope. Pins that stick
/// out horizontally (|dx| >= |dy|) set the width inset by a pin stub; the body
/// must still enclose the perpendicular spread of the other pins so no pin
/// origin falls inside a face it does not belong to.
fn symbol_body_half_extents(center: PointNm, pins: &[SymbolPin]) -> (i64, i64) {
    if pins.is_empty() {
        return (SYMBOL_HALF_WIDTH_NM, SYMBOL_HALF_HEIGHT_NM);
    }
    let mut stick_x = 0_i64; // furthest horizontal pin reach
    let mut stick_y = 0_i64; // furthest vertical pin reach
    let mut enclose_x = 0_i64; // widest x spread among vertical pins
    let mut enclose_y = 0_i64; // tallest y spread among horizontal pins
    for pin in pins {
        let dx = (pin.position.x - center.x).abs();
        let dy = (pin.position.y - center.y).abs();
        if dx >= dy {
            stick_x = stick_x.max(dx);
            enclose_y = enclose_y.max(dy);
        } else {
            stick_y = stick_y.max(dy);
            enclose_x = enclose_x.max(dx);
        }
    }
    let pin_stub = (stick_x.max(stick_y) / 3).clamp(MIN_PIN_STUB_NM, MAX_PIN_STUB_NM);
    let half_w = if stick_x > 0 {
        (stick_x - pin_stub).max(MIN_BODY_HALF_NM).max(enclose_x)
    } else {
        MIN_BODY_HALF_NM.max(enclose_x)
    };
    let half_h = if stick_y > 0 {
        (stick_y - pin_stub).max(MIN_BODY_HALF_NM).max(enclose_y)
    } else {
        MIN_BODY_HALF_NM.max(enclose_y)
    };
    (half_w, half_h)
}

#[allow(clippy::too_many_arguments)]
fn push_symbol_pin(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    symbol_uuid: Uuid,
    center: PointNm,
    half_w: i64,
    half_h: i64,
    index: usize,
    pin: &SymbolPin,
) {
    let terminal = point_nm(pin.position);
    let dx = terminal.x - center.x;
    let dy = terminal.y - center.y;
    let horizontal = dx.abs() >= dy.abs();

    // Body-edge attach point for this pin, on the face it exits.
    let (edge, name_anchor, name_h, name_v, number_anchor, number_h, number_v) = if horizontal {
        let sign = if dx >= 0 { 1 } else { -1 };
        let edge = PointNm {
            x: center.x + sign * half_w,
            y: terminal.y,
        };
        // Name inside the body next to the edge; number outside on the stub.
        let (name_h, number_h) = if sign >= 0 {
            (TextHAlign::Right, TextHAlign::Left)
        } else {
            (TextHAlign::Left, TextHAlign::Right)
        };
        (
            edge,
            PointNm {
                x: edge.x - sign * PIN_NAME_INSET_NM,
                y: terminal.y,
            },
            name_h,
            TextVAlign::Center,
            PointNm {
                x: edge.x + sign * PIN_NUMBER_OUTSET_NM,
                y: terminal.y,
            },
            number_h,
            TextVAlign::Bottom,
        )
    } else {
        let sign = if dy >= 0 { 1 } else { -1 };
        let edge = PointNm {
            x: terminal.x,
            y: center.y + sign * half_h,
        };
        let (name_v, number_v) = if sign >= 0 {
            (TextVAlign::Top, TextVAlign::Bottom)
        } else {
            (TextVAlign::Bottom, TextVAlign::Top)
        };
        (
            edge,
            PointNm {
                x: terminal.x,
                y: edge.y - sign * PIN_NAME_INSET_NM,
            },
            TextHAlign::Center,
            name_v,
            PointNm {
                x: terminal.x,
                y: edge.y + sign * PIN_NUMBER_OUTSET_NM,
            },
            TextHAlign::Center,
            number_v,
        )
    };

    // Pin line body-edge -> terminal, only when the terminal actually sits
    // outside the body face (degenerate/inner pins get just a marker).
    let outside = if horizontal {
        (terminal.x - edge.x).signum() == dx.signum() && terminal.x != edge.x
    } else {
        (terminal.y - edge.y).signum() == dy.signum() && terminal.y != edge.y
    };
    if outside {
        let path = vec![edge, terminal];
        points.extend(path.iter().copied());
        graphics.push(BoardGraphicPrimitive {
            object_id: format!("schematic-symbol-pin:{}:{index}", symbol_uuid),
            object_kind: "schematic_graphic".to_string(),
            primitive_kind: "line".to_string(),
            source_object_uuid: pin.uuid.to_string(),
            layer_id: SCHEMATIC_SYMBOL_LAYER.to_string(),
            path,
            holes: Vec::new(),
            width_nm: Some(PIN_STROKE_NM),
        });
    }

    // Terminal marker at the wire attach point.
    push_circle_graphic(
        graphics,
        points,
        format!("schematic-symbol-pin-terminal:{}:{index}", symbol_uuid),
        pin.uuid,
        pin.position,
        PIN_TERMINAL_RADIUS_NM,
        SCHEMATIC_SYMBOL_LAYER,
    );

    // Pin name (inside the body) and pin number (outside on the stub).
    text.push(
        points,
        symbol_uuid,
        &format!("pin-name:{index}"),
        &pin.name,
        name_anchor,
        PIN_NAME_HEIGHT_NM,
        name_h,
        name_v,
        SCHEMATIC_PIN_NAME_TEXT_LAYER_INT,
    );
    text.push(
        points,
        symbol_uuid,
        &format!("pin-number:{index}"),
        &pin.number,
        number_anchor,
        PIN_NUMBER_HEIGHT_NM,
        number_h,
        number_v,
        SCHEMATIC_PIN_NUMBER_TEXT_LAYER_INT,
    );
}
