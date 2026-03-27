use super::*;
use eda_engine::board::Keepout;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_keepout_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-keepouts",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_board_keepout_mutations_round_trip_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-keepout");
    create_native_project(&root, Some("Board Keepout Demo".to_string()))
        .expect("initial scaffold should succeed");

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-keepout",
        root.to_str().unwrap(),
        "--kind",
        "copper",
        "--layer",
        "1",
        "--layer",
        "16",
        "--vertex",
        "0:0",
        "--vertex",
        "1000:0",
        "--vertex",
        "1000:500",
        "--vertex",
        "0:500",
    ])
    .expect("CLI should parse");

    let placed_output = execute(place_cli).expect("place board keepout should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&placed_output).expect("place output should parse");
    let keepout_uuid = placed["keepout_uuid"].as_str().unwrap().to_string();

    let keepouts_output =
        execute(board_keepout_query_cli(&root)).expect("board keepout query should succeed");
    let keepouts: Vec<Keepout> =
        serde_json::from_str(&keepouts_output).expect("query output should parse");
    assert_eq!(keepouts.len(), 1);
    assert_eq!(keepouts[0].uuid.to_string(), keepout_uuid);
    assert_eq!(keepouts[0].kind, "copper");
    assert_eq!(keepouts[0].layers, vec![1, 16]);
    assert_eq!(keepouts[0].polygon.vertices.len(), 4);
    assert!(keepouts[0].polygon.closed);

    let edit_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "edit-board-keepout",
        root.to_str().unwrap(),
        "--keepout",
        &keepout_uuid,
        "--kind",
        "mixed",
        "--layer",
        "2",
        "--vertex",
        "10:10",
        "--vertex",
        "1010:10",
        "--vertex",
        "1010:510",
        "--vertex",
        "10:510",
    ])
    .expect("CLI should parse");
    let _ = execute(edit_cli).expect("edit board keepout should succeed");

    let keepouts_output =
        execute(board_keepout_query_cli(&root)).expect("board keepout query should succeed");
    let keepouts: Vec<Keepout> =
        serde_json::from_str(&keepouts_output).expect("query output should parse");
    assert_eq!(keepouts.len(), 1);
    assert_eq!(keepouts[0].kind, "mixed");
    assert_eq!(keepouts[0].layers, vec![2]);
    assert_eq!(keepouts[0].polygon.vertices[0].x, 10);
    assert_eq!(keepouts[0].polygon.vertices[0].y, 10);

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-board-keepout",
        root.to_str().unwrap(),
        "--keepout",
        &keepout_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("delete board keepout should succeed");
    assert!(delete_output.contains("action: delete_board_keepout"));

    let keepouts_output =
        execute(board_keepout_query_cli(&root)).expect("board keepout query should succeed");
    let keepouts: Vec<Keepout> =
        serde_json::from_str(&keepouts_output).expect("query output should parse");
    assert!(keepouts.is_empty());

    let summary_cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "summary",
    ])
    .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_keepouts: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_keepouts_reads_existing_native_board_file() {
    let root = unique_project_root("datum-eda-cli-project-board-keepout-query");
    create_native_project(&root, Some("Board Keepout Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let keepout_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Keepout Query Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "keepouts": [{
                    "uuid": keepout_uuid,
                    "polygon": {
                        "vertices": [
                            { "x": 0, "y": 0 },
                            { "x": 500, "y": 0 },
                            { "x": 500, "y": 500 }
                        ],
                        "closed": true
                    },
                    "layers": [1],
                    "kind": "via"
                }],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output = execute(board_keepout_query_cli(&root)).expect("board keepout query should succeed");
    let keepouts: Vec<Keepout> = serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(keepouts.len(), 1);
    assert_eq!(keepouts[0].uuid, keepout_uuid);
    assert_eq!(keepouts[0].kind, "via");
    assert_eq!(keepouts[0].layers, vec![1]);
    assert_eq!(keepouts[0].polygon.vertices.len(), 3);

    let _ = std::fs::remove_dir_all(&root);
}
