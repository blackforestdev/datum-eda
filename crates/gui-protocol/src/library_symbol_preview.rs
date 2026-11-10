//! Preview the native symbol with the same body/pin projection as placed symbols.
use crate::{BoardReviewSceneV1, ReviewWorkspaceState, SceneBounds};
use anyhow::{Context, Result, ensure};
use eda_engine::ir::geometry::Point;
use eda_engine::pool::{Symbol, SymbolPinDecoration};
use eda_engine::schematic::{HiddenPowerBehavior, PlacedSymbol, SymbolDisplayMode, SymbolPin};
use serde_json::Value;

pub fn project(raw: &Value, unit: Option<&Value>) -> Result<BoardReviewSceneV1> {
    let symbol: Symbol = serde_json::from_value(raw.clone())?;
    let pins = unit.and_then(|u| u.get("pins")).and_then(Value::as_object);
    let mut placed_pins = Vec::new();
    for anchor in &symbol.pin_anchors {
        ensure!(
            anchor.style.decoration == SymbolPinDecoration::None,
            "Preview unavailable for decorated pins"
        );
        super::pin_stub_points(anchor).context("Pin direction or length unavailable")?;
        let pin = pins.and_then(|p| p.get(&anchor.pin.to_string()));
        let name = pin
            .and_then(|p| p.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("Unresolved pin")
            .to_owned();
        placed_pins.push(SymbolPin {
            uuid: anchor.pin,
            library_pin: Some(anchor.pin),
            number: anchor.number.clone().unwrap_or_else(|| name.clone()),
            name,
            electrical_type: eda_engine::schematic::PinElectricalType::Passive,
            position: anchor.position,
        });
    }
    let placed = PlacedSymbol {
        uuid: symbol.uuid,
        part: None,
        entity: None,
        gate: None,
        lib_id: Some(symbol.uuid.to_string()),
        reference: String::new(),
        value: symbol.name.clone(),
        fields: Vec::new(),
        pins: placed_pins,
        position: Point::new(0, 0),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: Vec::new(),
        hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
    };
    let mut scene = ReviewWorkspaceState::unopened().scene;
    crate::schematic_scene_import::native_library::replace_body(&mut scene, &placed, &symbol)?;
    // Reuse captured world layout, including rotated bearings and advances.
    // Anchor positions alone do not describe the visible text envelope.
    let mut text_extents = Vec::new();
    for geometry in &scene.board_text_geometries {
        if let Some(corners) = crate::selection_geometry_catalog::layout_polygon(geometry) {
            for [x, y] in corners {
                let outward = |value: crate::selection_geometry::Rational| -> Result<(i64, i64)> {
                    let floor = value.numerator.div_euclid(value.denominator);
                    let ceil =
                        floor + i128::from(value.numerator.rem_euclid(value.denominator) != 0);
                    Ok((
                        i64::try_from(floor).context("Preview bounds overflow")?,
                        i64::try_from(ceil).context("Preview bounds overflow")?,
                    ))
                };
                let (min_x, max_x) = outward(x)?;
                let (min_y, max_y) = outward(y)?;
                text_extents.extend([
                    crate::PointNm { x: min_x, y: min_y },
                    crate::PointNm { x: max_x, y: max_y },
                ]);
            }
        }
        // Legacy fragments can coexist with an empty mesh layout basis.
        for fill in &geometry.fills {
            text_extents.extend(fill.outer.iter().copied());
            text_extents.extend(fill.holes.iter().flatten().copied());
        }
        for stroke in &geometry.strokes {
            let radius = (i128::from(stroke.width_nm).abs() + 1) / 2;
            for point in [stroke.from, stroke.to] {
                for sign in [-1i128, 1] {
                    text_extents.push(crate::PointNm {
                        x: i64::try_from(i128::from(point.x) + sign * radius)
                            .context("Preview bounds overflow")?,
                        y: i64::try_from(i128::from(point.y) + sign * radius)
                            .context("Preview bounds overflow")?,
                    });
                }
            }
        }
    }
    let points = scene
        .board_graphics
        .iter()
        .flat_map(|g| g.path.iter())
        .chain(scene.board_texts.iter().map(|t| &t.position))
        .chain(text_extents.iter())
        .collect::<Vec<_>>();
    let min_x = points.iter().map(|p| p.x).min().unwrap_or(-1);
    let min_y = points.iter().map(|p| p.y).min().unwrap_or(-1);
    let max_x = points.iter().map(|p| p.x).max().unwrap_or(1);
    let max_y = points.iter().map(|p| p.y).max().unwrap_or(1);
    ensure!(
        max_x.checked_sub(min_x).is_some() && max_y.checked_sub(min_y).is_some(),
        "Preview bounds overflow"
    );
    scene.bounds = SceneBounds {
        min_x,
        min_y,
        max_x,
        max_y,
    };
    Ok(scene)
}
