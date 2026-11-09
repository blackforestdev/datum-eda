//! Existing native placements keep their body, pin graphics and electrical identity.
use super::*;
use eda_engine::api::native_write::{
    self, PreparedWrite, WriteProvenance, commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
    library,
    library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec},
    registry::{NativeWriteContext, find_native_write_verb},
};
use eda_engine::substrate::{CommitSource, DesignModel, ProjectResolver};
use serde_json::{Value, json};
use std::path::PathBuf;
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "frozen-source-test",
        CommitSource::Manual,
        "Edit saved library",
    )
}
fn fixture() -> (PathBuf, DesignModel, Uuid, Uuid, Uuid) {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.datum/scratch/codex-delivery")
        .join(format!("frozen-symbol-{}", Uuid::new_v4()));
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
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    (root, model, unit, pin, symbol)
}

fn verb(model: &DesignModel, root: &std::path::Path, id: &str, params: Value) -> PreparedWrite {
    (find_native_write_verb(id).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap()
}
fn add_symbol_binding(prepared: &mut PreparedWrite, symbol: Uuid, revision: u64) {
    let mut count = 0;
    for op in &mut prepared.batch.operations {
        if let eda_engine::substrate::Operation::CreateComponentInstance {
            component_instance,
            ..
        } = op
        {
            component_instance["library_bindings"].as_object_mut().unwrap().insert(Uuid::new_v4().to_string(),json!({"target_object_id":symbol,"pinned_object_revision":revision,"binding_role":"symbol"}));
            count += 1;
        }
    }
    assert_eq!(
        count, 1,
        "fixture must create an actual authored Symbol LibraryBinding"
    );
}

fn raw(model: &DesignModel, kind: &str, id: Uuid) -> Value {
    library::pool_library_object_payload(model, &format!("pool/{kind}/{id}.json"), id).unwrap()
}
fn edit(model: &mut DesignModel, root: &std::path::Path, kind: &str, id: Uuid, fields: Value) {
    let prepared = verb(
        model,
        root,
        "datum.library.edit",
        json!({"kind":kind,"object_id":id,"pool_path":"pool","fields":fields}),
    );
    commit_prepared(model, root, prepared).unwrap();
}
fn placed_snapshot(model: &DesignModel, root: &std::path::Path) -> Value {
    let snapshot = eda_engine::substrate::ElectricalSelectionSnapshot::capture(model).unwrap();
    let source = snapshot.schematic_source().unwrap();
    let scene = crate::native_schematic_scene::project_native_schematic_scene(model, root)
        .unwrap()
        .unwrap();
    let mut value = json!({"sheets":source.sheets,"instances":model.component_instances,
        "graphics":scene.board_graphics,"texts":scene.board_texts,"text_geometry":scene.board_text_geometries});
    // Map iteration is not primitive identity; keep every primitive and its
    // complete ordered geometry while comparing scene collections by identity.
    for key in ["graphics", "texts", "text_geometry"] {
        value[key]
            .as_array_mut()
            .unwrap()
            .sort_by_cached_key(Value::to_string);
    }
    value
}
#[test]
fn saved_library_edits_freeze_bound_and_unbound_placed_sources_through_history_and_cold() {
    for bound in [false, true] {
        let (root, mut model, unit, pin, symbol) = fixture();
        if bound {
            let (entity, gate, package, part) = (
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
            );
            let prepared = library::build_pool_library_write(
                &model,
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
                        object: library::pool_package_payload(
                            package,
                            "Logical package",
                            json!({}),
                        ),
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
            commit_prepared(&mut model, &root, prepared).unwrap();
        }
        let sheet = Uuid::new_v4();
        let prepared = verb(
            &model,
            &root,
            "datum.schematic.create_sheet",
            json!({"sheet":sheet,"name":"Root"}),
        );
        commit_prepared(&mut model, &root, prepared).unwrap();
        for (reference, angle, mirrored) in [("U1", 0, false), ("U2", 90, true)] {
            let mut prepared = verb(
                &model,
                &root,
                "datum.schematic.place_symbol",
                json!({"sheet":sheet,"library_symbol":symbol,"reference":reference,"value":"original","x_nm":10000000,"y_nm":20000000,"rotation_deg":angle,"mirrored":mirrored}),
            );
            if bound {
                add_symbol_binding(
                    &mut prepared,
                    symbol,
                    model.objects[&symbol].object_revision.0,
                );
            }
            if bound && reference == "U1" {
                // Binding was prepared from the accepted model; the preceding
                // staged Set must not erase that authenticated revision basis.
                let previous = raw(&model, "symbols", symbol);
                let mut next = previous.clone();
                next["name"] = json!("Updated before atomic binding");
                prepared.batch.operations.insert(
                    0,
                    eda_engine::substrate::Operation::SetPoolLibraryObject {
                        object_id: symbol,
                        relative_path: format!("pool/symbols/{symbol}.json"),
                        object_kind: "symbols".into(),
                        previous_object: previous,
                        object: next,
                    },
                );
                prepared.batch =
                    native_write::build_batch(&model, provenance(), prepared.batch.operations)
                        .unwrap();
            }
            commit_prepared(&mut model, &root, prepared).unwrap();
        }
        assert_eq!(!model.component_instances.is_empty(), bound);
        if bound {
            let placed = *model
                .component_instances
                .values()
                .next()
                .unwrap()
                .placed_symbol_refs
                .first()
                .unwrap();
            let mut invalid = model.clone();
            invalid.component_instances.clear();
            assert!(
                invalid
                    .materialized_placed_symbol_definition(placed, symbol)
                    .is_err(),
                "declared ownership cannot fall back to birth"
            );
            let unavailable =
                crate::native_schematic_scene::project_native_schematic_scene(&invalid, &root)
                    .unwrap()
                    .unwrap();
            assert!(
                serde_json::to_string(&unavailable.board_texts)
                    .unwrap()
                    .contains("Library source unavailable")
            );
            assert!(
                !unavailable
                    .board_graphics
                    .iter()
                    .any(|g| g.source_object_uuid == placed.to_string()
                        && g.object_id.contains(":library:")),
                "unavailable declared source never uses latest body"
            );
        }
        let before = placed_snapshot(&model, &root);
        let mut pins = raw(&model, "units", unit)["pins"].clone();
        pins[pin.to_string()]["name"] = json!("Changed source pin");
        pins[pin.to_string()]["direction"] = json!("Passive");
        edit(&mut model, &root, "units", unit, json!({"pins":pins}));
        assert_eq!(
            placed_snapshot(&model, &root),
            before,
            "Unit metadata must stay materialized"
        );
        let original = raw(&model, "symbols", symbol);
        for move_anchor in [false, true] {
            let mut anchors = raw(&model, "symbols", symbol)["pin_anchors"].clone();
            anchors[0]["orientation"] = json!("Up");
            anchors[0]["length_nm"] = json!(1200000);
            if move_anchor {
                anchors[0]["position"]["x"] = json!(-2000000);
            }
            edit(
                &mut model,
                &root,
                "symbols",
                symbol,
                json!({"pin_anchors":anchors}),
            );
            assert_ne!(raw(&model, "symbols", symbol), original);
            assert_eq!(
                placed_snapshot(&model, &root),
                before,
                "warm frozen source, bound={bound} moved={move_anchor}"
            );
            let cold = ProjectResolver::new(&root).resolve().unwrap();
            assert_eq!(placed_snapshot(&cold, &root), before, "cold frozen source");
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
            assert_eq!(placed_snapshot(&model, &root), before, "Undo frozen source");
            model
                .commit_journal_redo(&root, provenance().into())
                .unwrap();
            assert_eq!(
                placed_snapshot(&ProjectResolver::new(&root).resolve().unwrap(), &root),
                before,
                "Redo cold frozen source"
            );
        }
        let prepared = verb(
            &model,
            &root,
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"library_symbol":symbol,"reference":"U3","value":"new definition","x_nm":0,"y_nm":0}),
        );
        let placed = prepared
            .batch
            .operations
            .iter()
            .find_map(|op| match op {
                eda_engine::substrate::Operation::CreateSchematicSymbol { symbol_id, .. } => {
                    Some(*symbol_id)
                }
                _ => None,
            })
            .unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
        assert_eq!(
            model
                .materialized_placed_symbol_definition(placed, symbol)
                .unwrap(),
            Some(raw(&model, "symbols", symbol)),
            "new placements consume the revised source"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn unbound_birth_uses_transaction_before_model_for_both_operation_orders() {
    use eda_engine::substrate::Operation;
    for edit_first in [false, true] {
        let (root, mut model, _, _, symbol) = fixture();
        let original = raw(&model, "symbols", symbol);
        let sheet = Uuid::new_v4();
        let prepared = verb(
            &model,
            &root,
            "datum.schematic.create_sheet",
            json!({"sheet":sheet,"name":"Root"}),
        );
        commit_prepared(&mut model, &root, prepared).unwrap();
        let mut prepared = verb(
            &model,
            &root,
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"library_symbol":symbol,"reference":"U1","value":"old basis","x_nm":0,"y_nm":0}),
        );
        let placed = prepared
            .batch
            .operations
            .iter()
            .find_map(|op| match op {
                Operation::CreateSchematicSymbol { symbol_id, .. } => Some(*symbol_id),
                _ => None,
            })
            .unwrap();
        let mut revised = original.clone();
        revised["pin_anchors"][0]["orientation"] = json!("Down");
        let set = Operation::SetPoolLibraryObject {
            object_id: symbol,
            relative_path: format!("pool/symbols/{symbol}.json"),
            object_kind: "symbols".into(),
            previous_object: original.clone(),
            object: revised,
        };
        if edit_first {
            prepared.batch.operations.insert(0, set);
        } else {
            prepared.batch.operations.push(set);
        }
        prepared.batch =
            native_write::build_batch(&model, provenance(), prepared.batch.operations).unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
        for candidate in [&model, &ProjectResolver::new(&root).resolve().unwrap()] {
            assert_eq!(
                candidate
                    .materialized_placed_symbol_definition(placed, symbol)
                    .unwrap(),
                Some(original.clone())
            );
            assert!(
                candidate
                    .materialized_placed_symbol_definition(placed, Uuid::new_v4())
                    .is_err()
            );
            let mut missing = candidate.clone();
            missing.journal.clear();
            missing.journal_cursor.applied_transaction_count = 0;
            assert!(
                missing
                    .materialized_placed_symbol_definition(placed, symbol)
                    .unwrap()
                    .is_none()
            );
            let mut ambiguous = candidate.clone();
            let birth = ambiguous.journal.last().unwrap().clone();
            ambiguous.journal.push(birth);
            ambiguous.journal_cursor.applied_transaction_count = ambiguous.journal.len();
            assert!(
                ambiguous
                    .materialized_placed_symbol_definition(placed, symbol)
                    .is_err()
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn restored_counter_collision_refuses_before_existing_pinned_user_loses_its_source() {
    use eda_engine::substrate::{ObjectRevision, Operation};
    let (root, mut model, unit, _, symbol) = fixture();
    let original = raw(&model, "symbols", symbol);
    let mut past = original.clone();
    past["pin_anchors"][0]["orientation"] = json!("Up");
    edit(
        &mut model,
        &root,
        "symbols",
        symbol,
        json!({"pin_anchors":past["pin_anchors"]}),
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(model.objects[&symbol].object_revision, ObjectRevision(0));
    assert_eq!(
        model
            .materialized_pool_object_revision(symbol, "symbols", ObjectRevision(1))
            .unwrap(),
        past
    );
    let (entity, gate, package, part, sheet) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let prepared = library::build_pool_library_write(
        &model,
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
                object: library::pool_package_payload(package, "Logical package", json!({})),
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
    commit_prepared(&mut model, &root, prepared).unwrap();
    let prepared = verb(
        &model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut prepared = verb(
        &model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":sheet,"library_symbol":symbol,"reference":"U1","value":"Pinned historical definition","x_nm":0,"y_nm":0}),
    );
    add_symbol_binding(&mut prepared, symbol, 1);
    commit_prepared(&mut model, &root, prepared).unwrap();
    assert!(
        model
            .component_instances
            .values()
            .any(|ci| ci.library_bindings.values().any(|b| b.binding_role
                == eda_engine::substrate::LibraryBindingRole::Symbol
                && b.pinned_object_revision == ObjectRevision(1)))
    );
    let before = placed_snapshot(&model, &root);
    let mut replacement = original.clone();
    replacement["pin_anchors"][0]["orientation"] = json!("Down");
    let journal = std::fs::read(eda_engine::substrate::transaction_journal_path(&root)).unwrap();
    let result = (find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"kind":"symbols","object_id":symbol,"pool_path":"pool","fields":{"pin_anchors":replacement["pin_anchors"]}}),
    );
    assert!(
        result.is_err(),
        "new counter1 must not alias historical pinned1 with another payload"
    );
    assert_eq!(
        std::fs::read(eda_engine::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    for set_first in [false, true] {
        let mut pending = verb(
            &model,
            &root,
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"library_symbol":symbol,"reference":"U2","value":"New staged binding","x_nm":5000000,"y_nm":0}),
        );
        add_symbol_binding(&mut pending, symbol, 1);
        let source_edit = Operation::SetPoolLibraryObject {
            object_id: symbol,
            relative_path: format!("pool/symbols/{symbol}.json"),
            object_kind: "symbols".into(),
            previous_object: original.clone(),
            object: replacement.clone(),
        };
        if set_first {
            pending.batch.operations.insert(0, source_edit);
        } else {
            pending.batch.operations.push(source_edit);
        }
        pending.batch =
            native_write::build_batch(&model, provenance(), pending.batch.operations).unwrap();
        assert!(
            commit_prepared(&mut model, &root, pending).is_err(),
            "new binding cannot admit conflicting restored counter in a pending batch"
        );
        assert_eq!(
            std::fs::read(eda_engine::substrate::transaction_journal_path(&root)).unwrap(),
            journal
        );
    }
    assert_eq!(raw(&model, "symbols", symbol), original);
    assert_eq!(
        placed_snapshot(&ProjectResolver::new(&root).resolve().unwrap(), &root),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[path = "native_library_frozen_rectangle_tests.rs"]
mod rectangle_tests;
