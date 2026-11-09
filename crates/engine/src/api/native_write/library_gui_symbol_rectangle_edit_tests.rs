//! Five raw rectangle leaves, frozen real users and atomic retained counter safety.
use super::*;
use crate::api::native_write::registry::find_native_write_verb;
use crate::substrate::{ElectricalSelectionSnapshot, ObjectRevision, Operation, SourceShardKind};
fn set(model: &mut DesignModel, root: &Path, symbol: Uuid, object: Value) {
    let p = library::build_pool_library_write(
        model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", "symbols", symbol),
            object,
        }],
    )
    .unwrap();
    commit_prepared(model, root, p).unwrap();
}
fn mixed(mut object: Value) -> Value {
    object["drawings"].as_array_mut().unwrap().insert(
        1,
        json!({"Rect":{
        "min":{"x":-2000000,"y":-1000000,"vendor":{"keep":[3,1,3]}},
        "max":{"x":2000000,"y":1000000,"optional":null},"width":100000,
        "extension":{"ordered":["z","a","z"]}}}),
    );
    object
}
fn changed(object: &Value, x: i64) -> Value {
    let mut drawings = object["drawings"].clone();
    drawings[1]["Rect"]["min"]["x"] = json!(x);
    drawings
}
fn verb(model: &DesignModel, root: &Path, name: &str, params: Value) -> PreparedWrite {
    (find_native_write_verb(name).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap()
}
fn component(model: &mut DesignModel, root: &Path, unit: Uuid, symbol: Uuid) {
    let (entity, gate, package, part) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let p = library::build_pool_library_write(
        model,
        provenance(),
        None,
        vec![
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "entities", entity),
                object: library::pool_entity_payload(
                    entity,
                    gate,
                    unit,
                    symbol,
                    "Component",
                    "U",
                    "Maker",
                    "A",
                ),
            },
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "packages", package),
                object: library::pool_package_payload(package, "Body", json!({})),
            },
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "parts", part),
                object: library::pool_part_payload(
                    part, entity, package, "MPN", "Maker", "Value", "", "", "Active",
                ),
            },
        ],
    )
    .unwrap();
    commit_prepared(model, root, p).unwrap();
}
fn place(
    model: &mut DesignModel,
    root: &Path,
    sheet: Uuid,
    symbol: Uuid,
    revision: Option<u64>,
    reference: &str,
) -> Uuid {
    let mut p = verb(
        model,
        root,
        "datum.schematic.place_symbol",
        json!({"sheet":sheet,"library_symbol":symbol,"reference":reference,"value":"Frozen","x_nm":10000000,"y_nm":20000000}),
    );
    let id = p.primary_object_id.unwrap();
    if let Some(revision) = revision {
        let mut count = 0;
        for op in &mut p.batch.operations {
            if let Operation::CreateComponentInstance {
                component_instance, ..
            } = op
            {
                component_instance["library_bindings"].as_object_mut().unwrap().insert(Uuid::new_v4().to_string(),json!({"target_object_id":symbol,"pinned_object_revision":revision,"binding_role":"symbol"}));
                count += 1;
            }
        }
        assert_eq!(count, 1, "actual authored binding fixture");
    }
    commit_prepared(model, root, p).unwrap();
    id
}
fn sheet(model: &mut DesignModel, root: &Path) -> Uuid {
    let id = Uuid::new_v4();
    let p = verb(
        model,
        root,
        "datum.schematic.create_sheet",
        json!({"sheet":id,"name":"Root"}),
    );
    commit_prepared(model, root, p).unwrap();
    id
}
fn preserved(model: &DesignModel, root: &Path) -> Value {
    let snapshot = ElectricalSelectionSnapshot::capture(model).unwrap();
    let sources: std::collections::BTreeMap<_, _> = model
        .source_shards
        .iter()
        .filter(|s| s.kind != SourceShardKind::Pool)
        .map(|s| {
            (
                s.relative_path.clone(),
                std::fs::read(root.join(&s.relative_path)).unwrap(),
            )
        })
        .collect();
    let members: std::collections::BTreeMap<_, _> = snapshot
        .net_ids()
        .into_iter()
        .map(|id| {
            let crate::substrate::ElectricalIdentity::Net { anchor, .. } =
                &model.electrical_identities[&id].identity
            else {
                panic!("Net")
            };
            (
                id,
                snapshot
                    .schematic_run(snapshot.revision(), anchor)
                    .unwrap()
                    .owned,
            )
        })
        .collect();
    json!({"sources":sources,"sheets":snapshot.schematic_source().unwrap().sheets,"ci":model.component_instances,"ei":model.electrical_identities,"members":members})
}
#[test]
fn symbol_rectangle_raw_patch_preview_stale_and_forbidden_changes_are_atomic() {
    let (root, mut model, _, _, symbol) = fixture();
    let original = mixed(raw(&model, "symbols", symbol));
    set(&mut model, &root, symbol, original.clone());
    let drawings = changed(&original, -3000000);
    let p = prepare(
        &model,
        &root,
        "symbols",
        symbol,
        json!({"drawings":drawings}),
    )
    .unwrap();
    assert!(p.batch.operations.iter().any(|op| matches!(op,
        Operation::GuardObjectRevision { object_id, expected_object_revision }
        if *object_id == symbol && *expected_object_revision == model.objects[&symbol].object_revision)));
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    assert_eq!(
        raw(
            &ProjectResolver::new(&root).resolve().unwrap(),
            "symbols",
            symbol
        ),
        original
    );
    let mut concurrent = original.clone();
    concurrent["name"] = json!("Concurrent");
    set(&mut model, &root, symbol, concurrent.clone());
    let after_competing = std::fs::read(&journal).unwrap();
    assert!(commit_prepared(&mut model, &root, p).is_err());
    assert_eq!(std::fs::read(&journal).unwrap(), after_competing);
    assert_ne!(bytes, after_competing);
    for wrong in [
        {
            let mut v = drawings.clone();
            v[0]["Circle"]["radius"] = json!(5);
            v
        },
        {
            let mut v = drawings.clone();
            v[1]["Rect"]["min"]["vendor"] = json!(false);
            v
        },
        {
            let mut v = drawings.clone();
            v[1]["Rect"].as_object_mut().unwrap().remove("extension");
            v
        },
        {
            let mut v = drawings.clone();
            v.as_array_mut().unwrap().swap(0, 2);
            v
        },
        {
            let mut v = drawings.clone();
            let repeated = v[1].clone();
            v.as_array_mut().unwrap().push(repeated);
            v
        },
        {
            let mut v = drawings.clone();
            v[1]["Rect"]["min"]["x"] = json!(2000000);
            v
        },
        {
            let mut v = drawings.clone();
            v[1]["Rect"]["width"] = json!(0);
            v
        },
        {
            let mut v = drawings.clone();
            v[1]["Rect"]["max"]["x"] = json!(1.5);
            v
        },
        {
            let mut v = drawings.clone();
            v[1]["Rect"]["width"] = json!("100000");
            v
        },
    ] {
        assert!(prepare(&model, &root, "symbols", symbol, json!({"drawings":wrong})).is_err());
        assert_eq!(std::fs::read(&journal).unwrap(), after_competing);
        assert_eq!(raw(&model, "symbols", symbol), concurrent);
    }
    let p = prepare(
        &model,
        &root,
        "symbols",
        symbol,
        json!({"drawings":drawings}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let mut expected = concurrent.clone();
    expected["drawings"] = drawings;
    assert_eq!(raw(&model, "symbols", symbol), expected);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, "symbols", symbol), concurrent);
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
    for zero_or_multiple in [
        json!([]),
        json!([original["drawings"][1], original["drawings"][1]]),
    ] {
        let mut source = expected.clone();
        source["drawings"] = zero_or_multiple.clone();
        set(&mut model, &root, symbol, source);
        assert!(
            prepare(
                &model,
                &root,
                "symbols",
                symbol,
                json!({"drawings":zero_or_multiple})
            )
            .is_err()
        );
        assert!(
            prepare(
                &model,
                &root,
                "symbols",
                symbol,
                json!({"name":"Metadata remains available"})
            )
            .is_ok()
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn symbol_rectangle_preserves_real_bound_and_birth_users_pins_ci_nets_history_and_cold() {
    for bound in [false, true] {
        let (root, mut model, unit, _, symbol) = fixture();
        let original = mixed(raw(&model, "symbols", symbol));
        set(&mut model, &root, symbol, original.clone());
        if bound {
            component(&mut model, &root, unit, symbol);
        }
        let sheet = sheet(&mut model, &root);
        let revision = model.objects[&symbol].object_revision.0;
        let placed = place(
            &mut model,
            &root,
            sheet,
            symbol,
            bound.then_some(revision),
            "U1",
        );
        let p = verb(
            &model,
            &root,
            "datum.schematic.draw_wire",
            json!({"sheet":sheet,"points":[{"x":9000000,"y":20000000},{"x":7000000,"y":20000000}]}),
        );
        commit_prepared(&mut model, &root, p).unwrap();
        assert_eq!(!model.component_instances.is_empty(), bound);
        assert!(
            !ElectricalSelectionSnapshot::capture(&model)
                .unwrap()
                .net_ids()
                .is_empty()
        );
        let before = preserved(&model, &root);
        let check = |model: &DesignModel| {
            assert_eq!(preserved(model, &root), before);
            assert_eq!(
                model
                    .materialized_placed_symbol_definition(placed, symbol)
                    .unwrap(),
                Some(original.clone())
            );
        };
        let mut next = original.clone();
        next["drawings"] = changed(&original, -4000000);
        next["drawings"][1]["Rect"]["max"]["y"] = json!(3000000);
        next["drawings"][1]["Rect"]["width"] = json!(200000);
        let p = prepare(
            &model,
            &root,
            "symbols",
            symbol,
            json!({"drawings":next["drawings"]}),
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        check(&model);
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        check(&model);
        assert_eq!(raw(&model, "symbols", symbol), original);
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        check(&model);
        model = ProjectResolver::new(&root).resolve().unwrap();
        check(&model);
        assert_eq!(raw(&model, "symbols", symbol), next);
        std::fs::remove_dir_all(root).unwrap();
    }
}
#[test]
fn symbol_rectangle_real_restored_counter_undo_refuses_collision_and_admits_equal_payload() {
    for equal in [false, true] {
        let (root, mut model, unit, _, template) = fixture();
        // A separate normal Create provides the same authenticated creation
        // Undo/Redo epoch used by the existing frozen-Symbol reader controls.
        let symbol = Uuid::new_v4();
        let mut original = mixed(raw(&model, "symbols", template));
        original["uuid"] = json!(symbol);
        let p = library::build_pool_library_write(
            &model,
            provenance(),
            None,
            vec![PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "symbols", symbol),
                object: original.clone(),
            }],
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        for x in [-3000000, -4000000, -5000000] {
            let mut v = original.clone();
            v["drawings"] = changed(&original, x);
            set(&mut model, &root, symbol, v);
        }
        assert_eq!(model.objects[&symbol].object_revision, ObjectRevision(3));
        let historical = raw(&model, "symbols", symbol);
        // Undo all three source edits and then the actual Symbol creation.
        for _ in 0..4 {
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
        }
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(model.objects[&symbol].object_revision, ObjectRevision(0));
        assert_eq!(raw(&model, "symbols", symbol), original);
        assert_eq!(
            model
                .materialized_pool_object_revision(symbol, "symbols", ObjectRevision(3))
                .unwrap(),
            historical
        );
        component(&mut model, &root, unit, symbol);
        let sheet = sheet(&mut model, &root);
        let placed = place(&mut model, &root, sheet, symbol, Some(3), "U1");
        let before = preserved(&model, &root);
        let first = if equal {
            historical["drawings"].clone()
        } else {
            changed(&historical, -3000000)
        };
        let p = prepare(&model, &root, "symbols", symbol, json!({"drawings":first})).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let second = changed(&historical, -7000000);
        let p = prepare(&model, &root, "symbols", symbol, json!({"drawings":second})).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        assert_eq!(model.objects[&symbol].object_revision, ObjectRevision(2));
        let journal = crate::substrate::transaction_journal_path(&root);
        let bytes = std::fs::read(&journal).unwrap();
        let source = raw(&model, "symbols", symbol);
        let result = model.commit_journal_undo(&root, provenance().into());
        if equal {
            result.unwrap();
            assert_eq!(model.objects[&symbol].object_revision, ObjectRevision(3));
            assert_eq!(raw(&model, "symbols", symbol), historical);
        } else {
            assert!(result.is_err());
            assert_eq!(model.objects[&symbol].object_revision, ObjectRevision(2));
            assert_eq!(raw(&model, "symbols", symbol), source);
            assert_eq!(std::fs::read(&journal).unwrap(), bytes);
        }
        let cold = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(preserved(&cold, &root), before);
        assert_eq!(
            cold.materialized_placed_symbol_definition(placed, symbol)
                .unwrap(),
            Some(historical.clone())
        );
        assert_eq!(
            cold.materialized_pool_object_revision(symbol, "symbols", ObjectRevision(3))
                .unwrap(),
            historical
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
