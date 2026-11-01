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
