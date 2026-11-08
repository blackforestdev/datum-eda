use super::{tests::*, *};
use crate::api::native_write::commit_prepared;
use crate::substrate::{DesignModel, ProjectResolver};
use std::path::Path;

fn edit(
    model: &DesignModel,
    root: &Path,
    id: Uuid,
    fields: Value,
) -> Result<PreparedWrite, EngineError> {
    super::super::build_definition(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        LibraryDefinitionRequest {
            kind: "entities".into(),
            object_id: id,
            pool_path: "pool".into(),
            fields: fields.as_object().unwrap().clone(),
        },
        false,
    )
}

#[test]
fn saved_metadata_patch_preserves_raw_siblings_and_history() {
    let (root, mut model, unit, symbol) = fixture();
    let (id, gate) = (Uuid::new_v4(), Uuid::new_v4());
    let p = prepare(&model, &root, "entities", id, fields(gate, unit, symbol)).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let target = PoolLibraryObjectTarget::new("pool", "entities", id);
    let mut raw = library::pool_library_object_payload(&model, &target.relative_path, id).unwrap();
    raw["manufacturer"] = json!("");
    raw["tags"] = json!(["z", "a", "z"]);
    raw["custom"] = json!({"ordered":[3,1,2]});
    raw["gates"][gate.to_string()]["custom"] = json!("retained gate metadata");
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: target.clone(),
            object: raw.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let board = std::fs::read(root.join("board/board.json")).unwrap();
    let journal_path = crate::substrate::transaction_journal_path(&root);
    let journal = std::fs::read(&journal_path).unwrap();
    let p = edit(
        &model,
        &root,
        id,
        json!({"name":"Revised name", "prefix":"IC"}),
    )
    .unwrap();
    assert_eq!(std::fs::read(&journal_path).unwrap(), journal);
    let stale = p.clone();
    commit_prepared(&mut model, &root, p).unwrap();
    let mut expected = raw.clone();
    expected["name"] = json!("Revised name");
    expected["prefix"] = json!("IC");
    assert_eq!(
        library::pool_library_object_payload(&model, &target.relative_path, id).unwrap(),
        expected
    );
    let committed = std::fs::read(&journal_path).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(std::fs::read(&journal_path).unwrap(), committed);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        library::pool_library_object_payload(&model, &target.relative_path, id).unwrap(),
        raw
    );
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        library::pool_library_object_payload(&cold, &target.relative_path, id).unwrap(),
        expected
    );
    assert_eq!(std::fs::read(root.join("board/board.json")).unwrap(), board);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn saved_metadata_rejects_structure_identity_types_and_wrong_kind_without_writes() {
    let (root, mut model, unit, symbol) = fixture();
    let id = Uuid::new_v4();
    let p = prepare(
        &model,
        &root,
        "entities",
        id,
        fields(Uuid::new_v4(), unit, symbol),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let journal_path = crate::substrate::transaction_journal_path(&root);
    let journal = std::fs::read(&journal_path).unwrap();
    for fields in [
        json!({"gates":{}}),
        json!({"uuid":id}),
        json!({"package":unit}),
        json!({"approval_status":"approved"}),
        json!({"name":"  "}),
        json!({"prefix":4}),
        json!({"manufacturer":null}),
        json!({"tags":[4]}),
    ] {
        assert!(edit(&model, &root, id, fields).is_err());
    }
    for wrong in [unit, symbol, Uuid::new_v4()] {
        assert!(edit(&model, &root, wrong, json!({"name":"Wrong target"})).is_err());
    }
    // Exercise adapter refusal against conflicting source evidence without
    // writing a malformed project or relaxing canonical admission.
    let owner = model.objects[&id].source_shard_id;
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == owner)
        .unwrap()
        .clone();
    model.source_shards.push(shard);
    assert!(
        edit(&model, &root, id, json!({"name":"Ambiguous source"}))
            .unwrap_err()
            .to_string()
            .contains("ambiguous")
    );
    model.source_shards.pop();
    model.objects.get_mut(&id).unwrap().source_shard_id = Uuid::new_v4();
    assert!(
        edit(&model, &root, id, json!({"name":"Wrong owner"}))
            .unwrap_err()
            .to_string()
            .contains("canonical project pool path")
    );
    assert_eq!(std::fs::read(&journal_path).unwrap(), journal);
    std::fs::remove_dir_all(root).unwrap();
}
