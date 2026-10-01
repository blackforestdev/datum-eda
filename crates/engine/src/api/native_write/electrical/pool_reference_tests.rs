//! Electrical evidence must see the same pool leaf revisions live and on reopen.
use super::super::{commit_prepared, genesis, library::*};
use super::tests::provenance;
use crate::substrate::{DesignModel, ObjectRevision, ProjectResolver};
use serde_json::json;
use uuid::Uuid;

fn agree(root: &std::path::Path, model: &DesignModel, ids: &[Uuid]) {
    let reopened = ProjectResolver::new(root).resolve().unwrap();
    for id in ids {
        assert_eq!(model.objects.get(id), reopened.objects.get(id), "{id}");
    }
}

#[test]
fn pool_leaf_reference_lifecycle_matches_live_undo_replay_and_reopen() {
    let root = super::super::test_support::temp_project_root("e1_pool_leaf_lifecycle");
    genesis::bootstrap_native_project(
        &root,
        genesis::GenesisSpec {
            project_name: "Pool leaf reference proof".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let unit = Uuid::new_v4();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let target = PoolLibraryObjectTarget::new("pool", "units", unit);
    let mut payload = json!({"schema_version":1,"uuid":unit,"name":"U","manufacturer":"","tags":[],"pins":{
        a.to_string():{"uuid":a,"name":"A","direction":"Passive","swap_group":0,"alternates":[]}, b.to_string():{"uuid":b,"name":"B","direction":"Passive","swap_group":0,"alternates":[]}
    }});
    let write = build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: target.clone(),
            object: payload.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    agree(&root, &model, &[unit, a, b]);
    assert_eq!(model.objects[&a].object_revision, ObjectRevision(0));
    payload["pins"][a.to_string()]["name"] = json!("changed");
    payload["pins"]
        .as_object_mut()
        .unwrap()
        .remove(&b.to_string());
    let write = build_pool_library_write(
        &model,
        provenance(),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: target.clone(),
            object: payload,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(model.objects[&a].object_revision, ObjectRevision(1));
    assert!(!model.objects.contains_key(&b));
    agree(&root, &model, &[unit, a, b]);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    agree(&root, &model, &[unit, a, b]);
    assert!(model.objects.contains_key(&b));
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    agree(&root, &model, &[unit, a, b]);
    let write = build_pool_library_write(
        &model,
        provenance(),
        None,
        vec![PoolLibraryOperationSpec::Delete { target }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(!model.objects.contains_key(&a));
    agree(&root, &model, &[unit, a, b]);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    agree(&root, &model, &[unit, a, b]);
    assert!(model.objects.contains_key(&a));
}
