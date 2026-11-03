use super::*;
use crate::{
    api::native_write::{
        commit_prepared,
        electrical::{binding_fixture, topology_tests},
        registry::find_native_write_verb,
        schematic_connectivity::*,
        schematic_sheets::*,
        schematic_symbols::build_place_schematic_symbol,
    },
    ir::geometry::Point,
    schematic::{
        HiddenPowerBehavior, LabelKind, NetLabel, NoConnectMarker, PinElectricalType, PlacedSymbol,
        SchematicWire, SheetDefinition, SheetInstance, SymbolDisplayMode, SymbolField, SymbolPin,
    },
    substrate::{CommitSource, DesignModel, Operation, ProjectResolver, SchematicMarkerKind},
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "edit schematic selection")
}
fn source(model: &DesignModel) -> Schematic {
    ElectricalSelectionSnapshot::capture(model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .clone()
}
fn write(
    model: &DesignModel,
    root: &Path,
    verb: &str,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
}
fn fixture(name: &str) -> (PathBuf, DesignModel, Uuid, Uuid, PlacedSymbol) {
    let (root, mut model, schematic, sheet) = topology_tests::fixture(name);
    let symbol = PlacedSymbol {
        uuid: Uuid::new_v4(),
        part: None,
        entity: None,
        gate: None,
        lib_id: None,
        reference: "R1".into(),
        value: "10k".into(),
        fields: vec![SymbolField {
            uuid: Uuid::new_v4(),
            key: "tolerance".into(),
            value: "1%".into(),
            position: Some(Point::new(100, 100)),
            visible: true,
        }],
        pins: vec![SymbolPin {
            uuid: Uuid::new_v4(),
            library_pin: None,
            number: "1".into(),
            name: "~".into(),
            electrical_type: PinElectricalType::Passive,
            position: Point::zero(),
        }],
        position: Point::new(0, 100),
        rotation: 90,
        mirrored: true,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![],
        hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
    };
    let mut prepared =
        build_place_schematic_symbol(&model, provenance(), sheet, &symbol, None).unwrap();
    prepared.batch =
        crate::substrate::bootstrap_empty_native_topology(&model, prepared.batch).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    (root, model, schematic, sheet, symbol)
}

fn roundtrip(model: &mut DesignModel, root: &Path, prepared: PreparedWrite) -> Schematic {
    let before = source(model);
    commit_prepared(model, root, prepared).unwrap();
    let after = source(model);
    let identities = model.electrical_identities.clone();
    let mut reopened = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(source(&reopened), after);
    assert_eq!(reopened.electrical_identities, identities);
    reopened
        .commit_journal_undo(root, provenance().into())
        .unwrap();
    let mut undone = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(source(&undone), before, "one undo restores the whole edit");
    undone
        .commit_journal_redo(root, provenance().into())
        .unwrap();
    *model = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(source(model), after);
    assert_eq!(model.electrical_identities, identities);
    after
}

fn persisted_bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(path: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(&path, result);
            } else {
                result.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut result = BTreeMap::new();
    collect(root, &mut result);
    result
}

#[test]
fn selected_symbol_reference_and_value_are_one_atomic_geometry_preserving_edit() {
    let (root, mut model, _, sheet, symbol) = fixture("selected_symbol_fields");
    let before = persisted_bytes(&root);
    let prepared = write(
        &model,
        &root,
        "datum.schematic.set_symbol_reference",
        serde_json::json!({"symbol":symbol.uuid,"instance_path":[],"reference":"R2","value":"4k7"}),
    )
    .unwrap();
    assert_eq!(
        persisted_bytes(&root),
        before,
        "prepare/cancel does not persist"
    );
    assert_eq!(
        prepared
            .batch
            .operations
            .iter()
            .filter(|op| matches!(op, Operation::SetSchematicSymbol { .. }))
            .count(),
        1
    );
    let after = roundtrip(&mut model, &root, prepared);
    let mut expected = symbol.clone();
    expected.reference = "R2".into();
    expected.value = "4k7".into();
    assert_eq!(after.sheets[&sheet].symbols[&symbol.uuid], expected);
    let prepared = write(
        &model,
        &root,
        "datum.schematic.set_symbol_value",
        serde_json::json!({"symbol":symbol.uuid,"instance_path":[],"value":""}),
    )
    .unwrap();
    let after = roundtrip(&mut model, &root, prepared);
    assert_eq!(after.sheets[&sheet].symbols[&symbol.uuid].reference, "R2");
    assert_eq!(after.sheets[&sheet].symbols[&symbol.uuid].value, "");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn selected_wire_power_label_and_symbol_deletion_roundtrip_without_orphan_markers() {
    let (root, mut model, _, sheet, symbol) = fixture("selected_object_delete");
    let wire = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::zero(),
        to: Point::new(1000, 0),
    };
    let prepared = build_create_schematic_wire(&model, provenance(), sheet, &wire).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: "GND".into(),
        kind: LabelKind::Power,
        position: wire.to,
    };
    let prepared = build_create_schematic_label(&model, provenance(), sheet, &label).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    for (verb, key, id) in [
        ("datum.schematic.delete_wire", "wire", wire.uuid),
        ("datum.schematic.delete_label", "label", label.uuid),
    ] {
        let mut params = serde_json::json!({"instance_path":[]});
        params[key] = serde_json::json!(id);
        let prepared = write(&model, &root, verb, params).unwrap();
        let after = roundtrip(&mut model, &root, prepared);
        if key == "wire" {
            assert!(!after.sheets[&sheet].wires.contains_key(&id));
        } else {
            assert!(!after.sheets[&sheet].labels.contains_key(&id));
        }
    }
    let mut other = symbol.clone();
    other.uuid = Uuid::new_v4();
    other.reference = "R2".into();
    other.pins[0].uuid = Uuid::new_v4();
    other.fields[0].uuid = Uuid::new_v4();
    let prepared = build_place_schematic_symbol(&model, provenance(), sheet, &other, None).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    // Coincident pins do not confer ownership of the other symbol's marker.
    let foreign_marker = NoConnectMarker {
        uuid: Uuid::new_v4(),
        symbol: other.uuid,
        pin: other.pins[0].uuid,
        position: Point::zero(),
    };
    let prepared = build_place_schematic_marker(
        &model,
        provenance(),
        sheet,
        foreign_marker.uuid,
        SchematicMarkerKind::NoConnect,
        serde_json::to_value(&foreign_marker).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let marker = NoConnectMarker {
        uuid: Uuid::new_v4(),
        symbol: symbol.uuid,
        pin: symbol.pins[0].uuid,
        position: Point::zero(),
    };
    let prepared = build_place_schematic_marker(
        &model,
        provenance(),
        sheet,
        marker.uuid,
        SchematicMarkerKind::NoConnect,
        serde_json::to_value(&marker).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let prepared = write(
        &model,
        &root,
        "datum.schematic.delete_symbol",
        serde_json::json!({"symbol":symbol.uuid,"instance_path":[]}),
    )
    .unwrap();
    let after = roundtrip(&mut model, &root, prepared);
    assert!(!after.sheets[&sheet].symbols.contains_key(&symbol.uuid));
    assert_eq!(after.sheets[&sheet].symbols[&other.uuid], other);
    assert!(!after.sheets[&sheet].noconnects.contains_key(&marker.uuid));
    assert_eq!(after.sheets[&sheet].noconnects.len(), 1);
    assert_eq!(
        after.sheets[&sheet].noconnects[&foreign_marker.uuid],
        foreign_marker
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_duplicate_stale_and_unknown_symbol_fields_cannot_write() {
    let (root, mut model, _, sheet, symbol) = fixture("selected_symbol_refusals");
    let mut other = symbol.clone();
    other.uuid = Uuid::new_v4();
    other.reference = "R2".into();
    other.pins[0].uuid = Uuid::new_v4();
    other.fields[0].uuid = Uuid::new_v4();
    let prepared = build_place_schematic_symbol(&model, provenance(), sheet, &other, None).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let before = persisted_bytes(&root);
    let base =
        serde_json::json!({"symbol":symbol.uuid,"instance_path":[],"reference":"R3","value":"1k"});
    for (field, value) in [
        ("reference", serde_json::json!("")),
        ("reference", serde_json::json!("R 3")),
        ("reference", serde_json::json!("R2")),
        ("value", serde_json::json!("bad\nvalue")),
        ("instance_path", serde_json::json!([Uuid::new_v4()])),
        ("symbol", serde_json::json!(Uuid::new_v4())),
        ("rotation", serde_json::json!(180)),
    ] {
        let mut params = base.clone();
        params[field] = value;
        assert!(
            write(
                &model,
                &root,
                "datum.schematic.set_symbol_reference",
                params
            )
            .is_err()
        );
    }
    assert!(
        write(
            &model,
            &root,
            "datum.schematic.set_symbol_value",
            serde_json::json!({"symbol":symbol.uuid,"instance_path":[],"reference":"R4"})
        )
        .is_err()
    );
    assert_eq!(persisted_bytes(&root), before);
    let stale = write(
        &model,
        &root,
        "datum.schematic.delete_symbol",
        serde_json::json!({"symbol":symbol.uuid,"instance_path":[]}),
    )
    .unwrap();
    let changed = write(
        &model,
        &root,
        "datum.schematic.set_symbol_value",
        serde_json::json!({"symbol":symbol.uuid,"instance_path":[],"value":"2k"}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, changed).unwrap();
    let before = persisted_bytes(&root);
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(persisted_bytes(&root), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn physical_component_instance_binding_blocks_symbol_edit_and_delete() {
    let f = binding_fixture::fixture("selected_bound_refusal");
    assert!(
        f.model
            .component_instances
            .values()
            .any(|instance| instance.placed_symbol_refs.contains(&f.symbol.uuid))
    );
    let before = persisted_bytes(&f.root);
    for (verb, params) in [
        (
            "datum.schematic.delete_symbol",
            serde_json::json!({"symbol":f.symbol.uuid,"instance_path":[]}),
        ),
        (
            "datum.schematic.set_symbol_reference",
            serde_json::json!({"symbol":f.symbol.uuid,"instance_path":[],"reference":"U9","value":"new"}),
        ),
        (
            "datum.schematic.set_symbol_value",
            serde_json::json!({"symbol":f.symbol.uuid,"instance_path":[],"value":"new"}),
        ),
    ] {
        let reason = if verb == "datum.schematic.delete_symbol" {
            "linked to a component"
        } else {
            "schematic-only"
        };
        assert!(
            write(&f.model, &f.root, verb, params)
                .unwrap_err()
                .to_string()
                .contains(reason)
        );
    }
    assert_eq!(persisted_bytes(&f.root), before);
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn repeated_source_occurrence_refuses_field_edits_and_all_supported_deletions() {
    let (root, mut model, schematic, sheet, symbol) = fixture("selected_repeated_occurrence");
    let wire = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::zero(),
        to: Point::new(1000, 0),
    };
    let prepared = build_create_schematic_wire(&model, provenance(), sheet, &wire).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: "GND".into(),
        kind: LabelKind::Power,
        position: wire.to,
    };
    let prepared = build_create_schematic_label(&model, provenance(), sheet, &label).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let definition = SheetDefinition {
        uuid: Uuid::new_v4(),
        root_sheet: sheet,
        name: "Repeated".into(),
    };
    let prepared = build_create_schematic_definition(
        &model,
        provenance(),
        schematic,
        definition.uuid,
        "definitions/repeated.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut paths = Vec::new();
    for name in ["First", "Second"] {
        let instance = SheetInstance {
            uuid: Uuid::new_v4(),
            definition: definition.uuid,
            parent_sheet: None,
            position: Point::zero(),
            name: name.into(),
            ports: vec![],
        };
        let prepared = build_create_schematic_sheet_instance(
            &model,
            provenance(),
            schematic,
            instance.uuid,
            serde_json::to_value(&instance).unwrap(),
        )
        .unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
        paths.push(instance.uuid);
    }
    let before = persisted_bytes(&root);
    for (verb, key, id) in [
        ("datum.schematic.delete_symbol", "symbol", symbol.uuid),
        ("datum.schematic.delete_wire", "wire", wire.uuid),
        ("datum.schematic.delete_label", "label", label.uuid),
        (
            "datum.schematic.set_symbol_reference",
            "symbol",
            symbol.uuid,
        ),
    ] {
        let mut params = serde_json::json!({"instance_path":[paths[0]]});
        params[key] = serde_json::json!(id);
        if verb.ends_with("reference") {
            params["reference"] = serde_json::json!("R9");
        }
        assert!(
            write(&model, &root, verb, params)
                .unwrap_err()
                .to_string()
                .contains("exactly one")
        );
    }
    assert_eq!(persisted_bytes(&root), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[path = "schematic_symbol_property_payload_tests.rs"]
mod property_payload;
