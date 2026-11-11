//! Designators preserve captured anchor data and feed existing native placement.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::path::{Path, PathBuf};

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "saved-definition-test",
        CommitSource::Manual,
        "Edit saved source",
    )
}

fn fixture() -> (PathBuf, DesignModel, Uuid, Uuid, Uuid) {
    let root = crate::api::native_write::test_support::temp_project_root("anchor_designator");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Saved definitions".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, pin, symbol) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let mut logical = library::pool_unit_payload(unit, "Logical", "Maker");
    logical["pins"] = json!({pin.to_string(): {"uuid":pin,"name":"Input","direction":"Input","swap_group":7,"alternates":[{"name":"alternate","kind":"alias"}],"vendor":{"retain":true}}});
    logical["extension"] = json!({"exact":[null,3]});
    let mut graphic = library::pool_symbol_payload(symbol, unit, "Graphic");
    graphic["drawings"] = json!([
        {"Circle":{"center":{"x":11,"y":22},"radius":300000,"width":1}},
        {"Line":{"from":{"x":1,"y":2},"to":{"x":3,"y":4},"width":1}}
    ]);
    graphic["fields"] =
        json!([{"key":"same","value":"first","extra":9},{"key":"same","value":"second"}]);
    graphic["pin_anchors"] = json!([{"pin":pin,"number":"1","position":{"x":-1000000,"y":0,"metadata":"keep"},"orientation":"Right","length_nm":500000,"extension":true}]);
    graphic["extension"] = json!({"untouched":true});
    let prepared = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "units", unit),
                object: logical,
            },
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "symbols", symbol),
                object: graphic,
            },
        ],
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    (root, model, unit, pin, symbol)
}

fn prepare(
    model: &DesignModel,
    root: &Path,
    kind: &str,
    id: Uuid,
    fields: Value,
) -> Result<PreparedWrite, EngineError> {
    super::super::build(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        LibraryDefinitionRequest {
            kind: kind.into(),
            object_id: id,
            pool_path: "pool".into(),
            fields: fields.as_object().unwrap().clone(),
        },
        false,
    )
}

#[test]
fn saved_anchor_designator_exact_pose_and_consumer_history() {
    use crate::api::native_write::schematic_library::materialize_pool_symbol_pins;
    let (root, mut model, _, pin, symbol) = fixture();
    let path = format!("pool/symbols/{symbol}.json");
    let read =
        |model: &DesignModel| library::pool_library_object_payload(model, &path, symbol).unwrap();
    let before = read(&model);
    let raw = &before["pin_anchors"][0];
    let mut poses = before["pin_anchors"].clone();
    poses[0]["position"]["y"] = json!(500000);
    let make =
        |previous: &Value, number: Value| json!({"index":0,"previous":previous,"number":number});
    for fields in [
        json!({"anchor_designators":[make(&json!({}),json!("A7"))]}),
        json!({"anchor_designators":[make(raw,json!(7))]}),
        json!({"anchor_designators":[make(raw,json!("A7")),make(raw,json!("A8"))]}),
        json!({"anchor_designators":[make(raw,json!("A7"))],"anchor_removals":[pin]}),
    ] {
        assert!(prepare(&model, &root, "symbols", symbol, fields).is_err());
        assert_eq!(read(&model), before);
    }
    let request = json!({"pin_anchors":poses,"anchor_designators":[make(raw,json!(" A7 Ω "))]});
    let stale = prepare(&model, &root, "symbols", symbol, request.clone()).unwrap();
    let prepared = prepare(&model, &root, "symbols", symbol, request).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    let mut expected = before.clone();
    expected["pin_anchors"] = poses;
    expected["pin_anchors"][0]["number"] = json!(" A7 Ω ");
    assert_eq!(read(&model), expected);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(read(&model), before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(read(&cold), expected);
    let check = |model: &DesignModel, number| {
        let pins = materialize_pool_symbol_pins(model, Some(&symbol.to_string())).unwrap();
        assert_eq!(pins.len(), 1);
        assert_eq!(pins[0].library_pin, Some(pin));
        assert_eq!(pins[0].name, "Input");
        assert_eq!(pins[0].number, number);
    };
    check(&cold, " A7 Ω ");
    for (number, expected_number) in [(json!(""), ""), (Value::Null, "Input")] {
        let raw = read(&model);
        let p = prepare(
            &model,
            &root,
            "symbols",
            symbol,
            json!({"anchor_designators":[make(&raw["pin_anchors"][0],number.clone())]}),
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        check(&model, expected_number);
        if number.is_null() {
            assert!(read(&model)["pin_anchors"][0].get("number").is_none());
        }
    }
}
