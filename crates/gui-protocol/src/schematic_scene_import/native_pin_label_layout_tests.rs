//! Shared native pin labels retain identity and clear the actual rendered ink.
use super::*;
use eda_engine::schematic::PinElectricalType;
use serde_json::json;

fn pin() -> SymbolPin {
    SymbolPin {
        uuid: Uuid::from_u128(7),
        library_pin: Some(Uuid::from_u128(3)),
        number: "S17".into(),
        name: "IN".into(),
        electrical_type: PinElectricalType::Input,
        position: Point { x: -5080000, y: 0 },
    }
}

fn body(from: PointNm, to: PointNm) -> BoardGraphicPrimitive {
    BoardGraphicPrimitive {
        object_id: "thick-body-edge".into(),
        object_kind: "schematic_symbol".into(),
        primitive_kind: "line".into(),
        source_object_uuid: Uuid::from_u128(1).to_string(),
        layer_id: SCHEMATIC_SYMBOL_LAYER.into(),
        path: vec![from, to],
        holes: vec![],
        width_nm: Some(254000),
    }
}

#[test]
fn native_pin_label_clearance_uses_real_stroke_and_terminal_through_cardinal_poses() {
    let pin = pin();
    let id = Uuid::from_u128(1);
    for (from, to, a, b) in [
        (
            (-5080000, 0),
            (-2540000, 0),
            (-2540000, -2540000),
            (-2540000, 2540000),
        ),
        (
            (5080000, 0),
            (2540000, 0),
            (2540000, -2540000),
            (2540000, 2540000),
        ),
        (
            (0, -5080000),
            (0, -2540000),
            (-2540000, -2540000),
            (2540000, -2540000),
        ),
        (
            (0, 5080000),
            (0, 2540000),
            (-2540000, 2540000),
            (2540000, 2540000),
        ),
    ] {
        let point = |(x, y)| PointNm { x, y };
        let (from, to, a, b) = (point(from), point(to), point(a), point(b));
        let mut scene = ReviewWorkspaceState::unopened().scene;
        scene.board_graphics.push(body(a, b));
        let unchanged = scene.board_graphics.clone();
        project(
            &mut scene,
            0,
            id,
            0,
            &pin,
            from,
            to,
            [
                i128::from(to.x) - i128::from(from.x),
                i128::from(to.y) - i128::from(from.y),
            ],
            LabelStyle::default(),
        )
        .unwrap();
        assert_eq!(scene.board_graphics, unchanged);
        let mut labels = Vec::new();
        for (key, text) in [("pin-name:0", "IN"), ("pin-number:0", "S17")] {
            let uuid = text_uuid(id, key).to_string();
            let primitive = scene
                .board_texts
                .iter()
                .find(|t| t.text_uuid == uuid)
                .unwrap();
            assert_eq!(primitive.text, text);
            assert_eq!(primitive.source_object_uuid, Some(pin.uuid.to_string()));
            let geometry = scene
                .board_text_geometries
                .iter()
                .find(|g| g.text_uuid == uuid)
                .unwrap();
            assert!(!geometry.glyphs.is_empty());
            let ink = ink_bounds(geometry).unwrap();
            for obstacle in [
                Bounds::stroke(a, b, radius(schematic_stroke::SYMBOL_OUTLINE_NM)),
                Bounds::stroke(from, to, radius(schematic_stroke::PIN_LINE_NM)),
                Bounds::stroke(from, from, LabelStyle::default().terminal_radius.into()),
            ] {
                assert!(!ink.overlaps(obstacle));
            }
            labels.push(ink);
        }
        assert!(!labels[0].overlaps(labels[1]));
    }
}

#[test]
fn native_pin_label_clearance_preview_reuses_source_geometry_and_label() {
    let pin = pin();
    let raw = json!({"uuid":Uuid::from_u128(1),"name":"Thick Symbol","unit":Uuid::from_u128(2),
        "drawings":[{"Rect":{"min":{"x":-2540000,"y":-2540000},"max":{"x":2540000,"y":2540000},"width":254000}}],
        "pin_anchors":[{"pin":pin.library_pin.unwrap(),"number":"S17","position":pin.position,
            "style":{"orientation":"Right","length_nm":2540000,"decoration":"none"}}]});
    let unit = json!({"pins":{pin.library_pin.unwrap().to_string():{"name":"IN"}}});
    let scene =
        crate::library_browser::symbol_geometry::preview::project(&raw, Some(&unit)).unwrap();
    let stub = scene
        .board_graphics
        .iter()
        .find(|g| g.object_kind == "schematic_pin")
        .unwrap();
    assert_eq!(
        stub.path,
        [PointNm { x: -5080000, y: 0 }, PointNm { x: -2540000, y: 0 }]
    );
    assert_eq!(stub.width_nm, Some(PIN_STROKE_NM));
    let labels: Vec<_> = scene.board_texts.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(labels, ["S17", "IN"]);
}

#[test]
fn native_pin_label_clearance_fitted_preview_preserves_readable_scale_and_square_marker() {
    let pin = pin();
    let raw = json!({"uuid":Uuid::from_u128(1),"name":"Large Symbol","unit":Uuid::from_u128(2),
        "drawings":[{"Rect":{"min":{"x":-50000000,"y":-1000000},"max":{"x":50000000,"y":1000000},"width":254000}}],
        "pin_anchors":[{"pin":pin.library_pin.unwrap(),"number":"S17","position":{"x":-51000000,"y":0},
            "style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]});
    let unit = json!({"pins":{pin.library_pin.unwrap().to_string():{"name":"IN"}}});
    let mut scene =
        crate::library_browser::symbol_geometry::preview::project(&raw, Some(&unit)).unwrap();
    let graphics = scene.board_graphics.clone();
    let nm_per_pixel = 600_000.;
    crate::library_browser::symbol_geometry::preview::fit_pin_labels(
        &mut scene,
        &raw,
        Some(&unit),
        nm_per_pixel,
    )
    .unwrap();
    assert_eq!(scene.board_graphics, graphics);
    let marker = PointNm { x: -51000000, y: 0 };
    let marker = Bounds::stroke(marker, marker, 1_200_000);
    for text in &scene.board_texts {
        assert_eq!(text.height_nm, 6_000_000);
        let geometry = scene
            .board_text_geometries
            .iter()
            .find(|g| g.text_uuid == text.text_uuid)
            .unwrap();
        assert!(!ink_bounds(geometry).unwrap().overlaps(marker));
    }
    assert_eq!(scene.board_texts.len(), 2);
}
