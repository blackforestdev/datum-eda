//! Connected-set geometry and source-history proofs over an actual native project.
use super::super::tests::fixture as geometry_fixture;
use super::*;
use crate::api::native_write::{
    commit_prepared, electrical::binding_fixture::BindingFixture, schematic_connectivity::*,
    schematic_edit::prepare_native_birth, schematic_symbols::build_place_schematic_symbol,
};
use crate::schematic::{LabelKind, NetLabel, SymbolField};
use crate::substrate::{CommitSource, ProjectResolver, electrical_test_partitions};
use serde_json::{Value, json};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "group Symbol transform")
}
fn raw(f: &BindingFixture) -> Value {
    source_sheet_payload(&f.model, f.sheet, f.symbol.uuid).unwrap()
}
fn setup(name: &str) -> (BindingFixture, Uuid, Uuid) {
    let mut f = geometry_fixture(name);
    let mut first = f.symbol.clone();
    first.position = Point::zero();
    first.pins[0].position = Point::zero();
    first.fields = vec![
        SymbolField {
            uuid: Uuid::new_v4(),
            key: "same".into(),
            value: "visible".into(),
            position: Some(Point::new(100, 200)),
            visible: true,
        },
        SymbolField {
            uuid: Uuid::new_v4(),
            key: "same".into(),
            value: "hidden".into(),
            position: Some(Point::new(200, 300)),
            visible: false,
        },
        SymbolField {
            uuid: Uuid::new_v4(),
            key: "none".into(),
            value: "none".into(),
            position: None,
            visible: false,
        },
    ];
    let mut second = first.clone();
    second.uuid = Uuid::new_v4();
    second.reference = "U2".into();
    second.part = None;
    second.entity = None;
    second.gate = None;
    second.lib_id = None;
    second.position = Point::new(1000, 0);
    second.rotation = 90;
    second.mirrored = true;
    for pin in &mut second.pins {
        pin.uuid = Uuid::new_v4();
        pin.position = Point::new(1000, 0);
    }
    for field in &mut second.fields {
        field.uuid = Uuid::new_v4();
    }
    let wire = Uuid::new_v4();
    let mut first_raw = serde_json::to_value(&first).unwrap();
    first_raw["extension"] = json!({"keep":"symbol"});
    first_raw["pins"][0]["extension"] = json!("pin");
    first_raw["fields"][0]["extension"] = json!("field");
    first_raw["fields"][0]["position"]["extension"] = json!("coordinate");
    let mut write =
        build_place_schematic_symbol(&f.model, provenance(), f.sheet, &second, None).unwrap();
    write.batch.operations.push(Operation::SetSchematicSymbol {
        sheet_id: f.sheet,
        symbol_id: first.uuid,
        symbol: first_raw,
    });
    write.batch.operations.push(Operation::CreateSchematicWire{sheet_id:f.sheet,wire_id:wire,
        wire:json!({"uuid":wire,"from":{"x":0,"y":0,"extension":"from"},"to":{"x":1000,"y":0,"extension":"to"},"extension":{"keep":"wire"}})});
    let write = prepare_native_birth(&f.model, write).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    f.symbol = first;
    (f, second.uuid, wire)
}
fn request(f: &BindingFixture, second: Uuid) -> SymbolMoveSetRequest {
    SymbolMoveSetRequest {
        symbols: vec![f.symbol.uuid, second],
        instance_path: vec![],
        reference: Point::new(500, 100),
        x_nm: 500,
        y_nm: 1100,
        rotation_deg: 90,
        mirrored: false,
    }
}
fn write(f: &BindingFixture, request: &SymbolMoveSetRequest) -> Result<PreparedWrite, EngineError> {
    move_symbol(
        &NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        provenance(),
        serde_json::to_value(request).unwrap(),
    )
}
fn install(f: &mut BindingFixture, id: Uuid, mutate: impl FnOnce(&mut Value)) {
    let mut symbol = raw(f)["symbols"][id.to_string()].clone();
    mutate(&mut symbol);
    let write = BatchComposer::compose(&f.model, provenance())
        .push_op(Operation::SetSchematicSymbol {
            sheet_id: f.sheet,
            symbol_id: id,
            symbol,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
}
#[test]
fn symbol_group_transform_connected_pair_shared_wire_raw_fields_and_atomic_history() {
    let (mut f, second, wire) = setup("symbol_group_history");
    let before = raw(&f);
    let identities = f.model.electrical_identities.clone();
    let members = electrical_test_partitions(&f.model, &[]).unwrap();
    let bindings = f.model.component_instances.clone();
    let journal = f.model.journal.len();
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let params = request(&f, second);
    let basis = SymbolMoveSetBasis::capture(&snapshot, &params.symbols, &[]).unwrap();
    let moved = basis
        .destination_with_transform(params.reference, Point::new(500, 1100), 90, false)
        .unwrap();
    let first = moved
        .symbols
        .iter()
        .find(|s| s.uuid == f.symbol.uuid)
        .unwrap();
    assert_eq!(first.position, Point::new(600, 600));
    assert_eq!(first.pins[0].position, Point::new(600, 600));
    assert_eq!(first.fields[0].position, Some(Point::new(400, 700)));
    assert_eq!(first.fields[1].position, Some(Point::new(300, 800)));
    assert_eq!(first.fields[2].position, None);
    assert!(!first.fields[1].visible);
    assert_eq!(moved.wires.len(), 1);
    assert_eq!(moved.wires[0].from, Point::new(600, 600));
    assert_eq!(moved.wires[0].to, Point::new(600, 1600));
    let prepared = write(&f, &params).unwrap();
    assert_eq!(
        prepared
            .batch
            .operations
            .iter()
            .filter(|o| matches!(o,Operation::DeleteSchematicWire{wire_id,..} if *wire_id==wire))
            .count(),
        1
    );
    assert_eq!(
        prepared
            .batch
            .operations
            .iter()
            .filter(|o| matches!(o,Operation::CreateSchematicWire{wire_id,..} if *wire_id==wire))
            .count(),
        1
    );
    assert_eq!(raw(&f), before);
    assert_eq!(
        f.model.journal.len(),
        journal,
        "preview/cancel has no journal write"
    );
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    assert_eq!(f.model.journal.len(), journal + 1);
    let after = raw(&f);
    assert_eq!(
        after["symbols"][f.symbol.uuid.to_string()]["extension"],
        before["symbols"][f.symbol.uuid.to_string()]["extension"]
    );
    assert_eq!(
        after["symbols"][f.symbol.uuid.to_string()]["pins"][0]["extension"],
        json!("pin")
    );
    assert_eq!(
        after["symbols"][f.symbol.uuid.to_string()]["fields"][0]["extension"],
        json!("field")
    );
    assert_eq!(
        after["symbols"][f.symbol.uuid.to_string()]["fields"][0]["position"]["extension"],
        json!("coordinate")
    );
    assert_eq!(
        after["wires"][wire.to_string()]["extension"],
        json!({"keep":"wire"})
    );
    assert_eq!(
        after["wires"][wire.to_string()]["from"]["extension"],
        json!("from")
    );
    assert_eq!(
        after["wires"][wire.to_string()]["to"]["extension"],
        json!("to")
    );
    assert_eq!(electrical_test_partitions(&f.model, &[]).unwrap(), members);
    assert_eq!(f.model.component_instances, bindings);
    assert_eq!(f.model.electrical_identities, identities);
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(raw(&f), after);
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(raw(&f), before);
    assert_eq!(f.model.electrical_identities, identities);
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(raw(&f), after);
    assert_eq!(electrical_test_partitions(&f.model, &[]).unwrap(), members);
    std::fs::remove_dir_all(f.root).unwrap();
}
#[test]
fn symbol_group_transform_mirror_uses_common_reference_and_composes_source_pose() {
    let (f, second, _) = setup("symbol_group_mirror");
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let basis = SymbolMoveSetBasis::capture(&snapshot, &[f.symbol.uuid, second], &[]).unwrap();
    let moved = basis
        .destination_with_transform(Point::new(500, 100), Point::new(500, 100), 0, true)
        .unwrap();
    let first = moved
        .symbols
        .iter()
        .find(|s| s.uuid == f.symbol.uuid)
        .unwrap();
    let second = moved.symbols.iter().find(|s| s.uuid == second).unwrap();
    assert_eq!(first.position, Point::new(1000, 0));
    assert_eq!(second.position, Point::zero());
    assert_eq!(first.mirrored, !f.symbol.mirrored);
    assert_eq!(second.rotation, 270);
    assert!(!second.mirrored);
    assert_eq!(moved.wires[0].from, Point::new(1000, 0));
    assert_eq!(moved.wires[0].to, Point::zero());
    let vertical = basis
        .destination_with_transform(Point::new(500, 100), Point::new(500, 100), 180, true)
        .unwrap();
    assert_eq!(
        vertical
            .symbols
            .iter()
            .find(|s| s.uuid == f.symbol.uuid)
            .unwrap()
            .position,
        Point::new(0, 200)
    );
    std::fs::remove_dir_all(f.root).unwrap();
}
#[test]
fn symbol_group_transform_complete_set_schema_lock_fixed_contact_and_overflow_refusals() {
    let (mut f, second, wire) = setup("symbol_group_refusals");
    let base = request(&f, second);
    let before = raw(&f);
    let tip = crate::revision::transaction_tip(&f.model.journal);
    let mut bad = serde_json::to_value(&base).unwrap();
    bad["symbol"] = json!(f.symbol.uuid);
    assert!(
        move_symbol(
            &NativeWriteContext {
                model: &f.model,
                project_root: &f.root
            },
            provenance(),
            bad
        )
        .is_err(),
        "absolute singleton and group delta shapes cannot mix"
    );
    for params in [
        SymbolMoveSetRequest {
            symbols: vec![],
            ..base.clone()
        },
        SymbolMoveSetRequest {
            symbols: vec![f.symbol.uuid, f.symbol.uuid],
            ..base.clone()
        },
        SymbolMoveSetRequest {
            symbols: vec![f.symbol.uuid, wire],
            ..base.clone()
        },
        SymbolMoveSetRequest {
            instance_path: vec![Uuid::new_v4()],
            ..base.clone()
        },
        SymbolMoveSetRequest {
            rotation_deg: 45,
            ..base.clone()
        },
        SymbolMoveSetRequest {
            x_nm: i64::MAX,
            y_nm: i64::MAX,
            ..base.clone()
        },
        SymbolMoveSetRequest {
            x_nm: base.reference.x,
            y_nm: base.reference.y,
            rotation_deg: 0,
            ..base.clone()
        },
    ] {
        assert!(write(&f, &params).is_err());
    }
    assert_eq!(raw(&f), before);
    assert_eq!(crate::revision::transaction_tip(&f.model.journal), tip);
    install(&mut f, second, |symbol| symbol["locked"] = json!(true));
    assert!(
        write(&f, &base)
            .unwrap_err()
            .to_string()
            .contains("locked Symbol")
    );
    install(&mut f, second, |symbol| symbol["locked"] = json!(false));
    for locked in [true, false] {
        let previous = raw(&f)["wires"][wire.to_string()].clone();
        let mut changed = previous.clone();
        changed["locked"] = json!(locked);
        let edit = BatchComposer::compose(&f.model, provenance())
            .push_op(Operation::DeleteSchematicWire {
                sheet_id: f.sheet,
                wire_id: wire,
                wire: previous,
            })
            .push_op(Operation::CreateSchematicWire {
                sheet_id: f.sheet,
                wire_id: wire,
                wire: changed,
            })
            .finish()
            .unwrap();
        commit_prepared(&mut f.model, &f.root, edit).unwrap();
        if locked {
            assert!(
                write(&f, &base)
                    .unwrap_err()
                    .to_string()
                    .contains("locked attached Wire")
            );
        }
    }
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: "fixed".into(),
        kind: LabelKind::Local,
        position: Point::new(500, 0),
    };
    let prepared = build_create_schematic_label(&f.model, provenance(), f.sheet, &label).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let before = raw(&f);
    let tip = crate::revision::transaction_tip(&f.model.journal);
    assert!(
        write(&f, &base).is_err(),
        "fixed interior attachment cannot be dragged away"
    );
    assert_eq!(raw(&f), before);
    assert_eq!(crate::revision::transaction_tip(&f.model.journal), tip);
    std::fs::remove_dir_all(f.root).unwrap();
}
#[test]
fn symbol_group_transform_stale_commit_cannot_partially_move_selection() {
    let (mut f, second, _) = setup("symbol_group_stale");
    let prepared = write(&f, &request(&f, second)).unwrap();
    install(&mut f, second, |symbol| {
        symbol["value"] = json!("changed later")
    });
    let before = raw(&f);
    let tip = crate::revision::transaction_tip(&f.model.journal);
    assert!(commit_prepared(&mut f.model, &f.root, prepared).is_err());
    assert_eq!(raw(&f), before);
    assert_eq!(crate::revision::transaction_tip(&f.model.journal), tip);
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn symbol_group_transform_repeated_source_refuses_complete_set_without_writes() {
    use crate::api::native_write::schematic_sheets::{
        build_create_schematic_definition, build_create_schematic_sheet_instance,
    };
    use crate::schematic::{SheetDefinition, SheetInstance};
    let (mut f, second, _) = setup("symbol_group_repeated");
    let definition = SheetDefinition {
        uuid: Uuid::new_v4(),
        root_sheet: f.sheet,
        name: "Repeated".into(),
    };
    let edit = build_create_schematic_definition(
        &f.model,
        provenance(),
        f.schematic,
        definition.uuid,
        "definitions/repeated.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, edit).unwrap();
    for n in 0..2 {
        let instance = SheetInstance {
            uuid: Uuid::new_v4(),
            definition: definition.uuid,
            parent_sheet: None,
            position: Point::new(n, 0),
            name: format!("I{n}"),
            ports: vec![],
        };
        let edit = build_create_schematic_sheet_instance(
            &f.model,
            provenance(),
            f.schematic,
            instance.uuid,
            serde_json::to_value(&instance).unwrap(),
        )
        .unwrap();
        commit_prepared(&mut f.model, &f.root, edit).unwrap();
    }
    let before = raw(&f);
    let tip = crate::revision::transaction_tip(&f.model.journal);
    assert!(write(&f, &request(&f, second)).is_err());
    assert_eq!(raw(&f), before);
    assert_eq!(crate::revision::transaction_tip(&f.model.journal), tip);
    std::fs::remove_dir_all(f.root).unwrap();
}
