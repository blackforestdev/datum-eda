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
fn project_place_edit_and_delete_drawing_line_update_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-drawing");
    create_native_project(&root, Some("Drawing Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-drawing-line",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--from-x-nm",
        "100",
        "--from-y-nm",
        "200",
        "--to-x-nm",
        "300",
        "--to-y-nm",
        "400",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-drawing-line should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-drawing-line JSON should parse");
    let drawing_uuid = placed["drawing_uuid"].as_str().unwrap().to_string();
    assert_eq!(placed["kind"], "line");

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "drawings",
    ])
    .expect("CLI should parse");
    let drawings_output = execute(query_cli).expect("project query drawings should succeed");
    let drawings: serde_json::Value =
        serde_json::from_str(&drawings_output).expect("drawings JSON should parse");
    assert_eq!(drawings.as_array().unwrap().len(), 1);
    assert_eq!(drawings[0]["uuid"], drawing_uuid);
    assert_eq!(drawings[0]["sheet"], sheet_uuid.to_string());
    assert_eq!(drawings[0]["kind"], "line");
    assert_eq!(drawings[0]["from"]["x"], 100);
    assert_eq!(drawings[0]["from"]["y"], 200);
    assert_eq!(drawings[0]["to"]["x"], 300);
    assert_eq!(drawings[0]["to"]["y"], 400);

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_drawings: 1"));

    let edit_cli = Cli::try_parse_from([
        "eda",
        "project",
        "edit-drawing-line",
        root.to_str().unwrap(),
        "--drawing",
        &drawing_uuid,
        "--from-x-nm",
        "500",
        "--from-y-nm",
        "600",
        "--to-x-nm",
        "700",
        "--to-y-nm",
        "800",
    ])
    .expect("CLI should parse");
    let edit_output = execute(edit_cli).expect("project edit-drawing-line should succeed");
    assert!(edit_output.contains("action: edit_drawing_line"));
    assert!(edit_output.contains("from_x_nm: 500"));
    assert!(edit_output.contains("to_y_nm: 800"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "drawings",
    ])
    .expect("CLI should parse");
    let drawings_output = execute(query_cli).expect("project query drawings should succeed");
    let drawings: serde_json::Value =
        serde_json::from_str(&drawings_output).expect("drawings JSON should parse");
    assert_eq!(drawings.as_array().unwrap().len(), 1);
    assert_eq!(drawings[0]["from"]["x"], 500);
    assert_eq!(drawings[0]["from"]["y"], 600);
    assert_eq!(drawings[0]["to"]["x"], 700);
    assert_eq!(drawings[0]["to"]["y"], 800);

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-drawing",
        root.to_str().unwrap(),
        "--drawing",
        &drawing_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("project delete-drawing should succeed");
    assert!(delete_output.contains("action: delete_drawing"));
    assert!(delete_output.contains("kind: line"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "drawings",
    ])
    .expect("CLI should parse");
    let drawings_output = execute(query_cli).expect("project query drawings should succeed");
    let drawings: serde_json::Value =
        serde_json::from_str(&drawings_output).expect("drawings JSON should parse");
    assert_eq!(drawings.as_array().unwrap().len(), 0);

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_drawings: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_place_and_edit_rect_circle_and_arc_update_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-drawing-shapes");
    create_native_project(&root, Some("Drawing Shapes Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);

    let rect_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-drawing-rect",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--min-x-nm",
        "10",
        "--min-y-nm",
        "20",
        "--max-x-nm",
        "30",
        "--max-y-nm",
        "40",
    ])
    .expect("CLI should parse");
    let rect_output = execute(rect_cli).expect("project place-drawing-rect should succeed");
    let rect: serde_json::Value =
        serde_json::from_str(&rect_output).expect("place-drawing-rect JSON should parse");
    let rect_uuid = rect["drawing_uuid"].as_str().unwrap().to_string();
    assert_eq!(rect["kind"], "rect");

    let circle_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-drawing-circle",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--center-x-nm",
        "100",
        "--center-y-nm",
        "200",
        "--radius-nm",
        "50",
    ])
    .expect("CLI should parse");
    let circle_output = execute(circle_cli).expect("project place-drawing-circle should succeed");
    let circle: serde_json::Value =
        serde_json::from_str(&circle_output).expect("place-drawing-circle JSON should parse");
    let circle_uuid = circle["drawing_uuid"].as_str().unwrap().to_string();
    assert_eq!(circle["kind"], "circle");

    let arc_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-drawing-arc",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--center-x-nm",
        "500",
        "--center-y-nm",
        "600",
        "--radius-nm",
        "70",
        "--start-angle-mdeg",
        "0",
        "--end-angle-mdeg",
        "90000",
    ])
    .expect("CLI should parse");
    let arc_output = execute(arc_cli).expect("project place-drawing-arc should succeed");
    let arc: serde_json::Value =
        serde_json::from_str(&arc_output).expect("place-drawing-arc JSON should parse");
    let arc_uuid = arc["drawing_uuid"].as_str().unwrap().to_string();
    assert_eq!(arc["kind"], "arc");

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "drawings",
    ])
    .expect("CLI should parse");
    let drawings_output = execute(query_cli).expect("project query drawings should succeed");
    let drawings: serde_json::Value =
        serde_json::from_str(&drawings_output).expect("drawings JSON should parse");
    assert_eq!(drawings.as_array().unwrap().len(), 3);

    let edit_rect_cli = Cli::try_parse_from([
        "eda",
        "project",
        "edit-drawing-rect",
        root.to_str().unwrap(),
        "--drawing",
        &rect_uuid,
        "--min-x-nm",
        "11",
        "--min-y-nm",
        "22",
        "--max-x-nm",
        "33",
        "--max-y-nm",
        "44",
    ])
    .expect("CLI should parse");
    let edit_rect_output =
        execute(edit_rect_cli).expect("project edit-drawing-rect should succeed");
    assert!(edit_rect_output.contains("action: edit_drawing_rect"));

    let edit_circle_cli = Cli::try_parse_from([
        "eda",
        "project",
        "edit-drawing-circle",
        root.to_str().unwrap(),
        "--drawing",
        &circle_uuid,
        "--center-x-nm",
        "101",
        "--center-y-nm",
        "202",
        "--radius-nm",
        "55",
    ])
    .expect("CLI should parse");
    let edit_circle_output =
        execute(edit_circle_cli).expect("project edit-drawing-circle should succeed");
    assert!(edit_circle_output.contains("action: edit_drawing_circle"));

    let edit_arc_cli = Cli::try_parse_from([
        "eda",
        "project",
        "edit-drawing-arc",
        root.to_str().unwrap(),
        "--drawing",
        &arc_uuid,
        "--center-x-nm",
        "501",
        "--center-y-nm",
        "602",
        "--radius-nm",
        "75",
        "--start-angle-mdeg",
        "1000",
        "--end-angle-mdeg",
        "91000",
    ])
    .expect("CLI should parse");
    let edit_arc_output = execute(edit_arc_cli).expect("project edit-drawing-arc should succeed");
    assert!(edit_arc_output.contains("action: edit_drawing_arc"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "drawings",
    ])
    .expect("CLI should parse");
    let drawings_output = execute(query_cli).expect("project query drawings should succeed");
    let drawings: serde_json::Value =
        serde_json::from_str(&drawings_output).expect("drawings JSON should parse");
    assert_eq!(drawings.as_array().unwrap().len(), 3);
    assert!(
        drawings
            .as_array()
            .unwrap()
            .iter()
            .any(|drawing| drawing["kind"] == "rect")
    );
    assert!(
        drawings
            .as_array()
            .unwrap()
            .iter()
            .any(|drawing| drawing["kind"] == "circle")
    );
    assert!(
        drawings
            .as_array()
            .unwrap()
            .iter()
            .any(|drawing| drawing["kind"] == "arc")
    );

    let _ = std::fs::remove_dir_all(&root);
}
