//! A successful unused logical pin removal preserves real connected Complete authority.
use super::*;
use crate::substrate::{NetCorrespondenceStatus, net_correspondence_status};

#[test]
fn unused_unit_pin_remove_preserves_connected_complete_and_held_fresh_membership_history_cold() {
    let (root, mut model, symbol, ci, part, package, map, _) =
        physical_fixture::eight_pad_fixture("unit-remove-pin-complete");
    let (net, _, members) = map_tests::connected(&mut model, &root, symbol, package, ci, map);
    let entity: Uuid = serde_json::from_value(raw(&model, part)["entity"].clone()).unwrap();
    let entity_source = raw(&model, entity);
    let unit: Uuid = serde_json::from_value(
        entity_source["gates"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap()["unit"]
            .clone(),
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
    let unchanged = |model: &DesignModel| -> BTreeMap<String, Value> {
        model
            .source_shards
            .iter()
            .filter(|s| s.shard_id != model.objects[&unit].source_shard_id)
            .map(|s| {
                (
                    s.relative_path.clone(),
                    model
                        .materialized_source_shard_value_by_relative_path(&s.relative_path)
                        .unwrap(),
                )
            })
            .collect()
    };
    let before = unchanged(&model);
    let unused = Uuid::new_v4();
    // One existing canonical AddPin transaction creates an actually unused pin.
    let add=(find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext{model:&model,project_root:&root},provenance(),
        json!({"kind":"units","object_id":unit,"pool_path":"pool","fields":{
            "pin_additions":{unused.to_string():{"name":"Unused sense","electrical_type":"Input"}}}})).unwrap();
    commit_prepared(&mut model, &root, add).unwrap();
    let with_unused = raw(&model, unit);
    assert!(model.objects.contains_key(&unused));
    assert_eq!(unchanged(&model), before);
    let held = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let remove=(find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext{model:&model,project_root:&root},provenance(),
        json!({"kind":"units","object_id":unit,"pool_path":"pool","fields":{"pin_removals":[unused]}})).unwrap();
    commit_prepared(&mut model, &root, remove).unwrap();
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
                with_unused.clone()
            } else {
                original.clone()
            }
        );
        assert_eq!(model.objects.contains_key(&unused), phase == 1);
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
}
