use super::*;
use eda_engine::board::PlacedPackage;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_components_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-components",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_board_component_place_move_rotate_and_lock_round_trip_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-component");
    create_native_project(&root, Some("Board Component Demo".to_string()))
        .expect("initial scaffold should succeed");

    let part_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-component",
        root.to_str().unwrap(),
        "--part",
        &part_uuid.to_string(),
        "--package",
        &package_uuid.to_string(),
        "--reference",
        "U1",
        "--value",
        "MCU",
        "--x-nm",
        "1000",
        "--y-nm",
        "2000",
        "--layer",
        "1",
    ])
    .expect("CLI should parse");

    let placed_output = execute(place_cli).expect("place board component should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&placed_output).expect("place output should parse");
    let component_uuid = placed["component_uuid"].as_str().unwrap().to_string();

    let components_output = execute(board_components_query_cli(&root))
        .expect("board components query should succeed");
    let components: Vec<PlacedPackage> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].uuid.to_string(), component_uuid);
    assert_eq!(components[0].part, part_uuid);
    assert_eq!(components[0].package, package_uuid);
    assert_eq!(components[0].reference, "U1");
    assert_eq!(components[0].value, "MCU");
    assert_eq!(components[0].position.x, 1000);
    assert_eq!(components[0].position.y, 2000);
    assert_eq!(components[0].rotation, 0);
    assert_eq!(components[0].layer, 1);
    assert!(!components[0].locked);

    let move_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "move-board-component",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
        "--x-nm",
        "3000",
        "--y-nm",
        "4000",
    ])
    .expect("CLI should parse");
    let _ = execute(move_cli).expect("move board component should succeed");

    let components_output = execute(board_components_query_cli(&root))
        .expect("board components query should succeed");
    let components: Vec<PlacedPackage> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].position.x, 3000);
    assert_eq!(components[0].position.y, 4000);
    assert_eq!(components[0].rotation, 0);
    assert_eq!(components[0].layer, 1);
    assert!(!components[0].locked);

    let rotate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "rotate-board-component",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
        "--rotation-deg",
        "180",
    ])
    .expect("CLI should parse");
    let _ = execute(rotate_cli).expect("rotate board component should succeed");

    let components_output = execute(board_components_query_cli(&root))
        .expect("board components query should succeed");
    let components: Vec<PlacedPackage> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].rotation, 180);
    assert!(!components[0].locked);

    let lock_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-board-component-locked",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
    ])
    .expect("CLI should parse");
    let _ = execute(lock_cli).expect("lock board component should succeed");

    let components_output = execute(board_components_query_cli(&root))
        .expect("board components query should succeed");
    let components: Vec<PlacedPackage> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].rotation, 180);
    assert!(components[0].locked);

    let unlock_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "clear-board-component-locked",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
    ])
    .expect("CLI should parse");
    let _ = execute(unlock_cli).expect("unlock board component should succeed");

    let components_output = execute(board_components_query_cli(&root))
        .expect("board components query should succeed");
    let components: Vec<PlacedPackage> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].rotation, 180);
    assert!(!components[0].locked);

    let delete_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "delete-board-component",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
    ])
    .expect("CLI should parse");
    let deleted_output = execute(delete_cli).expect("delete board component should succeed");
    let deleted: serde_json::Value =
        serde_json::from_str(&deleted_output).expect("delete output should parse");
    assert_eq!(deleted["action"].as_str(), Some("delete_board_component"));
    assert_eq!(deleted["component_uuid"].as_str(), Some(component_uuid.as_str()));

    let components_output = execute(board_components_query_cli(&root))
        .expect("board components query should succeed");
    let components: Vec<PlacedPackage> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert!(components.is_empty());

    let summary_cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "summary",
    ])
    .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_components: 0"));

    let _ = std::fs::remove_dir_all(&root);
}
