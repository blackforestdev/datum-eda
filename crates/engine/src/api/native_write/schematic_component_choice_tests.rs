//! Complete component selection over the same real canonical library graph.
use super::*;

fn replace(f: &mut F, kind: &str, id: Uuid, value: Value) {
    use native::library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec};
    let prepared = native::library::build_pool_library_write(
        &f.model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", kind, id),
            object: value,
        }],
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
}

#[test]
fn complete_choice_is_read_only_captures_all_sources_and_guards_placement() {
    let f = fixture();
    let before = journal(&f);
    let choice = CompletePartSelection::capture(&f.model, f.other, f.b, f.symbol).unwrap();
    assert_eq!(choice.value, "Second value");
    assert_eq!(choice.package_name, "Body");
    assert_eq!(choice.footprint_name, "Land");
    assert_eq!(choice.source_revisions.len(), 8);
    choice.verify(&f.model).unwrap();
    let mut request = params(&f, Some(&choice.part));
    request["component"] = serde_json::to_value(&choice).unwrap();
    request["value"] = json!(choice.value);
    let prepared = prepare(&f, "datum.schematic.place_symbol", request.clone()).unwrap();
    for (id, revision) in &choice.source_revisions {
        assert!(prepared.batch.operations.iter().any(|op| matches!(op,
            Operation::GuardObjectRevision { object_id, expected_object_revision }
            if object_id == id && expected_object_revision.0 == *revision)));
    }
    let mut stale = choice.clone();
    stale.footprint = Uuid::new_v4();
    assert!(stale.verify(&f.model).is_err());
    request["value"] = json!("An unrelated component value");
    assert!(prepare(&f, "datum.schematic.place_symbol", request).is_err());
    assert_eq!(journal(&f), before);
}

#[test]
fn incomplete_mapping_and_changed_source_refuse_without_selection_writes() {
    let mut f = fixture();
    assert!(CompletePartSelection::capture(&f.model, f.part, f.a, f.symbol).is_err());
    let choice = CompletePartSelection::capture(&f.model, f.other, f.b, f.symbol).unwrap();
    let mut map = materialized_pool_object(&f.model, choice.pin_pad_map, "pin_pad_maps").unwrap();
    let key = map["mappings"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone();
    map["mappings"].as_object_mut().unwrap().remove(&key);
    replace(&mut f, "pin_pad_maps", choice.pin_pad_map, map);
    let before = journal(&f);
    assert!(CompletePartSelection::capture(&f.model, f.other, f.b, f.symbol).is_err());
    assert!(choice.verify(&f.model).is_err());
    assert_eq!(journal(&f), before);
}

#[test]
fn changing_only_padstack_invalidates_retained_component_choice() {
    let mut f = fixture();
    let choice = CompletePartSelection::capture(&f.model, f.other, f.b, f.symbol).unwrap();
    let footprint: crate::pool::Footprint = serde_json::from_value(
        materialized_pool_object(&f.model, choice.footprint, "footprints").unwrap(),
    )
    .unwrap();
    let stack = footprint.pads.values().next().unwrap().padstack;
    let mut request = params(&f, Some(&choice.part));
    request["component"] = serde_json::to_value(&choice).unwrap();
    request["value"] = json!(choice.value);
    let prepared = prepare(&f, "datum.schematic.place_symbol", request.clone()).unwrap();
    let mut value = materialized_pool_object(&f.model, stack, "padstacks").unwrap();
    value["name"] = json!("Revised land source");
    replace(&mut f, "padstacks", stack, value);
    let before = journal(&f);
    assert!(choice.verify(&f.model).is_err());
    assert!(prepare(&f, "datum.schematic.place_symbol", request).is_err());
    assert!(commit_prepared(&mut f.model, &f.root, prepared).is_err());
    CompletePartSelection::capture(&f.model, f.other, f.b, f.symbol).unwrap();
    assert_eq!(journal(&f), before);
}

#[test]
fn changing_another_gates_symbol_invalidates_the_complete_component() {
    let mut f = fixture();
    let sibling = Uuid::new_v4();
    let mut body = materialized_pool_object(&f.model, f.symbol, "symbols").unwrap();
    body["uuid"] = json!(sibling);
    body["name"] = json!("Second gate body");
    use native::library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec};
    let prepared = native::library::build_pool_library_write(
        &f.model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "symbols", sibling),
            object: body.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let mut entity = materialized_pool_object(&f.model, f.entity, "entities").unwrap();
    entity["gates"][f.b.to_string()]["symbol"] = json!(sibling);
    let entity_id = f.entity;
    replace(&mut f, "entities", entity_id, entity);
    let choice = CompletePartSelection::capture(&f.model, f.other, f.a, f.symbol).unwrap();
    assert!(choice.source_revisions.contains_key(&sibling));
    body["name"] = json!("Revised second gate body");
    replace(&mut f, "symbols", sibling, body);
    let before = journal(&f);
    assert!(choice.verify(&f.model).is_err());
    CompletePartSelection::capture(&f.model, f.other, f.a, f.symbol).unwrap();
    assert_eq!(journal(&f), before);
}
