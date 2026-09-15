use super::main_tests_project_journal_support::assert_journal_transaction;
use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn seed_native_sheet(root: &Path) -> Uuid {
    let sheet_uuid = Uuid::new_v4();
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

    let schematic_json = root.join("schematic/schematic.json");
    let mut schematic_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schematic_json).expect("schematic.json should read"),
    )
    .expect("schematic.json should parse");
    schematic_value["sheets"] = serde_json::json!({
        sheet_uuid.to_string(): format!("sheets/{sheet_uuid}.json")
    });
    std::fs::write(
        &schematic_json,
        format!(
            "{}\n",
            to_json_deterministic(&schematic_value)
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("schematic.json should write");

    sheet_uuid
}

fn assert_bus_journal_transaction(root: &Path, index: usize, reason: &str, operations: u64) {
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
    let journal: serde_json::Value =
        serde_json::from_str(&output).expect("journal-list JSON should parse");
    assert_eq!(journal["transactions"][index]["reason"], reason);
    assert_eq!(journal["transactions"][index]["operations"], operations);
}

#[test]
fn project_bus_and_bus_entry_family_update_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-bus");
    create_native_project(&root, Some("Bus Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);

    let create_bus_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "create-bus",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--name",
        "DATA",
        "--member",
        "D0",
        "--member",
        "D1",
    ])
    .expect("CLI should parse");
    let create_bus_output = execute(create_bus_cli).expect("project create-bus should succeed");
    let created_bus: serde_json::Value =
        serde_json::from_str(&create_bus_output).expect("create-bus JSON should parse");
    let bus_uuid = created_bus["bus_uuid"].as_str().unwrap().to_string();

    let query_buses_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "buses",
    ])
    .expect("CLI should parse");
    let buses_output = execute(query_buses_cli).expect("project query buses should succeed");
    let buses: serde_json::Value =
        serde_json::from_str(&buses_output).expect("buses JSON should parse");
    assert_eq!(buses.as_array().unwrap().len(), 1);
    assert_eq!(buses[0]["name"], "DATA");
    assert_eq!(buses[0]["members"].as_array().unwrap().len(), 2);
    assert_journal_transaction(&root, "create schematic bus", 1);

    let edit_bus_cli = Cli::try_parse_from([
        "eda",
        "project",
        "edit-bus-members",
        root.to_str().unwrap(),
        "--bus",
        &bus_uuid,
        "--member",
        "D0",
        "--member",
        "D1",
        "--member",
        "D2",
    ])
    .expect("CLI should parse");
    let edit_bus_output = execute(edit_bus_cli).expect("project edit-bus-members should succeed");
    assert!(edit_bus_output.contains("action: edit_bus_members"));

    let query_buses_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "buses",
    ])
    .expect("CLI should parse");
    let buses_output = execute(query_buses_cli).expect("project query buses should succeed");
    let buses: serde_json::Value =
        serde_json::from_str(&buses_output).expect("buses JSON should parse");
    assert_eq!(buses[0]["members"].as_array().unwrap().len(), 3);
    assert_bus_journal_transaction(&root, 0, "create schematic bus", 1);
    assert_bus_journal_transaction(&root, 1, "edit schematic bus members", 1);

    let place_entry_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-bus-entry",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--bus",
        &bus_uuid,
        "--x-nm",
        "100",
        "--y-nm",
        "200",
    ])
    .expect("CLI should parse");
    let place_entry_output =
        execute(place_entry_cli).expect("project place-bus-entry should succeed");
    let placed_entry: serde_json::Value =
        serde_json::from_str(&place_entry_output).expect("place-bus-entry JSON should parse");
    let bus_entry_uuid = placed_entry["bus_entry_uuid"].as_str().unwrap().to_string();

    let query_entries_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "bus-entries",
    ])
    .expect("CLI should parse");
    let entries_output =
        execute(query_entries_cli).expect("project query bus-entries should succeed");
    let entries: serde_json::Value =
        serde_json::from_str(&entries_output).expect("bus entries JSON should parse");
    assert_eq!(entries.as_array().unwrap().len(), 1);
    assert_eq!(entries[0]["uuid"], bus_entry_uuid);
    assert_eq!(entries[0]["bus"], bus_uuid);
    assert_bus_journal_transaction(&root, 2, "place schematic bus entry", 1);

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_buses: 1"));
    assert!(summary_output.contains("schematic_bus_entries: 1"));
    let sheet_path = PathBuf::from(created_bus["sheet_path"].as_str().unwrap());
    let mut stale_sheet: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&sheet_path).expect("sheet should read"))
            .expect("sheet should parse");
    stale_sheet["buses"] = serde_json::json!({});
    stale_sheet["bus_entries"] = serde_json::json!({});
    std::fs::write(
        &sheet_path,
        format!(
            "{}\n",
            to_json_deterministic(&stale_sheet).expect("sheet should serialize")
        ),
    )
    .expect("stale sheet should write");

    let query_entries_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "bus-entries",
    ])
    .expect("CLI should parse");
    let replayed_entries_output =
        execute(query_entries_cli).expect("project query bus-entries should succeed");
    let replayed_entries: serde_json::Value =
        serde_json::from_str(&replayed_entries_output).expect("bus entries JSON should parse");
    assert_eq!(replayed_entries.as_array().unwrap().len(), 1);
    assert_eq!(replayed_entries[0]["uuid"], bus_entry_uuid);

    let blocked_delete_bus_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-bus",
        root.to_str().unwrap(),
        "--bus",
        &bus_uuid,
    ])
    .expect("CLI should parse");
    let blocked_delete_bus = execute(blocked_delete_bus_cli)
        .expect_err("project delete-bus should reject referenced buses");
    assert!(
        blocked_delete_bus
            .to_string()
            .contains("still referenced by bus entry")
    );

    let delete_entry_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-bus-entry",
        root.to_str().unwrap(),
        "--bus-entry",
        &bus_entry_uuid,
    ])
    .expect("CLI should parse");
    let delete_entry_output =
        execute(delete_entry_cli).expect("project delete-bus-entry should succeed");
    assert!(delete_entry_output.contains("action: delete_bus_entry"));
    assert_bus_journal_transaction(&root, 3, "delete schematic bus entry", 1);

    let query_entries_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "bus-entries",
    ])
    .expect("CLI should parse");
    let entries_output =
        execute(query_entries_cli).expect("project query bus-entries should succeed");
    let entries: serde_json::Value =
        serde_json::from_str(&entries_output).expect("bus entries JSON should parse");
    assert_eq!(entries.as_array().unwrap().len(), 0);

    let delete_bus_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "delete-bus",
        root.to_str().unwrap(),
        "--bus",
        &bus_uuid,
    ])
    .expect("CLI should parse");
    let delete_bus_output = execute(delete_bus_cli).expect("project delete-bus should succeed");
    let deleted_bus: serde_json::Value =
        serde_json::from_str(&delete_bus_output).expect("delete-bus JSON should parse");
    assert_eq!(deleted_bus["action"], "delete_bus");
    assert_eq!(deleted_bus["bus_uuid"], bus_uuid);
    assert_bus_journal_transaction(&root, 4, "delete schematic bus", 1);

    let query_buses_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "buses",
    ])
    .expect("CLI should parse");
    let buses_output = execute(query_buses_cli).expect("project query buses should succeed");
    let buses: serde_json::Value =
        serde_json::from_str(&buses_output).expect("buses JSON should parse");
    assert_eq!(buses.as_array().unwrap().len(), 0);

    let _ = std::fs::remove_dir_all(&root);
}
