//! Canonical declaration births, binding transitions and refusal atomicity.
use super::*;

fn declaration_net(model: &DesignModel, sheet: Uuid, name: &str) -> Uuid {
    let snapshot = ElectricalSelectionSnapshot::capture(model).unwrap();
    let id = snapshot.schematic_source().unwrap().sheets[&sheet]
        .net_declarations
        .values()
        .find(|d| d.name == name)
        .unwrap()
        .uuid;
    let net = snapshot
        .schematic_member_net(&members::origin("net_declarations", id, &[]))
        .unwrap();
    assert_ne!(net, id, "source UUID is not the scalar Net identity");
    net
}

#[test]
fn new_bus_members_mixed_with_existing_are_atomic_stable_and_independent() {
    let (root, mut model, sheet, _) = setup("bus_declaration_births");
    let before = source(&model);
    let before_raw = model
        .materialized_source_shard_value_by_relative_path("schematic/sheets/main.json")
        .unwrap();
    let before_nets = net_ids(&model);
    let mut p = params(sheet);
    p["members"] = serde_json::json!(["D0", "NEW_A", "NEW_B"]);
    let write = build(&model, &root, "datum.schematic.create_bus", p).unwrap();
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap()),
        before,
        "preparation/Cancel writes nothing"
    );
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let after = source(&model);
    assert_eq!(after.sheets[&sheet].net_declarations.len(), 2);
    assert_eq!(after.sheets[&sheet].labels, before.sheets[&sheet].labels);
    assert_eq!(after.sheets[&sheet].wires, before.sheets[&sheet].wires);
    let a = declaration_net(&model, sheet, "NEW_A");
    let b = declaration_net(&model, sheet, "NEW_B");
    assert_ne!(a, b);
    assert_eq!(net_ids(&model).len(), before_nets.len() + 2);
    history(&mut model, &root, false);
    assert_eq!(source(&model), before);
    assert_eq!(
        model
            .materialized_source_shard_value_by_relative_path("schematic/sheets/main.json")
            .unwrap(),
        before_raw
    );
    assert_eq!(net_ids(&model), before_nets);
    history(&mut model, &root, true);
    assert_eq!(source(&model), after);
    assert_eq!(declaration_net(&model, sheet, "NEW_A"), a);
    assert_eq!(declaration_net(&model, sheet, "NEW_B"), b);
}

#[test]
fn explicit_new_member_entry_merges_only_selected_net_and_removal_splits() {
    let (root, mut model, sheet, wires) = setup("bus_declaration_binding");
    let mut p = params(sheet);
    p["members"] = serde_json::json!(["NEW_A", "NEW_B"]);
    let mut entry = tap(wires[0].uuid);
    entry["member"] = "NEW_A".into();
    p["entries"] = serde_json::json!([entry]);
    let write = build(&model, &root, "datum.schematic.create_bus", p).unwrap();
    let bus = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let wire_net = snapshot
        .schematic_member_net(&members::origin("wires", wires[0].uuid, &[]))
        .unwrap();
    assert_eq!(declaration_net(&model, sheet, "NEW_A"), wire_net);
    let b = declaration_net(&model, sheet, "NEW_B");
    assert_ne!(b, wire_net);
    assert_eq!(net_ids(&model).len(), 5);
    let bound = source(&model);
    let entry = bound.sheets[&sheet].bus_entries.values().next().unwrap();
    let write = build(
        &model,
        &root,
        "datum.schematic.delete_bus_entry",
        serde_json::json!({"bus":bus,"bus_entry":entry.uuid,"instance_path":[]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(net_ids(&model).len(), 6);
    assert_eq!(declaration_net(&model, sheet, "NEW_B"), b);
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert_ne!(
        declaration_net(&model, sheet, "NEW_A"),
        snapshot
            .schematic_member_net(&members::origin("wires", wires[0].uuid, &[]))
            .unwrap()
    );
    history(&mut model, &root, false);
    assert_eq!(source(&model), bound);
    assert_eq!(declaration_net(&model, sheet, "NEW_A"), wire_net);
    history(&mut model, &root, true);
    assert_eq!(net_ids(&model).len(), 6);
}

#[test]
fn member_conflicts_and_colliding_labels_refuse_without_partial_writes() {
    let (root, mut model, sheet, wires) = setup("bus_declaration_refusals");
    let mut p = params(sheet);
    p["members"] = serde_json::json!(["D0", "NEW_A"]);
    let write = build(&model, &root, "datum.schematic.create_bus", p).unwrap();
    let bus = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = source(&model);
    let identities = model.electrical_identities.clone();
    let mut entry = tap(wires[0].uuid);
    entry["member"] = "NEW_A".into();
    let error = build(
        &model,
        &root,
        "datum.schematic.place_bus_entry",
        serde_json::json!({"sheet":sheet,"bus":bus,"instance_path":[],"entries":[entry]}),
    )
    .unwrap_err();
    assert!(error.to_string().contains("different member"), "{error}");
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: "NEW_A".into(),
        kind: LabelKind::Local,
        position: Point::new(4000, 4000),
    };
    let write = build_create_schematic_label(&model, provenance(), sheet, &label).unwrap();
    assert!(commit_prepared(&mut model, &root, write).is_err());
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model), before);
    assert_eq!(model.electrical_identities, identities);
}

#[test]
fn new_declaration_never_adopts_legacy_groups_even_on_later_edits() {
    let (root, mut model, _, sheet) = fixture("bus_declaration_legacy_boundary");
    let old = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(0, 0),
        to: Point::new(100, 0),
    };
    let write = build_create_schematic_wire(&model, provenance(), sheet, &old).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(net_ids(&model).is_empty());
    let mut p = params(sheet);
    p["members"] = serde_json::json!(["NEW_A"]);
    let write = build(&model, &root, "datum.schematic.create_bus", p).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(net_ids(&model).len(), 1);
    let new = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(0, 2000),
        to: Point::new(100, 2000),
    };
    let write = build_create_schematic_wire(&model, provenance(), sheet, &new).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(net_ids(&model).len(), 2);
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert!(
        snapshot
            .schematic_member_net(&members::origin("wires", old.uuid, &[]))
            .is_err()
    );
    assert!(crate::connectivity::native_schematic_net_info(&model, &source(&model)).is_err());
    let write = crate::api::native_write::electrical::build_adopt_schematic_net_identities(
        &model,
        provenance(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(net_ids(&model).len(), 3);
    let nets = crate::connectivity::native_schematic_net_info(&model, &source(&model)).unwrap();
    assert_eq!(
        nets.iter().map(|n| n.uuid).collect::<BTreeSet<_>>(),
        net_ids(&model)
    );
}

#[test]
fn multiple_entries_share_one_native_query_net_and_keep_remaining_binding() {
    let (root, mut model, sheet, wires) = setup("bus_declaration_query");
    for (index, wire) in wires[..2].iter().enumerate() {
        use crate::schematic::{
            HiddenPowerBehavior, PinElectricalType, PlacedSymbol, SymbolDisplayMode, SymbolPin,
        };
        let symbol = PlacedSymbol {
            uuid: Uuid::new_v4(),
            part: None,
            entity: None,
            gate: None,
            lib_id: Some("Device:IC".into()),
            reference: format!("U{index}"),
            value: "Driver".into(),
            fields: vec![],
            pins: vec![SymbolPin {
                uuid: Uuid::new_v4(),
                library_pin: None,
                number: "1".into(),
                name: "OUT".into(),
                electrical_type: PinElectricalType::Output,
                position: wire.from,
            }],
            position: wire.from,
            rotation: 0,
            mirrored: false,
            unit_selection: None,
            display_mode: SymbolDisplayMode::LibraryDefault,
            pin_overrides: vec![],
            hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
        };
        let write = crate::api::native_write::schematic_symbols::build_place_schematic_symbol(
            &model,
            provenance(),
            sheet,
            &symbol,
            None,
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
    }

    let mut p = params(sheet);
    p["members"] = serde_json::json!(["NEW_A", "NEW_B"]);
    let mut a = tap(wires[0].uuid);
    a["member"] = "NEW_A".into();
    let mut b = tap(wires[1].uuid);
    b["member"] = "NEW_A".into();
    b["y_nm"] = 1100.into();
    p["entries"] = serde_json::json!([a, b]);
    let write = build(&model, &root, "datum.schematic.create_bus", p).unwrap();
    let bus = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let placed = source(&model);
    let mut shared = params(sheet);
    shared["name"] = "SECOND".into();
    shared["members"] = serde_json::json!(["NEW_A"]);
    let write = build(&model, &root, "datum.schematic.create_bus", shared).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let net = declaration_net(&model, sheet, "NEW_A");
    let nets = crate::connectivity::native_schematic_net_info(&model, &placed).unwrap();
    assert_eq!(
        nets.iter().map(|n| n.uuid).collect::<BTreeSet<_>>(),
        net_ids(&model)
    );
    assert_eq!(nets.iter().find(|n| n.uuid == net).unwrap().labels, 2);
    assert!(nets.iter().any(|n| n.name == "NEW_B" && n.labels == 0));
    let findings = crate::erc::run_prechecks_with_nets(&placed, &Default::default(), &[], nets);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "output_to_output_conflict")
    );
    let entry = placed.sheets[&sheet]
        .bus_entries
        .values()
        .find(|e| e.wire == Some(wires[0].uuid))
        .unwrap();
    let write = build(
        &model,
        &root,
        "datum.schematic.delete_bus_entry",
        serde_json::json!({"bus":bus,"bus_entry":entry.uuid,"instance_path":[]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let nets = crate::connectivity::native_schematic_net_info(&model, &source(&model)).unwrap();
    assert_eq!(
        nets.iter()
            .find(|n| n.uuid == declaration_net(&model, sheet, "NEW_A"))
            .unwrap()
            .labels,
        1
    );
    let findings =
        crate::erc::run_prechecks_with_nets(&source(&model), &Default::default(), &[], nets);
    assert!(
        !findings
            .iter()
            .any(|f| f.code == "output_to_output_conflict")
    );
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert_eq!(
        snapshot
            .schematic_member_net(&members::origin("wires", wires[1].uuid, &[]))
            .unwrap(),
        declaration_net(&model, sheet, "NEW_A")
    );
    let p = serde_json::json!({"sheet":sheet,"bus":bus,"instance_path":[],"members":["NEW_B"]});
    assert!(build(&model, &root, "datum.schematic.create_bus", p).is_err());
    let write = build(
        &model,
        &root,
        "datum.schematic.delete_bus",
        serde_json::json!({"bus":bus,"instance_path":[]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model).sheets[&sheet].net_declarations.len(), 2);
    assert!(source(&model).sheets[&sheet].bus_entries.is_empty());
    assert_eq!(net_ids(&model).len(), 6);
    let expected = declaration_net(&model, sheet, "NEW_A");
    let buses: Vec<_> = model
        .electrical_identities
        .values()
        .filter_map(|r| match &r.identity {
            ElectricalIdentity::Bus {
                scalar_nets,
                scalar_declarations,
                retired: false,
                ..
            } if !scalar_declarations.is_empty() => Some((scalar_nets, scalar_declarations)),
            _ => None,
        })
        .collect();
    assert_eq!(buses.len(), 2);
    assert!(buses.iter().all(|(nets, _)| nets.contains(&expected)));
    assert!(model.electrical_identities.values().any(|r| matches!(&r.identity, ElectricalIdentity::Bus { representations, scalar_declarations, retired: false, .. } if representations.is_empty() && scalar_declarations.len() == 2)));
    let zero_drawing = model.electrical_identities.clone();
    history(&mut model, &root, false);
    assert_eq!(source(&model).sheets[&sheet].bus_entries.len(), 1);
    history(&mut model, &root, true);
    assert_eq!(model.electrical_identities, zero_drawing);
}

#[test]
fn native_projection_preserves_custom_power_label_metadata() {
    let (root, mut model, sheet, _) = setup("bus_declaration_power_projection");
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: "CUSTOM_RAIL".into(),
        kind: LabelKind::Power,
        position: Point::new(4000, 4000),
    };
    let write = build_create_schematic_label(&model, provenance(), sheet, &label).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut p = params(sheet);
    p["members"] = serde_json::json!(["NEW_A"]);
    let write = build(&model, &root, "datum.schematic.create_bus", p).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let source = source(&model);
    let nets = crate::connectivity::native_schematic_net_info(&model, &source).unwrap();
    assert_eq!(
        nets.iter()
            .find(|n| n.name == "CUSTOM_RAIL")
            .unwrap()
            .semantic_class
            .as_deref(),
        Some("power")
    );
    let erc = crate::erc::run_prechecks_with_nets(&source, &Default::default(), &[], nets);
    assert!(erc.iter().any(|f| f.code == "undriven_power_net" && f.net_name.as_deref() == Some("CUSTOM_RAIL")));
}

#[test]
fn declaration_member_removal_requires_a_semantic_edit_and_retains_scalar_source() {
    let (root, mut model, sheet, _) = setup("bus_declaration_member_edit");
    let mut p = params(sheet);
    p["members"] = serde_json::json!(["NEW_A", "NEW_B"]);
    let write = build(&model, &root, "datum.schematic.create_bus", p).unwrap();
    let bus_id = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = source(&model);
    let identities = model.electrical_identities.clone();
    let a = declaration_net(&model, sheet, "NEW_A");
    let mut bus = before.sheets[&sheet].buses[&bus_id].clone();
    bus.members = vec!["NEW_B".into()];
    let write = build_set_schematic_bus(&model, provenance(), sheet, &bus).unwrap();
    assert!(
        commit_prepared(&mut model, &root, write)
            .unwrap_err()
            .to_string()
            .contains("declaration binding")
    );
    assert_eq!(source(&model), before);
    assert_eq!(model.electrical_identities, identities);
    let p = serde_json::json!({"sheet":sheet,"bus":bus_id,"instance_path":[],"members":["NEW_B"]});
    let write = build(&model, &root, "datum.schematic.create_bus", p).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(source(&model).sheets[&sheet].net_declarations.len(), 2);
    assert_eq!(declaration_net(&model, sheet, "NEW_A"), a);
    assert!(model.electrical_identities.values().any(|r| matches!(&r.identity, ElectricalIdentity::Bus { scalar_declarations, scalar_nets, .. } if scalar_declarations.len() == 1 && scalar_nets.len() == 1)));
    history(&mut model, &root, false);
    assert_eq!(source(&model), before);
    assert_eq!(model.electrical_identities, identities);
}

#[test]
fn malformed_declaration_sources_and_direct_deletion_refuse() {
    use crate::substrate::Operation;
    let (root, mut model, sheet, wires) = setup("bus_declaration_validation");
    let mut p = params(sheet);
    p["members"] = serde_json::json!(["NEW_A", "NEW_B"]);
    let write = build(&model, &root, "datum.schematic.create_bus", p).unwrap();
    let bus = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let source = source(&model);
    let declaration = source.sheets[&sheet]
        .net_declarations
        .values()
        .find(|d| d.name == "NEW_A")
        .unwrap();
    let before = model.model_revision.clone();
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::DeleteSchematicNetDeclaration {
            sheet_id: sheet,
            declaration_id: declaration.uuid,
            declaration: serde_json::to_value(declaration).unwrap(),
            restore_collection_absence: false,
        })
        .finish()
        .unwrap();
    assert!(
        commit_prepared(&mut model, &root, write)
            .unwrap_err()
            .to_string()
            .contains("compensation")
    );
    assert_eq!(model.model_revision, before);
    let mut omitted = serde_json::to_value(&source.sheets[&sheet]).unwrap();
    omitted.as_object_mut().unwrap().remove("net_declarations");
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::DeleteSchematicSheet {
            schematic_id: source.uuid,
            sheet_id: sheet,
            relative_path: "sheets/main.json".into(),
            sheet: omitted,
        })
        .finish()
        .unwrap();
    assert!(
        commit_prepared(&mut model, &root, write)
            .unwrap_err()
            .to_string()
            .contains("independent scalar declarations")
    );
    assert_eq!(model.model_revision, before);
    let mut bad = source.clone();
    bad.sheets
        .get_mut(&sheet)
        .unwrap()
        .net_declarations
        .get_mut(&declaration.uuid)
        .unwrap()
        .uuid = Uuid::new_v4();
    assert!(crate::connectivity::partitions(&bad).is_err());
    let mut raw = serde_json::to_value(&source.sheets[&sheet]).unwrap();
    raw["net_declarations"][declaration.uuid.to_string()]["position"] =
        serde_json::json!({"x":0,"y":0});
    assert!(serde_json::from_value::<crate::schematic::Sheet>(raw).is_err());
    let mut tap = tap(wires[0].uuid);
    tap["member"] = "NEW_A".into();
    let params = serde_json::json!({"sheet":sheet,"bus":bus,"instance_path":[],"entries":[tap]});
    for field in ["source_id", "class", "instance_path"] {
        let mut write = build(
            &model,
            &root,
            "datum.schematic.place_bus_entry",
            params.clone(),
        )
        .unwrap();
        let entry = write
            .batch
            .operations
            .iter_mut()
            .find_map(|op| match op {
                Operation::CreateSchematicBusEntry { bus_entry, .. } => Some(bus_entry),
                _ => None,
            })
            .unwrap();
        entry["member_declaration"][field] = match field {
            "source_id" => serde_json::json!(Uuid::new_v4()),
            "class" => "labels".into(),
            _ => serde_json::json!([Uuid::new_v4()]),
        };
        assert!(commit_prepared(&mut model, &root, write).is_err());
        assert_eq!(model.model_revision, before);
    }
    let raw = serde_json::json!({"uuid":Uuid::new_v4(),"bus":bus,"wire":wires[0].uuid,"member_declaration":null,"position":{"x":0,"y":0}});
    assert!(serde_json::from_value::<BusEntry>(raw).is_err());
}
