use super::*;
use crate::api::native_write::{
    commit_prepared, electrical::binding_fixture::fixture, schematic_connectivity::*,
    schematic_sheets::*, schematic_symbols::*,
};
use crate::schematic::{
    Bus, BusEntry, HierarchicalPort, Junction, LabelKind, NetLabel, NoConnectMarker,
    PinDisplayOverride, PortDirection, SchematicWire, SheetDefinition, SheetInstance, SymbolField,
};
use crate::substrate::{
    CommitSource, DesignModel, Operation, ProjectResolver, SchematicMarkerKind,
};
use std::{collections::BTreeMap, path::Path};

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "move native schematic symbol")
}
fn p(x: i64, y: i64) -> Point {
    Point::new(x, y)
}
fn build(
    model: &DesignModel,
    root: &Path,
    id: Uuid,
    position: Point,
) -> Result<PreparedWrite, EngineError> {
    move_symbol(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        serde_json::json!({"symbol":id,"instance_path":[],"x_nm":position.x,"y_nm":position.y}),
    )
}
fn bytes(root: &Path, model: &DesignModel) -> BTreeMap<String, Vec<u8>> {
    let mut result: BTreeMap<_, _> = model
        .source_shards
        .iter()
        .map(|s| {
            (
                s.relative_path.clone(),
                std::fs::read(root.join(&s.relative_path)).unwrap(),
            )
        })
        .collect();
    fn journal(path: &Path, result: &mut BTreeMap<String, Vec<u8>>) {
        if !path.exists() {
            return;
        }
        for e in std::fs::read_dir(path).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                journal(&p, result);
            } else {
                result.insert(p.display().to_string(), std::fs::read(p).unwrap());
            }
        }
    }
    journal(&root.join(".datum/journal"), &mut result);
    result
}

#[test]
fn symbol_move_canonical_commit_preserves_identity_and_roundtrips_history() {
    let mut f = fixture("native_symbol_move_history");
    let mut original = f.symbol.clone();
    original.position = p(100, -50);
    original.pins[0].position = p(0, 0);
    original.rotation = 90;
    original.mirrored = true;
    let mut power_pin = original.pins[0].clone();
    power_pin.uuid = Uuid::new_v4();
    power_pin.number = "2".into();
    power_pin.name = "VCC".into();
    power_pin.electrical_type = crate::schematic::PinElectricalType::PowerIn;
    power_pin.position = p(300, -400);
    original.pins.push(power_pin.clone());
    original.fields.push(SymbolField {
        uuid: Uuid::new_v4(),
        key: "reference".into(),
        value: "U1".into(),
        position: None,
        visible: false,
    });
    original.pin_overrides.push(PinDisplayOverride {
        pin: original.pins[0].uuid,
        visible: false,
        position: None,
    });
    original.pin_overrides.push(PinDisplayOverride {
        pin: power_pin.uuid,
        visible: false,
        position: None,
    });
    let write = build_set_schematic_symbol(&f.model, provenance(), f.sheet, &original).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let basis = SymbolMoveBasis::capture(&snapshot, original.uuid, &[]).unwrap();
    let before = bytes(&f.root, &f.model);
    let moved = basis.destination(p(1100, 1950)).unwrap();
    assert_eq!(
        bytes(&f.root, &f.model),
        before,
        "preview/cancel has no writes"
    );
    let mut expected = original.clone();
    expected.position = p(1100, 1950);
    expected.pins[0].position = p(1000, 2000);
    expected.pins[1].position = p(1300, 1600);
    assert_eq!(
        moved, expected,
        "all other source fields and hidden-power policy unchanged"
    );
    let identities = f.model.electrical_identities.clone();
    let write = build(&f.model, &f.root, original.uuid, moved.position).unwrap();
    assert_eq!(
        write.batch.expected_model_revision,
        Some(f.model.model_revision.clone())
    );
    assert_eq!(
        write
            .batch
            .operations
            .iter()
            .filter(|op| matches!(op, Operation::SetSchematicSymbol { .. }))
            .count(),
        1
    );
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let mut reopened = ProjectResolver::new(&f.root).resolve().unwrap();
    let source = ElectricalSelectionSnapshot::capture(&reopened).unwrap();
    assert_eq!(
        source.schematic_source().unwrap().sheets[&f.sheet].symbols[&original.uuid],
        expected
    );
    assert_eq!(
        source.net_ids(),
        snapshot.net_ids(),
        "logical isolated pin assignment survives"
    );
    assert_eq!(reopened.electrical_identities, identities);
    reopened
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    let undone = ElectricalSelectionSnapshot::capture(&reopened).unwrap();
    assert_eq!(
        undone.schematic_source().unwrap().sheets[&f.sheet].symbols[&original.uuid],
        original
    );
    reopened
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    let redo = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        ElectricalSelectionSnapshot::capture(&redo)
            .unwrap()
            .schematic_source()
            .unwrap()
            .sheets[&f.sheet]
            .symbols[&original.uuid],
        expected
    );
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn symbol_move_refuses_all_existing_and_destination_contact_kinds_without_writes() {
    for kind in [
        "wire",
        "pin",
        "label",
        "port",
        "junction",
        "noconnect",
        "bus",
        "entry",
    ] {
        for position in [p(0, 0), p(1000, 2000)] {
            let mut f = fixture(&format!("symbol_move_contact_{kind}_{}", position.x));
            let write = match kind {
                "wire" => build_create_schematic_wire(
                    &f.model,
                    provenance(),
                    f.sheet,
                    &SchematicWire {
                        uuid: Uuid::new_v4(),
                        from: p(position.x - 10, position.y),
                        to: p(position.x + 10, position.y),
                    },
                )
                .unwrap(),
                "pin" => {
                    let mut foreign = f.symbol.clone();
                    foreign.uuid = Uuid::new_v4();
                    foreign.reference = "U2".into();
                    foreign.pins[0].uuid = Uuid::new_v4();
                    foreign.pins[0].position = position;
                    build_place_schematic_symbol(&f.model, provenance(), f.sheet, &foreign, None)
                        .unwrap()
                }
                "label" => build_create_schematic_label(
                    &f.model,
                    provenance(),
                    f.sheet,
                    &NetLabel {
                        uuid: Uuid::new_v4(),
                        kind: LabelKind::Local,
                        name: "N".into(),
                        position,
                    },
                )
                .unwrap(),
                "port" => build_create_schematic_port(
                    &f.model,
                    provenance(),
                    f.sheet,
                    &HierarchicalPort {
                        uuid: Uuid::new_v4(),
                        name: "P".into(),
                        direction: PortDirection::Input,
                        position,
                    },
                )
                .unwrap(),
                "junction" => build_create_schematic_junction(
                    &f.model,
                    provenance(),
                    f.sheet,
                    &Junction {
                        uuid: Uuid::new_v4(),
                        position,
                    },
                )
                .unwrap(),
                "noconnect" => {
                    let marker = NoConnectMarker {
                        uuid: Uuid::new_v4(),
                        symbol: f.symbol.uuid,
                        pin: f.symbol.pins[0].uuid,
                        position,
                    };
                    build_place_schematic_marker(
                        &f.model,
                        provenance(),
                        f.sheet,
                        marker.uuid,
                        SchematicMarkerKind::NoConnect,
                        serde_json::to_value(marker).unwrap(),
                    )
                    .unwrap()
                }
                "bus" => build_create_schematic_bus(
                    &f.model,
                    provenance(),
                    f.sheet,
                    &Bus {
                        uuid: Uuid::new_v4(),
                        name: "B".into(),
                        members: vec![],
                        segments: vec![
                            p(position.x - 10, position.y),
                            p(position.x + 10, position.y),
                        ],
                    },
                )
                .unwrap(),
                "entry" => {
                    let bus = Bus {
                        uuid: Uuid::new_v4(),
                        name: "B".into(),
                        members: vec![],
                        segments: vec![p(10000, 10000), p(20000, 10000)],
                    };
                    let write =
                        build_create_schematic_bus(&f.model, provenance(), f.sheet, &bus).unwrap();
                    commit_prepared(&mut f.model, &f.root, write).unwrap();
                    build_create_schematic_bus_entry(
                        &f.model,
                        provenance(),
                        f.sheet,
                        &BusEntry {
                            uuid: Uuid::new_v4(),
                            bus: bus.uuid,
                            wire: None,
                            position: p(position.x - 1, position.y - 1),
                            size: p(2, 2),
                        },
                    )
                    .unwrap()
                }
                _ => unreachable!(),
            };
            commit_prepared(&mut f.model, &f.root, write).unwrap();
            let before = bytes(&f.root, &f.model);
            let error = build(&f.model, &f.root, f.symbol.uuid, p(1000, 2000)).unwrap_err();
            assert!(
                error.to_string().contains("geometric pin contact"),
                "{kind}: {error}"
            );
            assert_eq!(bytes(&f.root, &f.model), before, "{kind}: atomic refusal");
            std::fs::remove_dir_all(f.root).unwrap();
        }
    }
}

#[test]
fn symbol_move_refuses_positioned_owners_and_repeated_or_wrong_occurrence() {
    let mut f = fixture("symbol_move_owned_geometry");
    for field in [true, false] {
        let mut symbol = f.symbol.clone();
        if field {
            symbol.fields.push(SymbolField {
                uuid: Uuid::new_v4(),
                key: "value".into(),
                value: "V".into(),
                position: Some(p(3, 4)),
                visible: true,
            });
        } else {
            symbol.pin_overrides.push(PinDisplayOverride {
                pin: symbol.pins[0].uuid,
                visible: true,
                position: Some(p(3, 4)),
            });
        }
        let write = build_set_schematic_symbol(&f.model, provenance(), f.sheet, &symbol).unwrap();
        commit_prepared(&mut f.model, &f.root, write).unwrap();
        let before = bytes(&f.root, &f.model);
        assert!(
            build(&f.model, &f.root, symbol.uuid, p(1000, 2000))
                .unwrap_err()
                .to_string()
                .contains("positioned")
        );
        assert_eq!(bytes(&f.root, &f.model), before);
    }
    let write = build_set_schematic_symbol(&f.model, provenance(), f.sheet, &f.symbol).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    assert!(SymbolMoveBasis::capture(&snapshot, f.symbol.uuid, &[Uuid::new_v4()]).is_err());
    let definition = SheetDefinition {
        uuid: Uuid::new_v4(),
        root_sheet: f.sheet,
        name: "Repeated".into(),
    };
    let write = build_create_schematic_definition(
        &f.model,
        provenance(),
        f.schematic,
        definition.uuid,
        "definitions/repeated.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    for n in 0..2 {
        let instance = SheetInstance {
            uuid: Uuid::new_v4(),
            definition: definition.uuid,
            parent_sheet: None,
            position: p(n, 0),
            name: format!("I{n}"),
            ports: vec![],
        };
        let write = build_create_schematic_sheet_instance(
            &f.model,
            provenance(),
            f.schematic,
            instance.uuid,
            serde_json::to_value(&instance).unwrap(),
        )
        .unwrap();
        commit_prepared(&mut f.model, &f.root, write).unwrap();
        if n == 0 {
            let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
            let basis =
                SymbolMoveBasis::capture(&snapshot, f.symbol.uuid, &[instance.uuid]).unwrap();
            assert_eq!(basis.sheet_id, f.sheet);
            assert!(SymbolMoveBasis::capture(&snapshot, f.symbol.uuid, &[]).is_err());
        }
    }
    let before = bytes(&f.root, &f.model);
    assert!(
        build(&f.model, &f.root, f.symbol.uuid, p(1000, 2000))
            .unwrap_err()
            .to_string()
            .contains("exactly one")
    );
    assert_eq!(bytes(&f.root, &f.model), before);
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn symbol_move_checked_translation_and_stale_commit_are_atomic() {
    let mut f = fixture("symbol_move_stale_overflow");
    let mut symbol = f.symbol.clone();
    symbol.position = p(i64::MIN, 0);
    symbol.pins[0].position = p(i64::MIN, 0);
    let write = build_set_schematic_symbol(&f.model, provenance(), f.sheet, &symbol).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let basis = SymbolMoveBasis::capture(&snapshot, symbol.uuid, &[]).unwrap();
    assert_eq!(
        basis.destination(p(i64::MAX, 0)).unwrap().pins[0].position,
        p(i64::MAX, 0)
    );
    symbol.pins[0].position = p(i64::MIN + 1, 0);
    let write = build_set_schematic_symbol(&f.model, provenance(), f.sheet, &symbol).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let basis = SymbolMoveBasis::capture(&snapshot, symbol.uuid, &[]).unwrap();
    assert!(basis.destination(p(i64::MAX, 0)).is_err());
    let prepared = build(&f.model, &f.root, symbol.uuid, p(1000, 2000)).unwrap();
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        kind: LabelKind::Global,
        name: "POWER".into(),
        position: p(1001, 2000),
    };
    let write = build_create_schematic_label(&f.model, provenance(), f.sheet, &label).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let before = bytes(&f.root, &f.model);
    assert!(commit_prepared(&mut f.model, &f.root, prepared).is_err());
    assert_eq!(bytes(&f.root, &f.model), before);
    assert!(
        build(&f.model, &f.root, symbol.uuid, p(1000, 2000))
            .unwrap_err()
            .to_string()
            .contains("geometric pin contact"),
        "writer recaptures new destination contact"
    );
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn symbol_move_unavailable_contact_geometry_never_becomes_isolation() {
    for geometry in [
        "empty_bus",
        "degenerate_bus",
        "zero_entry",
        "overflow_entry",
    ] {
        let mut f = fixture(&format!("symbol_move_incomplete_{geometry}"));
        let bus = Bus {
            uuid: Uuid::new_v4(),
            name: "B".into(),
            members: vec![],
            segments: match geometry {
                "empty_bus" => vec![],
                "degenerate_bus" => vec![p(10000, 10000), p(10000, 10000)],
                _ => vec![p(10000, 10000), p(20000, 10000)],
            },
        };
        let write = build_create_schematic_bus(&f.model, provenance(), f.sheet, &bus).unwrap();
        commit_prepared(&mut f.model, &f.root, write).unwrap();
        if geometry.ends_with("entry") {
            let entry = BusEntry {
                uuid: Uuid::new_v4(),
                bus: bus.uuid,
                wire: None,
                position: p(i64::MAX, 10000),
                size: if geometry == "zero_entry" {
                    Point::zero()
                } else {
                    p(1, 1)
                },
            };
            let write =
                build_create_schematic_bus_entry(&f.model, provenance(), f.sheet, &entry).unwrap();
            commit_prepared(&mut f.model, &f.root, write).unwrap();
        }
        let before = bytes(&f.root, &f.model);
        let error = build(&f.model, &f.root, f.symbol.uuid, p(1000, 2000)).unwrap_err();
        assert!(
            error.to_string().contains("incomplete") || error.to_string().contains("overflow"),
            "{geometry}: {error}"
        );
        assert_eq!(bytes(&f.root, &f.model), before);
        std::fs::remove_dir_all(f.root).unwrap();
    }
}
