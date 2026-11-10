//! Existing pin removal from a library Symbol cannot alter frozen physical nets.
use super::*;
use crate::substrate::{NetCorrespondenceStatus, net_correspondence_status};
#[test]
fn saved_symbol_remove_anchor_preserves_connected_complete_eight_pads_and_held_membership_through_cold()
 {
    let (root, mut model, placed_symbol, ci, _, package, map, _) =
        physical_fixture::eight_pad_fixture("symbol-remove-anchor-complete");
    let (net, _, members) =
        map_tests::connected(&mut model, &root, placed_symbol, package, ci, map);
    let snap = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let placed = snap
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .find_map(|s| s.symbols.get(&placed_symbol))
        .unwrap();
    let symbol: Uuid = placed.lib_id.as_ref().unwrap().parse().unwrap();
    let original = raw(&model, symbol);
    let unit: Uuid = serde_json::from_value(original["unit"].clone()).unwrap();
    let pin: Uuid = serde_json::from_value(original["pin_anchors"][0]["pin"].clone()).unwrap();
    let counters: BTreeMap<_, _> = raw(&model, unit)["pins"]
        .as_object()
        .unwrap()
        .keys()
        .map(|k| {
            let id: Uuid = k.parse().unwrap();
            (id, model.objects[&id].object_revision)
        })
        .collect();
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
    let held = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let mut expected = original.clone();
    expected["pin_anchors"].as_array_mut().unwrap().remove(0);
    let p=(find_native_write_verb("datum.library.edit").unwrap().build)(&NativeWriteContext{model:&model,project_root:&root},provenance(),json!({"kind":"symbols","object_id":symbol,"pool_path":"pool","fields":{"anchor_removals":[pin]}})).unwrap();
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
        for (id, rev) in &counters {
            assert_eq!(model.objects[id].object_revision, *rev);
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
