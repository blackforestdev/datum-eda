use super::*;
use eda_engine::board::PlacedPackage;
use eda_engine::ir::serialization::to_json_deterministic;

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
fn project_query_board_components_reads_existing_native_board_file() {
    let root = unique_project_root("datum-eda-cli-project-board-component-query");
    create_native_project(&root, Some("Board Component Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let component_uuid = Uuid::new_v4();
    let part_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Component Query Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    component_uuid.to_string(): {
                        "uuid": component_uuid,
                        "part": part_uuid,
                        "package": package_uuid,
                        "reference": "U1",
                        "value": "MCU",
                        "position": { "x": 1000, "y": 2000 },
                        "rotation": 90,
                        "layer": 1,
                        "locked": false
                    }
                },
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
        execute(board_components_query_cli(&root)).expect("board components query should succeed");
    let components: Vec<PlacedPackage> =
        serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].uuid, component_uuid);
    assert_eq!(components[0].part, part_uuid);
    assert_eq!(components[0].package, package_uuid);
    assert_eq!(components[0].reference, "U1");
    assert_eq!(components[0].value, "MCU");
    assert_eq!(components[0].position.x, 1000);
    assert_eq!(components[0].rotation, 90);
    assert_eq!(components[0].layer, 1);
    assert!(!components[0].locked);

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_components: 1"));

    let _ = std::fs::remove_dir_all(&root);
}
