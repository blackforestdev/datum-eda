//! Actual managed source adoption reaches the ordinary schematic scene, while
//! preview/candidate history remains separate from placed source materialization.
use super::*;
use eda_engine::api::native_write::{
    self as native, PreparedWrite, WriteProvenance, commit_prepared,
    registry::{NativeWriteContext, find_native_write_verb},
};
use eda_engine::substrate::{CommitSource, DesignModel, ProjectResolver};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "managed-scene-test",
        CommitSource::Test,
        "Review shared Symbol extent",
    )
}
fn context<'a>(model: &'a DesignModel, root: &'a Path) -> NativeWriteContext<'a> {
    NativeWriteContext {
        model,
        project_root: root,
    }
}
fn prepare(model: &DesignModel, root: &Path, verb: &str, params: Value) -> PreparedWrite {
    (find_native_write_verb(verb).unwrap().build)(&context(model, root), provenance(), params)
        .unwrap()
}
fn write(model: &mut DesignModel, root: &Path, verb: &str, params: Value) -> Uuid {
    let prepared = prepare(model, root, verb, params);
    let id = prepared.primary_object_id.unwrap_or(Uuid::nil());
    commit_prepared(model, root, prepared).unwrap();
    id
}
fn fixture() -> PathBuf {
    let root = std::env::temp_dir().join(format!("datum_managed_scene_{}", Uuid::new_v4()));
    native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Managed scene".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, pin, symbol, entity, gate, package, stack, footprint, pad, part, map, sheet) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    for (kind, id, fields) in [
        (
            "units",
            unit,
            json!({"name":"Signal","pins":{pin.to_string():{"name":"A","electrical_type":"Passive"}}}),
        ),
        (
            "symbols",
            symbol,
            json!({"name":"Body","unit":unit,"drawings":[{"Rect":{
            "min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}],
            "pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0}}]}),
        ),
        (
            "entities",
            entity,
            json!({"name":"Device","manufacturer":"Designer","prefix":"U",
            "gates":{gate.to_string():{"uuid":gate,"name":"A","unit":unit,"symbol":symbol}}}),
        ),
        ("packages", package, json!({"name":"Package"})),
        (
            "padstacks",
            stack,
            json!({"name":"Copper","aperture":{"circle":{"diameter_nm":1000000}}}),
        ),
        (
            "footprints",
            footprint,
            json!({"name":"Land","package":package,"pads":{
            pad.to_string():{"uuid":pad,"name":"1","position":{"x":0,"y":0},"padstack":stack,"layer":1}}}),
        ),
        (
            "parts",
            part,
            json!({"manufacturer":"Designer","mpn":"SCENE","description":"","datasheet":"",
            "value":"Test","entity":entity,"package":package,"default_footprint":footprint}),
        ),
        (
            "pin_pad_maps",
            map,
            json!({"part":part,"footprint":footprint,"assign_default":true,
            "mappings":{pad.to_string():{"gate":gate,"pin":pin}}}),
        ),
    ] {
        write(
            &mut model,
            &root,
            "datum.library.create",
            json!({"kind":kind,"object_id":id,"pool_path":"pool","fields":fields}),
        );
    }
    write(
        &mut model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    for (reference, x) in [("U1", 0), ("U2", 10000000)] {
        let choice = native::schematic_library::part_placement::CompletePartSelection::capture(
            &model, part, gate, symbol,
        )
        .unwrap();
        write(
            &mut model,
            &root,
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"library_symbol":symbol,
            "component":choice,"reference":reference,"value":"Test","x_nm":x,"y_nm":0}),
        );
    }
    write(
        &mut model,
        &root,
        "datum.schematic.draw_wire",
        json!({
        "sheet":sheet,"instance_path":[],
        "points":[{"x":-2000000,"y":0},{"x":8000000,"y":0}]}),
    );
    root
}
fn inventory(path: &Path) -> (usize, usize, usize) {
    if path.is_file() {
        return (std::fs::metadata(path).unwrap().len() as usize, 1, 0);
    }
    std::fs::read_dir(path)
        .unwrap()
        .map(|entry| inventory(&entry.unwrap().path()))
        .fold((0, 0, 0), |(b, f, d), (nb, nf, nd)| {
            (b + nb, f + nf, d.max(nd + 1))
        })
}
fn inspect(model: &DesignModel, root: &Path) -> Value {
    native::library_assets::inspect_local_library(&context(model, root)).unwrap()
}
fn change(model: &mut DesignModel, root: &Path, params: Value) -> Value {
    let change =
        native::library_assets::prepare_library_change(&context(model, root), provenance(), params)
            .unwrap();
    commit_prepared(model, root, change.prepared).unwrap();
    change.summary
}
fn selection(model: &DesignModel, root: &Path, reference: &Value) -> Value {
    json!({"action":"adopt_selection","request":{"expected_catalog":inspect(model,root)["catalog_manifest"],"selected":reference}})
}
fn scene_geometry(model: &DesignModel, root: &Path) -> BTreeMap<String, Vec<PointNm>> {
    let sources = eda_engine::substrate::ElectricalSelectionSnapshot::capture(model).unwrap();
    let schematic = sources.schematic_source().unwrap();
    let sheet = schematic
        .sheets
        .values()
        .find(|sheet| !sheet.symbols.is_empty())
        .unwrap();
    let mut scene =
        project_schematic_sheet_source(schematic, sheet, &root.join("schematic/root.json"))
            .unwrap();
    super::super::native_library::project_bodies(&mut scene, model, schematic, sheet.uuid).unwrap();
    scene
        .board_graphics
        .into_iter()
        .filter(|graphic| graphic.object_id.contains(":library:"))
        .map(|graphic| (graphic.object_id, graphic.path))
        .collect()
}

#[test]
fn managed_shared_symbol_extent_reaches_scene_through_exact_adoption_and_history() {
    const KEY: &str = "DATUM_MANAGED_SCENE_FIXTURE";
    let Some(root) = std::env::var_os(KEY) else {
        // Only this exact child receives a fixture-derived host policy, avoiding
        // shared process policy changes or arbitrary production defaults.
        let root = fixture();
        let (bytes, files, depth) = inventory(&root);
        let transitions = [
            "format",
            "capture",
            "baseline",
            "candidate",
            "adopt",
            "undo",
            "redo",
            "old-selection",
        ];
        let policy = root.with_extension("policy.json");
        std::fs::write(
            &policy,
            serde_json::to_vec(&json!({
            "metadata_bytes":bytes*transitions.len(),"blob_bytes":bytes*transitions.len(),
            "files":files*transitions.len(),"depth":depth+transitions.len(),
            "total_bytes":bytes*files*transitions.len()}))
            .unwrap(),
        )
        .unwrap();
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact",concat!(module_path!(),"::managed_shared_symbol_extent_reaches_scene_through_exact_adoption_and_history")
                .strip_prefix(concat!(env!("CARGO_CRATE_NAME"),"::")).unwrap(),"--nocapture"])
            .env(KEY,&root).env("DATUM_LIBRARY_RESOURCE_POLICY_FILE",&policy).output().unwrap();
        assert!(
            result.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(String::from_utf8_lossy(&result.stdout).contains("1 passed"));
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_file(policy).unwrap();
        return;
    };
    let root = PathBuf::from(root);
    native::library_assets::initialize_library_service_from_environment().unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let ci = model.component_instances.values().next().unwrap();
    let part = ci.part_ref.unwrap();
    let symbol = model
        .objects
        .values()
        .find(|object| object.domain == "pool" && object.kind == "symbols")
        .unwrap()
        .object_id;
    assert!(model.component_instances.values().all(|ci| {
        ci.library_bindings.values().all(|binding| {
            binding.binding_role != eda_engine::substrate::LibraryBindingRole::Symbol
        })
    }));
    let original = scene_geometry(&model, &root);
    assert!(
        !eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model)
            .unwrap()
            .schematic_source()
            .unwrap()
            .sheets
            .values()
            .next()
            .unwrap()
            .wires
            .is_empty()
    );
    assert_eq!(original.len(), 2);
    let sheet_bytes: BTreeMap<_, _> = model
        .source_shards
        .iter()
        .filter(|s| {
            matches!(
                s.kind,
                eda_engine::substrate::SourceShardKind::SchematicSheet
                    | eda_engine::substrate::SourceShardKind::SchematicRoot
            )
        })
        .map(|s| (s.relative_path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    change(&mut model, &root, json!({"action":"adopt_profile"}));
    let request = json!({"action":"save_candidate","request":{"expected_catalog":inspect(&model,&root)["catalog_manifest"],
        "change":{"action":"capture_native","asset_id":part}}});
    let part_ref = change(&mut model, &root, request)["candidate"].clone();
    let request = selection(&model, &root, &part_ref);
    change(&mut model, &root, request);
    let before = inspect(&model, &root)["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|asset| asset["revision"]["asset_id"] == json!(symbol))
        .unwrap()["revision"]
        .clone();
    let mut payload = native::library::pool_library_object_payload(
        &model,
        &format!("pool/symbols/{symbol}.json"),
        symbol,
    )
    .unwrap();
    payload["drawings"][0]["Rect"]["max"]["x"] = json!(3500000);
    payload["drawings"]
        .as_array_mut()
        .unwrap()
        .push(json!({"Line":{
        "from":{"x":0,"y":0},"to":{"x":4500000,"y":2000000},"width":100000}}));
    // Portable payloads exclude native counters through the normal capture owner.
    payload.as_object_mut().unwrap().remove("object_revision");
    payload.as_object_mut().unwrap().remove("model_revision");
    let request = json!({"action":"save_candidate","request":{"expected_catalog":inspect(&model,&root)["catalog_manifest"],
        "change":{"action":"revise","base":before,"payload_json":serde_json::to_string(&payload).unwrap()}}});
    let target = change(&mut model, &root, request)["candidate"].clone();
    assert_eq!(scene_geometry(&model, &root), original);
    let request = selection(&model, &root, &target);
    let preview = native::library_assets::prepare_library_change(
        &context(&model, &root),
        provenance(),
        request.clone(),
    )
    .unwrap();
    assert_eq!(scene_geometry(&model, &root), original);
    drop(preview);
    change(&mut model, &root, request);
    let adopted = scene_geometry(&model, &root);
    assert_eq!(adopted.len(), 4);
    assert_ne!(adopted, original);
    // Independently specified source extent and line endpoint, with the second
    // component's known 10mm translation. This catches birth-source rendering.
    for x in [3500000, 13500000] {
        assert!(adopted.values().any(|path| path.iter().any(|p| p.x == x)));
    }
    for x in [4500000, 14500000] {
        assert!(
            adopted
                .values()
                .any(|path| path.last().is_some_and(|p| p.x == x && p.y == 2000000))
        );
    }
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(scene_geometry(&cold, &root), adopted);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(scene_geometry(&model, &root), original);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(scene_geometry(&model, &root), adopted);
    let request = selection(&model, &root, &before);
    change(&mut model, &root, request);
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(scene_geometry(&cold, &root), original);
    for (path, bytes) in sheet_bytes {
        assert_eq!(std::fs::read(root.join(path)).unwrap(), bytes);
    }
}
