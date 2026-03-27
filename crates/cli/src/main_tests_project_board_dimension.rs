use super::*;
use eda_engine::board::Dimension;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_dimension_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-dimensions",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_board_dimension_mutations_round_trip_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-dimension");
    create_native_project(&root, Some("Board Dimension Demo".to_string()))
        .expect("initial scaffold should succeed");

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-dimension",
        root.to_str().unwrap(),
        "--from-x-nm",
        "0",
        "--from-y-nm",
        "0",
        "--to-x-nm",
        "1000",
        "--to-y-nm",
        "500",
        "--text",
        "1000x500",
    ])
    .expect("CLI should parse");

    let placed_output = execute(place_cli).expect("place board dimension should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&placed_output).expect("place output should parse");
    let dimension_uuid = placed["dimension_uuid"].as_str().unwrap().to_string();

    let dimensions_output =
        execute(board_dimension_query_cli(&root)).expect("board dimension query should succeed");
    let dimensions: Vec<Dimension> =
        serde_json::from_str(&dimensions_output).expect("query output should parse");
    assert_eq!(dimensions.len(), 1);
    assert_eq!(dimensions[0].uuid.to_string(), dimension_uuid);
    assert_eq!(dimensions[0].from.x, 0);
    assert_eq!(dimensions[0].from.y, 0);
    assert_eq!(dimensions[0].to.x, 1000);
    assert_eq!(dimensions[0].to.y, 500);
    assert_eq!(dimensions[0].text.as_deref(), Some("1000x500"));

    let edit_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "edit-board-dimension",
        root.to_str().unwrap(),
        "--dimension",
        &dimension_uuid,
        "--from-x-nm",
        "10",
        "--from-y-nm",
        "20",
        "--to-x-nm",
        "1010",
        "--to-y-nm",
        "520",
        "--text",
        "revised",
    ])
    .expect("CLI should parse");
    let _ = execute(edit_cli).expect("edit board dimension should succeed");

    let dimensions_output =
        execute(board_dimension_query_cli(&root)).expect("board dimension query should succeed");
    let dimensions: Vec<Dimension> =
        serde_json::from_str(&dimensions_output).expect("query output should parse");
    assert_eq!(dimensions.len(), 1);
    assert_eq!(dimensions[0].from.x, 10);
    assert_eq!(dimensions[0].from.y, 20);
    assert_eq!(dimensions[0].to.x, 1010);
    assert_eq!(dimensions[0].to.y, 520);
    assert_eq!(dimensions[0].text.as_deref(), Some("revised"));

    let clear_text_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "edit-board-dimension",
        root.to_str().unwrap(),
        "--dimension",
        &dimension_uuid,
        "--clear-text",
    ])
    .expect("CLI should parse");
    let _ = execute(clear_text_cli).expect("clear board dimension text should succeed");

    let dimensions_output =
        execute(board_dimension_query_cli(&root)).expect("board dimension query should succeed");
    let dimensions: Vec<Dimension> =
        serde_json::from_str(&dimensions_output).expect("query output should parse");
    assert_eq!(dimensions.len(), 1);
    assert_eq!(dimensions[0].text, None);

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-board-dimension",
        root.to_str().unwrap(),
        "--dimension",
        &dimension_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("delete board dimension should succeed");
    assert!(delete_output.contains("action: delete_board_dimension"));

    let dimensions_output =
        execute(board_dimension_query_cli(&root)).expect("board dimension query should succeed");
    let dimensions: Vec<Dimension> =
        serde_json::from_str(&dimensions_output).expect("query output should parse");
    assert!(dimensions.is_empty());

    let summary_cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "summary",
    ])
    .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_dimensions: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_dimensions_reads_existing_native_board_file() {
    let root = unique_project_root("datum-eda-cli-project-board-dimension-query");
    create_native_project(&root, Some("Board Dimension Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let dimension_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Dimension Query Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "keepouts": [],
                "dimensions": [{
                    "uuid": dimension_uuid,
                    "from": { "x": 5, "y": 10 },
                    "to": { "x": 15, "y": 20 },
                    "text": "10mm"
                }],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output = execute(board_dimension_query_cli(&root)).expect("board dimension query should succeed");
    let dimensions: Vec<Dimension> = serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(dimensions.len(), 1);
    assert_eq!(dimensions[0].uuid, dimension_uuid);
    assert_eq!(dimensions[0].from.x, 5);
    assert_eq!(dimensions[0].to.y, 20);
    assert_eq!(dimensions[0].text.as_deref(), Some("10mm"));

    let _ = std::fs::remove_dir_all(&root);
}
