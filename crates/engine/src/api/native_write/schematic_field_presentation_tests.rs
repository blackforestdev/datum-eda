use super::*;
use crate::{
    api::native_write::schematic_symbol_move::SymbolMoveBasis,
    ir::geometry::Point,
    substrate::{ElectricalSelectionSnapshot, electrical_test_partitions},
};

fn move_request(
    f: &BindingFixture,
    at: Point,
    angle: i32,
    mirrored: bool,
) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb("datum.schematic.move_symbol")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        provenance(),
        json!({"symbol":f.symbol.uuid,"instance_path":[],"x_nm":at.x,"y_nm":at.y,
            "rotation_deg":angle,"mirrored":mirrored}),
    )
}

fn install(f: &mut BindingFixture, symbol: Value) {
    let prepared = BatchComposer::compose(&f.model, provenance())
        .push_op(Operation::SetSchematicSymbol {
            sheet_id: f.sheet,
            symbol_id: f.symbol.uuid,
            symbol,
        })
        .primary_object(f.symbol.uuid)
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
}

#[test]
fn native_field_presentation_edits_position_visibility_and_metadata_atomically() {
    let (mut f, first, second) = setup("native_field_presentation_history");
    let mut expected = raw(&f.model, f.symbol.uuid);
    expected["fields"][0]["position"]["coordinate_extension"] = json!("retained");
    install(&mut f, expected.clone());
    let before = bytes(&f.root);
    let prepared = build(
        &f,
        json!({"field_id":first,"key":"Changed","value":"data",
        "visible":false,"position":{"x":50,"y":-60}}),
    )
    .unwrap();
    assert_eq!(
        bytes(&f.root),
        before,
        "prepare/cancel preserves source and journal"
    );
    expected["fields"][0]["key"] = json!("Changed");
    expected["fields"][0]["value"] = json!("data");
    expected["fields"][0]["visible"] = json!(false);
    expected["fields"][0]["position"]["x"] = json!(50);
    expected["fields"][0]["position"]["y"] = json!(-60);
    roundtrip(&mut f, prepared, &expected);
    let prepared = build(&f, json!({"field_id":first,"visible":true})).unwrap();
    expected["fields"][0]["visible"] = json!(true);
    roundtrip(&mut f, prepared, &expected);
    let prepared = build(
        &f,
        json!({"field_id":second,"visible":true,"position":{"x":80,"y":90}}),
    )
    .unwrap();
    expected["fields"][1]["visible"] = json!(true);
    expected["fields"][1]["position"] = json!({"x":80,"y":90});
    roundtrip(&mut f, prepared, &expected);
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn native_field_presentation_invalid_combinations_and_noops_do_not_write() {
    let (f, first, _) = setup("native_field_presentation_invalid");
    let before = bytes(&f.root);
    for patch in [
        json!({"key":"new","value":"x","visible":false}),
        json!({"key":"new","value":"x","position":{"x":1,"y":2}}),
        json!({"field_id":first,"clear":true,"visible":false}),
        json!({"field_id":first,"clear":true,"position":{"x":1,"y":2}}),
        json!({"field_id":first,"position":{"x":1}}),
        json!({"field_id":first,"position":{"x":1.5,"y":2}}),
        json!({"field_id":first,"visible":true}),
        json!({"field_id":first,"position":{"x":123,"y":-456}}),
        json!({"field_id":Uuid::new_v4(),"position":{"x":1,"y":2}}),
    ] {
        assert!(build(&f, patch.clone()).is_err(), "accepted {patch}");
        assert_eq!(bytes(&f.root), before);
    }
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn native_field_presentation_parent_pose_carries_hidden_fields_and_preserves_raw_history() {
    let (mut f, _, _) = setup("native_field_presentation_parent_pose");
    let mut original = raw(&f.model, f.symbol.uuid);
    original["position"]["coordinate_extension"] = json!("origin");
    original["pins"][0]["position"]["coordinate_extension"] = json!("pin");
    original["fields"][0]["position"] = json!({"x":30,"y":10,"coordinate_extension":"field"});
    original["fields"][1]["position"] = json!({"x":-20,"y":40});
    original["fields"]
        .as_array_mut()
        .unwrap()
        .push(json!({"uuid":Uuid::new_v4(),
        "key":"unpositioned","value":"retained","visible":false,"extension":7}));
    install(&mut f, original);
    let partitions = electrical_test_partitions(&f.model, &[]).unwrap();
    let cases = [
        (0, false, [(30, 10), (-20, 40)]),
        (90, false, [(-10, 30), (-40, -20)]),
        (180, false, [(-30, -10), (20, -40)]),
        (270, false, [(10, -30), (40, 20)]),
        (0, true, [(-30, 10), (20, 40)]),
        (90, true, [(-10, -30), (-40, 20)]),
        (180, true, [(30, -10), (-20, -40)]),
        (270, true, [(10, 30), (40, -20)]),
    ];
    for (angle, mirrored, positions) in cases {
        let at = Point::new(1000, 2000);
        let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
        let basis = SymbolMoveBasis::capture(&snapshot, f.symbol.uuid, &[]).unwrap();
        let before = bytes(&f.root);
        let preview = basis
            .destination_with_pose(at, Some(angle), Some(mirrored))
            .unwrap();
        assert_eq!(bytes(&f.root), before);
        let mut expected = raw(&f.model, f.symbol.uuid);
        expected["position"]["x"] = json!(at.x);
        expected["position"]["y"] = json!(at.y);
        expected["rotation"] = json!(angle);
        expected["mirrored"] = json!(mirrored);
        expected["pins"][0]["position"]["x"] = json!(at.x);
        expected["pins"][0]["position"]["y"] = json!(at.y);
        for (index, (x, y)) in positions.into_iter().enumerate() {
            let position = Point::new(at.x + x, at.y + y);
            assert_eq!(preview.symbol.fields[index].position, Some(position));
            expected["fields"][index]["position"]["x"] = json!(position.x);
            expected["fields"][index]["position"]["y"] = json!(position.y);
        }
        assert!(!preview.symbol.fields[1].visible);
        assert!(preview.symbol.fields[2].position.is_none());
        let prepared = move_request(&f, at, angle, mirrored).unwrap();
        roundtrip(&mut f, prepared, &expected);
        assert_eq!(
            electrical_test_partitions(&f.model, &[]).unwrap(),
            partitions
        );
    }
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn native_field_presentation_parent_refuses_field_overflow_without_writes() {
    let (mut f, first, _) = setup("native_field_presentation_overflow");
    let prepared = build(
        &f,
        json!({"field_id":first,"position":{"x":i64::MAX,"y":0}}),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let before = bytes(&f.root);
    let error = move_request(&f, Point::new(1, 0), 0, false).unwrap_err();
    assert!(error.to_string().contains("overflow"), "{error}");
    assert_eq!(bytes(&f.root), before);
    std::fs::remove_dir_all(f.root).unwrap();
}
