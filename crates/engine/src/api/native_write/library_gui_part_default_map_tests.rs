//! Saved default Map source edits preserve raw data and validate the whole pair.
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
fn store(model: &mut DesignModel, root: &Path, kind: &str, id: Uuid, object: Value, create: bool) {
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
fn map(model: &mut DesignModel, root: &Path, id: Uuid, part: Uuid, foot: Option<Uuid>) {
    store(
        model,
        root,
        "pin_pad_maps",
        id,
        json!({"schema_version":1,"uuid":id,"part":part,"footprint":foot,"mappings":{},"tags":[]}),
        true,
    );
}
#[test]
fn saved_default_map_raw_optional_preview_revision_guards_history_and_cold() {
    let (root, mut model, entity, package, _, foot) = fixture();
    for representation in 0..4 {
        let (part, a, b) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let p = prepare(&model, &root, part, fields(entity, package, Some(foot))).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        map(&mut model, &root, a, part, Some(foot));
        map(&mut model, &root, b, part, Some(foot));
        let mut original = raw(&model, "parts", part);
        original["custom"] = json!({"ordered":[3,1,3]});
        original["tags"] = json!(["z", "a", "z"]);
        original["default_pin_pad_map"] = match representation {
            0 => json!(a),
            3 => json!(a.simple().to_string()),
            _ => Value::Null,
        };
        if representation == 2 {
            original
                .as_object_mut()
                .unwrap()
                .remove("default_pin_pad_map");
        }
        store(&mut model, &root, "parts", part, original.clone(), false);
        let p = edit(
            &model,
            &root,
            part,
            json!({"description":"Explicit metadata"}),
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        original["description"] = json!("Explicit metadata");
        assert_eq!(raw(&model, "parts", part), original);
        let journal = crate::substrate::transaction_journal_path(&root);
        let bytes = std::fs::read(&journal).unwrap();
        let prepared = edit(&model, &root, part, json!({"default_pin_pad_map":b})).unwrap();
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
        assert_eq!(raw(&model, "parts", part), original);
        assert!(prepared.batch.operations.iter().any(|op|matches!(op,Operation::GuardObjectRevision{object_id,expected_object_revision} if *object_id==b && *expected_object_revision==model.objects[&b].object_revision)));
        // An actual concurrent source edit invalidates the prepared choice.
        let mut changed = raw(&model, "pin_pad_maps", b);
        changed["tags"] = json!(["changed before commit"]);
        store(&mut model, &root, "pin_pad_maps", b, changed.clone(), false);
        let after_competing = std::fs::read(&journal).unwrap();
        assert!(commit_prepared(&mut model, &root, prepared).is_err());
        assert_eq!(std::fs::read(&journal).unwrap(), after_competing);
        assert_eq!(raw(&model, "parts", part), original);
        let p = edit(&model, &root, part, json!({"default_pin_pad_map":b})).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let mut selected = original.clone();
        selected["default_pin_pad_map"] = json!(b);
        assert_eq!(raw(&model, "parts", part), selected);
        assert_eq!(raw(&model, "pin_pad_maps", b), changed);
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        assert_eq!(raw(&model, "parts", part), original);
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&model, "parts", part), selected);
        let p = edit(&model, &root, part, json!({"default_pin_pad_map":null})).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let mut cleared = selected.clone();
        cleared
            .as_object_mut()
            .unwrap()
            .remove("default_pin_pad_map");
        assert_eq!(raw(&model, "parts", part), cleared);
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        assert_eq!(raw(&model, "parts", part), selected);
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&model, "parts", part), cleared);
        assert_eq!(raw(&model, "pin_pad_maps", b), changed);
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn saved_default_map_composes_with_footprint_and_refuses_invalid_pair_without_writes() {
    let (root, mut model, entity, package, other, first) = fixture();
    let (part, other_part, second, wrong, a, b, c) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    for id in [part, other_part] {
        let p = prepare(&model, &root, id, fields(entity, package, Some(first))).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
    }
    for (id, pkg) in [(second, package), (wrong, other)] {
        store(
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
    map(&mut model, &root, a, part, Some(first));
    map(&mut model, &root, b, part, Some(second));
    map(&mut model, &root, c, other_part, Some(first));
    let p = edit(&model, &root, part, json!({"default_pin_pad_map":a})).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let original = raw(&model, "parts", part);
    let map_a = raw(&model, "pin_pad_maps", a);
    let map_b = raw(&model, "pin_pad_maps", b);
    let journal = crate::substrate::transaction_journal_path(&root);
    for intent in [
        json!({"default_pin_pad_map":b}),
        json!({"default_footprint":second}),
        json!({"default_pin_pad_map":c}),
        json!({"default_pin_pad_map":package}),
        json!({"default_pin_pad_map":Uuid::new_v4()}),
        json!({"default_pin_pad_map":""}),
        json!({"default_pin_pad_map":true}),
        json!({"default_pin_pad_map":null,"package":other}),
        json!({"default_pin_pad_map":a,"mappings":{}}),
        json!({"default_pin_pad_map":b,"default_footprint":wrong}),
    ] {
        let bytes = std::fs::read(&journal).unwrap();
        assert!(edit(&model, &root, part, intent).is_err());
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
        assert_eq!(raw(&model, "parts", part), original);
    }
    let p = edit(
        &model,
        &root,
        part,
        json!({"default_pin_pad_map":b,"default_footprint":second}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let mut selected = original.clone();
    selected["default_footprint"] = json!(second);
    selected["default_pin_pad_map"] = json!(b);
    assert_eq!(raw(&model, "parts", part), selected);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, "parts", part), original);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, "parts", part), selected);
    // Both explicit clears are one valid raw patch; neither silently clears the other.
    let p = edit(
        &model,
        &root,
        part,
        json!({"default_pin_pad_map":null,"default_footprint":null}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let mut clear = selected;
    clear.as_object_mut().unwrap().remove("default_footprint");
    clear.as_object_mut().unwrap().remove("default_pin_pad_map");
    assert_eq!(raw(&model, "parts", part), clear);
    assert_eq!(raw(&model, "pin_pad_maps", a), map_a);
    assert_eq!(raw(&model, "pin_pad_maps", b), map_b);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn saved_default_map_undo_refuses_real_legacy_part_counter_collision_atomically() {
    let (root, mut model, part, ci, historical) = super::edit_tests::legacy_fixture_at_revision(3);
    let foot = model
        .objects
        .values()
        .find(|o| o.domain == "pool" && o.kind == "footprints")
        .unwrap()
        .object_id;
    let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
    map(&mut model, &root, a, part, Some(foot));
    map(&mut model, &root, b, part, Some(foot));
    let ci_before = model.component_instances[&ci].clone();
    for id in [a, b] {
        let p = edit(&model, &root, part, json!({"default_pin_pad_map":id})).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
    }
    let before = raw(&model, "parts", part);
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    assert_eq!(model.objects[&part].object_revision.0, 2);
    assert!(
        model
            .commit_journal_undo(&root, provenance().into())
            .is_err()
    );
    assert_eq!(model.objects[&part].object_revision.0, 2);
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
