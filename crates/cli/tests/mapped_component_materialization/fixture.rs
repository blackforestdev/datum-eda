//! Subprocess manual choices over the governed native library fixture.
use eda_engine::{
    api::native_write::{self as write, WriteProvenance, commit_prepared},
    substrate::{CommitSource, DesignModel, ProjectResolver, SourceShardKind},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use uuid::Uuid;

pub(super) const PART: &str = "90000000-0000-4000-8000-000000000001";
pub(super) const PACKAGE: &str = "50000000-0000-4000-8000-000000000001";
pub(super) const MAP: &str = "a0000000-0000-4000-8000-000000000001";
pub(super) const FOOTPRINT: &str = "60000000-0000-4000-8000-000000000001";
pub(super) fn id(value: &str) -> Uuid {
    value.parse().unwrap()
}
pub(super) fn provenance() -> WriteProvenance {
    WriteProvenance::new("fixture", CommitSource::Manual, "native fixture seed")
}
pub(super) fn model(root: &Path) -> DesignModel {
    ProjectResolver::new(root).resolve().unwrap()
}
pub(super) fn board(model: &DesignModel) -> Value {
    model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap()
}
pub(super) fn raw(model: &DesignModel, object: Uuid) -> Value {
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == model.objects[&object].source_shard_id)
        .unwrap();
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()
}
pub(super) fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}
pub(super) fn fixture() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.datum/scratch/codex-delivery/mapped-cli-tests")
        .join(Uuid::new_v4().to_string());
    copy_tree(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../test-harness/testdata/library/native_authored_baseline_v1"),
        &root,
    );
    // Complete the governed empty board for the CLI read model before resolution.
    let path = root.join("board/board.json");
    let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    value["stackup"] = json!({"layers":[]});
    value["outline"] = json!({"vertices":[],"closed":true});
    value["pads"] = json!({});
    for key in ["rules", "keepouts", "dimensions", "texts"] {
        value[key] = json!([]);
    }
    for key in [
        "component_silkscreen",
        "component_silkscreen_texts",
        "component_silkscreen_arcs",
        "component_silkscreen_circles",
        "component_silkscreen_polygons",
        "component_silkscreen_polylines",
        "component_mechanical_lines",
        "component_mechanical_texts",
        "component_mechanical_polygons",
        "component_mechanical_polylines",
        "component_mechanical_circles",
        "component_mechanical_arcs",
        "component_pads",
        "component_models_3d",
    ] {
        value[key] = json!({});
    }
    std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    println!("native CLI fixture {}", root.display());
    success(cli(&[
        "project".into(),
        "add-default-top-stackup".into(),
        root.display().to_string(),
    ]));
    root
}
pub(super) fn cli(args: &[String]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_datum-eda"))
        .args(["--format", "json"])
        .args(args)
        .env_remove("DATUM_COMMIT_SOURCE")
        .output()
        .unwrap();
    println!("CLI {:?}: {}", args, output.status);
    output
}
pub(super) fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
pub(super) fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in std::fs::read_dir(root).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                visit(&p, out);
            } else {
                out.insert(p.clone(), std::fs::read(p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, &mut out);
    out
}
pub(super) fn refuse(root: &Path, args: &[String]) {
    let before = files(root);
    let output = cli(args);
    assert!(!output.status.success(), "unexpected success {:?}", args);
    assert_eq!(
        files(root),
        before,
        "refusal must not write canonical sources or journal"
    );
}
pub(super) fn create(root: &Path, kind: &str, object: Value) -> Uuid {
    use write::library::{
        self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec,
    };
    let mut m = model(root);
    let uuid = serde_json::from_value(object["uuid"].clone()).unwrap();
    let p = library::build_pool_library_write(
        &m,
        provenance(),
        Some("pool"),
        vec![Spec::Create {
            target: Target::new("pool", kind, uuid),
            object,
        }],
    )
    .unwrap();
    commit_prepared(&mut m, root, p).unwrap();
    uuid
}
pub(super) fn seed(root: &Path, index: i64) -> (Uuid, Uuid) {
    use write::{
        board_components::{BoardPackagePlacement, build_place_board_package},
        registry::{NativeWriteContext, find_native_write_verb},
        schematic_symbols::build_set_schematic_symbol,
    };
    let mut m = model(root);
    let sheet = id("10000000-0000-4000-8000-000000000004");
    let prepared = (find_native_write_verb("datum.schematic.place_symbol")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &m,
            project_root: root,
        },
        provenance(),
        json!({"sheet":sheet,"library_symbol":"30000000-0000-4000-8000-000000000001",
            "reference":format!("U{index}"),"value":"native","x_nm":index*40_000_000,"y_nm":0}),
    )
    .unwrap();
    let symbol = prepared.primary_object_id.unwrap();
    commit_prepared(&mut m, root, prepared).unwrap();
    let mut placed: eda_engine::schematic::PlacedSymbol =
        serde_json::from_value(raw(&m, symbol)["symbols"][symbol.to_string()].clone()).unwrap();
    placed.part = Some(id(PART));
    placed.entity = Some(id("40000000-0000-4000-8000-000000000001"));
    placed.gate = Some(id("40000000-0000-4000-8000-000000000101"));
    let p = build_set_schematic_symbol(&m, provenance(), sheet, &placed).unwrap();
    commit_prepared(&mut m, root, p).unwrap();
    let package = eda_engine::board::PlacedPackage {
        uuid: Uuid::new_v4(),
        part: id(PART),
        package: id(PACKAGE),
        reference: format!("U{index}"),
        value: "native".into(),
        position: eda_engine::ir::geometry::Point::new(index * 40_000_000, 0),
        rotation: 0,
        layer: 1,
        locked: false,
    };
    let p = build_place_board_package(
        &m,
        provenance(),
        &BoardPackagePlacement {
            package: package.clone(),
            materialized: json!({}),
        },
    )
    .unwrap();
    commit_prepared(&mut m, root, p).unwrap();
    (symbol, package.uuid)
}
pub(super) fn adopt(root: &Path) {
    let report = success(cli(&[
        "project".into(),
        "place-board-component".into(),
        root.display().to_string(),
        "--part".into(),
        PART.into(),
        "--package".into(),
        PACKAGE.into(),
        "--reference".into(),
        "U_PROFILE".into(),
        "--value".into(),
        "profile adoption".into(),
        "--x-nm".into(),
        "120000000".into(),
        "--y-nm".into(),
        "0".into(),
        "--layer".into(),
        "1".into(),
    ]));
    assert!(
        report["source_profile_adoption_transaction"]
            .as_str()
            .is_some()
    );
    assert_eq!(board(&model(root))["schema_version"], 4);
}
pub(super) fn bind_args(
    root: &Path,
    command: &str,
    symbol: Uuid,
    package: Uuid,
    ci: Uuid,
    map: Option<Uuid>,
) -> Vec<String> {
    let mut args = vec![
        "project".into(),
        command.into(),
        root.display().to_string(),
        "--component-instance".into(),
        ci.to_string(),
        "--symbol".into(),
        symbol.to_string(),
        "--package".into(),
        package.to_string(),
        "--part".into(),
        PART.into(),
    ];
    if let Some(map) = map {
        args.extend(["--pin-pad-map".into(), map.to_string()]);
    }
    args
}
pub(super) fn materialize_args(root: &Path, package: Uuid, ci: Uuid) -> Vec<String> {
    vec![
        "project".into(),
        "set-board-component-package".into(),
        root.display().to_string(),
        "--component".into(),
        package.to_string(),
        "--package".into(),
        PACKAGE.into(),
        "--component-instance".into(),
        ci.to_string(),
        "--pin-pad-map".into(),
        MAP.into(),
    ]
}
pub(super) fn history(root: &Path, command: &str) {
    let m = model(root);
    success(cli(&[
        "project".into(),
        command.into(),
        root.display().to_string(),
        "--expected-model-revision".into(),
        m.model_revision.0,
        "--expected-tip-transaction".into(),
        eda_engine::revision::transaction_tip(&m.journal)
            .unwrap()
            .to_string(),
    ]));
}
pub(super) fn pads(root: &Path, package: Uuid) -> Value {
    board(&model(root))["component_pads"][package.to_string()].clone()
}
pub(super) fn assert_sources(root: &Path, package: Uuid, ci: Uuid) {
    let m = model(root);
    let values = pads(root, package);
    assert_eq!(values.as_array().unwrap().len(), 8);
    for p in values.as_array().unwrap() {
        let b = &p["source_binding"];
        assert_eq!(b["component_instance"]["object_id"], json!(ci));
        assert_eq!(
            b["component_instance"]["object_revision"],
            json!(m.objects[&ci].object_revision)
        );
        assert_eq!(b["pin_pad_map"]["object_id"], MAP);
        assert_eq!(
            b["pin_pad_map"]["object_revision"],
            json!(m.objects[&id(MAP)].object_revision)
        );
        assert_eq!(b["land_pattern"]["object_id"], FOOTPRINT);
    }
    let projected =
        eda_engine::substrate::component_pad_source::component_pad_projection(&m, &board(&m))
            .unwrap();
    assert!(
        projected.unavailable.is_empty(),
        "{:?}",
        projected.unavailable
    );
}

pub(super) fn connect(root: &Path, symbol: Uuid, package: Uuid, ci: Uuid) -> Uuid {
    use eda_engine::{
        ir::geometry::Point,
        substrate::{
            ElectricalIdentity, ElectricalIdentityRecord, ElectricalOccurrence,
            ElectricalSelectionSnapshot, NetCorrespondence, NetRelationshipIntent, ObjectRevision,
            Operation, RevisionedRef,
        },
    };
    let mut m = model(root);
    let snapshot = ElectricalSelectionSnapshot::capture(&m).unwrap();
    let (sheet, placed) = snapshot
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .find_map(|s| s.symbols.get(&symbol).map(|p| (s.uuid, p.clone())))
        .unwrap();
    let mapping = raw(&m, id(MAP));
    let (pin, library_pad) = placed
        .pins
        .iter()
        .find_map(|pin| {
            let logical = pin.library_pin?;
            mapping["mappings"]
                .as_object()?
                .iter()
                .find_map(|(pad, cell)| {
                    (cell["gate"] == json!(placed.gate?) && cell["pin"] == json!(logical))
                        .then(|| (pin.clone(), id(pad)))
                })
        })
        .unwrap();
    let wire = eda_engine::schematic::SchematicWire {
        uuid: Uuid::new_v4(),
        from: pin.position,
        to: Point::new(pin.position.x - 5_000_000, pin.position.y),
    };
    let p =
        write::schematic_connectivity::build_create_schematic_wire(&m, provenance(), sheet, &wire)
            .unwrap();
    commit_prepared(&mut m, root, p).unwrap();
    let name = format!("MANUAL_MAP_{ci}");
    let label = eda_engine::schematic::NetLabel {
        uuid: Uuid::new_v4(),
        kind: eda_engine::schematic::LabelKind::Global,
        name: name.clone(),
        position: wire.to,
    };
    let p = write::schematic_connectivity::build_create_schematic_label(
        &m,
        provenance(),
        sheet,
        &label,
    )
    .unwrap();
    commit_prepared(&mut m, root, p).unwrap();
    let pad: eda_engine::substrate::component_pad_source::ComponentPadSource = pads(root, package)
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            serde_json::from_value::<
                    eda_engine::substrate::component_pad_source::ComponentPadSource,
                >(v.clone())
                .unwrap()
        })
        .find(|p| p.source_binding.as_ref().unwrap().library_pad.object_id == library_pad)
        .unwrap();
    let projection =
        eda_engine::substrate::component_pad_source::component_pad_projection(&m, &board(&m))
            .unwrap();
    let terminal = projection.query_pads[&pad.uuid].terminal();
    let net = eda_engine::board::Net::new(Uuid::new_v4(), &name, Uuid::nil());
    let track = eda_engine::board::Track::straight(
        Uuid::new_v4(),
        net.uuid,
        terminal.position,
        Point::new(terminal.position.x - 1_000_000, terminal.position.y),
        100_000,
        terminal.layer,
    );
    let p = write::BatchComposer::compose(&m, provenance())
        .push_ops([
            Operation::CreateBoardNet {
                net_id: net.uuid,
                net: serde_json::to_value(&net).unwrap(),
            },
            Operation::CreateBoardTrack {
                track_id: track.uuid,
                track: serde_json::to_value(&track).unwrap(),
            },
        ])
        .finish()
        .unwrap();
    commit_prepared(&mut m, root, p).unwrap();
    let occurrence = ElectricalOccurrence {
        class: "pins".into(),
        source_id: pin.uuid,
        instance_path: vec![],
    };
    // global_net requires the Complete relationship we are about to author.
    // Use the actual automatic identity anchored in this closed native circuit,
    // never create a duplicate logical Net or infer it from a display name.
    let logical = m
        .electrical_identities
        .values()
        .filter_map(|record| match &record.identity {
            ElectricalIdentity::Net {
                anchor,
                retired: false,
                ..
            } if [pin.uuid, wire.uuid, label.uuid].contains(&anchor.source_id) => Some(record.id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        logical.len(),
        1,
        "one actual identity must anchor this native terminal/wire/label"
    );
    let logical = logical[0];
    let reference = |object_id| RevisionedRef {
        object_id,
        object_revision: m.objects[&object_id].object_revision,
    };
    let record = ElectricalIdentityRecord {
        id: Uuid::new_v4(),
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::NetRelationship {
            logical_net: logical,
            board_net: Some(reference(net.uuid)),
            intent: NetRelationshipIntent::Implemented,
            evidence: vec![NetCorrespondence {
                component_instance: reference(ci),
                pin_pad_map: reference(id(MAP)),
                schematic_terminal: occurrence.clone(),
                board_pad: reference(pad.uuid),
                library_pad: Some(reference(library_pad)),
                library_pin: Some(reference(pin.library_pin.unwrap())),
            }],
        },
    };
    let p = write::electrical::build_adopt_pin_correspondence(
        &m,
        provenance(),
        record,
        &[write::electrical::PlacedPinAdoption {
            sheet_id: sheet,
            symbol_id: symbol,
            placed_pin_id: pin.uuid,
            library_pin_id: pin.library_pin.unwrap(),
        }],
    )
    .unwrap();
    commit_prepared(&mut m, root, p).unwrap();
    assert_eq!(
        eda_engine::substrate::net_correspondence_status(&m, logical),
        eda_engine::substrate::NetCorrespondenceStatus::Complete
    );
    let snapshot = ElectricalSelectionSnapshot::capture(&m).unwrap();
    assert!(
        snapshot
            .global_net(snapshot.revision(), logical)
            .unwrap()
            .owned
            .contains(&occurrence)
    );
    logical
}
