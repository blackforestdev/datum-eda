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

#[test]
fn project_place_edit_and_delete_text_update_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-text");
    create_native_project(&root, Some("Text Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-text",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--text",
        "VIN",
        "--x-nm",
        "700",
        "--y-nm",
        "800",
        "--rotation-deg",
        "90",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-text should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-text JSON should parse");
    let text_uuid = placed["text_uuid"].as_str().unwrap().to_string();
    assert_eq!(placed["text"], "VIN");
    assert_eq!(placed["rotation_deg"], 90);

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "texts",
    ])
    .expect("CLI should parse");
    let texts_output = execute(query_cli).expect("project query texts should succeed");
    let texts: serde_json::Value =
        serde_json::from_str(&texts_output).expect("texts JSON should parse");
    assert_eq!(texts.as_array().unwrap().len(), 1);
    assert_eq!(texts[0]["uuid"], text_uuid);
    assert_eq!(texts[0]["sheet"], sheet_uuid.to_string());
    assert_eq!(texts[0]["text"], "VIN");
    assert_eq!(texts[0]["position"]["x"], 700);
    assert_eq!(texts[0]["position"]["y"], 800);
    assert_eq!(texts[0]["rotation"], 90);

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_texts: 1"));

    let edit_cli = Cli::try_parse_from([
        "eda",
        "project",
        "edit-text",
        root.to_str().unwrap(),
        "--text",
        &text_uuid,
        "--value",
        "VOUT",
        "--x-nm",
        "900",
        "--y-nm",
        "1000",
        "--rotation-deg",
        "180",
    ])
    .expect("CLI should parse");
    let edit_output = execute(edit_cli).expect("project edit-text should succeed");
    assert!(edit_output.contains("action: edit_text"));
    assert!(edit_output.contains("text: VOUT"));
    assert!(edit_output.contains("rotation_deg: 180"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "texts",
    ])
    .expect("CLI should parse");
    let texts_output = execute(query_cli).expect("project query texts should succeed");
    let texts: serde_json::Value =
        serde_json::from_str(&texts_output).expect("texts JSON should parse");
    assert_eq!(texts.as_array().unwrap().len(), 1);
    assert_eq!(texts[0]["text"], "VOUT");
    assert_eq!(texts[0]["position"]["x"], 900);
    assert_eq!(texts[0]["position"]["y"], 1000);
    assert_eq!(texts[0]["rotation"], 180);

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-text",
        root.to_str().unwrap(),
        "--text",
        &text_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("project delete-text should succeed");
    assert!(delete_output.contains("action: delete_text"));
    assert!(delete_output.contains("text: VOUT"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "texts",
    ])
    .expect("CLI should parse");
    let texts_output = execute(query_cli).expect("project query texts should succeed");
    let texts: serde_json::Value =
        serde_json::from_str(&texts_output).expect("texts JSON should parse");
    assert_eq!(texts.as_array().unwrap().len(), 0);

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_texts: 0"));

    let _ = std::fs::remove_dir_all(&root);
}
