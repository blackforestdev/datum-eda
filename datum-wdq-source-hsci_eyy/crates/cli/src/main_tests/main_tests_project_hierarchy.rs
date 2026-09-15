use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).expect("json file should read"))
        .expect("json file should parse")
}

fn query_hierarchy_json(root: &Path) -> serde_json::Value {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "hierarchy",
        ])
        .expect("CLI should parse"),
    )
    .expect("hierarchy query should succeed");
    serde_json::from_str(&output).expect("hierarchy JSON should parse")
}

fn query_canonical_hierarchy_json(root: &Path) -> serde_json::Value {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "hierarchy",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("canonical hierarchy query should succeed");
    serde_json::from_str(&output).expect("canonical hierarchy JSON should parse")
}

#[test]
fn project_query_hierarchy_reports_native_sheet_instances() {
    let root = unique_project_root("datum-eda-cli-project-query-hierarchy");
    create_native_project(&root, Some("Hierarchy Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = Uuid::new_v4();
    let definition_uuid = Uuid::new_v4();
    let instance_uuid = Uuid::new_v4();
    let sheet_path = root
        .join("schematic/sheets")
        .join(format!("{sheet_uuid}.json"));
    std::fs::write(
        &sheet_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": sheet_uuid,
                "name": "Main",
                "frame": null,
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
            }))
            .expect("sheet JSON should serialize")
        ),
    )
    .expect("sheet file should write");
    let definition_path = root
        .join("schematic/definitions")
        .join(format!("{definition_uuid}.json"));
    std::fs::write(
        &definition_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": definition_uuid,
                "root_sheet": sheet_uuid,
                "name": "Main Definition"
            }))
            .expect("definition JSON should serialize")
        ),
    )
    .expect("definition file should write");
    let schematic_json = root.join("schematic/schematic.json");
    let mut schematic_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schematic_json).expect("schematic.json should read"),
    )
    .expect("schematic.json should parse");
    schematic_value["sheets"] = serde_json::json!({
        sheet_uuid.to_string(): format!("sheets/{sheet_uuid}.json")
    });
    schematic_value["definitions"] = serde_json::json!({
        definition_uuid.to_string(): format!("definitions/{definition_uuid}.json")
    });
    schematic_value["instances"] = serde_json::json!([{
        "uuid": instance_uuid,
        "definition": definition_uuid,
        "parent_sheet": sheet_uuid,
        "position": { "x": 100, "y": 200 },
        "name": "Main Instance"
    }]);
    std::fs::write(
        &schematic_json,
        format!(
            "{}\n",
            to_json_deterministic(&schematic_value)
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("schematic.json should write");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "hierarchy",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query hierarchy should succeed");
    let hierarchy: serde_json::Value =
        serde_json::from_str(&output).expect("hierarchy JSON should parse");
    assert_eq!(hierarchy["instances"].as_array().unwrap().len(), 1);
    assert_eq!(hierarchy["instances"][0]["uuid"], instance_uuid.to_string());
    assert_eq!(
        hierarchy["instances"][0]["definition"],
        definition_uuid.to_string()
    );
    assert_eq!(
        hierarchy["instances"][0]["parent_sheet"],
        sheet_uuid.to_string()
    );
    assert_eq!(hierarchy["instances"][0]["name"], "Main Instance");
    assert_eq!(hierarchy["instances"][0]["position"]["x"], 100);
    assert_eq!(hierarchy["links"].as_array().unwrap().len(), 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_bind_sheet_instance_port_enables_hierarchy_link() {
    let root = unique_project_root("datum-eda-cli-project-sheet-instance-port");
    create_native_project(&root, Some("Sheet Port Binding Demo".to_string()))
        .expect("initial scaffold should succeed");
    let parent_sheet = Uuid::new_v4();
    let child_sheet = Uuid::new_v4();
    let definition_uuid = Uuid::new_v4();
    let instance_uuid = Uuid::new_v4();

    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "create-sheet",
            root.to_str().unwrap(),
            "--name",
            "Parent",
            "--sheet",
            &parent_sheet.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("parent sheet should create");
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "create-sheet",
            root.to_str().unwrap(),
            "--name",
            "Child",
            "--sheet",
            &child_sheet.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("child sheet should create");
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "create-sheet-definition",
            root.to_str().unwrap(),
            "--root-sheet",
            &child_sheet.to_string(),
            "--name",
            "Child Definition",
            "--definition",
            &definition_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("definition should create");
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "create-sheet-instance",
            root.to_str().unwrap(),
            "--definition",
            &definition_uuid.to_string(),
            "--parent-sheet",
            &parent_sheet.to_string(),
            "--name",
            "Child Instance",
            "--x-nm",
            "10",
            "--y-nm",
            "20",
            "--instance",
            &instance_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("instance should create");

    let port_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-port",
            root.to_str().unwrap(),
            "--sheet",
            &parent_sheet.to_string(),
            "--name",
            "SUB_IN",
            "--direction",
            "input",
            "--x-nm",
            "100",
            "--y-nm",
            "200",
        ])
        .expect("CLI should parse"),
    )
    .expect("port should place");
    let port: serde_json::Value =
        serde_json::from_str(&port_output).expect("port JSON should parse");
    let port_uuid = port["port_uuid"].as_str().unwrap().to_string();
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "place-label",
            root.to_str().unwrap(),
            "--sheet",
            &child_sheet.to_string(),
            "--name",
            "SUB_IN",
            "--kind",
            "hierarchical",
            "--x-nm",
            "100",
            "--y-nm",
            "200",
        ])
        .expect("CLI should parse"),
    )
    .expect("child label should place");

    let bind_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "bind-sheet-instance-port",
            root.to_str().unwrap(),
            "--instance",
            &instance_uuid.to_string(),
            "--port",
            &port_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("port should bind");
    let bound: serde_json::Value =
        serde_json::from_str(&bind_output).expect("bind JSON should parse");
    assert_eq!(bound["action"], "bind_sheet_instance_port");
    assert_eq!(bound["port_uuid"], port_uuid);
    assert_eq!(
        read_json(&root.join("schematic/schematic.json"))["instances"][0]["ports"][0],
        port_uuid
    );

    std::fs::remove_file(root.join(format!("schematic/definitions/{definition_uuid}.json")))
        .expect("promoted definition should remove");
    let hierarchy = query_hierarchy_json(&root);
    assert_eq!(hierarchy["links"].as_array().unwrap().len(), 1);
    assert_eq!(hierarchy["links"][0]["parent_port"], port_uuid);
    assert_eq!(
        query_canonical_hierarchy_json(&root)["links"][0]["parent_port"],
        port_uuid
    );

    execute(
        Cli::try_parse_from(["eda", "project", "undo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("undo should remove binding");
    assert_eq!(
        query_hierarchy_json(&root)["links"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    execute(
        Cli::try_parse_from(["eda", "project", "redo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("redo should restore binding");
    assert_eq!(
        query_hierarchy_json(&root)["links"][0]["parent_port"],
        port_uuid
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "unbind-sheet-instance-port",
            root.to_str().unwrap(),
            "--instance",
            &instance_uuid.to_string(),
            "--port",
            &port_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("port should unbind");
    let unbound_hierarchy = query_hierarchy_json(&root);
    assert_eq!(unbound_hierarchy["links"].as_array().unwrap().len(), 0);

    let _ = std::fs::remove_dir_all(&root);
}
