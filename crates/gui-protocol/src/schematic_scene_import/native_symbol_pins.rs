//! Replace only declared native pin stubs; electrical terminals stay immutable.
use super::*;
use crate::library_browser::symbol_geometry::pin_stub_points;

pub(super) fn project(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    placed: &PlacedSymbol,
    symbol: &Symbol,
) -> Result<()> {
    for (index, pin) in placed.pins.iter().enumerate() {
        let Some((from, to, _)) = declared_stub(placed, pin, symbol)? else {
            continue;
        };
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

fn declared_stub(
    placed: &PlacedSymbol,
    pin: &eda_engine::schematic::SymbolPin,
    symbol: &Symbol,
) -> Result<Option<(PointNm, PointNm, [i128; 2])>> {
    let Some(identity) = pin.library_pin else {
        return Ok(None);
    };
    let mut matches = symbol
        .pin_anchors
        .iter()
        .filter(|anchor| anchor.pin == identity);
    let Some(anchor) = matches.next() else {
        return Ok(None);
    };
    if matches.next().is_some() {
        return Ok(None);
    }
    let Some((from, to)) = pin_stub_points(anchor) else {
        return Ok(None);
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
        return Ok(None);
    }
    let to = transform(to)?;
    let direction = if from == to {
        use eda_engine::pool::SymbolPinOrientation::*;
        let vector = match anchor.style.orientation {
            Left => Point::new(-1_000_000, 0),
            Right => Point::new(1_000_000, 0),
            Up => Point::new(0, -1_000_000),
            Down => Point::new(0, 1_000_000),
        };
        let vector = display_point(vector, Point::new(0, 0), placed.rotation, placed.mirrored)?;
        [vector.x.into(), vector.y.into()]
    } else {
        [
            i128::from(to.x) - i128::from(from.x),
            i128::from(to.y) - i128::from(from.y),
        ]
    };
    Ok(Some((from, to, direction)))
}

pub(super) fn project_labels(
    scene: &mut BoardReviewSceneV1,
    body_start: usize,
    placed: &PlacedSymbol,
    symbol: &Symbol,
    style: super::native_pin_label_layout::LabelStyle,
) -> Result<()> {
    for (index, pin) in placed.pins.iter().enumerate() {
        if let Some((from, to, direction)) = declared_stub(placed, pin, symbol)? {
            super::native_pin_label_layout::project(
                scene,
                body_start,
                placed.uuid,
                index,
                pin,
                from,
                to,
                direction,
                style,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "native_symbol_pins_tests.rs"]
mod tests;
