use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_export_gerber_outline_writes_rs274x_file() {
    let root = unique_project_root("datum-eda-cli-project-gerber-outline-export");
    create_native_project(&root, Some("Gerber Outline Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Outline Demo Board",
                "stackup": { "layers": [] },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 1000000, "y": 0 },
                        { "x": 1000000, "y": 500000 },
                        { "x": 0, "y": 500000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "rules": [],
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let gerber_path = root.join("outline.gbr");
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-outline",
        root.to_str().unwrap(),
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("gerber outline export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "export_gerber_outline");
    assert_eq!(report["production_classification"], "manual_debug_export");
    assert_eq!(report["outline_vertex_count"], 4);
    assert_eq!(report["outline_closed"], true);

    let gerber = std::fs::read_to_string(&gerber_path).expect("gerber should read");
    assert!(gerber.contains("%FSLAX46Y46*%"));
    assert!(gerber.contains("%MOMM*%"));
    assert!(gerber.contains("%ADD10C,0.100000*%"));
    assert!(gerber.contains("X0Y0D02*"));
    assert!(gerber.contains("X1000000Y0D01*"));
    assert!(gerber.contains("X1000000Y500000D01*"));
    assert!(gerber.contains("X0Y500000D01*"));
    assert!(gerber.contains("X0Y0D01*"));
    assert!(gerber.ends_with("M02*\n"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_export_gerber_outline_uses_resolver_materialized_board_state() {
    let root = unique_project_root("datum-eda-cli-project-gerber-outline-resolved-export");
    create_native_project(
        &root,
        Some("Gerber Outline Resolved Export Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let board_json = root.join("board/board.json");
    let stale_board = std::fs::read_to_string(&board_json).expect("board file should read");

    let set_outline_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-board-outline",
        root.to_str().unwrap(),
        "--vertex",
        "0:0",
        "--vertex",
        "2000000:0",
        "--vertex",
        "2000000:1000000",
        "--vertex",
        "0:1000000",
    ])
    .expect("CLI should parse");
    let _ = execute(set_outline_cli).expect("set board outline should succeed");
    std::fs::write(&board_json, stale_board).expect("stale board file should restore");

    let gerber_path = root.join("outline-resolved.gbr");
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-outline",
        root.to_str().unwrap(),
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("gerber outline export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["outline_vertex_count"], 4);

    let gerber = std::fs::read_to_string(&gerber_path).expect("gerber should read");
    assert!(gerber.contains("X2000000Y0D01*"));
    assert!(gerber.contains("X2000000Y1000000D01*"));
    assert!(gerber.contains("X0Y1000000D01*"));

    let _ = std::fs::remove_dir_all(&root);
}
