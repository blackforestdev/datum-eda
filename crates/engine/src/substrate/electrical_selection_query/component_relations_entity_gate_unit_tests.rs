//! Prospective Unit changes must retain actual connected map authority.
use super::*;
use crate::api::native_write::library::{
    self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec,
};
use crate::substrate::{NetCorrespondenceStatus, net_correspondence_status};
fn set(model: &mut DesignModel, root: &Path, kind: &str, id: Uuid, object: Value) {
    let p = library::build_pool_library_write(
        model,
        provenance(),
        Some("pool"),
        vec![Spec::Set {
            target: Target::new("pool", kind, id),
            object,
        }],
    )
    .unwrap();
    commit_prepared(model, root, p).unwrap();
}
fn edit(
    model: &DesignModel,
    root: &Path,
    entity: Uuid,
    gate: Uuid,
    unit: Uuid,
    symbol: Uuid,
) -> Result<crate::api::native_write::PreparedWrite, crate::error::EngineError> {
    (find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        json!({"kind":"entities","object_id":entity,"pool_path":"pool","fields":{"gate_units":{gate.to_string():unit},"gate_symbols":{gate.to_string():symbol}}}),
    )
}
#[test]
fn saved_unit_pair_refuses_retained_default_nondefault_and_legacy_maps_without_changing_complete() {
    let (root, mut model, symbol, ci, part, placed, map, _) =
        physical_fixture::eight_pad_fixture("entity-unit-map-refusal");
    let (net, _, members) = map_tests::connected(&mut model, &root, symbol, placed, ci, map);
    let entity: Uuid = serde_json::from_value(raw(&model, part)["entity"].clone()).unwrap();
    let original = raw(&model, entity);
    let (gate, old) = original["gates"]
        .as_object()
        .unwrap()
        .iter()
        .next()
        .unwrap();
    let gate: Uuid = gate.parse().unwrap();
    let old_unit: Uuid = serde_json::from_value(old["unit"].clone()).unwrap();
    let old_symbol: Uuid = serde_json::from_value(old["symbol"].clone()).unwrap();
    let (unit, body) = (Uuid::new_v4(), Uuid::new_v4());
    let mut logical = raw(&model, old_unit);
    logical["uuid"] = json!(unit);
    logical["name"] = json!("Different pins");
    let mut pins = serde_json::Map::new();
    let mut ids = BTreeMap::new();
    for (key, row) in logical["pins"].as_object().unwrap() {
        let id = Uuid::new_v4();
        let mut row = row.clone();
        row["uuid"] = json!(id);
        ids.insert(key.clone(), id);
        pins.insert(id.to_string(), row);
    }
    logical["pins"] = json!(pins);
    let mut graphic = raw(&model, old_symbol);
    graphic["uuid"] = json!(body);
    graphic["unit"] = json!(unit);
    graphic["name"] = json!("Different Unit body");
    for anchor in graphic["pin_anchors"].as_array_mut().unwrap() {
        anchor["pin"] = json!(ids[anchor["pin"].as_str().unwrap()]);
    }
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![
            Spec::Create {
                target: Target::new("pool", "units", unit),
                object: logical,
            },
            Spec::Create {
                target: Target::new("pool", "symbols", body),
                object: graphic,
            },
        ],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    for phase in 0..2 {
        let mut p = raw(&model, part);
        if phase == 1 {
            p["default_pin_pad_map"] = json!(map);
        }
        if phase > 0 {
            set(&mut model, &root, "parts", part, p);
        }
        let unchanged: BTreeMap<_, _> = model
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
        let error = edit(&model, &root, entity, gate, unit, body).unwrap_err();
        assert!(error.to_string().contains("pin"), "{error}");
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
        assert_eq!(model.model_revision, revision);
        for (path, raw) in &unchanged {
            assert_eq!(
                model
                    .materialized_source_shard_value_by_relative_path(path)
                    .unwrap(),
                *raw
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
        assert_eq!(
            ElectricalSelectionSnapshot::capture(&model)
                .unwrap()
                .global_net(&model.model_revision, net)
                .unwrap()
                .owned,
            members
        );
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&model, entity), original);
    }
    // A map on an unchanged gate must not block another existing gate's correction.
    let extra = Uuid::new_v4();
    let mut extra_entity = original.clone();
    let mut row = extra_entity["gates"][gate.to_string()].clone();
    row["uuid"] = json!(extra);
    row["name"] = json!("Unmapped gate");
    extra_entity["gates"][extra.to_string()] = row;
    set(&mut model, &root, "entities", entity, extra_entity);
    let legacy = Uuid::new_v4();
    let mut legacy_part = raw(&model, part);
    legacy_part["uuid"] = json!(legacy);
    legacy_part["mpn"] = json!("Legacy only fixture");
    legacy_part["default_pin_pad_map"] = Value::Null;
    let mut legacy_rows = raw(&model, map)["mappings"].clone();
    for row in legacy_rows.as_object_mut().unwrap().values_mut() {
        row["gate"] = json!(extra);
    }
    legacy_part["pad_map"] = legacy_rows;
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![Spec::Create {
            target: Target::new("pool", "parts", legacy),
            object: legacy_part.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    let unchanged: BTreeMap<_, _> = model
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
    let error = edit(&model, &root, entity, extra, unit, body).unwrap_err();
    assert!(
        error.to_string().contains("pad_map"),
        "legacy check must independently refuse: {error}"
    );
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    for (path, raw) in &unchanged {
        assert_eq!(
            model
                .materialized_source_shard_value_by_relative_path(path)
                .unwrap(),
            *raw
        );
    }
    assert_eq!(
        net_correspondence_status(&model, net),
        NetCorrespondenceStatus::Complete
    );
    legacy_part["pad_map"] = json!({});
    set(&mut model, &root, "parts", legacy, legacy_part);
    let p = edit(&model, &root, entity, extra, unit, body)
        .expect("unchanged mapped gate stays compatible");
    commit_prepared(&mut model, &root, p).unwrap();
    assert_eq!(
        net_correspondence_status(&model, net),
        NetCorrespondenceStatus::Complete
    );
    assert_eq!(
        raw(&model, entity)["gates"][gate.to_string()],
        original["gates"][gate.to_string()]
    );
    std::fs::remove_dir_all(root).unwrap();
}
