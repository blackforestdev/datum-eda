use super::*;
use crate::substrate::electrical_test_partitions;

fn wire(
    model: &mut DesignModel,
    root: &Path,
    sheet: Uuid,
    from: Point,
    to: Point,
) -> SchematicWire {
    let wire = SchematicWire {
        uuid: Uuid::new_v4(),
        from,
        to,
    };
    let write = build_create_schematic_wire(model, provenance(), sheet, &wire).unwrap();
    commit_prepared(model, root, write).unwrap();
    wire
}

#[test]
fn connected_move_previews_all_endpoints_and_roundtrips_as_one_identity_preserving_edit() {
    let mut f = fixture("connected_symbol_move_history");
    // A second pin of the moving component drives a separate net, and two
    // branches on its first pin must follow together, not just one chosen wire.
    let mut symbol = f.symbol.clone();
    let mut pin = symbol.pins[0].clone();
    pin.uuid = Uuid::new_v4();
    pin.number = "2".into();
    pin.position = p(100, 0);
    symbol.pins.push(pin);
    let write = build_set_schematic_symbol(&f.model, provenance(), f.sheet, &symbol).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let mut other = f.symbol.clone();
    other.uuid = Uuid::new_v4();
    other.reference = "R2".into();
    other.position = p(1000, 0);
    other.pins[0].uuid = Uuid::new_v4();
    other.pins[0].position = p(1000, 0);
    let write =
        build_place_schematic_symbol(&f.model, provenance(), f.sheet, &other, None).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let a = wire(&mut f.model, &f.root, f.sheet, p(0, 0), p(-1000, 0));
    let b = wire(&mut f.model, &f.root, f.sheet, p(0, 0), p(0, -1000));
    let c = wire(&mut f.model, &f.root, f.sheet, p(100, 0), p(1000, 0));
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let basis = SymbolMoveBasis::capture(&snapshot, symbol.uuid, &[]).unwrap();
    let source = snapshot.schematic_source().unwrap().clone();
    let partitions = electrical_test_partitions(&f.model, &[]).unwrap();
    let identities = f.model.electrical_identities.clone();
    let before = bytes(&f.root, &f.model);
    let destination = basis.destination_edit(p(0, 200)).unwrap();
    assert_eq!(
        bytes(&f.root, &f.model),
        before,
        "preview/cancel writes nothing"
    );
    assert_eq!(destination.wires.len(), 3);
    for (original, from) in [(&a, p(0, 200)), (&b, p(0, 200)), (&c, p(100, 200))] {
        let moved = destination
            .wires
            .iter()
            .find(|wire| wire.uuid == original.uuid)
            .unwrap();
        assert_eq!(moved.from, from);
        assert_eq!(moved.to, original.to);
    }
    let prepared = build(&f.model, &f.root, symbol.uuid, p(0, 200)).unwrap();
    assert_eq!(
        prepared
            .batch
            .operations
            .iter()
            .filter(|op| matches!(op, Operation::DeleteSchematicWire { .. }))
            .count(),
        3
    );
    assert_eq!(
        prepared
            .batch
            .operations
            .iter()
            .filter(|op| matches!(op, Operation::CreateSchematicWire { .. }))
            .count(),
        3
    );
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let mut reopened = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        electrical_test_partitions(&reopened, &[]).unwrap(),
        partitions
    );
    assert_eq!(reopened.electrical_identities, identities);
    let moved_source = ElectricalSelectionSnapshot::capture(&reopened)
        .unwrap()
        .schematic_source()
        .unwrap()
        .clone();
    assert_eq!(
        moved_source.sheets[&f.sheet].symbols[&symbol.uuid],
        destination.symbol
    );
    for moved in &destination.wires {
        assert_eq!(&moved_source.sheets[&f.sheet].wires[&moved.uuid], moved);
    }
    let electrical_findings = |source: &crate::schematic::Schematic| {
        let mut findings: Vec<_> = crate::erc::run_prechecks(source)
            .into_iter()
            .map(|f| {
                (
                    f.code,
                    format!("{:?}", f.severity),
                    f.component,
                    f.pin,
                    f.object_uuids,
                )
            })
            .collect();
        findings.sort();
        findings
    };
    assert_eq!(
        electrical_findings(&source),
        electrical_findings(&moved_source)
    );
    reopened
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    let mut undone = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        ElectricalSelectionSnapshot::capture(&undone)
            .unwrap()
            .schematic_source(),
        Some(&source)
    );
    assert_eq!(undone.electrical_identities, identities);
    undone
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    let redo = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        ElectricalSelectionSnapshot::capture(&redo)
            .unwrap()
            .schematic_source(),
        Some(&moved_source)
    );
    assert_eq!(redo.electrical_identities, identities);
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn connected_move_refuses_short_detachment_and_collapsed_wire_without_writes() {
    for scenario in ["short", "detachment", "collapse", "bus", "no_connect"] {
        let mut f = fixture(&format!("connected_move_refusal_{scenario}"));
        wire(&mut f.model, &f.root, f.sheet, p(0, 0), p(1000, 0));
        let target = if scenario == "collapse" {
            p(1000, 0)
        } else {
            p(0, 1000)
        };
        match scenario {
            "short" => {
                // The stretched segment passes through this distinct net's label.
                let label = NetLabel {
                    uuid: Uuid::new_v4(),
                    name: "Other".into(),
                    kind: LabelKind::Global,
                    position: p(500, 500),
                };
                let write =
                    build_create_schematic_label(&f.model, provenance(), f.sheet, &label).unwrap();
                commit_prepared(&mut f.model, &f.root, write).unwrap();
            }
            "detachment" => {
                let junction = Junction {
                    uuid: Uuid::new_v4(),
                    position: p(500, 0),
                };
                let write =
                    build_create_schematic_junction(&f.model, provenance(), f.sheet, &junction)
                        .unwrap();
                commit_prepared(&mut f.model, &f.root, write).unwrap();
                wire(&mut f.model, &f.root, f.sheet, p(500, 0), p(500, -1000));
            }
            "bus" => {
                let bus = Bus {
                    uuid: Uuid::new_v4(),
                    name: "B".into(),
                    members: vec![],
                    segments: vec![p(400, 500), p(600, 500)],
                };
                let write =
                    build_create_schematic_bus(&f.model, provenance(), f.sheet, &bus).unwrap();
                commit_prepared(&mut f.model, &f.root, write).unwrap();
            }
            "no_connect" => {
                let marker = NoConnectMarker {
                    uuid: Uuid::new_v4(),
                    symbol: f.symbol.uuid,
                    pin: f.symbol.pins[0].uuid,
                    position: p(500, 500),
                };
                let write = build_place_schematic_marker(
                    &f.model,
                    provenance(),
                    f.sheet,
                    marker.uuid,
                    SchematicMarkerKind::NoConnect,
                    serde_json::to_value(marker).unwrap(),
                )
                .unwrap();
                commit_prepared(&mut f.model, &f.root, write).unwrap();
            }
            _ => {}
        }
        let before = bytes(&f.root, &f.model);
        assert!(
            build(&f.model, &f.root, f.symbol.uuid, target).is_err(),
            "{scenario}"
        );
        assert_eq!(
            bytes(&f.root, &f.model),
            before,
            "{scenario}: atomic refusal"
        );
        std::fs::remove_dir_all(f.root).unwrap();
    }
}

#[test]
fn connected_move_recaptures_concurrent_wire_changes_and_updates_both_owned_endpoints() {
    let mut f = fixture("connected_move_stale");
    let first = wire(&mut f.model, &f.root, f.sheet, p(0, 0), p(1000, 0));
    let stale = build(&f.model, &f.root, f.symbol.uuid, p(0, 1000)).unwrap();
    let write = build_delete_schematic_wire(&f.model, provenance(), f.sheet, &first).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let before = bytes(&f.root, &f.model);
    assert!(commit_prepared(&mut f.model, &f.root, stale).is_err());
    assert_eq!(bytes(&f.root, &f.model), before);
    let mut symbol = f.symbol.clone();
    let mut pin = symbol.pins[0].clone();
    pin.uuid = Uuid::new_v4();
    pin.number = "2".into();
    pin.position = p(100, 0);
    symbol.pins.push(pin);
    let write = build_set_schematic_symbol(&f.model, provenance(), f.sheet, &symbol).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let loop_wire = wire(&mut f.model, &f.root, f.sheet, p(0, 0), p(100, 0));
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let basis = SymbolMoveBasis::capture(&snapshot, symbol.uuid, &[]).unwrap();
    let moved = basis.destination_edit(p(0, 1000)).unwrap();
    assert_eq!(
        moved.wires,
        vec![SchematicWire {
            uuid: loop_wire.uuid,
            from: p(0, 1000),
            to: p(100, 1000)
        }]
    );
    std::fs::remove_dir_all(f.root).unwrap();
}
