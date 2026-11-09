//! Real CLI materialization consumes corrected current defaults; old sources freeze.
use super::main_tests_project_board_component_pool_materialization::{
    configure_native_project_for_pool_materialization, place_component, silkscreen_stackup,
};
use super::*;
use eda_engine::{
    api::native_write::{
        WriteProvenance, commit_prepared,
        registry::{NativeWriteContext, find_native_write_verb},
    },
    substrate::{CommitSource, ProjectResolver, SourceShardKind},
};
use serde_json::{Value, json};
fn read(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[test]
fn saved_part_default_footprint_cli_materializes_changed_and_cleared_current_source() {
    let root = development_workspace::path("saved-part-default-footprint")
        .join(Uuid::new_v4().to_string());
    create_native_project(&root, Some("Saved default consumer".into())).unwrap();
    configure_native_project_for_pool_materialization(
        &root,
        json!([{"path":"pool","priority":1}]),
        silkscreen_stackup(21),
    );
    // Initial on-disk library fixture, before the first canonical transaction.
    let baseline = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../test-harness/testdata/library/native_authored_baseline_v1/pool");
    for kind in std::fs::read_dir(baseline).unwrap() {
        let kind = kind.unwrap();
        if !kind.file_type().unwrap().is_dir() {
            continue;
        }
        let destination = root.join("pool").join(kind.file_name());
        std::fs::create_dir_all(&destination).unwrap();
        for file in std::fs::read_dir(kind.path()).unwrap() {
            let file = file.unwrap();
            if file.path().extension().is_some_and(|e| e == "json") {
                std::fs::copy(file.path(), destination.join(file.file_name())).unwrap();
            }
        }
    }
    let part: Uuid = "90000000-0000-4000-8000-000000000001".parse().unwrap();
    let package: Uuid = "50000000-0000-4000-8000-000000000001".parse().unwrap();
    let first: Uuid = "60000000-0000-4000-8000-000000000001".parse().unwrap();
    let second: Uuid = "60000000-0000-4000-8000-000000000002".parse().unwrap();
    let part_path = root.join(format!("pool/parts/{part}.json"));
    let mut initial = read(&part_path);
    initial
        .as_object_mut()
        .unwrap()
        .remove("default_pin_pad_map");
    std::fs::write(&part_path, serde_json::to_vec_pretty(&initial).unwrap()).unwrap();
    let mut alternate = read(&root.join(format!("pool/footprints/{first}.json")));
    alternate["uuid"] = json!(second);
    alternate["name"] = json!("Different current CLI land");
    alternate.as_object_mut().unwrap().remove("ipc_basis");
    alternate["standards_basis"] = Value::Null;
    alternate["pads"] = Value::Object(
        alternate["pads"]
            .as_object()
            .unwrap()
            .values()
            .map(|pad| {
                let id = Uuid::new_v4();
                let mut pad = pad.clone();
                pad["uuid"] = json!(id);
                pad["position"]["x"] = json!(pad["position"]["x"].as_i64().unwrap() + 600_000);
                (id.to_string(), pad)
            })
            .collect(),
    );
    std::fs::write(
        root.join(format!("pool/footprints/{second}.json")),
        serde_json::to_vec_pretty(&alternate).unwrap(),
    )
    .unwrap();
    let mut frozen = Vec::new();
    let mut first_positions = None;
    for (choice, expected) in [
        (None, first),
        (Some(json!(second)), second),
        (Some(Value::Null), first),
    ] {
        let mut model = ProjectResolver::new(&root).resolve().unwrap();
        if let Some(value) = choice {
            let p = (find_native_write_verb("datum.library.edit").unwrap().build)(
                &NativeWriteContext { model:&model, project_root:&root },
                WriteProvenance::new("test", CommitSource::Manual, "Correct saved default"),
                json!({"kind":"parts","object_id":part,"pool_path":"pool","fields":{"default_footprint":value}}),
            ).unwrap();
            commit_prepared(&mut model, &root, p).unwrap();
        }
        let selected_revision = model.objects[&part].object_revision.0;
        let placed = place_component(&root, part, package);
        let board = read(&root.join("board/board.json"));
        let pads = board["component_pads"][&placed].as_array().unwrap();
        assert_eq!(pads.len(), 8);
        assert!(
            pads.iter()
                .all(
                    |p| p["source_binding"]["land_pattern"]["object_id"] == json!(expected)
                        && p["source_binding"]["library_pad"]["owner_id"] == json!(expected)
                        && p["source_binding"]["part"]["object_revision"]
                            == json!(selected_revision)
                )
        );
        let mut positions: Vec<_> = pads.iter().map(|p| p["position"].to_string()).collect();
        positions.sort();
        if let Some(first) = &first_positions {
            assert_eq!(positions == *first, expected != second);
        } else {
            first_positions = Some(positions);
        }
        for (id, package, pads) in &frozen {
            assert_eq!(&board["packages"][id], package);
            assert_eq!(&board["component_pads"][id], pads);
        }
        frozen.push((
            placed.clone(),
            board["packages"][&placed].clone(),
            board["component_pads"][&placed].clone(),
        ));
        let cold = ProjectResolver::new(&root).resolve().unwrap();
        let source = cold
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap();
        let projection =
            eda_engine::substrate::component_pad_source::component_pad_projection(&cold, &source)
                .unwrap();
        assert!(
            projection.unavailable.is_empty(),
            "{:?}",
            projection.unavailable
        );
        assert!(projection.statuses.values().all(|s| *s == eda_engine::substrate::component_pad_source::ComponentPadMaterializationStatus::Current));
    }
    let mut cleared = read(&part_path);
    assert!(cleared.get("default_footprint").is_none());
    cleared["default_footprint"] = initial["default_footprint"].clone();
    assert_eq!(
        cleared, initial,
        "default correction leaves all other raw Part fields unchanged"
    );
    std::fs::remove_dir_all(root).unwrap();
}
