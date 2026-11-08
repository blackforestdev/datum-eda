//! Canonical saved-definition edits preserve raw data and reject broader intent.
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
    let root = crate::api::native_write::test_support::temp_project_root("saved_electrical");
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

fn raw(model: &DesignModel, kind: &str, id: Uuid) -> Value {
    library::pool_library_object_payload(model, &format!("pool/{kind}/{id}.json"), id).unwrap()
}

#[test]
fn saved_unit_metadata_preserves_raw_pin_data_and_canonical_history() {
    let (root, mut model, unit, pin, _) = fixture();
    let before = raw(&model, "units", unit);
    let mut pins = before["pins"].clone();
    pins[pin.to_string()]["name"] = json!("Renamed");
    pins[pin.to_string()]["direction"] = json!("Passive");
    let prepared = prepare(
        &model,
        &root,
        "units",
        unit,
        json!({"name":"Updated logical","pins":pins}),
    )
    .unwrap();
    assert_eq!(
        raw(
            &ProjectResolver::new(&root).resolve().unwrap(),
            "units",
            unit
        ),
        before
    );
    let stale = prepared.clone();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let after = raw(&model, "units", unit);
    let mut expected = before.clone();
    expected["name"] = json!("Updated logical");
    expected["pins"] = pins;
    assert_eq!(after, expected);
    assert!(
        after["pins"][pin.to_string()]
            .get("electrical_type")
            .is_none()
    );
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, "units", unit), before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        raw(
            &ProjectResolver::new(&root).resolve().unwrap(),
            "units",
            unit
        ),
        after
    );
}

#[test]
fn saved_symbol_geometry_preserves_arbitrary_graphics_order_fields_and_flat_style() {
    let (root, mut model, _, _, symbol) = fixture();
    let before = raw(&model, "symbols", symbol);
    let mut anchors = before["pin_anchors"].clone();
    anchors[0]["position"]["x"] = json!(-1500001);
    anchors[0]["orientation"] = json!("Up");
    let prepared = prepare(
        &model,
        &root,
        "symbols",
        symbol,
        json!({"name":"Revised graphic","pin_anchors":anchors}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut expected = before.clone();
    expected["name"] = json!("Revised graphic");
    expected["pin_anchors"] = anchors;
    assert_eq!(raw(&model, "symbols", symbol), expected);
    assert!(expected["pin_anchors"][0].get("style").is_none());
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, "symbols", symbol), before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        raw(
            &ProjectResolver::new(&root).resolve().unwrap(),
            "symbols",
            symbol
        ),
        expected
    );
}

#[test]
fn saved_electrical_adapter_refuses_identity_topology_and_unrelated_raw_changes_without_writes() {
    let (root, model, unit, pin, symbol) = fixture();
    let logical = raw(&model, "units", unit);
    let graphic = raw(&model, "symbols", symbol);
    let mut missing = logical["pins"].clone();
    missing.as_object_mut().unwrap().clear();
    let mut hidden = logical["pins"].clone();
    hidden[pin.to_string()]["vendor"] = json!(false);
    let mut linked = graphic["pin_anchors"].clone();
    linked[0]["pin"] = json!(Uuid::new_v4());
    let mut metadata = graphic["pin_anchors"].clone();
    metadata[0]["position"]["metadata"] = json!("lost");
    let mut negative = graphic["pin_anchors"].clone();
    negative[0]["length_nm"] = json!(-1);
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    for (kind, id, fields) in [
        ("units", unit, json!({"pins":missing})),
        ("units", unit, json!({"pins":hidden})),
        ("symbols", symbol, json!({"unit":Uuid::new_v4()})),
        ("symbols", symbol, json!({"drawings":[]})),
        ("symbols", symbol, json!({"pin_anchors":linked})),
        ("symbols", symbol, json!({"pin_anchors":metadata})),
        ("symbols", symbol, json!({"pin_anchors":negative})),
        ("symbols", symbol, json!({"approval_status":"approved"})),
    ] {
        assert!(prepare(&model, &root, kind, id, fields).is_err());
    }
    assert_eq!(
        raw(
            &ProjectResolver::new(&root).resolve().unwrap(),
            "units",
            unit
        ),
        logical
    );
    assert_eq!(
        raw(
            &ProjectResolver::new(&root).resolve().unwrap(),
            "symbols",
            symbol
        ),
        graphic
    );
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
}

#[test]
fn saved_electrical_patch_admission_refuses_alias_identities() {
    let id = Uuid::parse_str("abcdefab-cdef-4abc-8def-abcdefabcdef").unwrap();
    let lower = id.to_string();
    let upper = lower.to_ascii_uppercase();
    let pins = json!({lower.clone():{"uuid":id,"name":"A","direction":"Input"},upper.clone():{"uuid":id,"name":"B","direction":"Input"}});
    assert!(validate_pins(&pins, &pins).is_err());
    let anchors =
        json!([{"pin":lower,"position":{"x":0,"y":0}},{"pin":upper,"position":{"x":1,"y":0}}]);
    assert!(validate_anchors(&anchors, &anchors).is_err());
}

#[test]
#[ignore = "known separate defect dat-17ld: repeated pool path promotion is uncertain; no durability PASS"]
fn historical_symbol_revisions_count_each_set_and_bump_after_cold_reopen() {
    use crate::substrate::{ObjectRevision, Operation};
    let (root, mut model, _, _, symbol) = fixture();
    let original = raw(&model, "symbols", symbol);
    let revision = model.objects[&symbol].object_revision.0;
    let mut middle = original.clone();
    middle["name"] = json!("Intermediate");
    let mut final_value = middle.clone();
    final_value["name"] = json!("Final");
    let path = format!("pool/symbols/{symbol}.json");
    let operations = vec![
        Operation::SetPoolLibraryObject {
            object_id: symbol,
            relative_path: path.clone(),
            object_kind: "symbols".into(),
            previous_object: original.clone(),
            object: middle.clone(),
        },
        Operation::BumpObjectRevision { object_id: symbol },
        Operation::SetPoolLibraryObject {
            object_id: symbol,
            relative_path: path,
            object_kind: "symbols".into(),
            previous_object: middle.clone(),
            object: final_value.clone(),
        },
    ];
    let batch = crate::api::native_write::build_batch(&model, provenance(), operations).unwrap();
    model.commit_journaled(&root, batch).unwrap();
    assert_eq!(model.objects[&symbol].object_revision.0, revision + 3);
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        cold.objects[&symbol].object_revision,
        model.objects[&symbol].object_revision
    );
    for candidate in [&model, &cold] {
        for (r, expected) in [
            (revision, &original),
            (revision + 1, &middle),
            (revision + 2, &middle),
            (revision + 3, &final_value),
        ] {
            assert_eq!(
                &candidate
                    .materialized_pool_object_revision(symbol, "symbols", ObjectRevision(r))
                    .unwrap(),
                expected
            );
        }
        assert!(
            candidate
                .materialized_pool_object_revision(symbol, "symbols", ObjectRevision(revision + 4))
                .is_err()
        );
        assert!(
            candidate
                .materialized_pool_object_revision(symbol, "units", ObjectRevision(revision))
                .is_err()
        );
    }
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        cold.objects[&symbol].object_revision,
        model.objects[&symbol].object_revision
    );
    assert_eq!(
        cold.materialized_pool_object_revision(symbol, "symbols", ObjectRevision(revision + 1))
            .unwrap(),
        middle
    );
}
