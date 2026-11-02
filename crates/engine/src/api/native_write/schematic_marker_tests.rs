use super::*;
use crate::{
    api::native_write::{
        commit_prepared, electrical::topology_tests::fixture, registry::find_native_write_verb,
        schematic_connectivity::build_create_schematic_wire,
        schematic_symbols::build_place_schematic_symbol,
    },
    connectivity::schematic_net_info,
    erc::run_prechecks,
    schematic::{
        HiddenPowerBehavior, PinElectricalType, PlacedSymbol, Schematic, SchematicWire,
        SymbolDisplayMode, SymbolPin,
    },
    substrate::{CommitSource, DesignModel, Operation, ProjectResolver},
};
use std::path::Path;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "native schematic marker")
}
fn source(model: &DesignModel) -> Schematic {
    ElectricalSelectionSnapshot::capture(model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .clone()
}
fn build(
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
fn place_params(sheet: Uuid, kind: &str) -> serde_json::Value {
    serde_json::json!({"sheet": sheet, "kind": kind, "x_nm":0, "y_nm":0, "symbol":null, "pin":null})
}
fn add_pin(model: &mut DesignModel, root: &Path, sheet: Uuid) -> PlacedSymbol {
    let symbol = PlacedSymbol {
        uuid: Uuid::new_v4(),
        part: None,
        entity: None,
        gate: None,
        lib_id: None,
        reference: "R1".into(),
        value: "10k".into(),
        fields: vec![],
        pins: vec![SymbolPin {
            uuid: Uuid::new_v4(),
            library_pin: None,
            number: "1".into(),
            name: "~".into(),
            electrical_type: PinElectricalType::Passive,
            position: Point::zero(),
        }],
        position: Point::zero(),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![],
        hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
    };
    let write = build_place_schematic_symbol(model, provenance(), sheet, &symbol, None).unwrap();
    let write = prepare_native_birth(model, write).unwrap();
    commit_prepared(model, root, write).unwrap();
    symbol
}
fn has_code(model: &DesignModel, code: &str) -> bool {
    run_prechecks(&source(model)).iter().any(|f| f.code == code)
}
fn history(model: &mut DesignModel, root: &Path, redo: bool) {
    if redo {
        model
            .commit_journal_redo(root, provenance().into())
            .unwrap();
    } else {
        model
            .commit_journal_undo(root, provenance().into())
            .unwrap();
    }
    *model = ProjectResolver::new(root).resolve().unwrap();
}

#[test]
fn native_marker_noconnect_suppresses_open_pin_and_history_restores_warning() {
    let (root, mut model, _, sheet) = fixture("marker_nc_semantics");
    let symbol = add_pin(&mut model, &root, sheet);
    assert!(has_code(&model, "unconnected_component_pin"));
    let before = source(&model);
    let write = build(
        &model,
        &root,
        "datum.schematic.place_marker",
        place_params(sheet, "noconnect"),
    )
    .unwrap();
    let id = write.primary_object_id.unwrap();
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap()),
        before,
        "prepare/cancel does not persist"
    );
    assert!(write.batch.operations.iter().any(|op| matches!(
        op,
        Operation::PlaceSchematicMarker {
            marker_kind: SchematicMarkerKind::NoConnect,
            ..
        }
    )));
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let marked = source(&model);
    assert_eq!(marked.sheets[&sheet].noconnects[&id].symbol, symbol.uuid);
    assert_eq!(
        marked.sheets[&sheet].noconnects[&id].pin,
        symbol.pins[0].uuid
    );
    assert!(!has_code(&model, "unconnected_component_pin"));
    assert!(!has_code(&model, "noconnect_connected"));
    history(&mut model, &root, false);
    assert_eq!(source(&model), before);
    assert!(has_code(&model, "unconnected_component_pin"));
    history(&mut model, &root, true);
    assert_eq!(source(&model), marked);
    assert!(!has_code(&model, "unconnected_component_pin"));
    let write = build(
        &model,
        &root,
        "datum.schematic.delete_noconnect",
        serde_json::json!({"noconnect":id,"instance_path":[]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(has_code(&model, "unconnected_component_pin"));
    history(&mut model, &root, false);
    assert_eq!(source(&model), marked);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_marker_junction_merges_crossing_and_history_separates_nets() {
    let (root, mut model, _, sheet) = fixture("marker_junction_semantics");
    for (from, to) in [
        (Point::new(-100, 0), Point::new(100, 0)),
        (Point::new(0, -100), Point::new(0, 100)),
    ] {
        let wire = SchematicWire {
            uuid: Uuid::new_v4(),
            from,
            to,
        };
        let write = build_create_schematic_wire(&model, provenance(), sheet, &wire).unwrap();
        let write = prepare_native_birth(&model, write).unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
    }
    assert_eq!(schematic_net_info(&source(&model)).len(), 2);
    let before = source(&model);
    let write = build(
        &model,
        &root,
        "datum.schematic.place_marker",
        place_params(sheet, "junction"),
    )
    .unwrap();
    let id = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let marked = source(&model);
    assert_eq!(schematic_net_info(&marked).len(), 1);
    history(&mut model, &root, false);
    assert_eq!(source(&model), before);
    assert_eq!(schematic_net_info(&source(&model)).len(), 2);
    history(&mut model, &root, true);
    assert_eq!(source(&model), marked);
    assert_eq!(schematic_net_info(&source(&model)).len(), 1);
    let write = build(
        &model,
        &root,
        "datum.schematic.delete_junction",
        serde_json::json!({"junction":id,"instance_path":[]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(schematic_net_info(&source(&model)).len(), 2);
    history(&mut model, &root, false);
    assert_eq!(source(&model), marked);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_marker_aliases_refuse_duplicate_mismatched_and_unqualified_targets() {
    let (root, mut model, _, sheet) = fixture("marker_admission");
    let symbol = add_pin(&mut model, &root, sheet);
    let mut nc = place_params(sheet, "noconnect");
    nc.as_object_mut().unwrap().remove("kind");
    nc["symbol"] = serde_json::json!(symbol.uuid);
    nc["pin"] = serde_json::json!(symbol.pins[0].uuid);
    let mut stacked = symbol.clone();
    stacked.uuid = Uuid::new_v4();
    stacked.reference = "R2".into();
    stacked.pins[0].uuid = Uuid::new_v4();
    let write = build_place_schematic_symbol(&model, provenance(), sheet, &stacked, None).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(build(&model, &root, "datum.schematic.place_noconnect", nc.clone()).is_err());
    assert!(
        build(
            &model,
            &root,
            "datum.schematic.place_marker",
            place_params(sheet, "noconnect")
        )
        .is_err()
    );
    history(&mut model, &root, false);
    for (field, value) in [
        ("pin", serde_json::json!(Uuid::new_v4())),
        ("symbol", serde_json::Value::Null),
        ("x_nm", serde_json::json!(1)),
    ] {
        let mut bad = nc.clone();
        bad[field] = value;
        assert!(build(&model, &root, "datum.schematic.place_noconnect", bad).is_err());
    }
    let write = build(&model, &root, "datum.schematic.place_noconnect", nc.clone()).unwrap();
    let nc_id = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(build(&model, &root, "datum.schematic.place_noconnect", nc).is_err());
    assert!(
        build(
            &model,
            &root,
            "datum.schematic.delete_noconnect",
            serde_json::json!({"noconnect":nc_id,"instance_path":[Uuid::new_v4()]})
        )
        .is_err()
    );
    let mut junction = place_params(sheet, "junction");
    junction.as_object_mut().unwrap().remove("kind");
    let write = build(
        &model,
        &root,
        "datum.schematic.place_junction",
        junction.clone(),
    )
    .unwrap();
    let junction_id = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(build(&model, &root, "datum.schematic.place_junction", junction).is_err());
    assert!(
        build(
            &model,
            &root,
            "datum.schematic.place_junction",
            place_params(sheet, "noconnect")
        )
        .is_err()
    );
    assert!(
        build(
            &model,
            &root,
            "datum.schematic.delete_junction",
            serde_json::json!({"junction":junction_id,"instance_path":[Uuid::new_v4()]})
        )
        .is_err()
    );
    let before = source(&model);
    let revision = model.model_revision.clone();
    let mut no_pin = place_params(sheet, "noconnect");
    no_pin["x_nm"] = serde_json::json!(100);
    assert!(build(&model, &root, "datum.schematic.place_marker", no_pin).is_err());
    assert_eq!(model.model_revision, revision);
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap()),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}
