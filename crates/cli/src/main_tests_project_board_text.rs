use super::*;
use eda_engine::board::BoardText;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_text_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-texts",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_board_text_mutations_round_trip_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-text");
    create_native_project(&root, Some("Board Text Demo".to_string()))
        .expect("initial scaffold should succeed");

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-text",
        root.to_str().unwrap(),
        "--text",
        "PCB TOP",
        "--x-nm",
        "1000",
        "--y-nm",
        "2000",
        "--rotation-deg",
        "90",
        "--layer",
        "1",
    ])
    .expect("CLI should parse");

    let placed_output = execute(place_cli).expect("place board text should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&placed_output).expect("place output should parse");
    let text_uuid = placed["text_uuid"].as_str().unwrap().to_string();

    let texts_output =
        execute(board_text_query_cli(&root)).expect("board text query should succeed");
    let texts: Vec<BoardText> =
        serde_json::from_str(&texts_output).expect("query output should parse");
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].uuid.to_string(), text_uuid);
    assert_eq!(texts[0].text, "PCB TOP");
    assert_eq!(texts[0].position.x, 1000);
    assert_eq!(texts[0].position.y, 2000);
    assert_eq!(texts[0].rotation, 90);
    assert_eq!(texts[0].layer, 1);

    let edit_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "edit-board-text",
        root.to_str().unwrap(),
        "--text",
        &text_uuid,
        "--value",
        "PCB BOT",
        "--x-nm",
        "3000",
        "--y-nm",
        "4000",
        "--rotation-deg",
        "180",
        "--layer",
        "2",
    ])
    .expect("CLI should parse");
    let _ = execute(edit_cli).expect("edit board text should succeed");

    let texts_output =
        execute(board_text_query_cli(&root)).expect("board text query should succeed");
    let texts: Vec<BoardText> =
        serde_json::from_str(&texts_output).expect("query output should parse");
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "PCB BOT");
    assert_eq!(texts[0].position.x, 3000);
    assert_eq!(texts[0].position.y, 4000);
    assert_eq!(texts[0].rotation, 180);
    assert_eq!(texts[0].layer, 2);

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-board-text",
        root.to_str().unwrap(),
        "--text",
        &text_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("delete board text should succeed");
    assert!(delete_output.contains("action: delete_board_text"));

    let texts_output =
        execute(board_text_query_cli(&root)).expect("board text query should succeed");
    let texts: Vec<BoardText> =
        serde_json::from_str(&texts_output).expect("query output should parse");
    assert!(texts.is_empty());

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_texts: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_texts_reads_existing_native_board_file() {
    let root = unique_project_root("datum-eda-cli-project-board-text-query");
    create_native_project(&root, Some("Board Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Query Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "keepouts": [],
                "dimensions": [],
                "texts": [{
                    "uuid": Uuid::new_v4(),
                    "text": "FAB",
                    "position": { "x": 10, "y": 20 },
                    "rotation": 0,
                    "layer": 21
                }]
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output = execute(board_text_query_cli(&root)).expect("board text query should succeed");
    let texts: Vec<BoardText> = serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "FAB");
    assert_eq!(texts[0].layer, 21);

    let _ = std::fs::remove_dir_all(&root);
}
