use super::{tests::*, *};
use crate::api::native_write::{commit_prepared, registry::find_native_write_verb};
use crate::substrate::ProjectResolver;

fn revise(
    model: &crate::substrate::DesignModel,
    root: &std::path::Path,
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
fn verb(
    model: &mut crate::substrate::DesignModel,
    root: &std::path::Path,
    name: &str,
    params: Value,
) -> Uuid {
    let p = (find_native_write_verb(name).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    let id = p.primary_object_id.unwrap();
    commit_prepared(model, root, p).unwrap();
    id
}
fn legacy_fixture(
    restored: bool,
) -> (
    std::path::PathBuf,
    crate::substrate::DesignModel,
    Uuid,
    Uuid,
    Value,
) {
    legacy_fixture_at_revision(u64::from(restored))
}
pub(super) fn legacy_fixture_at_revision(
    pinned_revision: u64,
) -> (
    std::path::PathBuf,
    crate::substrate::DesignModel,
    Uuid,
    Uuid,
    Value,
) {
    use crate::api::native_write::BatchComposer;
    use crate::substrate::{ElectricalSelectionSnapshot, Operation};
    let (root, mut model, entity, package, _, _) = fixture();
    let part = Uuid::new_v4();
    let p = prepare(&model, &root, part, fields(entity, package, None)).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    if pinned_revision > 0 {
        for i in 0..pinned_revision {
            let p = revise(
                &model,
                &root,
                part,
                json!({"value":if i == 0 { "Historical pinned value".into() } else { format!("Historical pinned value {i}") }}),
            )
            .unwrap();
            commit_prepared(&mut model, &root, p).unwrap();
        }
        for _ in 0..pinned_revision {
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
        }
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(model.objects[&part].object_revision.0, 0);
    }
    let revision = crate::substrate::ObjectRevision(pinned_revision);
    let historical = model
        .materialized_pool_object_revision(part, "parts", revision)
        .unwrap();
    let entity_raw = library::pool_library_object_payload(
        &model,
        &format!("pool/entities/{entity}.json"),
        entity,
    )
    .unwrap();
    let symbol = entity_raw["gates"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()["symbol"]
        .clone();
    let sheet = Uuid::new_v4();
    verb(
        &mut model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Legacy Part reference"}),
    );
    let placed = verb(
        &mut model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":sheet,"library_symbol":symbol,"reference":"U1","value":"Keep placed value","x_nm":0,"y_nm":0}),
    );
    let authored = model
        .component_instances
        .values()
        .find(|c| {
            c.authority == crate::substrate::ComponentInstanceAuthority::Authored
                && c.placed_symbol_refs.contains(&placed)
        })
        .map(|c| c.id);
    let ci = authored.unwrap_or_else(Uuid::new_v4);
    let operation = if authored.is_some() {
        let owner = &model.objects[&ci];
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.shard_id == owner.source_shard_id)
            .unwrap();
        let previous = model
            .materialized_source_shard_value_by_relative_path(&shard.relative_path)
            .unwrap()["component_instance"]
            .clone();
        let mut next = previous.clone();
        next["object_revision"] = json!(owner.object_revision.0 + 1);
        next["part_ref"] = json!({"object_id":part,"object_revision":revision.0});
        next.as_object_mut().unwrap().remove("library_bindings");
        Operation::SetComponentInstance {
            component_instance_id: ci,
            previous_component_instance: previous,
            component_instance: next,
        }
    } else {
        Operation::CreateComponentInstance {
            component_instance_id: ci,
            component_instance: json!({"uuid":ci,"object_revision":0,
            "part_ref":{"object_id":part,"object_revision":revision.0},
            "placed_symbol_refs":[{"object_id":placed,"object_revision":model.objects[&placed].object_revision.0}],"placed_package_refs":[]}),
        }
    };
    let p = BatchComposer::compose(&model, provenance())
        .push_op(operation)
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(model.component_instances[&ci].library_bindings.is_empty());
    assert_eq!(model.component_instances[&ci].part_ref, Some(part));
    assert_eq!(
        ElectricalSelectionSnapshot::capture(&model)
            .unwrap()
            .schematic_source()
            .unwrap()
            .sheets[&sheet]
            .symbols[&placed]
            .value,
        "Keep placed value"
    );
    (root, model, part, ci, historical)
}

#[test]
fn legacy_part_ref_only_restored_counter_collision_refuses_and_same_payload_survives() {
    let (root, mut model, part, ci, historical) = legacy_fixture(true);
    let before = model.component_instances[&ci].clone();
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    let path = root.join(format!("pool/parts/{part}.json"));
    let raw = std::fs::read(&path).unwrap();
    assert!(
        revise(
            &model,
            &root,
            part,
            json!({"value":"Conflicting counter one"})
        )
        .is_err()
    );
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    assert_eq!(std::fs::read(&path).unwrap(), raw);
    assert_eq!(model.component_instances[&ci], before);
    let p = revise(
        &model,
        &root,
        part,
        json!({"value":"Historical pinned value"}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    for model in [&model, &ProjectResolver::new(&root).resolve().unwrap()] {
        assert_eq!(model.component_instances[&ci], before);
        assert!(model.component_instances[&ci].library_bindings.is_empty());
        assert!(
            !model
                .diagnostics
                .iter()
                .any(|d| d.code.starts_with("component_instance_"))
        );
        assert_eq!(
            model
                .materialized_pool_object_revision(
                    part,
                    "parts",
                    crate::substrate::ObjectRevision(1)
                )
                .unwrap(),
            historical
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn ordinary_legacy_part_ref_only_metadata_history_keeps_exact_declared_source() {
    let (root, mut model, part, ci, historical) = legacy_fixture(false);
    let before = model.component_instances[&ci].clone();
    let p = revise(
        &model,
        &root,
        part,
        json!({"value":"New unpropagated source"}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(cold.component_instances[&ci], before);
    assert!(cold.component_instances[&ci].library_bindings.is_empty());
    assert!(
        !cold
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("component_instance_"))
    );
    assert_eq!(
        cold.materialized_pool_object_revision(part, "parts", crate::substrate::ObjectRevision(0))
            .unwrap(),
        historical
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn saved_part_patch_preserves_unedited_optional_ordered_unknown_metadata_and_refuses_links() {
    let (root, mut model, entity, package, _, footprint) = fixture();
    let id = Uuid::new_v4();
    let p = prepare(&model, &root, id, fields(entity, package, Some(footprint))).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let target = PoolLibraryObjectTarget::new("pool", "parts", id);
    let mut original =
        library::pool_library_object_payload(&model, &target.relative_path, id).unwrap();
    for key in [
        "packaging_options",
        "manufacturer_jep106",
        "behavioural_models",
        "thermal",
    ] {
        original.as_object_mut().unwrap().remove(key);
    }
    original["tags"] = json!(["z", "a", "z"]);
    original["orderable_mpns"] = json!(["PN-2", "PN-1", "PN-2"]);
    original["parametric"] = json!({"unrelated":"as entered"});
    original["custom"] = json!({"ordered":[3,1,2]});
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: target.clone(),
            object: original.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let edit = |model: &crate::substrate::DesignModel, fields: Value| {
        (find_native_write_verb("datum.library.edit").unwrap().build)(
            &NativeWriteContext {
                model,
                project_root: &root,
            },
            provenance(),
            json!({"kind":"parts","object_id":id,"pool_path":"pool","fields":fields}),
        )
    };
    let journal = crate::substrate::transaction_journal_path(&root);
    let before = std::fs::read(&journal).unwrap();
    for fields in [
        json!({"uuid":id}),
        json!({"entity":entity}),
        json!({"package":package}),
        json!({"pad_map":{}}),
        json!({"lifecycle":"Active"}),
        json!({"approval_status":"approved"}),
        json!({"mpn":" "}),
        json!({"manufacturer":""}),
        json!({"value":1}),
        json!({"tags":[7]}),
    ] {
        assert!(edit(&model, fields).is_err());
    }
    assert_eq!(std::fs::read(&journal).unwrap(), before);
    let p = edit(&model, json!({"description":"Corrected description"})).unwrap();
    assert_eq!(std::fs::read(&journal).unwrap(), before);
    let stale = p.clone();
    commit_prepared(&mut model, &root, p).unwrap();
    let mut expected = original.clone();
    expected["description"] = json!("Corrected description");
    assert_eq!(
        library::pool_library_object_payload(&model, &target.relative_path, id).unwrap(),
        expected
    );
    let committed = std::fs::read(&journal).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(std::fs::read(&journal).unwrap(), committed);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        library::pool_library_object_payload(&model, &target.relative_path, id).unwrap(),
        original
    );
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        library::pool_library_object_payload(&cold, &target.relative_path, id).unwrap(),
        expected
    );
    std::fs::remove_dir_all(root).unwrap();
}
