//! Replace only declared native pin stubs; electrical terminals stay immutable.
use super::*;
use crate::library_browser::symbol_geometry::pin_stub_points;

pub(super) fn project(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    placed: &PlacedSymbol,
    symbol: &Symbol,
) -> Result<()> {
    for (index, pin) in placed.pins.iter().enumerate() {
        let Some(identity) = pin.library_pin else {
            continue;
        };
        let mut matches = symbol
            .pin_anchors
            .iter()
            .filter(|anchor| anchor.pin == identity);
        let Some(anchor) = matches.next() else {
            continue;
        };
        if matches.next().is_some() {
            continue;
        }
        let Some((from, to)) = pin_stub_points(anchor) else {
            continue;
        };
        let transform = |p: PointNm| {
            display_point(
                Point { x: p.x, y: p.y },
                placed.position,
                placed.rotation,
                placed.mirrored,
            )
        };
        let from = transform(from)?;
        // A newer definition must never relocate a placed electrical connection.
        if from != point_nm(pin.position) {
            continue;
        }
        let to = transform(to)?;
        let id = format!("schematic-symbol-pin:{}:{index}", placed.uuid);
        if from == to {
            graphics.retain(|g| g.object_id != id);
        } else if let Some(graphic) = graphics.iter_mut().find(|g| g.object_id == id) {
            graphic.path = vec![from, to];
        } else {
            graphics.push(BoardGraphicPrimitive {
                object_id: id,
                object_kind: "schematic_pin".into(),
                primitive_kind: "line".into(),
                source_object_uuid: pin.uuid.to_string(),
                layer_id: SCHEMATIC_SYMBOL_LAYER.into(),
                path: vec![from, to],
                holes: Vec::new(),
                width_nm: Some(PIN_STROKE_NM),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "native_symbol_pins_tests.rs"]
mod tests;
