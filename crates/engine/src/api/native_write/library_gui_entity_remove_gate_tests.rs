//! Unused saved gates retire through the canonical nested-object lifecycle.
use super::{tests::*, *};
use crate::api::native_write::{commit_prepared, schematic_gate_placement as placement};
use crate::substrate::{DesignModel, ProjectResolver};
use std::path::Path;
fn raw(model: &DesignModel, id: Uuid) -> Value {
    library::pool_library_object_payload(
        model,
        &PoolLibraryObjectTarget::new("pool", "entities", id).relative_path,
        id,
    )
    .unwrap()
}
fn remove(
    model: &DesignModel,
    root: &Path,
    entity: Uuid,
    ids: Value,
) -> Result<PreparedWrite, EngineError> {
    super::super::build_definition(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        LibraryDefinitionRequest {
            kind: "entities".into(),
            object_id: entity,
            pool_path: "pool".into(),
            fields: json!({"gate_removals":ids}).as_object().unwrap().clone(),
        },
        false,
    )
}
#[test]
fn unused_removal_preserves_raw_aliases_guards_and_nested_history_cold() {
    let (root, mut model, unit, symbol) = fixture();
    let (entity, keep, retired) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let p = prepare(
        &model,
        &root,
        "entities",
        entity,
        fields(keep, unit, symbol),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let mut original = raw(&model, entity);
    let alias = keep.simple().to_string().to_uppercase();
    let gate = original["gates"]
        .as_object_mut()
        .unwrap()
        .remove(&keep.to_string())
        .unwrap();
    original["gates"][&alias] = gate;
    original["gates"][&alias]["custom"] = json!([3, 1]);
    original["custom"] = json!(true);
    let remove_key = retired.simple().to_string().to_uppercase();
    original["gates"][&remove_key] = json!({"uuid":retired,"name":"Unused","unit":unit,"symbol":symbol,"custom":"retire only this"});
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", "entities", entity),
            object: original.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let counter = model.objects[&keep].object_revision;
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    for ids in [
        json!([]),
        json!([Uuid::new_v4()]),
        json!([retired, retired]),
        json!([retired, retired.simple().to_string()]),
        json!([entity]),
    ] {
        assert!(remove(&model, &root, entity, ids).is_err());
        assert_eq!(raw(&model, entity), original);
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    }
    let p = remove(&model, &root, entity, json!([retired])).unwrap();
    assert_eq!(raw(&model, entity), original);
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    for id in [unit, symbol] {
        assert!(
            p.batch.operations.iter().any(
                |op| matches!(op,Operation::GuardObjectRevision{object_id,..} if *object_id==id)
            )
        );
    }
    let mut changed = model.clone();
    changed.objects.get_mut(&unit).unwrap().object_revision.0 += 1;
    assert!(changed.commit(p.batch.clone()).is_err());
    let stale = p.clone();
    commit_prepared(&mut model, &root, p).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    let mut expected = original.clone();
    expected["gates"]
        .as_object_mut()
        .unwrap()
        .remove(&remove_key);
    assert_eq!(raw(&model, entity), expected);
    assert!(!model.objects.contains_key(&retired));
    assert_eq!(model.objects[&keep].object_revision, counter);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, entity), original);
    assert!(model.objects.contains_key(&retired));
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, entity), expected);
    assert!(!model.objects.contains_key(&retired));
    assert_eq!(model.objects[&keep].object_revision, counter);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn actual_placement_refuses_gate_removal_without_source_or_journal_writes() {
    let (root, model, first, request) = placement::tests::setup("remove-used-gate");
    let entity = placement::options(&model, first, &[]).unwrap().entity;
    let sheet = placement::tests::raw(&model, first);
    let gate: Uuid =
        serde_json::from_value(sheet["symbols"][first.to_string()]["gate"].clone()).unwrap();
    let before = raw(&model, entity);
    let ci = placement::tests::binding(&model, request.component_instance);
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    let error = remove(&model, &root, entity, json!([gate]))
        .unwrap_err()
        .to_string();
    assert!(error.contains("schematic placement"), "{error}");
    assert_eq!(raw(&model, entity), before);
    assert_eq!(placement::tests::raw(&model, first), sheet);
    assert_eq!(
        placement::tests::binding(&model, request.component_instance),
        ci
    );
    assert_eq!(std::fs::read(journal).unwrap(), bytes);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn placed_gate_still_refuses_after_its_part_relinks_to_another_entity() {
    use crate::api::native_write::registry::find_native_write_verb;
    let (root, mut model, first, _) = placement::tests::setup("remove-old-placed-gate");
    let options = placement::options(&model, first, &[]).unwrap();
    let entity = options.entity;
    let sheet = placement::tests::raw(&model, first);
    let gate: Uuid =
        serde_json::from_value(sheet["symbols"][first.to_string()]["gate"].clone()).unwrap();
    let other = Uuid::new_v4();
    let mut replacement = raw(&model, entity);
    replacement["uuid"] = json!(other);
    replacement["gates"] = Value::Object(
        replacement["gates"]
            .as_object()
            .unwrap()
            .values()
            .map(|row| {
                let id = Uuid::new_v4();
                let mut row = row.clone();
                row["uuid"] = json!(id);
                (id.to_string(), row)
            })
            .collect(),
    );
    let setup = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "entities", other),
            object: replacement,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, setup).unwrap();
    let setup=(find_native_write_verb("datum.library.edit").unwrap().build)(&NativeWriteContext{model:&model,project_root:&root},provenance(),
        json!({"kind":"parts","object_id":options.part,"pool_path":"pool","fields":{"entity":other}})).unwrap();
    commit_prepared(&mut model, &root, setup).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let before: std::collections::BTreeMap<_, _> = model
        .source_shards
        .iter()
        .map(|s| {
            (
                s.relative_path.clone(),
                model
                    .materialized_source_shard_value_by_relative_path(&s.relative_path)
                    .unwrap(),
            )
        })
        .collect();
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    let revision = model.model_revision.clone();
    let error = remove(&model, &root, entity, json!([gate])).unwrap_err();
    assert!(error.to_string().contains("schematic placement"), "{error}");
    assert_eq!(model.model_revision, revision);
    assert_eq!(std::fs::read(journal).unwrap(), bytes);
    for (path, source) in before {
        assert_eq!(
            model
                .materialized_source_shard_value_by_relative_path(&path)
                .unwrap(),
            source
        );
    }
    assert_eq!(placement::tests::raw(&model, first), sheet);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn last_gate_removal_refuses_but_complete_distinct_replacement_is_valid() {
    let (root, mut model, unit, symbol) = fixture();
    let (entity, old, new) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let setup = prepare(&model, &root, "entities", entity, fields(old, unit, symbol)).unwrap();
    commit_prepared(&mut model, &root, setup).unwrap();
    let before = raw(&model, entity);
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    assert!(
        remove(&model, &root, entity, json!([old]))
            .unwrap_err()
            .to_string()
            .contains("one or more")
    );
    assert_eq!(raw(&model, entity), before);
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    let request = |id: Uuid| {
        LibraryDefinitionRequest{kind:"entities".into(),object_id:entity,pool_path:"pool".into(),fields:json!({"gate_removals":[old],"gate_additions":{id.to_string():{"uuid":id,"name":"Replacement","unit":unit,"symbol":symbol}}}).as_object().unwrap().clone()}
    };
    assert!(
        super::super::build_definition(
            &NativeWriteContext {
                model: &model,
                project_root: &root
            },
            provenance(),
            request(old),
            false
        )
        .is_err()
    );
    let p = super::super::build_definition(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        request(new),
        false,
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    assert!(!model.objects.contains_key(&old));
    assert!(model.objects.contains_key(&new));
    assert_eq!(raw(&model, entity)["gates"].as_object().unwrap().len(), 1);
    std::fs::remove_dir_all(root).unwrap();
}
