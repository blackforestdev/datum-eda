//! Source-default changes retain nonempty physical users and their authenticated pins.
use super::*;
use crate::api::native_write::library::{
    self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec,
};
fn edit(
    model: &DesignModel,
    root: &Path,
    part: Uuid,
    value: Value,
) -> Result<crate::api::native_write::PreparedWrite, crate::error::EngineError> {
    (find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        json!({"kind":"parts","object_id":part,"pool_path":"pool","fields":{"default_footprint":value}}),
    )
}
#[test]
fn saved_part_default_footprint_preserves_eight_pads_connected_identity_history_and_cold() {
    use crate::substrate::component_pad_source as pins;
    let (root, mut model, symbol, ci, part, package, map, _request) =
        physical_fixture::eight_pad_fixture("part-default-frozen-eight");
    let (net, _, members) =
        super::map_tests::connected(&mut model, &root, symbol, package, ci, map);
    assert!(!members.is_empty());
    let old_foot: Uuid = "60000000-0000-4000-8000-000000000001".parse().unwrap();
    let next_foot = Uuid::new_v4();
    let mut next = raw(&model, old_foot);
    next["uuid"] = json!(next_foot);
    next["name"] = json!("Different current source land");
    let mut pad_ids = BTreeMap::new();
    next["pads"] = Value::Object(
        next["pads"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(id, raw)| {
                let new = Uuid::new_v4();
                pad_ids.insert(id.clone(), new);
                let mut raw = raw.clone();
                raw["uuid"] = json!(new);
                raw["position"]["x"] = json!(raw["position"]["x"].as_i64().unwrap() + 100_000);
                (new.to_string(), raw)
            })
            .collect(),
    );
    next.as_object_mut().unwrap().remove("ipc_basis");
    next["standards_basis"] = Value::Null;
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![Spec::Create {
            target: Target::new("pool", "footprints", next_foot),
            object: next,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let original = raw(&model, part);
    let revision = model.objects[&part].object_revision;
    let board = model
        .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
        .unwrap();
    let board_bytes = std::fs::read(root.join("board/board.json")).unwrap();
    let ci_raw = raw(&model, ci);
    let ci_summary = model.component_instances[&ci].clone();
    let old_map = raw(&model, map);
    let identities = model.electrical_identities.clone();
    let partitions = crate::substrate::electrical_test_partitions(&model, &[]).unwrap();
    let projection = |model: &DesignModel| {
        let p = pins::component_pad_projection(model, &board).unwrap();
        assert!(p.unavailable.is_empty(), "{:?}", p.unavailable);
        assert_eq!(p.query_pads.len(), 8);
        json!({"pads":p.pads,"queries":p.query_pads.iter().map(|(id,p)|(*id,serde_json::to_value(p.terminal()).unwrap())).collect::<BTreeMap<_,_>>()})
    };
    let before = projection(&model);
    let verify = |model: &DesignModel| {
        let snapshot = ElectricalSelectionSnapshot::capture(model).unwrap();
        assert_eq!(
            snapshot.global_net(snapshot.revision(), net).unwrap().owned,
            members
        );
        assert_eq!(
            std::fs::read(root.join("board/board.json")).unwrap(),
            board_bytes
        );
        assert_eq!(raw(model, ci), ci_raw);
        assert_eq!(model.component_instances[&ci], ci_summary);
        assert_eq!(raw(model, map), old_map);
        assert_eq!(model.electrical_identities, identities);
        assert_eq!(
            crate::substrate::electrical_test_partitions(model, &[]).unwrap(),
            partitions
        );
        assert_eq!(
            projection(model),
            before,
            "old eight Pad geometry/process/mapping query projections are frozen"
        );
        assert_eq!(
            model
                .materialized_pool_object_revision(part, "parts", revision)
                .unwrap(),
            original
        );
        assert!(
            model.component_instances[&ci]
                .placed_package_refs
                .contains(&package)
        );
    };
    for value in [json!(next_foot), Value::Null] {
        let old = raw(&model, part);
        let p = edit(&model, &root, part, value.clone()).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let expected = raw(&model, part);
        if value.is_null() {
            assert!(expected.get("default_footprint").is_none());
        } else {
            assert_eq!(expected["default_footprint"], value);
        }
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
    // A footprint-less default Map inherits its Part default. Clearing that
    // source must validate Package pads rather than retain an implicit old land.
    let fallback_map = Uuid::new_v4();
    let mut map_raw = old_map.clone();
    map_raw["uuid"] = json!(fallback_map);
    map_raw["footprint"] = Value::Null;
    map_raw["mappings"] = Value::Object(
        map_raw["mappings"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(old, cell)| (pad_ids[old].to_string(), cell.clone()))
            .collect(),
    );
    let mut part_raw = raw(&model, part);
    part_raw["default_footprint"] = json!(next_foot);
    part_raw["default_pin_pad_map"] = json!(fallback_map);
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![
            Spec::Create {
                target: Target::new("pool", "pin_pad_maps", fallback_map),
                object: map_raw.clone(),
            },
            Spec::Set {
                target: Target::new("pool", "parts", part),
                object: part_raw.clone(),
            },
        ],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    assert!(
        edit(&model, &root, part, Value::Null).is_err(),
        "old Footprint pad IDs cannot silently become Package pad identities"
    );
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    assert_eq!(raw(&model, part), part_raw);
    assert_eq!(raw(&model, fallback_map), map_raw);
    // Map validation consumes only its fixed mapped Gate's Unit, and guards it.
    let entity: Uuid = serde_json::from_value(part_raw["entity"].clone()).unwrap();
    let mut entity_raw = raw(&model, entity);
    let mapped_gate = map_raw["mappings"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()["gate"]
        .as_str()
        .unwrap();
    let gate = entity_raw["gates"][mapped_gate].clone();
    let unit: Uuid = serde_json::from_value(gate["unit"].clone()).unwrap();
    let p = edit(&model, &root, part, json!(next_foot)).unwrap();
    assert!(p.batch.operations.iter().any(
        |op| matches!(op, Operation::GuardObjectRevision { object_id, .. } if *object_id == unit)
    ));
    // Omitted optional Footprint pads means empty, not unknown membership.
    let complete = raw(&model, next_foot);
    let mut empty = complete.clone();
    empty.as_object_mut().unwrap().remove("pads");
    for object in [empty, complete] {
        let omitted = object.get("pads").is_none();
        let p = library::build_pool_library_write(
            &model,
            provenance(),
            Some("pool"),
            vec![Spec::Set {
                target: Target::new("pool", "footprints", next_foot),
                object,
            }],
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let bytes = std::fs::read(&journal).unwrap();
        let result = edit(&model, &root, part, json!(next_foot));
        assert_eq!(result.is_err(), omitted);
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
        assert_eq!(raw(&model, part), part_raw);
    }
    // A typed Map can coalesce Pad-key aliases. Both raw rows still consume
    // their Gate/Unit dependencies, including a row with a missing Unit.
    let bad_gate = Uuid::new_v4();
    let mut bad_entity = entity_raw.clone();
    let mut bad_gate_raw = gate.clone();
    bad_gate_raw["uuid"] = json!(bad_gate);
    bad_gate_raw["unit"] = json!(Uuid::new_v4());
    bad_entity["gates"][bad_gate.to_string()] = bad_gate_raw;
    let mut aliases = map_raw.clone();
    let pad_key = aliases["mappings"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone();
    let good_row = aliases["mappings"][&pad_key].clone();
    aliases["mappings"][&pad_key]["gate"] = json!(bad_gate);
    let pad_id: Uuid = pad_key.parse().unwrap();
    aliases["mappings"][pad_id.simple().to_string()] = good_row.clone();
    let coalesced: crate::pool::PinPadMap = serde_json::from_value(aliases.clone()).unwrap();
    assert_eq!(json!(coalesced.mappings[&pad_id].gate), good_row["gate"]);
    for (entity_object, map_object, invalid) in [
        (bad_entity, aliases, true),
        (entity_raw.clone(), map_raw.clone(), false),
    ] {
        let p = library::build_pool_library_write(
            &model,
            provenance(),
            Some("pool"),
            vec![
                Spec::Set {
                    target: Target::new("pool", "entities", entity),
                    object: entity_object,
                },
                Spec::Set {
                    target: Target::new("pool", "pin_pad_maps", fallback_map),
                    object: map_object,
                },
            ],
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let bytes = std::fs::read(&journal).unwrap();
        assert_eq!(
            edit(&model, &root, part, json!(next_foot)).is_err(),
            invalid
        );
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
        assert_eq!(raw(&model, part), part_raw);
    }
    // Raw aliases must not be coalesced into an apparently unique typed Gate.
    let id: Uuid = mapped_gate.parse().unwrap();
    entity_raw["gates"][id.simple().to_string()] = gate;
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![Spec::Set {
            target: Target::new("pool", "entities", entity),
            object: entity_raw,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    assert!(edit(&model, &root, part, json!(next_foot)).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn saved_part_default_footprint_accepts_authenticated_frozen_only_physical_user() {
    use crate::api::native_write::board_components::{
        BoardPackagePlacement, build_place_board_package,
        pad_materialization::{PadLandPatternSource, build_materialize_component_pad_sources},
    };
    use crate::substrate::component_pad_source::{
        ComponentPadMaterializationStatus, component_pad_projection,
    };
    let (root, mut model, _, _, part, _, _, mut request) =
        physical_fixture::eight_pad_fixture("part-default-frozen-no-ci");
    request.placement.uuid = Uuid::new_v4();
    request.placement.reference = "U2".into();
    request.component_instance = None;
    request.pin_pad_map = None;
    let placed = request.placement.uuid;
    let p = build_place_board_package(
        &model,
        provenance(),
        &BoardPackagePlacement {
            package: request.placement.clone(),
            materialized: json!({}),
        },
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let p = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    assert!(!model.component_instances.values().any(|ci| ci.authority
        == crate::substrate::ComponentInstanceAuthority::Authored
        && ci.placed_package_refs.contains(&placed)));
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let pads = board["component_pads"][placed.to_string()]
        .as_array()
        .unwrap();
    assert_eq!(pads.len(), 8);
    assert!(
        pads.iter()
            .all(|p| p["source_binding"]["component_instance"].is_null())
    );
    let board_bytes = std::fs::read(root.join("board/board.json")).unwrap();
    let cis = model.component_instances.clone();
    let old = raw(&model, part);
    let old_revision = model.objects[&part].object_revision;
    let PadLandPatternSource::Footprint(footprint) = request.land_pattern else {
        panic!("Footprint fixture")
    };
    for value in [json!(footprint), Value::Null] {
        let p = edit(&model, &root, part, value).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(
            std::fs::read(root.join("board/board.json")).unwrap(),
            board_bytes
        );
        assert_eq!(model.component_instances, cis);
        assert_eq!(
            model
                .materialized_pool_object_revision(part, "parts", old_revision)
                .unwrap(),
            old
        );
        let projection = component_pad_projection(&model, &board).unwrap();
        for pad in pads {
            let id = serde_json::from_value(pad["uuid"].clone()).unwrap();
            assert_eq!(
                projection.statuses.get(&id),
                Some(&ComponentPadMaterializationStatus::Current)
            );
            assert!(projection.query_pads.contains_key(&id));
            assert!(!projection.unavailable.contains_key(&id));
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
