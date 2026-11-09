//! Actual native query subprocesses retain authored identity and write nothing.
use eda_engine::{
    api::native_write::{self as write, WriteProvenance, commit_prepared},
    ir::geometry::Point,
    schematic::{
        HiddenPowerBehavior, Junction, PinElectricalType, PlacedSymbol, Schematic, SchematicWire,
        SymbolDisplayMode, SymbolPin,
    },
    substrate::{
        CommitSource, DesignModel, ElectricalIdentity, ElectricalSelectionSnapshot, ProjectResolver,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::Command,
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
        eda_engine::substrate::transaction_journal_path(root),
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
        .join("../../.datum/scratch/codex-delivery/vbd3-cli-tests")
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

fn query(f: &Fixture) -> serde_json::Value {
    let before = bytes(&f.root, &f.model);
    let output = Command::new(env!("CARGO_BIN_EXE_datum-eda"))
        .args(["--format", "json", "query", "schematic-nets"])
        .arg(&f.root)
        .env_remove("DATUM_COMMIT_SOURCE")
        .output()
        .unwrap();
    println!(
        "schematic-nets {} exit={}\nstdout={}\nstderr={}",
        f.root.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        bytes(&f.root, &f.model),
        before,
        "CLI query must not write sources or journal"
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn actual_native_cli_reports_recorded_nets_across_geometry_history_without_writes() {
    let mut f = fixture(true);
    let before = source(&f.model);
    assert!(
        before
            .sheets
            .values()
            .all(|s| s.net_declarations.is_empty())
    );
    let ids: Vec<_> = f
        .model
        .electrical_identities
        .values()
        .filter(|r| matches!(r.identity, ElectricalIdentity::Net { retired: false, .. }))
        .map(|r| r.id)
        .collect();
    assert_eq!(ids.len(), 1);
    let identities = f.model.electrical_identities.clone();
    let expected = query(&f);
    let nets = expected
        .as_array()
        .expect("public schematic-nets query returns its existing array schema");
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0]["uuid"], ids[0].to_string());
    assert_eq!(
        nets[0]["name"],
        format!("N${}", &ids[0].as_simple().to_string()[..8])
    );
    assert_eq!(nets[0]["pins"].as_array().unwrap().len(), 1);
    assert_eq!(nets[0]["pins"][0]["uuid"], f.pin.to_string());
    assert_eq!(
        query(&f),
        expected,
        "cold repeated query reuses recorded identity without adoption"
    );
    let p = (write::registry::find_native_write_verb("datum.schematic.move")
        .unwrap()
        .build)(
        &write::registry::NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        provenance(),
        serde_json::json!({"object":f.junction,"instance_path":[],"x_nm":5_000_000,"y_nm":0}),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let after = source(&f.model);
    assert_ne!(
        after.sheets[&f.sheet].junctions,
        before.sheets[&f.sheet].junctions
    );
    assert_eq!(
        after.sheets[&f.sheet].symbols,
        before.sheets[&f.sheet].symbols
    );
    assert_eq!(f.model.electrical_identities, identities);
    assert_eq!(query(&f), expected);
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
        assert_eq!(
            source(&f.model),
            if redo { after.clone() } else { before.clone() }
        );
        assert_eq!(query(&f), expected);
    }
    println!(
        "native CLI identity {} preserved across move/undo/redo at {}",
        ids[0],
        f.root.display()
    );
}

// The portable fixture above runs everywhere. This separate real-circuit case
// requires the preserved, machine-local canonical input copy and is never
// counted as executed by a default run that lacks that evidence.
#[test]
#[ignore = "requires DATUM_VBD3_OWNER_INPUT containing the retained canonical owner-circuit copy"]
fn retained_owner_circuit_query_ids_survive_the_recorded_junction_move() {
    let input = PathBuf::from(
        std::env::var_os("DATUM_VBD3_OWNER_INPUT").expect("explicit retained input required"),
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.datum/scratch/codex-delivery/vbd3-owner-cli-tests")
        .join(Uuid::new_v4().to_string());
    fn copy(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let kind = entry.file_type().unwrap();
            assert!(
                !kind.is_symlink(),
                "canonical input must not redirect source"
            );
            if kind.is_dir() {
                copy(&entry.path(), &to.join(entry.file_name()));
            } else {
                assert!(kind.is_file());
                std::fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
            }
        }
    }
    copy(&input, &root);
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let sheet = "ef48f5e2-eec6-4345-bc2b-0f3fffcb8c28"
        .parse::<Uuid>()
        .unwrap();
    let junction = "54bf64cc-59e4-44ea-852a-f921c765a689"
        .parse::<Uuid>()
        .unwrap();
    let loaded = source(&model);
    assert_eq!(
        loaded.sheets[&sheet].junctions[&junction].position,
        Point::new(-2_400_000, 700_000)
    );
    let pin = loaded.sheets[&sheet]
        .symbols
        .values()
        .flat_map(|s| s.pins.iter())
        .next()
        .unwrap()
        .uuid;
    let mut f = Fixture {
        root,
        model,
        sheet,
        junction,
        pin,
    };
    // The canonical input's applied final transaction is the proven original
    // move. Undo it using actual history, never by rewriting source or cursor.
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let before = source(&f.model);
    assert!(
        before
            .sheets
            .values()
            .all(|s| s.net_declarations.is_empty())
    );
    assert_eq!(
        before.sheets[&sheet].junctions[&junction].position,
        Point::new(-2_400_000, 1_200_000)
    );
    assert_eq!(f.model.electrical_identities.len(), 21);
    let ids = f
        .model
        .electrical_identities
        .values()
        .filter(|r| matches!(r.identity, ElectricalIdentity::Net { retired: false, .. }))
        .map(|r| r.id)
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), 5);
    let identities = f.model.electrical_identities.clone();
    let expected = query(&f);
    let nets = expected.as_array().unwrap();
    assert_eq!(nets.len(), 5);
    let actual = nets
        .iter()
        .map(|n| n["uuid"].as_str().unwrap().parse::<Uuid>().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual, ids,
        "query must use all five recorded active identities"
    );
    assert_eq!(
        nets.iter()
            .map(|n| n["pins"].as_array().unwrap().len())
            .sum::<usize>(),
        9
    );
    let p = (write::registry::find_native_write_verb("datum.schematic.move")
        .unwrap()
        .build)(
        &write::registry::NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        provenance(),
        serde_json::json!({"object":junction,"instance_path":[],"x_nm":-2_400_000,"y_nm":700_000}),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let after = source(&f.model);
    assert_ne!(
        after.sheets[&sheet].junctions,
        before.sheets[&sheet].junctions
    );
    assert_eq!(f.model.electrical_identities, identities);
    assert_eq!(
        query(&f),
        expected,
        "recorded IDs/names and all complete pin memberships must remain stable"
    );
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
        assert_eq!(
            source(&f.model),
            if redo { after.clone() } else { before.clone() }
        );
        assert_eq!(query(&f), expected);
    }
    println!(
        "retained owner CLI witness: five recorded Nets/nine pins stable, no query writes; {}",
        f.root.display()
    );
}
