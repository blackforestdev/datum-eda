//! Raw optional-reference corrections and effective retained-Map compatibility.
use super::{tests::*, *};
use crate::api::native_write::{commit_prepared, registry::find_native_write_verb};
use crate::substrate::{DesignModel, Operation, ProjectResolver};
use std::path::Path;
fn edit(
    model: &DesignModel,
    root: &Path,
    id: Uuid,
    fields: Value,
) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        json!({"kind":"parts","object_id":id,"pool_path":"pool","fields":fields}),
    )
}
fn raw(model: &DesignModel, kind: &str, id: Uuid) -> Value {
    library::pool_library_object_payload(
        model,
        &PoolLibraryObjectTarget::new("pool", kind, id).relative_path,
        id,
    )
    .unwrap()
}
fn write(model: &mut DesignModel, root: &Path, kind: &str, id: Uuid, object: Value, create: bool) {
    let target = PoolLibraryObjectTarget::new("pool", kind, id);
    let spec = if create {
        PoolLibraryOperationSpec::Create { target, object }
    } else {
        PoolLibraryOperationSpec::Set { target, object }
    };
    let p =
        library::build_pool_library_write(model, provenance(), Some("pool"), vec![spec]).unwrap();
    commit_prepared(model, root, p).unwrap();
}
#[test]
fn saved_default_footprint_raw_optional_patch_preview_guards_and_cold_history() {
    let (root, mut model, entity, package, _, first) = fixture();
    let second = Uuid::new_v4();
    write(
        &mut model,
        &root,
        "footprints",
        second,
        super::super::super::library_footprint::pool_footprint_payload(
            second,
            package,
            "Alternate land",
        ),
        true,
    );
    for representation in 0..3 {
        let part = Uuid::new_v4();
        let p = prepare(&model, &root, part, fields(entity, package, Some(first))).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let mut before = raw(&model, "parts", part);
        before["custom"] = json!({"ordered":[3,1]});
        before["tags"] = json!(["z", "a", "z"]);
        if representation == 1 {
            before["default_footprint"] = Value::Null;
        }
        if representation == 2 {
            before.as_object_mut().unwrap().remove("default_footprint");
        }
        write(&mut model, &root, "parts", part, before.clone(), false);
        let p = edit(
            &model,
            &root,
            part,
            json!({"description":"Explicit metadata"}),
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        before["description"] = json!("Explicit metadata");
        assert_eq!(
            raw(&model, "parts", part),
            before,
            "untouched optional representation is exact"
        );
        let journal = crate::substrate::transaction_journal_path(&root);
        let bytes = std::fs::read(&journal).unwrap();
        let p = edit(&model, &root, part, json!({"default_footprint":second})).unwrap();
        assert_eq!(
            std::fs::read(&journal).unwrap(),
            bytes,
            "preview/cancel has no writes"
        );
        for id in [entity, package, second] {
            assert!(p.batch.operations.iter().any(|op| matches!(op,
            Operation::GuardObjectRevision{object_id,..} if *object_id==id)));
        }
        let mut stale_ref = model.clone();
        stale_ref
            .objects
            .get_mut(&second)
            .unwrap()
            .object_revision
            .0 += 1;
        assert!(
            stale_ref
                .commit(p.batch.clone())
                .unwrap_err()
                .to_string()
                .contains("object revision mismatch")
        );
        let stale = p.clone();
        commit_prepared(&mut model, &root, p).unwrap();
        let mut expected = before.clone();
        expected["default_footprint"] = json!(second);
        assert_eq!(raw(&model, "parts", part), expected);
        assert!(commit_prepared(&mut model, &root, stale).is_err());
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        assert_eq!(raw(&model, "parts", part), before);
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&model, "parts", part), expected);
        let p = edit(&model, &root, part, json!({"default_footprint":null})).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let mut cleared = expected.clone();
        cleared.as_object_mut().unwrap().remove("default_footprint");
        assert_eq!(raw(&model, "parts", part), cleared);
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        assert_eq!(raw(&model, "parts", part), expected);
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&model, "parts", part), cleared);
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn retained_default_map_uses_existing_effective_footprint_then_package_rules() {
    let (root, mut model, entity, package, other, first) = fixture();
    let (part, map, second, wrong) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let p = prepare(&model, &root, part, fields(entity, package, Some(first))).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    for (id, pkg) in [(second, package), (wrong, other)] {
        write(
            &mut model,
            &root,
            "footprints",
            id,
            super::super::super::library_footprint::pool_footprint_payload(
                id,
                pkg,
                "Existing land",
            ),
            true,
        );
    }
    // Empty mapping still exercises the existing explicit Footprint compatibility
    // separately from the nonempty physical fixture in the shared query tests.
    write(
        &mut model,
        &root,
        "pin_pad_maps",
        map,
        json!({"schema_version":1,"uuid":map,"part":part,"footprint":first,"tags":[]}),
        true,
    );
    let mut original = raw(&model, "parts", part);
    original["default_pin_pad_map"] = json!(map);
    write(&mut model, &root, "parts", part, original.clone(), false);
    let map_bytes = std::fs::read(root.join(format!("pool/pin_pad_maps/{map}.json"))).unwrap();
    let journal = crate::substrate::transaction_journal_path(&root);
    for intent in [
        json!({"default_footprint":second}),
        json!({"default_footprint":wrong}),
        json!({"default_footprint":package}),
        json!({"default_footprint":Uuid::new_v4()}),
        json!({"default_footprint":""}),
        json!({"default_footprint":3}),
        json!({"default_footprint":null,"default_pin_pad_map":null}),
        json!({"default_footprint":first,"package":other}),
    ] {
        let before = std::fs::read(&journal).unwrap();
        assert!(edit(&model, &root, part, intent).is_err());
        assert_eq!(std::fs::read(&journal).unwrap(), before);
        assert_eq!(raw(&model, "parts", part), original);
    }
    let p = edit(&model, &root, part, json!({"default_footprint":null})).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let cleared = raw(&model, "parts", part);
    assert!(cleared.get("default_footprint").is_none());
    assert_eq!(cleared["default_pin_pad_map"], json!(map));
    assert_eq!(
        std::fs::read(root.join(format!("pool/pin_pad_maps/{map}.json"))).unwrap(),
        map_bytes
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, "parts", part), original);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&cold, "parts", part), cleared);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn undo_default_footprint_refuses_future_pinned_part_counter_collision_atomically() {
    let (root, mut model, part, ci, historical) = super::edit_tests::legacy_fixture_at_revision(3);
    let footprint = model
        .objects
        .values()
        .find(|o| o.domain == "pool" && o.kind == "footprints")
        .unwrap()
        .object_id;
    for value in [json!(footprint), Value::Null] {
        let p = edit(&model, &root, part, json!({"default_footprint":value})).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
    }
    assert_eq!(model.objects[&part].object_revision.0, 2);
    let before = raw(&model, "parts", part);
    let ci_before = model.component_instances[&ci].clone();
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    assert!(
        model
            .commit_journal_undo(&root, provenance().into())
            .is_err(),
        "Undo must not reuse pinned historical Part revision three for a different payload"
    );
    assert_eq!(raw(&model, "parts", part), before);
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(cold.component_instances[&ci], ci_before);
    assert_eq!(
        cold.materialized_pool_object_revision(part, "parts", crate::substrate::ObjectRevision(3))
            .unwrap(),
        historical
    );
    std::fs::remove_dir_all(root).unwrap();
}
