use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn execute(cli: Cli) -> Result<String> {
    let (output, _) = execute_with_exit_code(cli)?;
    Ok(output)
}

fn read_project_core_files(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    [
        "project.json",
        "schematic/schematic.json",
        "board/board.json",
        "rules/rules.json",
    ]
    .into_iter()
    .map(|relative| {
        let path = root.join(relative);
        let bytes = std::fs::read(&path).expect("project core file should read");
        (path, bytes)
    })
    .collect()
}

#[test]
fn project_query_relationships_and_variants_are_resolver_backed() {
    let root = unique_project_root("datum-eda-cli-project-query-relationships");
    create_native_project(&root, Some("Relationship Query Demo".to_string()))
        .expect("initial scaffold should succeed");
    let before = read_project_core_files(&root);
    let board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    let board_id = Uuid::parse_str(board["uuid"].as_str().unwrap()).unwrap();
    let relationship_id = Uuid::new_v4();
    let variant_id = Uuid::new_v4();
    let resolve_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("resolve-debug should succeed");
    let resolve_report: serde_json::Value = serde_json::from_str(&resolve_output).unwrap();

    let relationship_path = root.join(".datum/relationships/direct.json");
    std::fs::create_dir_all(relationship_path.parent().unwrap()).unwrap();
    std::fs::write(
        &relationship_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "relationships": [{
                    "id": relationship_id,
                    "kind": "implemented_by",
                    "from": [{ "object_id": board_id, "object_revision": 0 }],
                    "to": [{ "object_id": board_id, "object_revision": 0 }],
                    "authored_intent": [],
                    "object_revision": 0
                }]
            }))
            .unwrap()
        ),
    )
    .unwrap();
    let variant_path = root.join(".datum/variants/no-board.json");
    std::fs::create_dir_all(variant_path.parent().unwrap()).unwrap();
    std::fs::write(
        &variant_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "variants": [{
                    "id": variant_id,
                    "name": "No board",
                    "base_model_revision": resolve_report["model_revision"],
                    "variant_revision": 0,
                    "fitted": { board_id.to_string(): "unfitted" },
                    "relationship_overrides": {},
                    "property_overrides": {}
                }]
            }))
            .unwrap()
        ),
    )
    .unwrap();

    let relationships_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "relationships",
        ])
        .expect("CLI should parse"),
    )
    .expect("relationships query should succeed");
    let relationships: serde_json::Value = serde_json::from_str(&relationships_output).unwrap();
    assert_eq!(relationships["contract"], "relationships_query_v1");
    assert_eq!(relationships["relationship_count"], 1);
    assert_eq!(
        relationships["statuses"][relationship_id.to_string()],
        "implemented"
    );

    let variants_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "variants",
        ])
        .expect("CLI should parse"),
    )
    .expect("variants query should succeed");
    let variants: serde_json::Value = serde_json::from_str(&variants_output).unwrap();
    assert_eq!(variants["contract"], "variants_query_v1");
    assert_eq!(variants["variant_count"], 1);
    assert_eq!(
        variants["populations"][variant_id.to_string()][board_id.to_string()],
        "not_applicable_for_variant"
    );
    for (path, bytes) in before {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}

#[test]
fn project_component_instance_commands_are_journal_backed() {
    let root = unique_project_root("datum-eda-cli-project-component-instance");
    create_native_project(&root, Some("Component Instance Demo".to_string()))
        .expect("initial scaffold should succeed");
    let project: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("project.json")).unwrap()).unwrap();
    let project_id = Uuid::parse_str(project["uuid"].as_str().unwrap()).unwrap();
    let board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    let board_id = Uuid::parse_str(board["uuid"].as_str().unwrap()).unwrap();
    let mut schematic: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("schematic/schematic.json")).unwrap())
            .unwrap();
    let sheet_id = Uuid::new_v4();
    let sheet_path = format!("sheets/{sheet_id}.json");
    schematic["sheets"][sheet_id.to_string()] = serde_json::Value::String(sheet_path.clone());
    std::fs::write(
        root.join("schematic/schematic.json"),
        format!("{}\n", to_json_deterministic(&schematic).unwrap()),
    )
    .unwrap();
    let sheet_file = root.join("schematic").join(&sheet_path);
    std::fs::create_dir_all(sheet_file.parent().unwrap()).unwrap();
    let symbol_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    let mut sheet = serde_json::json!({
        "schema_version": 1,
        "uuid": sheet_id,
        "name": "Main",
        "symbols": {},
        "wires": {},
        "junctions": {},
        "labels": {},
        "buses": {},
        "bus_entries": {},
        "ports": {},
        "noconnects": {},
        "texts": {},
        "drawings": {}
    });
    sheet["symbols"][symbol_id.to_string()] = serde_json::json!({
        "uuid": symbol_id,
        "part": part_id,
        "entity": Uuid::new_v5(&project_id, b"entity"),
        "gate": Uuid::new_v5(&project_id, b"gate"),
        "lib_id": "test:R",
        "reference": "U1",
        "value": "OLD",
        "fields": [],
        "pins": [],
        "position": { "x": 0, "y": 0 },
        "rotation": 0,
        "mirrored": false,
        "unit_selection": null,
        "display_mode": "LibraryDefault",
        "pin_overrides": [],
        "hidden_power_behavior": "SourceDefinedImplicit"
    });
    std::fs::write(
        &sheet_file,
        format!("{}\n", to_json_deterministic(&sheet).unwrap()),
    )
    .unwrap();
    let package_id = Uuid::new_v4();
    let alternate_package_id = Uuid::new_v4();
    let mut board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    board["packages"][package_id.to_string()] =
        board_package_json(project_id, part_id, package_id, "U1", 0);
    board["packages"][alternate_package_id.to_string()] =
        board_package_json(project_id, part_id, alternate_package_id, "U1", 10);
    std::fs::write(
        root.join("board/board.json"),
        format!("{}\n", to_json_deterministic(&board).unwrap()),
    )
    .unwrap();

    let component_instance_id = Uuid::new_v4();
    let bind_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "bind-component-instance",
            root.to_str().unwrap(),
            "--component-instance",
            &component_instance_id.to_string(),
            "--symbol",
            &symbol_id.to_string(),
            "--package",
            &package_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("bind component instance should succeed");
    let bind_report: serde_json::Value = serde_json::from_str(&bind_output).unwrap();
    assert_eq!(bind_report["contract"], "component_instance_mutation_v1");
    assert_eq!(bind_report["action"], "bind_component_instance");

    let query_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "component-instances",
        ])
        .expect("CLI should parse"),
    )
    .expect("component-instances query should succeed");
    let query: serde_json::Value = serde_json::from_str(&query_output).unwrap();
    assert_eq!(query["contract"], "component_instances_query_v1");
    assert_eq!(query["component_instance_count"], 1);
    assert_eq!(
        query["component_instances"][component_instance_id.to_string()]["placed_package_refs"][0],
        package_id.to_string()
    );
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-component-instance",
            root.to_str().unwrap(),
            "--component-instance",
            &component_instance_id.to_string(),
            "--symbol",
            &symbol_id.to_string(),
            "--package",
            &alternate_package_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("set component instance should succeed");
    let set_query_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "component-instances",
        ])
        .expect("CLI should parse"),
    )
    .expect("component-instances query after set should succeed");
    let set_query: serde_json::Value = serde_json::from_str(&set_query_output).unwrap();
    assert_eq!(
        set_query["component_instances"][component_instance_id.to_string()]["placed_package_refs"]
            [0],
        alternate_package_id.to_string()
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "delete-component-instance",
            root.to_str().unwrap(),
            "--component-instance",
            &component_instance_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("delete component instance should succeed");
    let deleted_query_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "component-instances",
        ])
        .expect("CLI should parse"),
    )
    .expect("component-instances query after delete should succeed");
    let deleted_query: serde_json::Value = serde_json::from_str(&deleted_query_output).unwrap();
    assert_eq!(deleted_query["component_instance_count"], 0);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "undo",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("undo delete should succeed");
    let undo_query_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "component-instances",
        ])
        .expect("CLI should parse"),
    )
    .expect("component-instances query after undo should succeed");
    let undo_query: serde_json::Value = serde_json::from_str(&undo_query_output).unwrap();
    assert_eq!(undo_query["component_instance_count"], 1);
    assert_eq!(
        undo_query["component_instances"][component_instance_id.to_string()]["placed_package_refs"]
            [0],
        alternate_package_id.to_string()
    );
    let resolve_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("resolve-debug should succeed");
    let resolve_report: serde_json::Value = serde_json::from_str(&resolve_output).unwrap();
    let variant_id = Uuid::new_v4();
    let variant_path = root.join(".datum/variants/no-component.json");
    std::fs::create_dir_all(variant_path.parent().unwrap()).unwrap();
    std::fs::write(
        &variant_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "variants": [{
                    "id": variant_id,
                    "name": "No U1",
                    "base_model_revision": resolve_report["model_revision"],
                    "variant_revision": 0,
                    "fitted": { component_instance_id.to_string(): "unfitted" },
                    "relationship_overrides": {},
                    "property_overrides": {}
                }]
            }))
            .unwrap()
        ),
    )
    .unwrap();
    let variants_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "variants",
        ])
        .expect("CLI should parse"),
    )
    .expect("variants query should succeed");
    let variants: serde_json::Value = serde_json::from_str(&variants_output).unwrap();
    let population = &variants["populations"][variant_id.to_string()];
    assert_eq!(
        population[component_instance_id.to_string()],
        "not_applicable_for_variant"
    );
    assert_eq!(
        population[symbol_id.to_string()],
        "not_applicable_for_variant"
    );
    assert_eq!(
        population[alternate_package_id.to_string()],
        "not_applicable_for_variant"
    );
    assert_eq!(board_id.to_string(), board["uuid"].as_str().unwrap());
}

fn board_package_json(
    project_id: Uuid,
    part_id: Uuid,
    package_id: Uuid,
    reference: &str,
    x: i64,
) -> serde_json::Value {
    serde_json::json!({
        "uuid": package_id,
        "part": part_id,
        "package": Uuid::new_v5(&project_id, format!("package-{package_id}").as_bytes()),
        "reference": reference,
        "value": "OLD",
        "position": { "x": x, "y": 0 },
        "rotation": 0,
        "layer": 0,
        "locked": false
    })
}
