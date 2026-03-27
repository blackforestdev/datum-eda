use super::*;

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

use eda_engine::ir::serialization::to_json_deterministic;

#[test]
fn project_place_label_writes_native_sheet_and_query_labels_reports_it() {
    let root = unique_project_root("datum-eda-cli-project-place-label");
    create_native_project(&root, Some("Label Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);

    let cli = Cli::try_parse_from([
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
        "123",
        "--y-nm",
        "456",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project place-label should succeed");
    assert!(output.contains("name: VIN"));
    assert!(output.contains("kind: global"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "labels",
    ])
    .expect("CLI should parse");
    let labels_output = execute(query_cli).expect("project query labels should succeed");
    let labels: serde_json::Value =
        serde_json::from_str(&labels_output).expect("labels JSON should parse");
    assert_eq!(labels.as_array().unwrap().len(), 1);
    assert_eq!(labels[0]["name"], "VIN");
    assert_eq!(labels[0]["kind"], "Global");
    assert_eq!(labels[0]["position"]["x"], 123);
    assert_eq!(labels[0]["position"]["y"], 456);

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_labels: 1"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_rename_and_delete_label_update_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-rename-delete-label");
    create_native_project(&root, Some("Label Edit Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-label",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--name",
        "OLD_NAME",
        "--x-nm",
        "10",
        "--y-nm",
        "20",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-label should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-label JSON should parse");
    let label_uuid = placed["label_uuid"].as_str().unwrap().to_string();

    let rename_cli = Cli::try_parse_from([
        "eda",
        "project",
        "rename-label",
        root.to_str().unwrap(),
        "--label",
        &label_uuid,
        "--name",
        "NEW_NAME",
    ])
    .expect("CLI should parse");
    let rename_output = execute(rename_cli).expect("project rename-label should succeed");
    assert!(rename_output.contains("action: rename_label"));
    assert!(rename_output.contains("name: NEW_NAME"));

    let labels_query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "labels",
    ])
    .expect("CLI should parse");
    let renamed_labels_output =
        execute(labels_query_cli).expect("project query labels should succeed");
    let renamed_labels: serde_json::Value =
        serde_json::from_str(&renamed_labels_output).expect("labels JSON should parse");
    assert_eq!(renamed_labels.as_array().unwrap().len(), 1);
    assert_eq!(renamed_labels[0]["uuid"], label_uuid);
    assert_eq!(renamed_labels[0]["name"], "NEW_NAME");

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-label",
        root.to_str().unwrap(),
        "--label",
        &label_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("project delete-label should succeed");
    assert!(delete_output.contains("action: delete_label"));
    assert!(delete_output.contains("name: NEW_NAME"));

    let labels_query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "labels",
    ])
    .expect("CLI should parse");
    let deleted_labels_output =
        execute(labels_query_cli).expect("project query labels should succeed");
    let deleted_labels: serde_json::Value =
        serde_json::from_str(&deleted_labels_output).expect("labels JSON should parse");
    assert_eq!(deleted_labels.as_array().unwrap().len(), 0);

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_labels: 0"));

    let _ = std::fs::remove_dir_all(&root);
}
