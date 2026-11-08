//! Explicit gate identity/reference admission and canonical history.
use super::super::build_definition;
use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::path::Path;
pub(super) fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Create logical Entity")
}
pub(super) fn prepare(
    model: &DesignModel,
    root: &Path,
    kind: &str,
    id: Uuid,
    fields: Value,
) -> Result<PreparedWrite, EngineError> {
    let request = LibraryDefinitionRequest {
        kind: kind.into(),
        object_id: id,
        pool_path: "pool".into(),
        fields: fields.as_object().unwrap().clone(),
    };
    (super::super::super::registry::find_native_write_verb("datum.library.create")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        serde_json::to_value(request).unwrap(),
    )
}
pub(super) fn fixture() -> (std::path::PathBuf, DesignModel, Uuid, Uuid) {
    let root = super::super::super::test_support::temp_project_root("entity_gui");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Logical gates".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, pin, symbol) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let p = prepare(&model,&root,"units",unit,json!({"name":"Gate Unit","pins":{pin.to_string():{"name":"IN","electrical_type":"Input"}}})).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let p = prepare(&model,&root,"symbols",symbol,json!({"name":"Gate Symbol","unit":unit,
      "drawings":[{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}],
      "pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]})).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    (root, model, unit, symbol)
}
pub(super) fn fields(gate: Uuid, unit: Uuid, symbol: Uuid) -> Value {
    json!({"name":"Designer Entity","prefix":"U","manufacturer":"Designer","tags":["logic"],"gates":{gate.to_string():{"uuid":gate,"name":"A","unit":unit,"symbol":symbol}}})
}
#[test]
fn entity_create_preview_identity_history_and_cold_reopen() {
    let (root, mut model, unit, symbol) = fixture();
    let (id, a, b) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let mut intent = fields(a, unit, symbol);
    intent["gates"][b.to_string()] = json!({"uuid":b,"name":"B","unit":unit,"symbol":symbol});
    let board = std::fs::read(root.join("board/board.json")).unwrap();
    let unit_bytes = std::fs::read(root.join(format!("pool/units/{unit}.json"))).unwrap();
    let symbol_bytes = std::fs::read(root.join(format!("pool/symbols/{symbol}.json"))).unwrap();
    let journal_path = crate::substrate::transaction_journal_path(&root);
    let before = std::fs::read(&journal_path).unwrap();
    let prepared = prepare(&model, &root, "entities", id, intent.clone()).unwrap();
    assert_eq!(std::fs::read(&journal_path).unwrap(), before);
    assert!(!root.join(format!("pool/entities/{id}.json")).exists());
    let stale = prepared.clone();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let path = format!("pool/entities/{id}.json");
    let raw = library::pool_library_object_payload(&model, &path, id).unwrap();
    assert_eq!(raw["gates"], intent["gates"]);
    assert_eq!(raw["tags"], intent["tags"]);
    assert!(raw.get("approval_status").is_none());
    let committed = std::fs::read(&journal_path).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(std::fs::read(&journal_path).unwrap(), committed);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert!(!root.join(&path).exists());
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        library::pool_library_object_payload(&cold, &path, id).unwrap(),
        raw
    );
    for (p, expected) in [
        ("board/board.json".into(), board),
        (format!("pool/units/{unit}.json"), unit_bytes),
        (format!("pool/symbols/{symbol}.json"), symbol_bytes),
    ] {
        assert_eq!(std::fs::read(root.join(p)).unwrap(), expected);
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn entity_create_refuses_invalid_identity_missing_kind_mismatch_and_hidden_fields_without_writes() {
    let (root, mut model, unit, symbol) = fixture();
    let other_unit = Uuid::new_v4();
    let pin = Uuid::new_v4();
    let p = prepare(&model, &root, "units", other_unit, json!({"name":"Other Unit","pins":{pin.to_string():{"name":"Other","electrical_type":"Passive"}}})).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let (id, gate) = (Uuid::new_v4(), Uuid::new_v4());
    let valid = fields(gate, unit, symbol);
    let mut cases = Vec::new();
    for (pointer, value) in [
        ("/name", json!(" ")),
        ("/prefix", json!(7)),
        ("/gates", json!({})),
        ("/approval_status", json!("approved")),
    ] {
        let mut v = valid.clone();
        if pointer == "/approval_status" {
            v["approval_status"] = value;
        } else {
            *v.pointer_mut(pointer).unwrap() = value;
        }
        cases.push(v);
    }
    for (key, value) in [
        ("uuid", json!(id)),
        ("name", json!(" ")),
        ("unit", json!(Uuid::new_v4())),
        ("unit", json!(symbol)),
        ("symbol", json!(unit)),
        ("symbol", json!(Uuid::new_v4())),
        ("approval", json!(true)),
    ] {
        let mut v = valid.clone();
        v["gates"][gate.to_string()][key] = value;
        cases.push(v);
    }
    let mut aliases = valid.clone();
    aliases["gates"][gate.simple().to_string()] = aliases["gates"][gate.to_string()].clone();
    cases.push(aliases);
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    for v in cases {
        assert!(prepare(&model, &root, "entities", id, v).is_err());
    }
    let mut wrong_pair = valid.clone();
    wrong_pair["gates"][gate.to_string()]["unit"] = json!(other_unit);
    assert!(prepare(&model, &root, "entities", id, wrong_pair).is_err());
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    let edit = LibraryDefinitionRequest {
        kind: "entities".into(),
        object_id: id,
        pool_path: "pool".into(),
        fields: valid.as_object().unwrap().clone(),
    };
    assert!(
        build_definition(
            &NativeWriteContext {
                model: &model,
                project_root: &root
            },
            provenance(),
            edit,
            false
        )
        .is_err()
    );
    assert!(!root.join("pool/entities").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn entity_preview_guards_the_canonical_before_model_referenced_definitions() {
    let (root, model, unit, symbol) = fixture();
    let id = Uuid::new_v4();
    let prepared = prepare(
        &model,
        &root,
        "entities",
        id,
        fields(Uuid::new_v4(), unit, symbol),
    )
    .unwrap();
    let guards: HashSet<_> = prepared
        .batch
        .operations
        .iter()
        .filter_map(|op| {
            if let Operation::GuardObjectRevision { object_id, .. } = op {
                Some(*object_id)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(guards, HashSet::from([unit, symbol]));
    let path = crate::substrate::transaction_journal_path(&root);
    let before = std::fs::read(&path).unwrap();
    for reference in [unit, symbol] {
        let mut changed = model.clone();
        changed
            .objects
            .get_mut(&reference)
            .unwrap()
            .object_revision
            .0 += 1;
        assert_eq!(changed.model_revision, model.model_revision);
        // Journaled commit re-resolves the authoritative disk model; a synthetic
        // in-memory change must exercise canonical in-memory admission instead.
        assert!(changed.commit(prepared.batch.clone()).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(!root.join(format!("pool/entities/{id}.json")).exists());
    }
    std::fs::remove_dir_all(root).unwrap();
}
