//! Current saved defaults inform explicit later choices; no automatic Map binding.
use super::main_tests_project_board_component_pool_materialization::{
    configure_native_project_for_pool_materialization, place_component, silkscreen_stackup,
};
use super::main_tests_project_pool_pin_pad_map::{
    create_default_pin_pad_map, create_fixture, query_pool_object_payload, run_project_command,
};
use super::*;
use eda_engine::{
    api::native_write::{
        WriteProvenance, commit_prepared,
        library::{self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec},
        registry::{NativeWriteContext, find_native_write_verb},
        schematic_assignment,
        schematic_library::part_placement::PartGateSelection,
        schematic_symbols::placed_symbol_component_instance_id,
    },
    substrate::{CommitSource, ProjectResolver, SourceShardKind},
};
use serde_json::{Value, json};
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "test",
        CommitSource::Manual,
        "Explicit saved default consumer",
    )
}
fn command(root: &Path, verb: &str, arguments: &[String]) -> Result<String> {
    let mut args = vec![
        "datum-eda".to_string(),
        "--format".into(),
        "json".into(),
        "project".into(),
        verb.into(),
        root.display().to_string(),
    ];
    args.extend_from_slice(arguments);
    run_project_command(&args.iter().map(String::as_str).collect::<Vec<_>>())
}
#[test]
fn saved_part_default_map_current_selection_explicit_cli_use_and_clear() {
    let root =
        development_workspace::path("saved-part-default-map").join(Uuid::new_v4().to_string());
    create_native_project(&root, Some("Saved map consumer".into())).unwrap();
    configure_native_project_for_pool_materialization(
        &root,
        json!([{"path":"pool","priority":1}]),
        silkscreen_stackup(21),
    );
    let f = create_fixture(&root, &["A", "B"], &["1", "2"]);
    let map_a = create_default_pin_pad_map(
        &root,
        &f,
        &[(f.pin_ids[0], f.pad_ids[0]), (f.pin_ids[1], f.pad_ids[1])],
    );
    let map_b = Uuid::new_v4();
    let mut alternate = query_pool_object_payload(&root, "pin_pad_maps", map_a);
    alternate["uuid"] = json!(map_b);
    // Different electrical meaning on the same land pattern, never inferred by geometry.
    let a = f.pad_ids[0].to_string();
    let b = f.pad_ids[1].to_string();
    let pin = alternate["mappings"][&a].clone();
    alternate["mappings"][&a] = alternate["mappings"][&b].clone();
    alternate["mappings"][&b] = pin;
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![Spec::Create {
            target: Target::new("pool", "pin_pad_maps", map_b),
            object: alternate,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let sheet = Uuid::new_v4();
    let p = (find_native_write_verb("datum.schematic.create_sheet")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"sheet":sheet,"name":"Explicit current default consumers"}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let before = query_pool_object_payload(&root, "parts", f.part_id);
    let package: Uuid = serde_json::from_value(before["package"].clone()).unwrap();
    let maps = [map_a, map_b].map(|id| (id, query_pool_object_payload(&root, "pin_pad_maps", id)));
    let mut frozen = Vec::<(Uuid, Value, Value, Value)>::new();
    for (index, selected) in [Some(map_a), Some(map_b), None].into_iter().enumerate() {
        let mut model = ProjectResolver::new(&root).resolve().unwrap();
        if index != 0 {
            let p = (find_native_write_verb("datum.library.edit").unwrap().build)(
                &NativeWriteContext{model:&model,project_root:&root},provenance(),
                json!({"kind":"parts","object_id":f.part_id,"pool_path":"pool","fields":{"default_pin_pad_map":selected}})).unwrap();
            commit_prepared(&mut model, &root, p).unwrap();
        }
        let current = query_pool_object_payload(&root, "parts", f.part_id);
        assert_eq!(
            current
                .get("default_pin_pad_map")
                .and_then(Value::as_str)
                .map(|s| s.parse::<Uuid>().unwrap()),
            selected
        );
        let selection =
            PartGateSelection::capture(&model, f.part_id, f.gate_id, f.symbol_id).unwrap();
        assert_eq!(
            selection.part_revision,
            model.objects[&f.part_id].object_revision.0
        );
        let p = (find_native_write_verb("datum.schematic.place_symbol").unwrap().build)(
            &NativeWriteContext{model:&model,project_root:&root},provenance(),
            json!({"sheet":sheet,"library_symbol":f.symbol_id,"explicit_part":selection,"reference":format!("U{index}"),"value":"Chosen current Part","x_nm":(index as i64)*40000000,"y_nm":0})).unwrap();
        let symbol = p.primary_object_id.unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let ci =
            placed_symbol_component_instance_id(&model.project.project_id, f.symbol_id, symbol);
        assert_eq!(
            model.component_instances[&ci].library_bindings.len(),
            1,
            "initial placement binds only Part, never an automatic Map"
        );
        assert!(
            schematic_assignment::options(&model, symbol, &[])
                .unwrap()
                .choices
                .iter()
                .any(|c| c.part == f.part_id && c.gate == f.gate_id),
            "actual gate compatibility reads the current Part/default"
        );
        let placed: Uuid = place_component(&root, f.part_id, package).parse().unwrap();
        if let Some(selected) = selected {
            command(
                &root,
                "set-component-instance",
                &[
                    "--component-instance".into(),
                    ci.to_string(),
                    "--symbol".into(),
                    symbol.to_string(),
                    "--package".into(),
                    placed.to_string(),
                    "--part".into(),
                    f.part_id.to_string(),
                    "--pin-pad-map".into(),
                    selected.to_string(),
                ],
            )
            .unwrap();
            command(
                &root,
                "set-board-component-package",
                &[
                    "--component".into(),
                    placed.to_string(),
                    "--package".into(),
                    package.to_string(),
                    "--component-instance".into(),
                    ci.to_string(),
                    "--pin-pad-map".into(),
                    selected.to_string(),
                ],
            )
            .unwrap();
        }
        let cold = ProjectResolver::new(&root).resolve().unwrap();
        let board = cold
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap();
        let pads = &board["component_pads"][placed.to_string()];
        assert_eq!(pads.as_array().unwrap().len(), 2);
        for pad in pads.as_array().unwrap() {
            if let Some(selected) = selected {
                assert_eq!(
                    pad["source_binding"]["pin_pad_map"]["object_id"],
                    json!(selected)
                );
                assert_eq!(
                    pad["source_binding"]["part"]["object_revision"],
                    json!(selection.part_revision)
                );
                let map = query_pool_object_payload(&root, "pin_pad_maps", selected);
                let library_pad = &pad["source_binding"]["library_pad"]["object_id"];
                assert!(
                    map["mappings"]
                        .as_object()
                        .unwrap()
                        .contains_key(library_pad.as_str().unwrap())
                );
            } else {
                assert!(pad["source_binding"]["pin_pad_map"].is_null());
                assert_eq!(cold.component_instances[&ci].library_bindings.len(), 1);
            }
        }
        for (old, old_package, old_pads, old_ci) in &frozen {
            assert_eq!(board["packages"][old.to_string()], *old_package);
            assert_eq!(board["component_pads"][old.to_string()], *old_pads);
            assert_eq!(
                serde_json::to_value(
                    &cold.component_instances
                        [&serde_json::from_value::<Uuid>(old_ci["id"].clone()).unwrap()]
                )
                .unwrap(),
                *old_ci
            );
        }
        frozen.push((
            placed,
            board["packages"][placed.to_string()].clone(),
            pads.clone(),
            serde_json::to_value(&cold.component_instances[&ci]).unwrap(),
        ));
        let projection =
            eda_engine::substrate::component_pad_source::component_pad_projection(&cold, &board)
                .unwrap();
        assert!(
            projection.unavailable.is_empty(),
            "{:?}",
            projection.unavailable
        );
        for (id, raw) in &maps {
            assert_eq!(query_pool_object_payload(&root, "pin_pad_maps", *id), *raw);
        }
    }
    let mut cleared = query_pool_object_payload(&root, "parts", f.part_id);
    assert!(cleared.get("default_pin_pad_map").is_none());
    cleared["default_pin_pad_map"] = before["default_pin_pad_map"].clone();
    assert_eq!(cleared, before);
    std::fs::remove_dir_all(root).unwrap();
}
