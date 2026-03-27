use super::*;
use eda_engine::ir::geometry::Polygon;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_outline_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-outline",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_board_outline_replacement_round_trips_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-outline");
    create_native_project(&root, Some("Board Outline Demo".to_string()))
        .expect("initial scaffold should succeed");

    let set_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-board-outline",
        root.to_str().unwrap(),
        "--vertex",
        "0:0",
        "--vertex",
        "2000:0",
        "--vertex",
        "1500:1000",
        "--vertex",
        "0:1000",
    ])
    .expect("CLI should parse");

    let output = execute(set_cli).expect("set board outline should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");
    assert_eq!(report["vertex_count"], 4);
    assert_eq!(report["closed"], true);

    let outline_output =
        execute(board_outline_query_cli(&root)).expect("board outline query should succeed");
    let outline: Polygon =
        serde_json::from_str(&outline_output).expect("query output should parse");
    assert_eq!(outline.vertices.len(), 4);
    assert!(outline.closed);
    assert_eq!(outline.vertices[1].x, 2000);
    assert_eq!(outline.vertices[2].y, 1000);

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_dimensions: 0"));
    assert!(summary_output.contains("board_keepouts: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_outline_reads_existing_native_board_file() {
    let root = unique_project_root("datum-eda-cli-project-board-outline-query");
    create_native_project(&root, Some("Board Outline Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Outline Query Demo Board",
                "stackup": { "layers": [] },
                "outline": {
                    "vertices": [
                        { "x": 1, "y": 2 },
                        { "x": 3, "y": 4 },
                        { "x": 5, "y": 6 }
                    ],
                    "closed": true
                },
                "packages": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output =
        execute(board_outline_query_cli(&root)).expect("board outline query should succeed");
    let outline: Polygon = serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(outline.vertices.len(), 3);
    assert_eq!(outline.vertices[0].x, 1);
    assert_eq!(outline.vertices[2].y, 6);
    assert!(outline.closed);

    let _ = std::fs::remove_dir_all(&root);
}
