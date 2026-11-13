//! Actual native projection of one child definition used twice.
use crate::{
    api::native_write::{self as write, WriteProvenance, commit_prepared},
    connectivity::native_schematic_net_info,
    erc::{ErcConfig, ErcFinding, run_prechecks_with_nets},
    ir::geometry::Point,
    schematic::{
        CheckDomain, CheckWaiver, HiddenPowerBehavior, LabelKind, NetLabel, PinElectricalType,
        PlacedSymbol, Schematic, SymbolDisplayMode, SymbolPin, WaiverTarget,
    },
    substrate::{CommitSource, DesignModel, ElectricalSelectionSnapshot, ProjectResolver},
};
use std::path::PathBuf;
use uuid::Uuid;

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "occurrence-test",
        CommitSource::Test,
        "Repeated definition ERC",
    )
}
fn source(model: &DesignModel) -> Schematic {
    ElectricalSelectionSnapshot::capture(model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .clone()
}
fn conflicts(model: &DesignModel, waivers: &[CheckWaiver]) -> Vec<ErcFinding> {
    let source = source(model);
    run_prechecks_with_nets(
        &source,
        &ErcConfig::default(),
        waivers,
        native_schematic_net_info(model, &source).unwrap(),
    )
    .into_iter()
    .filter(|f| f.code == "output_to_output_conflict")
    .collect()
}
fn fixture() -> (PathBuf, DesignModel, Uuid, [Uuid; 2]) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.datum/scratch/erc-occurrence-tests")
        .join(Uuid::new_v4().to_string());
    let ids = write::genesis::bootstrap_native_project(
        &root,
        write::genesis::GenesisSpec {
            project_name: "Repeated definition ERC".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let sheet = Uuid::new_v4();
    let prepared = write::schematic_sheets::build_create_schematic_sheet(
        &model,
        provenance(),
        ids.schematic_uuid,
        sheet,
        "sheets/child.json",
        serde_json::json!({
            "schema_version":1,"uuid":sheet,"name":"Child","symbols":{},"wires":{},
            "junctions":{},"labels":{},"ports":{},"buses":{},"bus_entries":{},
            "noconnects":{},"texts":{},"drawings":{}
        }),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    // Author one source before repetition; both instances retain these pin UUIDs.
    for reference in ["U1", "U2"] {
        let symbol = PlacedSymbol {
            uuid: Uuid::new_v4(),
            part: None,
            entity: None,
            gate: None,
            lib_id: None,
            reference: reference.into(),
            value: "output".into(),
            fields: vec![],
            pins: vec![SymbolPin {
                uuid: Uuid::new_v4(),
                library_pin: None,
                number: "1".into(),
                name: "OUT".into(),
                electrical_type: PinElectricalType::Output,
                position: Point::new(0, 0),
            }],
            position: Point::new(0, 0),
            rotation: 0,
            mirrored: false,
            unit_selection: None,
            display_mode: SymbolDisplayMode::LibraryDefault,
            pin_overrides: vec![],
            hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
        };
        let prepared = write::schematic_symbols::build_place_schematic_symbol(
            &model,
            provenance(),
            sheet,
            &symbol,
            None,
        )
        .unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
    }
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        kind: LabelKind::Local,
        name: "SIG".into(),
        position: Point::new(0, 0),
    };
    let prepared = write::schematic_connectivity::build_create_schematic_label(
        &model,
        provenance(),
        sheet,
        &label,
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let definition = Uuid::new_v4();
    let prepared = write::schematic_sheets::build_create_schematic_definition(
        &model,
        provenance(),
        ids.schematic_uuid,
        definition,
        "definitions/child.json",
        serde_json::json!({
            "schema_version":1,"uuid":definition,"root_sheet":sheet,"name":"Repeated Child"
        }),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let instances = [Uuid::new_v4(), Uuid::new_v4()];
    for instance in instances {
        let prepared = write::schematic_sheets::build_create_schematic_sheet_instance(
            &model,
            provenance(),
            ids.schematic_uuid,
            instance,
            serde_json::json!({
                "uuid":instance,"definition":definition,"parent_sheet":null,
                "position":{"x":0,"y":0},"name":"Repeated","ports":[]
            }),
        )
        .unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
    }
    // Raw symbol builders do not adopt scalar identities as public authoring does.
    let prepared =
        write::electrical::build_adopt_schematic_net_identities(&model, provenance()).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let nets = native_schematic_net_info(&model, &source(&model)).unwrap();
    assert_eq!(nets.len(), 2);
    assert_ne!(nets[0].uuid, nets[1].uuid);
    assert!(nets.iter().all(|net| !net.occurrences.is_empty()));
    (root, model, sheet, instances)
}

#[test]
fn repeated_source_conflicts_keep_qualified_identity_and_waive_only_one_occurrence() {
    let (root, model, sheet, instances) = fixture();
    let before = conflicts(&model, &[]);
    assert_eq!(before.len(), 2);
    assert_eq!(before[0].net_name.as_deref(), Some("SIG"));
    assert_eq!(before[0].object_uuids, before[1].object_uuids);
    assert_ne!(before[0].net_uuid, before[1].net_uuid);
    assert_ne!(before[0].id, before[1].id);
    assert_ne!(before[0].fingerprint, before[1].fingerprint);
    for finding in &before {
        let path = &finding.occurrences[0].instance_path;
        assert!(instances.iter().any(|id| path == &vec![*id]));
        assert_eq!(finding.occurrences.len(), 2);
        assert_eq!(finding.pin_evidence.len(), 2);
        for pin in &finding.pin_evidence {
            assert_eq!(&pin.instance_path, path);
            assert!(finding.object_uuids.contains(&pin.pin_uuid));
            assert_eq!(pin.canonical_pin_type, "output");
        }
    }
    let waiver = CheckWaiver {
        uuid: Uuid::new_v4(),
        domain: CheckDomain::ERC,
        target: WaiverTarget::Fingerprint(before[0].fingerprint.clone().unwrap()),
        rationale: "Only this occurrence".into(),
        created_by: None,
    };
    let checked = conflicts(&model, std::slice::from_ref(&waiver));
    assert_eq!(checked.iter().filter(|f| f.waived).count(), 1);
    assert_eq!(checked.iter().find(|f| f.waived).unwrap().id, before[0].id);

    // Presentation changes keep native identity and electrical observations.
    let mut renamed = source(&model);
    renamed.sheets.get_mut(&sheet).unwrap().name = "Renamed Child".into();
    for symbol in renamed.sheets.get_mut(&sheet).unwrap().symbols.values_mut() {
        symbol.reference.push_str("_renamed");
    }
    for label in renamed.sheets.get_mut(&sheet).unwrap().labels.values_mut() {
        label.name = "RENAMED_SIG".into();
    }
    for instance in renamed.sheet_instances.values_mut() {
        instance.name = "Renamed".into();
    }
    let nets = native_schematic_net_info(&model, &renamed).unwrap();
    let renamed_findings = run_prechecks_with_nets(
        &renamed,
        &ErcConfig::default(),
        std::slice::from_ref(&waiver),
        nets,
    );
    for original in &before {
        let current = renamed_findings
            .iter()
            .find(|f| f.id == original.id)
            .unwrap();
        assert_eq!(current.fingerprint, original.fingerprint);
        assert_eq!(current.waived, original.id == before[0].id);
    }
    // An electrical change retaining the conflict must reopen the waived finding.
    renamed
        .sheets
        .get_mut(&sheet)
        .unwrap()
        .symbols
        .values_mut()
        .next()
        .unwrap()
        .pins[0]
        .electrical_type = PinElectricalType::PowerOut;
    let nets = native_schematic_net_info(&model, &renamed).unwrap();
    let changed = run_prechecks_with_nets(&renamed, &ErcConfig::default(), &[waiver], nets);
    assert_eq!(
        changed
            .iter()
            .filter(|f| f.code == "output_to_output_conflict")
            .count(),
        2
    );
    for current in changed
        .iter()
        .filter(|f| f.code == "output_to_output_conflict")
    {
        let original = before.iter().find(|f| f.id == current.id).unwrap();
        assert_ne!(current.fingerprint, original.fingerprint);
        assert!(!current.waived);
    }
    std::fs::remove_dir_all(root).unwrap();
}
