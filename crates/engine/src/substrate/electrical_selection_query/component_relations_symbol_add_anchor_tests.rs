//! Appending a library anchor retains actual connected physical correspondence.
use super::*;
use crate::substrate::{NetCorrespondenceStatus, net_correspondence_status};
#[test]
fn saved_symbol_add_anchor_preserves_connected_complete_eight_pads_through_history_and_cold() {
    let (root, mut model, placed_symbol, ci, part, package, map, _) =
        physical_fixture::eight_pad_fixture("symbol-add-anchor-complete");
    let (net, _, members) =
        map_tests::connected(&mut model, &root, placed_symbol, package, ci, map);
    let entity: Uuid = serde_json::from_value(raw(&model, part)["entity"].clone()).unwrap();
    let gate = raw(&model, entity)["gates"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .clone();
    let unit: Uuid = serde_json::from_value(gate["unit"].clone()).unwrap();
    let symbol: Uuid = serde_json::from_value(gate["symbol"].clone()).unwrap();
    let added = Uuid::new_v4();
    // One canonical fixture operation adds a named, currently unanchored pin.
    let edit = |m: &DesignModel, kind: &str, id: Uuid, fields: Value| {
        (find_native_write_verb("datum.library.edit").unwrap().build)(
            &NativeWriteContext {
                model: m,
                project_root: &root,
            },
            provenance(),
            json!({"kind":kind,"object_id":id,"pool_path":"pool","fields":fields}),
        )
        .unwrap()
    };
    let p = edit(
        &model,
        "units",
        unit,
        json!({"pin_additions":{added.to_string():{"name":"Enable","electrical_type":"Input"}}}),
    );
    commit_prepared(&mut model, &root, p).unwrap();
    let original = raw(&model, symbol);
    let unchanged = |m: &DesignModel| -> BTreeMap<String, Value> {
        m.source_shards
            .iter()
            .filter(|s| s.shard_id != m.objects[&symbol].source_shard_id)
            .map(|s| {
                (
                    s.relative_path.clone(),
                    m.materialized_source_shard_value_by_relative_path(&s.relative_path)
                        .unwrap(),
                )
            })
            .collect()
    };
    let before = unchanged(&model);
    let counters: BTreeMap<_, _> = raw(&model, unit)["pins"]
        .as_object()
        .unwrap()
        .keys()
        .map(|k| {
            let id: Uuid = k.parse().unwrap();
            (id, model.objects[&id].object_revision)
        })
        .collect();
    let held = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let row = json!({"pin":added,"position":{"x":0,"y":-3000000},"style":{"orientation":"Down","length_nm":1000000,"decoration":"none"}});
    let mut expected = original.clone();
    expected["pin_anchors"]
        .as_array_mut()
        .unwrap()
        .push(row.clone());
    let p = edit(&model, "symbols", symbol, json!({"anchor_additions":[row]}));
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
            raw(&model, symbol),
            if phase == 1 {
                original.clone()
            } else {
                expected.clone()
            }
        );
        assert_eq!(unchanged(&model), before);
        for (id, revision) in &counters {
            assert_eq!(model.objects[id].object_revision, *revision);
        }
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
