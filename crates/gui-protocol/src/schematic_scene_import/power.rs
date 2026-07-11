use eda_engine::schematic::PlacedSymbol;
use eda_engine::text::{TextHAlign, TextVAlign};
use uuid::Uuid;

use super::common::*;
use crate::*;

/// True for KiCad power symbols (`power:+3V3`, `power:GND`, ...), keyed on the
/// `power:` library prefix (case-insensitive).
pub(super) fn is_power_symbol(symbol: &PlacedSymbol) -> bool {
    symbol
        .lib_id
        .as_deref()
        .map(|lib| lib.to_ascii_lowercase().starts_with("power:"))
        .unwrap_or(false)
}

/// Distinguishes a ground symbol from a positive rail by name: the symbol part of
/// the lib_id (after `:`) containing GND / GROUND / VSS / EARTH is a ground stack;
/// anything else (VCC, +3V3, VEE, PWR_FLAG, ...) is a rail flag.
fn is_ground_power_symbol(lib_id: &str) -> bool {
    let name = lib_id.rsplit(':').next().unwrap_or(lib_id).to_ascii_uppercase();
    ["GND", "GROUND", "VSS", "EARTH"]
        .iter()
        .any(|token| name.contains(token))
}

/// The net name a power symbol labels (its `Value`, e.g. `+3V3`; falling back to
/// the lib_id symbol name).
fn power_net_name(symbol: &PlacedSymbol) -> String {
    if !symbol.value.trim().is_empty() {
        return symbol.value.clone();
    }
    symbol
        .lib_id
        .as_deref()
        .and_then(|lib| lib.rsplit(':').next())
        .unwrap_or("")
        .to_string()
}

/// P2.2e: a power symbol as prototype GEOMETRY on `Schematic.Power` (`--pwr`)
/// instead of a generic IEC box: a rail flag (stem + one bar) for a positive rail,
/// or a ground stack (stem + three shrinking bars) for a ground. Power symbols
/// skeleton-import with no pins, so geometry is anchored at the symbol origin —
/// rail extending up (-y), ground down (+y).
pub(super) fn push_power_symbol_graphics(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    text: &mut SchematicTextSink,
    symbol: &PlacedSymbol,
) {
    let origin = point_nm(symbol.position);
    let lib = symbol.lib_id.as_deref().unwrap_or("");
    let ground = is_ground_power_symbol(lib);
    // sign: +y (down) for ground, -y (up) for a rail flag.
    let sign = if ground { 1 } else { -1 };

    // Stem from the connection origin to the flag/stack.
    let stem_end = PointNm {
        x: origin.x,
        y: origin.y + sign * POWER_STEM_NM,
    };
    push_power_line(graphics, points, symbol.uuid, "stem", origin, stem_end);

    if ground {
        // Three shrinking horizontal bars beyond the stem.
        for (index, half) in [
            POWER_GND_BAR0_HALF_NM,
            POWER_GND_BAR1_HALF_NM,
            POWER_GND_BAR2_HALF_NM,
        ]
        .into_iter()
        .enumerate()
        {
            let y = stem_end.y + sign * (index as i64) * POWER_GND_BAR_GAP_NM;
            push_power_line(
                graphics,
                points,
                symbol.uuid,
                &format!("gnd-bar:{index}"),
                PointNm {
                    x: origin.x - half,
                    y,
                },
                PointNm {
                    x: origin.x + half,
                    y,
                },
            );
        }
    } else {
        // One horizontal bar at the top of the stem.
        push_power_line(
            graphics,
            points,
            symbol.uuid,
            "rail-bar",
            PointNm {
                x: origin.x - POWER_RAIL_BAR_HALF_NM,
                y: stem_end.y,
            },
            PointNm {
                x: origin.x + POWER_RAIL_BAR_HALF_NM,
                y: stem_end.y,
            },
        );
    }

    // Net name past the flag/stack (above a rail, below a ground stack).
    let far_y = if ground {
        stem_end.y + sign * (2 * POWER_GND_BAR_GAP_NM + POWER_LABEL_GAP_NM)
    } else {
        stem_end.y + sign * POWER_LABEL_GAP_NM
    };
    let (v_align, layer_int) = if ground {
        (TextVAlign::Top, SCHEMATIC_VALUE_TEXT_LAYER_INT)
    } else {
        (TextVAlign::Bottom, SCHEMATIC_VALUE_TEXT_LAYER_INT)
    };
    text.push(
        points,
        symbol.uuid,
        "power-net",
        &power_net_name(symbol),
        PointNm {
            x: origin.x,
            y: far_y,
        },
        POWER_LABEL_HEIGHT_NM,
        TextHAlign::Center,
        v_align,
        layer_int,
    );
}

fn push_power_line(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    symbol_uuid: Uuid,
    key: &str,
    from: PointNm,
    to: PointNm,
) {
    let path = vec![from, to];
    points.extend(path.iter().copied());
    graphics.push(BoardGraphicPrimitive {
        object_id: format!("schematic-power:{symbol_uuid}:{key}"),
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "line".to_string(),
        source_object_uuid: symbol_uuid.to_string(),
        layer_id: SCHEMATIC_POWER_LAYER.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(POWER_STROKE_NM),
    });
}
