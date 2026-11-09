//! Correct future defaults without rebinding a connected materialized component.
use super::*;
use crate::api::native_write::library::{
    self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec,
};
use crate::substrate::component_pad_source as pins;
fn edit(
    model: &DesignModel,
    root: &Path,
    part: Uuid,
    fields: Value,
) -> Result<crate::api::native_write::PreparedWrite, crate::error::EngineError> {
    (find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        json!({"kind":"parts","object_id":part,"pool_path":"pool","fields":fields}),
    )
}
#[test]
fn saved_default_map_and_composed_footprint_preserve_connected_eight_pads_and_cold_history() {
    let (root, mut model, symbol, ci, part, placed, a, _) =
        physical_fixture::eight_pad_fixture("part-default-map-frozen-eight");
    let (net, _, members) = super::map_tests::connected(&mut model, &root, symbol, placed, ci, a);
    assert!(!members.is_empty());
    let (b, c, foot_b) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let map_a = raw(&model, a);
    let foot_a: Uuid = serde_json::from_value(map_a["footprint"].clone()).unwrap();
    let mut next = raw(&model, foot_a);
    next["uuid"] = json!(foot_b);
    next["name"] = json!("Composed current land");
    next.as_object_mut().unwrap().remove("ipc_basis");
    next["standards_basis"] = Value::Null;
    let mut pad_ids = BTreeMap::new();
    next["pads"] = Value::Object(
        next["pads"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(id, value)| {
                let new = Uuid::new_v4();
                pad_ids.insert(id.clone(), new);
                let mut value = value.clone();
                value["uuid"] = json!(new);
                value["position"]["x"] = json!(value["position"]["x"].as_i64().unwrap() + 100000);
                (new.to_string(), value)
            })
            .collect(),
    );
    let mut map_b = map_a.clone();
    map_b["uuid"] = json!(b);
    let keys: Vec<_> = map_b["mappings"]
        .as_object()
        .unwrap()
        .keys()
        .take(2)
        .cloned()
        .collect();
    assert_eq!(keys.len(), 2);
    let first = map_b["mappings"][&keys[0]].clone();
    let second = map_b["mappings"][&keys[1]].clone();
    map_b["mappings"][&keys[0]] = second;
    map_b["mappings"][&keys[1]] = first;
    let mut map_c = map_a.clone();
    map_c["uuid"] = json!(c);
    map_c["footprint"] = json!(foot_b);
    map_c["mappings"] = Value::Object(
        map_c["mappings"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, row)| (pad_ids[key].to_string(), row.clone()))
            .collect(),
    );
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![
            Spec::Create {
                target: Target::new("pool", "footprints", foot_b),
                object: next,
            },
            Spec::Create {
                target: Target::new("pool", "pin_pad_maps", b),
                object: map_b.clone(),
            },
            Spec::Create {
                target: Target::new("pool", "pin_pad_maps", c),
                object: map_c.clone(),
            },
        ],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let original = raw(&model, part);
    let revision = model.objects[&part].object_revision;
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let board_bytes = std::fs::read(root.join("board/board.json")).unwrap();
    let ci_raw = raw(&model, ci);
    let ci_summary = model.component_instances[&ci].clone();
    let identities = model.electrical_identities.clone();
    let partitions = crate::substrate::electrical_test_partitions(&model, &[]).unwrap();
    let projected = |model: &DesignModel| {
        let p = pins::component_pad_projection(model, &board).unwrap();
        assert!(p.unavailable.is_empty(), "{:?}", p.unavailable);
        assert_eq!(p.query_pads.len(), 8);
        assert!(
            p.statuses
                .values()
                .all(|s| *s == pins::ComponentPadMaterializationStatus::Current)
        );
        json!({"pads":p.pads,"queries":p.query_pads.iter().map(|(id,p)|(*id,serde_json::to_value(p.terminal()).unwrap())).collect::<BTreeMap<_,_>>()})
    };
    let before = projected(&model);
    let verify = |model: &DesignModel| {
        assert_eq!(
            std::fs::read(root.join("board/board.json")).unwrap(),
            board_bytes
        );
        assert_eq!(raw(model, ci), ci_raw);
        assert_eq!(model.component_instances[&ci], ci_summary);
        assert_eq!(raw(model, a), map_a);
        assert_eq!(raw(model, b), map_b);
        assert_eq!(raw(model, c), map_c);
        assert_eq!(model.electrical_identities, identities);
        assert_eq!(
            crate::substrate::electrical_test_partitions(model, &[]).unwrap(),
            partitions
        );
        assert_eq!(
            projected(model),
            before,
            "old geometry/process/mapping/source-qualified queries frozen"
        );
        let snap = ElectricalSelectionSnapshot::capture(model).unwrap();
        assert_eq!(
            snap.global_net(snap.revision(), net).unwrap().owned,
            members
        );
        assert_eq!(
            model
                .materialized_pool_object_revision(part, "parts", revision)
                .unwrap(),
            original
        );
    };
    for fields in [
        json!({"default_pin_pad_map":a}),
        json!({"default_pin_pad_map":b}),
        json!({"default_pin_pad_map":null}),
        json!({"default_pin_pad_map":c,"default_footprint":foot_b}),
    ] {
        let old = raw(&model, part);
        let mut expected = old.clone();
        for (k, v) in fields.as_object().unwrap() {
            if v.is_null() {
                expected.as_object_mut().unwrap().remove(k);
            } else {
                expected[k] = v.clone();
            }
        }
        let p = edit(&model, &root, part, fields).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        assert_eq!(raw(&model, part), expected);
        verify(&model);
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        assert_eq!(raw(&model, part), old);
        verify(&model);
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&model, part), expected);
        verify(&model);
    }
    // A Map change cannot smuggle edits to its payload or fixed Part links.
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    for invalid in [
        json!({"default_pin_pad_map":a}),
        json!({"default_pin_pad_map":b,"default_footprint":foot_b}),
        json!({"default_pin_pad_map":null,"mappings":{}}),
    ] {
        assert!(edit(&model, &root, part, invalid).is_err());
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
        verify(&model);
    }
    std::fs::remove_dir_all(root).unwrap();
}
