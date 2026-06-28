use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).expect("json file should read"))
        .expect("json file should parse")
}

fn journal_list(root: &Path) -> serde_json::Value {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "journal-list",
        ])
        .expect("CLI should parse"),
    )
    .expect("journal-list should succeed");
    serde_json::from_str(&output).expect("journal-list JSON should parse")
}

#[test]
fn project_create_delete_sheet_undo_redo_uses_journaled_substrate() {
    let root = unique_project_root("datum-eda-cli-project-sheet");
    create_native_project(&root, Some("Sheet Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = Uuid::new_v4();

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-sheet",
            root.to_str().unwrap(),
            "--name",
            "Aux",
            "--sheet",
            &sheet_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("create-sheet should succeed");
    let created: serde_json::Value =
        serde_json::from_str(&create_output).expect("create-sheet JSON should parse");
    assert_eq!(created["action"], "create_sheet");
    assert_eq!(created["sheet_uuid"], sheet_uuid.to_string());
    assert_eq!(created["name"], "Aux");
    let journal = journal_list(&root);
    assert_eq!(journal["count"], 1);
    assert_eq!(
        journal["transactions"][0]["reason"],
        "create schematic sheet"
    );
    assert_eq!(journal["transactions"][0]["operations"], 1);

    let sheet_path = PathBuf::from(created["sheet_path"].as_str().unwrap());
    assert!(sheet_path.exists());
    assert_eq!(read_json(&sheet_path)["name"], "Aux");
    let schematic_path = root.join("schematic/schematic.json");
    let schematic = read_json(&schematic_path);
    assert_eq!(
        schematic["sheets"][sheet_uuid.to_string()],
        format!("sheets/{sheet_uuid}.json")
    );

    let rename_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "rename-sheet",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--name",
            "Renamed Aux",
        ])
        .expect("CLI should parse"),
    )
    .expect("rename-sheet should succeed");
    let renamed: serde_json::Value =
        serde_json::from_str(&rename_output).expect("rename-sheet JSON should parse");
    assert_eq!(renamed["action"], "rename_sheet");
    assert_eq!(renamed["sheet_uuid"], sheet_uuid.to_string());
    assert_eq!(renamed["name"], "Renamed Aux");
    assert_eq!(read_json(&sheet_path)["name"], "Renamed Aux");
    let journal = journal_list(&root);
    assert_eq!(journal["count"], 2);
    assert_eq!(
        journal["transactions"][1]["reason"],
        "rename schematic sheet"
    );

    execute(
        Cli::try_parse_from(["eda", "project", "undo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("rename undo should succeed");
    assert_eq!(read_json(&sheet_path)["name"], "Aux");

    execute(
        Cli::try_parse_from(["eda", "project", "redo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("rename redo should succeed");
    assert_eq!(read_json(&sheet_path)["name"], "Renamed Aux");

    let delete_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "delete-sheet",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("delete-sheet should succeed");
    let deleted: serde_json::Value =
        serde_json::from_str(&delete_output).expect("delete-sheet JSON should parse");
    assert_eq!(deleted["action"], "delete_sheet");
    assert_eq!(deleted["sheet_uuid"], sheet_uuid.to_string());
    assert!(!sheet_path.exists());
    assert!(
        !read_json(&schematic_path)["sheets"]
            .as_object()
            .unwrap()
            .contains_key(&sheet_uuid.to_string())
    );
    let journal = journal_list(&root);
    assert_eq!(journal["count"], 5);
    assert_eq!(
        journal["transactions"][4]["reason"],
        "delete schematic sheet"
    );
    assert_eq!(journal["transactions"][4]["operations"], 1);

    execute(
        Cli::try_parse_from(["eda", "project", "undo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("undo should succeed");
    assert!(sheet_path.exists());
    assert!(
        read_json(&schematic_path)["sheets"]
            .as_object()
            .unwrap()
            .contains_key(&sheet_uuid.to_string())
    );

    execute(
        Cli::try_parse_from(["eda", "project", "redo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("redo should succeed");
    assert!(!sheet_path.exists());
    assert!(
        !read_json(&schematic_path)["sheets"]
            .as_object()
            .unwrap()
            .contains_key(&sheet_uuid.to_string())
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_sheet_definition_undo_redo_uses_journaled_substrate() {
    let root = unique_project_root("datum-eda-cli-project-sheet-definition");
    create_native_project(&root, Some("Sheet Definition Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = Uuid::new_v4();
    let definition_uuid = Uuid::new_v4();

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-sheet",
            root.to_str().unwrap(),
            "--name",
            "Main",
            "--sheet",
            &sheet_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("create-sheet should succeed");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-sheet-definition",
            root.to_str().unwrap(),
            "--root-sheet",
            &sheet_uuid.to_string(),
            "--name",
            "Main Definition",
            "--definition",
            &definition_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("create-sheet-definition should succeed");
    let created: serde_json::Value =
        serde_json::from_str(&create_output).expect("create-sheet-definition JSON should parse");
    assert_eq!(created["action"], "create_sheet_definition");
    assert_eq!(created["definition_uuid"], definition_uuid.to_string());
    assert_eq!(created["root_sheet_uuid"], sheet_uuid.to_string());
    assert_eq!(created["name"], "Main Definition");

    let definition_path = PathBuf::from(created["definition_path"].as_str().unwrap());
    assert!(definition_path.exists());
    let definition = read_json(&definition_path);
    assert_eq!(definition["uuid"], definition_uuid.to_string());
    assert_eq!(definition["root_sheet"], sheet_uuid.to_string());
    assert_eq!(definition["name"], "Main Definition");

    let schematic_path = root.join("schematic/schematic.json");
    assert_eq!(
        read_json(&schematic_path)["definitions"][definition_uuid.to_string()],
        format!("definitions/{definition_uuid}.json")
    );
    let journal = journal_list(&root);
    assert_eq!(journal["count"], 2);
    assert_eq!(
        journal["transactions"][1]["reason"],
        "create schematic sheet definition"
    );
    assert_eq!(journal["transactions"][1]["operations"], 1);

    execute(
        Cli::try_parse_from(["eda", "project", "undo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("definition create undo should succeed");
    assert!(!definition_path.exists());
    assert!(
        !read_json(&schematic_path)["definitions"]
            .as_object()
            .unwrap()
            .contains_key(&definition_uuid.to_string())
    );

    execute(
        Cli::try_parse_from(["eda", "project", "redo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("definition create redo should succeed");
    assert!(definition_path.exists());
    assert_eq!(
        read_json(&schematic_path)["definitions"][definition_uuid.to_string()],
        format!("definitions/{definition_uuid}.json")
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_sheet_mutations_read_resolver_materialized_schematic_root() {
    let root = unique_project_root("datum-eda-cli-project-sheet-resolver-root");
    create_native_project(&root, Some("Sheet Resolver Demo".to_string()))
        .expect("initial scaffold should succeed");
    let schematic_path = root.join("schematic/schematic.json");
    let stale_schematic_root =
        std::fs::read_to_string(&schematic_path).expect("stale schematic root should read");
    let sheet_uuid = Uuid::new_v4();
    let definition_uuid = Uuid::new_v4();
    let instance_uuid = Uuid::new_v4();

    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "create-sheet",
            root.to_str().unwrap(),
            "--name",
            "Journal Sheet",
            "--sheet",
            &sheet_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("create-sheet should succeed");
    std::fs::write(&schematic_path, &stale_schematic_root)
        .expect("stale schematic root should restore");

    let renamed = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "rename-sheet",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--name",
            "Resolver Sheet",
        ])
        .expect("CLI should parse"),
    )
    .expect("rename-sheet should resolve journal-created sheet");
    let renamed: serde_json::Value =
        serde_json::from_str(&renamed).expect("rename-sheet JSON should parse");
    assert_eq!(renamed["name"], "Resolver Sheet");
    std::fs::write(&schematic_path, &stale_schematic_root)
        .expect("stale schematic root should restore");

    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "create-sheet-definition",
            root.to_str().unwrap(),
            "--root-sheet",
            &sheet_uuid.to_string(),
            "--name",
            "Resolver Definition",
            "--definition",
            &definition_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("create-sheet-definition should resolve journal-created sheet");
    std::fs::write(&schematic_path, &stale_schematic_root)
        .expect("stale schematic root should restore");

    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "create-sheet-instance",
            root.to_str().unwrap(),
            "--definition",
            &definition_uuid.to_string(),
            "--parent-sheet",
            &sheet_uuid.to_string(),
            "--name",
            "Resolver Instance",
            "--x-nm",
            "100",
            "--y-nm",
            "200",
            "--instance",
            &instance_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("create-sheet-instance should resolve journal-created definition and sheet");
    std::fs::write(&schematic_path, &stale_schematic_root)
        .expect("stale schematic root should restore");

    let moved = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "move-sheet-instance",
            root.to_str().unwrap(),
            "--instance",
            &instance_uuid.to_string(),
            "--x-nm",
            "300",
            "--y-nm",
            "400",
        ])
        .expect("CLI should parse"),
    )
    .expect("move-sheet-instance should resolve journal-created instance");
    let moved: serde_json::Value =
        serde_json::from_str(&moved).expect("move-sheet-instance JSON should parse");
    assert_eq!(moved["x_nm"], 300);
    assert_eq!(moved["y_nm"], 400);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_delete_sheet_instance_undo_redo_uses_journaled_substrate() {
    let root = unique_project_root("datum-eda-cli-project-sheet-instance");
    create_native_project(&root, Some("Sheet Instance Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = Uuid::new_v4();
    let definition_uuid = Uuid::new_v4();
    let instance_uuid = Uuid::new_v4();

    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "create-sheet",
            root.to_str().unwrap(),
            "--name",
            "Main",
            "--sheet",
            &sheet_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("create-sheet should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "create-sheet-definition",
            root.to_str().unwrap(),
            "--root-sheet",
            &sheet_uuid.to_string(),
            "--name",
            "Main Definition",
            "--definition",
            &definition_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("create-sheet-definition should succeed");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-sheet-instance",
            root.to_str().unwrap(),
            "--definition",
            &definition_uuid.to_string(),
            "--parent-sheet",
            &sheet_uuid.to_string(),
            "--name",
            "Main Instance",
            "--x-nm",
            "100",
            "--y-nm",
            "200",
            "--instance",
            &instance_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("create-sheet-instance should succeed");
    let created: serde_json::Value =
        serde_json::from_str(&create_output).expect("create-sheet-instance JSON should parse");
    assert_eq!(created["action"], "create_sheet_instance");
    assert_eq!(created["instance_uuid"], instance_uuid.to_string());
    assert_eq!(created["definition_uuid"], definition_uuid.to_string());
    assert_eq!(created["parent_sheet_uuid"], sheet_uuid.to_string());
    assert_eq!(created["x_nm"], 100);
    let schematic_path = root.join("schematic/schematic.json");
    assert_eq!(
        read_json(&schematic_path)["instances"][0]["uuid"],
        instance_uuid.to_string()
    );

    let hierarchy_output = execute(
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
    let hierarchy: serde_json::Value =
        serde_json::from_str(&hierarchy_output).expect("hierarchy JSON should parse");
    assert_eq!(hierarchy["instances"][0]["uuid"], instance_uuid.to_string());
    assert_eq!(hierarchy["instances"][0]["name"], "Main Instance");

    let move_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "move-sheet-instance",
            root.to_str().unwrap(),
            "--instance",
            &instance_uuid.to_string(),
            "--x-nm",
            "300",
            "--y-nm",
            "400",
        ])
        .expect("CLI should parse"),
    )
    .expect("move-sheet-instance should succeed");
    let moved: serde_json::Value =
        serde_json::from_str(&move_output).expect("move-sheet-instance JSON should parse");
    assert_eq!(moved["action"], "move_sheet_instance");
    assert_eq!(moved["x_nm"], 300);
    assert_eq!(
        read_json(&schematic_path)["instances"][0]["position"]["x"],
        300
    );
    execute(
        Cli::try_parse_from(["eda", "project", "undo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("move undo should succeed");
    assert_eq!(
        read_json(&schematic_path)["instances"][0]["position"]["x"],
        100
    );
    execute(
        Cli::try_parse_from(["eda", "project", "redo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("move redo should succeed");
    assert_eq!(
        read_json(&schematic_path)["instances"][0]["position"]["x"],
        300
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "delete-sheet-instance",
            root.to_str().unwrap(),
            "--instance",
            &instance_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("delete-sheet-instance should succeed");
    assert_eq!(
        read_json(&schematic_path)["instances"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    execute(
        Cli::try_parse_from(["eda", "project", "undo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("undo should succeed");
    assert_eq!(
        read_json(&schematic_path)["instances"][0]["uuid"],
        instance_uuid.to_string()
    );
    execute(
        Cli::try_parse_from(["eda", "project", "redo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("redo should succeed");
    assert_eq!(
        read_json(&schematic_path)["instances"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_delete_sheet_cascades_populated_sheet_and_undo_redo_restores_payload() {
    let root = unique_project_root("datum-eda-cli-project-sheet-cascade");
    create_native_project(&root, Some("Sheet Cascade Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = Uuid::new_v4();

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-sheet",
            root.to_str().unwrap(),
            "--name",
            "Aux",
            "--sheet",
            &sheet_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("create-sheet should succeed");
    let created: serde_json::Value =
        serde_json::from_str(&create_output).expect("create-sheet JSON should parse");
    let sheet_path = PathBuf::from(created["sheet_path"].as_str().unwrap());

    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "place-label",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--name",
            "VIN",
            "--kind",
            "global",
            "--x-nm",
            "10",
            "--y-nm",
            "20",
        ])
        .expect("CLI should parse"),
    )
    .expect("place-label should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "draw-wire",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--from-x-nm",
            "10",
            "--from-y-nm",
            "20",
            "--to-x-nm",
            "30",
            "--to-y-nm",
            "40",
        ])
        .expect("CLI should parse"),
    )
    .expect("draw-wire should succeed");
    let populated = read_json(&sheet_path);
    assert_eq!(populated["labels"].as_object().unwrap().len(), 1);
    assert_eq!(populated["wires"].as_object().unwrap().len(), 1);

    let delete_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "delete-sheet",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("delete-sheet should cascade populated sheet");
    let deleted: serde_json::Value =
        serde_json::from_str(&delete_output).expect("delete-sheet JSON should parse");
    assert_eq!(deleted["action"], "delete_sheet");
    assert_eq!(deleted["cascaded_objects"], 2);
    assert!(!sheet_path.exists());
    let journal = journal_list(&root);
    assert_eq!(journal["count"], 4);
    assert_eq!(
        journal["transactions"][3]["reason"],
        "delete schematic sheet"
    );
    assert_eq!(journal["transactions"][3]["operations"], 1);

    execute(
        Cli::try_parse_from(["eda", "project", "undo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("undo should succeed");
    let restored = read_json(&sheet_path);
    assert_eq!(restored["labels"].as_object().unwrap().len(), 1);
    assert_eq!(restored["wires"].as_object().unwrap().len(), 1);

    execute(
        Cli::try_parse_from(["eda", "project", "redo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("redo should succeed");
    assert!(!sheet_path.exists());

    let _ = std::fs::remove_dir_all(&root);
}
