use super::*;
use crate::substrate::electrical_test_partitions;

fn build_pose(
    model: &DesignModel,
    root: &Path,
    id: Uuid,
    at: Point,
    rotation: i32,
    mirrored: bool,
) -> Result<PreparedWrite, EngineError> {
    move_symbol(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        serde_json::json!({"symbol":id,"instance_path":[],"x_nm":at.x,"y_nm":at.y,"rotation_deg":rotation,"mirrored":mirrored}),
    )
}

#[test]
fn connected_asymmetric_symbol_rotation_and_reflection_commit_at_same_position_with_exact_history()
{
    let mut f = fixture("placed_symbol_pose_history");
    let mut symbol = f.symbol.clone();
    symbol.position = p(1000, 1000);
    symbol.rotation = 90;
    symbol.mirrored = true;
    symbol.pins[0].position = p(980, 900); // local (100,20), mirror-X then +90.
    let mut pin = symbol.pins[0].clone();
    pin.uuid = Uuid::new_v4();
    pin.number = "2".into();
    pin.position = p(920, 1040); // local (-40,80).
    symbol.pins.push(pin);
    let prepared = build_set_schematic_symbol(&f.model, provenance(), f.sheet, &symbol).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let wires = [
        (symbol.pins[0].position, p(2000, 900)),
        (symbol.pins[1].position, p(0, 1040)),
    ]
    .map(|(from, to)| SchematicWire {
        uuid: Uuid::new_v4(),
        from,
        to,
    });
    for wire in &wires {
        let prepared = build_create_schematic_wire(&f.model, provenance(), f.sheet, wire).unwrap();
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    }
    let partitions = electrical_test_partitions(&f.model, &[]).unwrap();
    let identities = f.model.electrical_identities.clone();
    for (mirrored, expected_pins) in [
        (false, [p(1100, 1020), p(960, 1080)]),
        (true, [p(900, 1020), p(1040, 1080)]),
    ] {
        let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
        let before = snapshot.schematic_source().unwrap().clone();
        let basis = SymbolMoveBasis::capture(&snapshot, symbol.uuid, &[]).unwrap();
        let persisted = bytes(&f.root, &f.model);
        let preview = basis
            .destination_with_pose(symbol.position, Some(0), Some(mirrored))
            .unwrap();
        assert_eq!(
            bytes(&f.root, &f.model),
            persisted,
            "pose preview/cancel never writes"
        );
        assert_eq!(preview.symbol.position, symbol.position);
        assert_eq!(
            preview
                .symbol
                .pins
                .iter()
                .map(|pin| pin.position)
                .collect::<Vec<_>>(),
            expected_pins
        );
        for (wire, expected) in wires.iter().zip(expected_pins) {
            let moved = preview
                .wires
                .iter()
                .find(|changed| changed.uuid == wire.uuid)
                .unwrap();
            assert_eq!(moved.from, expected);
            assert_eq!(moved.to, wire.to);
        }
        let prepared =
            build_pose(&f.model, &f.root, symbol.uuid, symbol.position, 0, mirrored).unwrap();
        assert_eq!(
            prepared
                .batch
                .operations
                .iter()
                .filter(|op| matches!(op, Operation::SetSchematicSymbol { .. }))
                .count(),
            1
        );
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        let mut reopened = ProjectResolver::new(&f.root).resolve().unwrap();
        let after = ElectricalSelectionSnapshot::capture(&reopened)
            .unwrap()
            .schematic_source()
            .unwrap()
            .clone();
        assert_eq!(after.sheets[&f.sheet].symbols[&symbol.uuid], preview.symbol);
        for wire in preview.wires {
            assert_eq!(after.sheets[&f.sheet].wires[&wire.uuid], wire);
        }
        assert_eq!(
            electrical_test_partitions(&reopened, &[]).unwrap(),
            partitions
        );
        assert_eq!(reopened.electrical_identities, identities);
        reopened
            .commit_journal_undo(&f.root, provenance().into())
            .unwrap();
        let mut undone = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(
            ElectricalSelectionSnapshot::capture(&undone)
                .unwrap()
                .schematic_source(),
            Some(&before)
        );
        undone
            .commit_journal_redo(&f.root, provenance().into())
            .unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(
            ElectricalSelectionSnapshot::capture(&f.model)
                .unwrap()
                .schematic_source(),
            Some(&after)
        );
        assert_eq!(f.model.electrical_identities, identities);
    }
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn existing_pose_is_inverted_before_every_cardinal_destination_without_accumulation() {
    let mut f = fixture("placed_symbol_pose_algebra");
    // Independent expected points for local anchors (30,10), (-20,40).
    let cases = [
        (0, false, [p(30, 10), p(-20, 40)]),
        (90, false, [p(-10, 30), p(-40, -20)]),
        (180, false, [p(-30, -10), p(20, -40)]),
        (270, false, [p(10, -30), p(40, 20)]),
        (0, true, [p(-30, 10), p(20, 40)]),
        (90, true, [p(-10, -30), p(-40, 20)]),
        (180, true, [p(30, -10), p(-20, -40)]),
        (270, true, [p(10, 30), p(40, -20)]),
    ];
    let mut original = f.symbol.clone();
    let mut second = original.pins[0].clone();
    second.uuid = Uuid::new_v4();
    second.number = "2".into();
    original.pins.push(second);
    for (source_rotation, source_mirror, source_points) in cases {
        original.position = p(100, 200);
        original.rotation = source_rotation;
        original.mirrored = source_mirror;
        for (pin, at) in original.pins.iter_mut().zip(source_points) {
            pin.position = p(at.x + 100, at.y + 200);
        }
        let prepared =
            build_set_schematic_symbol(&f.model, provenance(), f.sheet, &original).unwrap();
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
        let basis = SymbolMoveBasis::capture(&snapshot, original.uuid, &[]).unwrap();
        for (rotation, mirrored, points) in cases {
            let destination = basis
                .destination_with_pose(p(-300, 400), Some(rotation), Some(mirrored))
                .unwrap();
            let expected: Vec<_> = points
                .into_iter()
                .map(|at| p(at.x - 300, at.y + 400))
                .collect();
            assert_eq!(
                destination
                    .symbol
                    .pins
                    .iter()
                    .map(|pin| pin.position)
                    .collect::<Vec<_>>(),
                expected,
                "source {source_rotation}/{source_mirror}, destination {rotation}/{mirrored}"
            );
            assert_eq!(destination.symbol.rotation, rotation);
            assert_eq!(destination.symbol.mirrored, mirrored);
            assert_eq!(destination.symbol.reference, original.reference);
            assert_eq!(destination.symbol.part, original.part);
        }
        assert_eq!(
            basis.destination_edit(original.position).unwrap().symbol,
            original
        );
    }
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn pose_uses_wide_offsets_and_refuses_invalid_or_overflowing_destinations_without_writes() {
    let mut f = fixture("placed_symbol_pose_extremes");
    let mut original = f.symbol.clone();
    original.position = p(i64::MIN, 0);
    original.pins[0].position = p(i64::MAX, 0);
    let prepared = build_set_schematic_symbol(&f.model, provenance(), f.sheet, &original).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let basis = SymbolMoveBasis::capture(&snapshot, original.uuid, &[]).unwrap();
    let before = bytes(&f.root, &f.model);
    assert_eq!(
        basis
            .destination_with_pose(p(i64::MAX, 0), Some(0), Some(true))
            .unwrap()
            .symbol
            .pins[0]
            .position,
        p(i64::MIN, 0)
    );
    assert!(
        basis
            .destination_with_pose(p(0, 0), Some(90), Some(false))
            .is_err()
    );
    assert!(
        build_pose(
            &f.model,
            &f.root,
            original.uuid,
            original.position,
            45,
            false
        )
        .is_err()
    );
    assert!(
        build_pose(
            &f.model,
            &f.root,
            original.uuid,
            original.position,
            0,
            false
        )
        .is_err(),
        "unchanged pose still refuses no-op"
    );
    assert_eq!(bytes(&f.root, &f.model), before);
    // Shared placement law must permit the same intermediate-overflow cancellation.
    assert_eq!(
        super::super::super::schematic_edit::place_local_point(p(i64::MIN, 0), p(-1, 0), 0, true)
            .unwrap(),
        p(i64::MAX, 0)
    );
    original.position = p(0, 0);
    original.pins[0].position = p(30, 10);
    original.rotation = 45;
    let prepared = build_set_schematic_symbol(&f.model, provenance(), f.sheet, &original).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let basis = SymbolMoveBasis::capture(&snapshot, original.uuid, &[]).unwrap();
    assert_eq!(
        basis
            .destination_with_pose(p(100, 200), Some(45), Some(false))
            .unwrap()
            .symbol
            .pins[0]
            .position,
        p(130, 210)
    );
    assert!(
        basis
            .destination_with_pose(p(100, 200), Some(90), Some(false))
            .is_err()
    );
    original.rotation = 0;
    let prepared = build_set_schematic_symbol(&f.model, provenance(), f.sheet, &original).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: "OTHER".into(),
        kind: LabelKind::Global,
        position: p(-10, 30),
    };
    let prepared = build_create_schematic_label(&f.model, provenance(), f.sheet, &label).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let before = bytes(&f.root, &f.model);
    assert!(
        build_pose(&f.model, &f.root, original.uuid, p(0, 0), 90, false)
            .unwrap_err()
            .to_string()
            .contains("geometric pin contact")
    );
    assert_eq!(
        bytes(&f.root, &f.model),
        before,
        "rotating onto another net refuses atomically"
    );
    std::fs::remove_dir_all(f.root).unwrap();
}
