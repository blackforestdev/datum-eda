//! Ordinary canonical Unit/Symbol creation, including refusal and stale preview.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
    schematic_library::materialize_pool_symbol_pins,
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::path::{Path, PathBuf};

fn fixture() -> (PathBuf, DesignModel) {
    let root = super::super::test_support::temp_project_root("electrical_library");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Manual symbol".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    (root, model)
}

fn prepare(
    model: &DesignModel,
    root: &Path,
    kind: &str,
    id: Uuid,
    fields: Value,
) -> Result<PreparedWrite, EngineError> {
    build_definition(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        WriteProvenance::new(
            "test",
            CommitSource::Manual,
            "Create electrical library object",
        ),
        LibraryDefinitionRequest {
            kind: kind.into(),
            object_id: id,
            pool_path: "pool".into(),
            fields: fields.as_object().unwrap().clone(),
        },
        true,
    )
}

fn unit_fields(pin: Uuid) -> Value {
    json!({"name":"Sense unit","pins":{pin.to_string():{"name":"Sense","electrical_type":"Input"}}})
}

fn symbol_fields(unit: Uuid, pin: Uuid) -> Value {
    json!({"name":"Sense symbol","unit":unit,
        "drawings":[{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}],
        "pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]})
}

#[test]
fn electrical_library_creation_preserves_identity_and_materializes_pin_geometry() {
    let (root, mut model) = fixture();
    let (unit, pin, symbol) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let before = model.model_revision.clone();
    let prepared = prepare(&model, &root, "units", unit, unit_fields(pin)).unwrap();
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .model_revision,
        before
    );
    assert!(!root.join(format!("pool/units/{unit}.json")).exists());
    commit_prepared(&mut model, &root, prepared).unwrap();
    let before_symbol = model.model_revision.clone();
    let prepared = prepare(&model, &root, "symbols", symbol, symbol_fields(unit, pin)).unwrap();
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .model_revision,
        before_symbol
    );
    assert!(!root.join(format!("pool/symbols/{symbol}.json")).exists());
    commit_prepared(&mut model, &root, prepared).unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    let pins = materialize_pool_symbol_pins(&reopened, Some(&symbol.to_string())).unwrap();
    assert_eq!(pins.len(), 1);
    assert_eq!(pins[0].library_pin, Some(pin));
    assert_eq!(pins[0].name, "Sense");
    assert_eq!(pins[0].position.x, -2000000);
    assert_eq!(
        pins[0].electrical_type,
        crate::schematic::PinElectricalType::Input
    );
    let target = PoolLibraryObjectTarget::new("pool", "symbols", symbol);
    let raw =
        library::pool_library_object_payload(&reopened, &target.relative_path, symbol).unwrap();
    assert_eq!(raw["unit"], json!(unit));
    assert_eq!(raw["drawings"], symbol_fields(unit, pin)["drawings"]);
    assert!(raw.get("provenance").is_none());
    assert!(raw.get("check_state").is_none());
    assert!(
        !reopened
            .objects
            .values()
            .any(|o| o.kind == "component_instance")
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn electrical_library_creation_refuses_bad_pin_intent_without_writes() {
    let (root, model) = fixture();
    let pin: Uuid = "abcdef00-0000-4000-8000-000000000001".parse().unwrap();
    let original = unit_fields(pin);
    let mut cases = Vec::new();
    let mut invalid = original.clone();
    invalid["pins"][pin.to_string()]["name"] = json!(" ");
    cases.push(invalid);
    let mut invalid = original.clone();
    invalid["pins"][pin.to_string()]["electrical_type"] = json!("InventedType");
    cases.push(invalid);
    let mut invalid = original.clone();
    invalid["pins"][pin.to_string().to_uppercase()] = invalid["pins"][pin.to_string()].clone();
    cases.push(invalid);
    let mut invalid = original.clone();
    invalid["approval"] = json!("Approved");
    cases.push(invalid);
    let mut invalid = original;
    invalid["pins"][pin.to_string()]["uuid"] = json!(Uuid::new_v4());
    cases.push(invalid);
    for fields in cases {
        assert!(prepare(&model, &root, "units", Uuid::new_v4(), fields).is_err());
        assert_eq!(
            ProjectResolver::new(&root)
                .resolve()
                .unwrap()
                .model_revision,
            model.model_revision
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn electrical_library_symbol_refuses_invalid_references_geometry_and_stale_preview() {
    let (root, mut model) = fixture();
    let (unit, pin, symbol) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let prepared = prepare(&model, &root, "units", unit, unit_fields(pin)).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let original = symbol_fields(unit, pin);
    let mut cases = Vec::new();
    let mut invalid = original.clone();
    invalid["unit"] = json!(Uuid::new_v4());
    cases.push(invalid);
    let mut invalid = original.clone();
    invalid["pin_anchors"][0]["pin"] = json!(Uuid::new_v4());
    cases.push(invalid);
    let mut invalid = original.clone();
    let anchor = invalid["pin_anchors"][0].clone();
    invalid["pin_anchors"].as_array_mut().unwrap().push(anchor);
    cases.push(invalid);
    let mut invalid = original.clone();
    invalid["drawings"][0]["Rect"]["width"] = json!(0);
    cases.push(invalid);
    let mut invalid = original.clone();
    invalid["drawings"][0]["Rect"]["max"]["x"] = json!(-2000000);
    cases.push(invalid);
    let mut invalid = original.clone();
    invalid["pin_anchors"][0]["style"]["length_nm"] = json!(-1);
    cases.push(invalid);
    for fields in cases {
        assert!(prepare(&model, &root, "symbols", symbol, fields).is_err());
        assert_eq!(
            ProjectResolver::new(&root)
                .resolve()
                .unwrap()
                .model_revision,
            model.model_revision
        );
    }
    let stale = prepare(&model, &root, "symbols", symbol, original).unwrap();
    let intervening = prepare(
        &model,
        &root,
        "units",
        Uuid::new_v4(),
        unit_fields(Uuid::new_v4()),
    )
    .unwrap();
    commit_prepared(&mut model, &root, intervening).unwrap();
    let before = model.model_revision.clone();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .model_revision,
        before
    );
    assert!(!root.join(format!("pool/symbols/{symbol}.json")).exists());
    std::fs::remove_dir_all(root).unwrap();
}
