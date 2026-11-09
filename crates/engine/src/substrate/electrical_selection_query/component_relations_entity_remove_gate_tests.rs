//! Successful unused gate removal preserve an actual connected Complete certificate.
use super::*;
use crate::substrate::{NetCorrespondenceStatus, net_correspondence_status};
#[test]
fn saved_gate_remove_preserves_connected_complete_eight_pads_and_held_membership_through_cold_history()
 {
    let (root, mut model, symbol, ci, part, placed, map, _) =
        physical_fixture::eight_pad_fixture("entity-remove-gate-complete");
    let (net, _, members) = map_tests::connected(&mut model, &root, symbol, placed, ci, map);
    let entity: Uuid = serde_json::from_value(raw(&model, part)["entity"].clone()).unwrap();
    let original = raw(&model, entity);
    let added = Uuid::new_v4();
    let old = original["gates"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap();
    let row = json!({"uuid":added,"name":"Auxiliary","unit":old["unit"],"symbol":old["symbol"]});
    let mut expected = original.clone();
    expected["gates"][added.to_string()] = row.clone();
    let unchanged = |m: &DesignModel| -> BTreeMap<String, Value> {
        m.source_shards
            .iter()
            .filter(|s| s.shard_id != m.objects[&entity].source_shard_id)
            .map(|s| {
                (
                    s.relative_path.clone(),
                    m.materialized_source_shard_value_by_relative_path(&s.relative_path)
                        .unwrap(),
                )
            })
            .collect()
    };
    let add=(find_native_write_verb("datum.library.edit").unwrap().build)(&NativeWriteContext{model:&model,project_root:&root},
        provenance(),json!({"kind":"entities","object_id":entity,"pool_path":"pool","fields":{"gate_additions":{added.to_string():row}}})).unwrap();
    commit_prepared(&mut model, &root, add).unwrap();
    let before = unchanged(&model);
    let held = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert_eq!(
        net_correspondence_status(&model, net),
        NetCorrespondenceStatus::Complete
    );
    let p=(find_native_write_verb("datum.library.edit").unwrap().build)(&NativeWriteContext{model:&model,project_root:&root},
        provenance(),json!({"kind":"entities","object_id":entity,"pool_path":"pool","fields":{"gate_removals":[added]}})).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    for phase in 0..4 {
        match phase {
            1 => {
                model
                    .commit_journal_undo(&root, provenance().into())
                    .unwrap();
            }
            2 => {
                model
                    .commit_journal_redo(&root, provenance().into())
                    .unwrap();
            }
            3 => {
                model = ProjectResolver::new(&root).resolve().unwrap();
            }
            _ => {}
        }
        assert_eq!(
            raw(&model, entity),
            if phase == 1 {
                expected.clone()
            } else {
                original.clone()
            }
        );
        assert_eq!(unchanged(&model), before);
        assert_eq!(
            net_correspondence_status(&model, net),
            NetCorrespondenceStatus::Complete
        );
        assert_eq!(
            held.global_net(held.revision(), net).unwrap().owned,
            members
        );
        let fresh = ElectricalSelectionSnapshot::capture(&model).unwrap();
        assert_eq!(
            fresh.global_net(fresh.revision(), net).unwrap().owned,
            members
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn independent_unplaced_default_nondefault_and_legacy_maps_refuse_removal_atomically() {
    use crate::api::native_write::library::{
        self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec,
    };
    let (root, mut model, symbol, ci, part, placed, map, _) =
        physical_fixture::eight_pad_fixture("entity-remove-mapped");
    let (net, _, members) = map_tests::connected(&mut model, &root, symbol, placed, ci, map);
    let entity: Uuid = serde_json::from_value(raw(&model, part)["entity"].clone()).unwrap();
    for phase in 0..3 {
        let (gate, new_part, new_map) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let old = raw(&model, entity)["gates"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap()
            .clone();
        let add=(find_native_write_verb("datum.library.edit").unwrap().build)(&NativeWriteContext{model:&model,project_root:&root},provenance(),
            json!({"kind":"entities","object_id":entity,"pool_path":"pool","fields":{"gate_additions":{gate.to_string():{"uuid":gate,"name":"Unplaced mapped","unit":old["unit"],"symbol":old["symbol"]}}}})).unwrap();
        commit_prepared(&mut model, &root, add).unwrap();
        let mut rows = raw(&model, map)["mappings"].clone();
        for row in rows.as_object_mut().unwrap().values_mut() {
            row["gate"] = json!(gate);
        }
        let mut p = raw(&model, part);
        p["uuid"] = json!(new_part);
        p["mpn"] = json!(format!("Independent removal case {phase}"));
        p["default_pin_pad_map"] = Value::Null;
        p["pad_map"] = if phase == 2 { rows.clone() } else { json!({}) };
        let mut specs = vec![Spec::Create {
            target: Target::new("pool", "parts", new_part),
            object: p.clone(),
        }];
        if phase < 2 {
            let mut m = raw(&model, map);
            m["uuid"] = json!(new_map);
            m["part"] = json!(new_part);
            m["mappings"] = rows;
            specs.push(Spec::Create {
                target: Target::new("pool", "pin_pad_maps", new_map),
                object: m,
            });
        }
        let setup =
            library::build_pool_library_write(&model, provenance(), Some("pool"), specs).unwrap();
        commit_prepared(&mut model, &root, setup).unwrap();
        if phase == 1 {
            p["default_pin_pad_map"] = json!(new_map);
            let setup = library::build_pool_library_write(
                &model,
                provenance(),
                Some("pool"),
                vec![Spec::Set {
                    target: Target::new("pool", "parts", new_part),
                    object: p,
                }],
            )
            .unwrap();
            commit_prepared(&mut model, &root, setup).unwrap();
        }
        let before: BTreeMap<_, _> = model
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
        let held = ElectricalSelectionSnapshot::capture(&model).unwrap();
        let error=(find_native_write_verb("datum.library.edit").unwrap().build)(&NativeWriteContext{model:&model,project_root:&root},provenance(),
            json!({"kind":"entities","object_id":entity,"pool_path":"pool","fields":{"gate_removals":[gate]}})).unwrap_err();
        assert!(
            error.to_string().contains("Mapped Gate"),
            "phase {phase}: {error}"
        );
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
        assert_eq!(model.model_revision, revision);
        for (path, source) in &before {
            assert_eq!(
                model
                    .materialized_source_shard_value_by_relative_path(path)
                    .unwrap(),
                *source
            );
        }
        assert_eq!(
            net_correspondence_status(&model, net),
            NetCorrespondenceStatus::Complete
        );
        assert_eq!(
            held.global_net(held.revision(), net).unwrap().owned,
            members
        );
        model = ProjectResolver::new(&root).resolve().unwrap();
        let fresh = ElectricalSelectionSnapshot::capture(&model).unwrap();
        assert_eq!(
            fresh.global_net(fresh.revision(), net).unwrap().owned,
            members
        );
        assert_eq!(
            net_correspondence_status(&model, net),
            NetCorrespondenceStatus::Complete
        );
        for (path, source) in &before {
            assert_eq!(
                model
                    .materialized_source_shard_value_by_relative_path(path)
                    .unwrap(),
                *source
            );
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
