//! Recorded native identities, canonical geometry history and read-only faults.
use super::*;
use crate::{
    api::native_write::{self as write, WriteProvenance, commit_prepared},
    ir::geometry::Point,
    schematic::{
        HiddenPowerBehavior, Junction, PinElectricalType, PlacedSymbol, SchematicWire,
        SymbolDisplayMode, SymbolPin,
    },
    substrate::{CommitSource, ElectricalSelectionSnapshot, ProjectResolver},
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "recorded native net query")
}
fn source(model: &DesignModel) -> Schematic {
    ElectricalSelectionSnapshot::capture(model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .clone()
}
fn bytes(root: &Path, model: &DesignModel) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut paths: Vec<_> = model.source_shards.iter().map(|s| s.path.clone()).collect();
    paths.extend([
        crate::substrate::transaction_journal_path(root),
        root.join(".datum/journal/cursor.json"),
    ]);
    paths
        .into_iter()
        .filter(|p| p.is_file())
        .map(|p| {
            let b = std::fs::read(&p).unwrap();
            (p, b)
        })
        .collect()
}
struct Fixture {
    root: PathBuf,
    model: DesignModel,
    sheet: Uuid,
    junction: Uuid,
    pin: Uuid,
}
fn fixture(adopt: bool) -> Fixture {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.datum/scratch/codex-delivery/vbd3-engine-tests")
        .join(Uuid::new_v4().to_string());
    let ids = write::genesis::bootstrap_native_project(
        &root,
        write::genesis::GenesisSpec {
            project_name: "native net identity".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let sheet = Uuid::new_v4();
    let p = write::schematic_sheets::build_create_schematic_sheet(&model,provenance(),ids.schematic_uuid,sheet,"sheets/main.json",
        serde_json::json!({"schema_version":1,"uuid":sheet,"name":"Main","symbols":{},"wires":{},"junctions":{},"labels":{},"ports":{},"buses":{},"bus_entries":{},"noconnects":{},"texts":{},"drawings":{}})).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let pin = Uuid::new_v4();
    let symbol = PlacedSymbol {
        uuid: Uuid::new_v4(),
        part: None,
        entity: None,
        gate: None,
        lib_id: None,
        reference: "J1".into(),
        value: "terminal".into(),
        fields: vec![],
        pins: vec![SymbolPin {
            uuid: pin,
            library_pin: None,
            number: "1".into(),
            name: "signal".into(),
            electrical_type: PinElectricalType::Passive,
            position: Point::new(10_000_000, 0),
        }],
        position: Point::new(10_000_000, 0),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![],
        hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
    };
    let p = write::schematic_symbols::build_place_schematic_symbol(
        &model,
        provenance(),
        sheet,
        &symbol,
        None,
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let wire = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(0, 0),
        to: Point::new(20_000_000, 0),
    };
    let p = write::schematic_connectivity::build_create_schematic_wire(
        &model,
        provenance(),
        sheet,
        &wire,
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let junction = Junction {
        uuid: Uuid::new_v4(),
        position: wire.from,
    };
    let p = write::schematic_connectivity::build_create_schematic_junction(
        &model,
        provenance(),
        sheet,
        &junction,
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    if adopt {
        let p =
            write::electrical::build_adopt_schematic_net_identities(&model, provenance()).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
    }
    let model = ProjectResolver::new(&root).resolve().unwrap();
    println!("native net query fixture {}", root.display());
    Fixture {
        root,
        model,
        sheet,
        junction: junction.uuid,
        pin,
    }
}

#[test]
fn recorded_net_without_declarations_survives_canonical_geometry_history() {
    let mut f = fixture(true);
    let before = source(&f.model);
    assert!(
        before
            .sheets
            .values()
            .all(|s| s.net_declarations.is_empty())
    );
    let groups = crate::connectivity::partitions(&before).unwrap();
    assert_eq!(groups.len(), 1);
    assert!(
        groups[0]
            .iter()
            .any(|m| m.class == "pins" && m.source_id == f.pin)
    );
    let owners = f.model.electrical_identities.clone();
    let id = *owners.keys().next().unwrap();
    let disk = bytes(&f.root, &f.model);
    let nets = native_schematic_net_info(&f.model, &before).unwrap();
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0].uuid, id);
    assert_eq!(
        nets[0].name,
        format!("N${}", &id.as_simple().to_string()[..8])
    );
    assert_eq!(
        nets[0].pins.iter().map(|p| p.uuid).collect::<Vec<_>>(),
        vec![f.pin]
    );
    assert_eq!(
        bytes(&f.root, &f.model),
        disk,
        "query writes no sources or history"
    );
    let context = write::registry::NativeWriteContext {
        model: &f.model,
        project_root: &f.root,
    };
    let p = (write::registry::find_native_write_verb("datum.schematic.move")
        .unwrap()
        .build)(
        &context,
        provenance(),
        serde_json::json!({"object":f.junction,"instance_path":[],"x_nm":5_000_000,"y_nm":0}),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let after = source(&f.model);
    assert_ne!(
        before.sheets[&f.sheet].junctions,
        after.sheets[&f.sheet].junctions
    );
    assert_eq!(crate::connectivity::partitions(&after).unwrap(), groups);
    assert_eq!(f.model.electrical_identities, owners);
    assert_ne!(
        crate::connectivity::schematic_net_info(&before)[0].uuid,
        crate::connectivity::schematic_net_info(&after)[0].uuid,
        "control reproduces the legacy coordinate-ID change"
    );
    let expected = serde_json::to_value(&nets).unwrap();
    let disk = bytes(&f.root, &f.model);
    assert_eq!(
        serde_json::to_value(native_schematic_net_info(&f.model, &after).unwrap()).unwrap(),
        expected
    );
    assert_eq!(bytes(&f.root, &f.model), disk);
    for redo in [false, true] {
        if redo {
            f.model
                .commit_journal_redo(&f.root, provenance().into())
                .unwrap();
        } else {
            f.model
                .commit_journal_undo(&f.root, provenance().into())
                .unwrap();
        }
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        let current = source(&f.model);
        assert_eq!(current, if redo { after.clone() } else { before.clone() });
        assert_eq!(crate::connectivity::partitions(&current).unwrap(), groups);
        assert_eq!(
            serde_json::to_value(native_schematic_net_info(&f.model, &current).unwrap()).unwrap(),
            expected
        );
    }
}

#[test]
fn genuine_legacy_without_recorded_net_authority_keeps_existing_projection() {
    let f = fixture(false);
    let source = source(&f.model);
    assert!(f.model.electrical_identities.is_empty());
    assert_eq!(
        serde_json::to_value(native_schematic_net_info(&f.model, &source).unwrap()).unwrap(),
        serde_json::to_value(crate::connectivity::schematic_net_info(&source)).unwrap()
    );
}

#[test]
fn malformed_retired_foreign_or_ambiguous_input_never_falls_back_to_coordinate_ids() {
    let f = fixture(true);
    let source = source(&f.model);
    let disk = bytes(&f.root, &f.model);
    // Fault-inject only isolated query inputs. No source/history is forged or
    // committed; the positive fixture above uses canonical writers throughout.
    for fault in ["retired", "foreign", "ambiguous"] {
        let mut model = f.model.clone();
        let mut record = model.electrical_identities.values().next().unwrap().clone();
        match fault {
            "retired" => {
                if let ElectricalIdentity::Net { retired, .. } = &mut record.identity {
                    *retired = true;
                }
            }
            "foreign" => {
                if let ElectricalIdentity::Net { anchor, .. } = &mut record.identity {
                    anchor.instance_path = vec![Uuid::new_v4()];
                }
            }
            "ambiguous" => record.id = Uuid::new_v4(),
            _ => unreachable!(),
        }
        model.electrical_identities.insert(record.id, record);
        assert!(
            native_schematic_net_info(&model, &source).is_err(),
            "{fault}"
        );
        assert_eq!(bytes(&f.root, &f.model), disk);
    }
    let mut retired = f.model.clone();
    let mut old = retired
        .electrical_identities
        .values()
        .next()
        .unwrap()
        .clone();
    old.id = Uuid::new_v4();
    if let ElectricalIdentity::Net { retired, .. } = &mut old.identity {
        *retired = true;
    }
    retired.electrical_identities.insert(old.id, old);
    assert_eq!(
        serde_json::to_value(native_schematic_net_info(&retired, &source).unwrap()).unwrap(),
        serde_json::to_value(native_schematic_net_info(&f.model, &source).unwrap()).unwrap(),
        "unrelated retired records do not displace a unique active owner"
    );
    let mut empty = source;
    empty.sheets.clear();
    assert!(
        native_schematic_net_info(&retired, &empty)
            .unwrap()
            .is_empty()
    );
}
