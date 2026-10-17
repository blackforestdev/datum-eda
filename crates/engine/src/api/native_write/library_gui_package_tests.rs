//! Creation, history, exact body/terminal admission and non-writing refusals.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
};
use crate::substrate::{CommitSource, ProjectResolver};

fn provenance() -> WriteProvenance {
    WriteProvenance::new("datum-gui", CommitSource::Manual, "Create physical Package")
}
fn project(label: &str) -> std::path::PathBuf {
    let root = super::super::test_support::temp_project_root(label);
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Package creation".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    root
}
fn request(id: Uuid, fields: Value) -> LibraryDefinitionRequest {
    LibraryDefinitionRequest {
        kind: "packages".into(),
        object_id: id,
        pool_path: "pool".into(),
        fields: fields.as_object().unwrap().clone(),
    }
}

#[test]
fn package_create_preserves_body_terminal_identity_history_and_reopen() {
    let root = project("package_gui_history");
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let board = std::fs::read(root.join("board/board.json")).unwrap();
    let id = Uuid::new_v4();
    let terminal = Uuid::new_v4();
    let fields = json!({"name":"Designer body","package_family":"Designer family","mounting_type":"Designer mounting","body_dimensions":{"x_nm":5000000,"y_nm":3000000},"body_height_nm":1500000,"terminals":{terminal.to_string():{"uuid":terminal,"name":"1","role":"signal"}}});
    let prepared = build_definition(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        request(id, fields.clone()),
        true,
    )
    .unwrap();
    assert!(
        !root.join("pool/packages").exists(),
        "preview is build-only"
    );
    assert!(!crate::substrate::transaction_journal_path(&root).exists());
    let stale = prepared.clone();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let path = format!("pool/packages/{id}.json");
    let mut expected = library::pool_package_payload(id, "Designer body", json!({}));
    expected
        .as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    assert_eq!(
        library::pool_library_object_payload(&model, &path, id).unwrap(),
        expected
    );
    let source = std::fs::read(root.join(&path)).unwrap();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(std::fs::read(root.join(&path)).unwrap(), source);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert!(!root.join(&path).exists());
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        library::pool_library_object_payload(&reopened, &path, id).unwrap(),
        expected
    );
    assert!(expected.get("package_code").is_none());
    assert!(expected.pointer("/body_dimensions/z_nm").is_none());
    assert_eq!(std::fs::read(root.join("board/board.json")).unwrap(), board);
}

#[test]
fn package_create_refuses_unsupported_shape_target_and_saved_edit_without_writes() {
    let root = project("package_gui_refusals");
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    let id = Uuid::new_v4();
    let t = Uuid::new_v4();
    for fields in [
        json!({"name":""}),
        json!({"name":"bad","body_dimensions":{"x_nm":"not a length"}}),
        json!({"name":"bad","body_dimensions":{"pitch_nm":100}}),
        json!({"name":"bad","body_height_nm":1e30}),
        json!({"name":"bad","pads":{}}),
        json!({"name":"bad","approval_status":"approved"}),
        json!({"name":"bad","terminals":{t.to_string():{"uuid":id,"name":"1"}}}),
        json!({"name":"alias mismatch","terminals":{
            t.to_string():{"uuid":t,"name":"valid"},
            t.simple().to_string():{"uuid":id,"name":"mismatching alias"}
        }}),
        json!({"name":"duplicate alias","terminals":{
            t.to_string():{"uuid":t,"name":"first"},
            t.simple().to_string():{"uuid":t,"name":"second"}
        }}),
        json!({"name":"bad","terminals":{t.to_string():{"uuid":t,"name":"1","position":{"x":0,"y":0}}}}),
    ] {
        assert!(build_definition(&context, provenance(), request(id, fields), true).is_err());
    }
    let mut wrong = request(id, json!({"name":"wrong"}));
    wrong.pool_path = "other-pool".into();
    assert!(build_definition(&context, provenance(), wrong, true).is_err());
    assert!(
        build_definition(
            &context,
            provenance(),
            request(id, json!({"name":"edit"})),
            false
        )
        .unwrap_err()
        .to_string()
        .contains("Saved Package editing")
    );
    assert!(!root.join("pool/packages").exists());
    assert!(!crate::substrate::transaction_journal_path(&root).exists());
}
