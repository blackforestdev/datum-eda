//! Successful Unit pin additions preserve an actual connected Complete certificate.
use super::*;
use crate::substrate::{NetCorrespondenceStatus, net_correspondence_status};
#[test]
fn saved_unit_pin_add_preserves_connected_complete_eight_pads_and_held_membership_through_cold_history()
 {
    let (root, mut model, symbol, ci, part, placed, map, _) =
        physical_fixture::eight_pad_fixture("unit-add-pin-complete");
    let (net, _, members) = map_tests::connected(&mut model, &root, symbol, placed, ci, map);
    let entity: Uuid = serde_json::from_value(raw(&model, part)["entity"].clone()).unwrap();
    let e = raw(&model, entity);
    let unit: Uuid = serde_json::from_value(
        e["gates"].as_object().unwrap().values().next().unwrap()["unit"].clone(),
    )
    .unwrap();
    let original = raw(&model, unit);
    let counters: BTreeMap<Uuid, _> = original["pins"]
        .as_object()
        .unwrap()
        .keys()
        .map(|key| {
            let id: Uuid = key.parse().unwrap();
            (id, model.objects[&id].object_revision)
        })
        .collect();
    let added = Uuid::new_v4();
    let row = json!({"name":"Enable","electrical_type":"Input"});
    let mut expected = original.clone();
    expected["pins"][added.to_string()] = json!({"uuid":added,"name":"Enable","direction":"Input","electrical_type":"Input","swap_group":0,"alternates":[]});
    let unchanged = |m: &DesignModel| -> BTreeMap<String, Value> {
        m.source_shards
            .iter()
            .filter(|s| s.shard_id != m.objects[&unit].source_shard_id)
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
    assert_eq!(
        net_correspondence_status(&model, net),
        NetCorrespondenceStatus::Complete
    );
    let p=(find_native_write_verb("datum.library.edit").unwrap().build)(&NativeWriteContext{model:&model,project_root:&root},
        provenance(),json!({"kind":"units","object_id":unit,"pool_path":"pool","fields":{"pin_additions":{added.to_string():row}}})).unwrap();
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
            raw(&model, unit),
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
