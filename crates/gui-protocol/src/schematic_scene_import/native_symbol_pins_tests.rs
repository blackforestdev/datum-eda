use super::*;
use eda_engine::schematic::{HiddenPowerBehavior, PinElectricalType, SymbolDisplayMode, SymbolPin};
use serde_json::json;

fn placed(rotation: i32, mirrored: bool, terminal: (i64, i64)) -> PlacedSymbol {
    PlacedSymbol {
        uuid: Uuid::new_v4(),
        part: None,
        entity: None,
        gate: None,
        lib_id: None,
        reference: "U1".into(),
        value: "User symbol".into(),
        fields: vec![],
        pins: vec![SymbolPin {
            uuid: Uuid::new_v4(),
            library_pin: Some(Uuid::from_u128(3)),
            number: "A".into(),
            name: "A".into(),
            electrical_type: PinElectricalType::Passive,
            position: Point {
                x: terminal.0,
                y: terminal.1,
            },
        }],
        position: Point {
            x: 10_000_000,
            y: 20_000_000,
        },
        rotation,
        mirrored,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![],
        hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
    }
}

fn definition() -> Symbol {
    serde_json::from_value(
        json!({"uuid":Uuid::new_v4(),"name":"User symbol","unit":Uuid::from_u128(1),
        "pin_anchors":[{"pin":Uuid::from_u128(3),"position":{"x":-2_000_000,"y":0},
            "style":{"orientation":"Right","length_nm":1_000_000,"decoration":"none"}}]}),
    )
    .unwrap()
}

fn old_graphics(placed: &PlacedSymbol) -> Vec<BoardGraphicPrimitive> {
    let pin = &placed.pins[0];
    let old = BoardGraphicPrimitive {
        object_id: format!("schematic-symbol-pin:{}:0", placed.uuid),
        object_kind: "schematic_pin".into(),
        primitive_kind: "line".into(),
        source_object_uuid: pin.uuid.to_string(),
        layer_id: SCHEMATIC_SYMBOL_LAYER.into(),
        path: vec![PointNm { x: 0, y: 0 }, point_nm(pin.position)],
        holes: vec![],
        width_nm: Some(PIN_STROKE_NM),
    };
    let mut marker = old.clone();
    marker.object_id = format!("schematic-symbol-pin-terminal:{}:0", placed.uuid);
    marker.primitive_kind = "circle".into();
    marker.path = vec![point_nm(pin.position)];
    vec![old, marker]
}

#[test]
fn declared_symbol_pin_projection_keeps_terminal_identity_through_cardinal_poses() {
    let definition = definition();
    for (rotation, mirrored, terminal, end) in [
        (0, false, (8_000_000, 20_000_000), (9_000_000, 20_000_000)),
        (
            90,
            false,
            (10_000_000, 18_000_000),
            (10_000_000, 19_000_000),
        ),
        (
            180,
            false,
            (12_000_000, 20_000_000),
            (11_000_000, 20_000_000),
        ),
        (
            270,
            false,
            (10_000_000, 22_000_000),
            (10_000_000, 21_000_000),
        ),
        (0, true, (12_000_000, 20_000_000), (11_000_000, 20_000_000)),
        (90, true, (10_000_000, 22_000_000), (10_000_000, 21_000_000)),
    ] {
        let source = placed(rotation, mirrored, terminal);
        let before = source.clone();
        let mut graphics = old_graphics(&source);
        let marker = graphics[1].clone();
        project(&mut graphics, &source, &definition).unwrap();
        assert_eq!(
            graphics[0].path,
            vec![
                PointNm {
                    x: terminal.0,
                    y: terminal.1
                },
                PointNm { x: end.0, y: end.1 }
            ]
        );
        assert_eq!(
            graphics[0].source_object_uuid,
            source.pins[0].uuid.to_string()
        );
        assert_eq!(
            graphics[0].schematic_hit_kind(),
            Some(SchematicHitKind::Pin)
        );
        assert_eq!(graphics[1], marker);
        assert_eq!(source, before);
    }
}

#[test]
fn declared_symbol_pin_projection_retains_missing_or_moved_source_fallback() {
    let source = placed(0, false, (8_000_000, 20_000_000));
    let original = old_graphics(&source);
    let mut definition = definition();
    definition.pin_anchors[0].style.length_nm = None;
    let mut graphics = original.clone();
    project(&mut graphics, &source, &definition).unwrap();
    assert_eq!(graphics, original);
    definition.pin_anchors[0].style.length_nm = Some(1_000_000);
    definition.pin_anchors[0].position.x = -3_000_000;
    project(&mut graphics, &source, &definition).unwrap();
    assert_eq!(graphics, original);
    definition.pin_anchors[0].position.x = -2_000_000;
    definition.pin_anchors[0].style.length_nm = Some(0);
    project(&mut graphics, &source, &definition).unwrap();
    assert_eq!(graphics, vec![original[1].clone()]);
}

#[test]
fn narrow_native_body_keeps_long_opposed_names_at_owning_anchors_through_poses() {
    let mut symbol = definition();
    symbol.drawings = vec![eda_engine::pool::Primitive::Rect {
        min: Point::new(-1_000_000, -500_000),
        max: Point::new(1_000_000, 500_000),
        width: 150_000,
    }];
    let mut opposite = symbol.pin_anchors[0].clone();
    opposite.pin = Uuid::from_u128(4);
    opposite.position.x = 2_000_000;
    opposite.style.orientation = eda_engine::pool::SymbolPinOrientation::Left;
    symbol.pin_anchors.push(opposite);
    let raw_before = serde_json::to_value(&symbol).unwrap();
    for rotation in [0, 90, 180, 270] {
        for mirrored in [false, true] {
            let mut source = placed(rotation, mirrored, (0, 0));
            let transform = |x| {
                super::super::display_point(Point::new(x, 0), source.position, rotation, mirrored)
                    .unwrap()
            };
            let left = transform(-2_000_000);
            let right = transform(2_000_000);
            source.pins[0].position = Point::new(left.x, left.y);
            source.pins[0].name = "TERU_IN".into();
            source.pins[0].number = "TERU_IN".into();
            let mut pin = source.pins[0].clone();
            pin.uuid = Uuid::new_v4();
            pin.library_pin = Some(Uuid::from_u128(4));
            pin.position = Point::new(right.x, right.y);
            pin.name = "TERU_OUT".into();
            pin.number = "TERU_OUT".into();
            source.pins.push(pin);
            let unchanged = serde_json::to_value(&source).unwrap();
            let mut scene = ReviewWorkspaceState::unopened().scene;
            super::super::replace_body(&mut scene, &source, &symbol).unwrap();
            for (index, pin) in source.pins.iter().enumerate() {
                let (terminal, body_side, direction) =
                    declared_stub(&source, pin, &symbol).unwrap().unwrap();
                let name = scene
                    .board_texts
                    .iter()
                    .find(|text| {
                        text.text_uuid
                            == text_uuid(source.uuid, &format!("pin-name:{index}")).to_string()
                    })
                    .unwrap();
                assert_eq!(name.text, pin.name);
                let expected = PointNm {
                    x: body_side.x + direction[0].signum() as i64 * 50_800,
                    y: body_side.y + direction[1].signum() as i64 * 50_800,
                };
                assert_eq!(
                    name.position, expected,
                    "only own stub ink clearance, rotation={rotation}, mirrored={mirrored}"
                );
                let corners = crate::selection_geometry_catalog::layout_polygon(
                    scene
                        .board_text_geometries
                        .iter()
                        .find(|g| g.text_uuid == name.text_uuid)
                        .unwrap(),
                )
                .unwrap();
                let min: [i128; 2] = std::array::from_fn(|i| {
                    corners
                        .iter()
                        .map(|p| p[i].numerator.div_euclid(p[i].denominator))
                        .min()
                        .unwrap()
                });
                let max: [i128; 2] = std::array::from_fn(|i| {
                    corners
                        .iter()
                        .map(|p| {
                            p[i].numerator.div_euclid(p[i].denominator)
                                + i128::from(p[i].numerator.rem_euclid(p[i].denominator) != 0)
                        })
                        .max()
                        .unwrap()
                });
                let a = [i128::from(terminal.x), i128::from(terminal.y)];
                let b = [i128::from(body_side.x), i128::from(body_side.y)];
                assert!((0..2).any(
                    |i| max[i] <= a[i].min(b[i]) - 50_800 || min[i] >= a[i].max(b[i]) + 50_800
                ));
                assert!((0..2).any(|i| max[i] <= a[i] - 240_000 || min[i] >= a[i] + 240_000));
                assert_eq!(name.source_object_uuid, Some(pin.uuid.to_string()));
                let number = scene
                    .board_texts
                    .iter()
                    .find(|text| {
                        text.text_uuid
                            == text_uuid(source.uuid, &format!("pin-number:{index}")).to_string()
                    })
                    .unwrap();
                assert_eq!(number.text, pin.number);
                assert_eq!(number.source_object_uuid, Some(pin.uuid.to_string()));
            }
            assert_eq!(serde_json::to_value(&source).unwrap(), unchanged);
            assert_eq!(serde_json::to_value(&symbol).unwrap(), raw_before);
        }
    }
}
