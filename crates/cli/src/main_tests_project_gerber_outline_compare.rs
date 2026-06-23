use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_compare_gerber_outline_is_semantic_and_reports_drift() {
    let root = unique_project_root("datum-eda-cli-project-gerber-outline-compare");
    create_native_project(&root, Some("Gerber Outline Compare Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Outline Compare Demo Board",
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
    std::fs::write(
        &gerber_path,
        concat!(
            "G04 semantic outline compare fixture*\n",
            "%MOMM*%\n",
            "%FSLAX46Y46*%\n",
            "%LPD*%\n",
            "%ADD11C,0.100000*%\n",
            "D11*\n",
            "X0Y0D02*\n",
            "X1000000Y0D01*\n",
            "X1000000Y500000D01*\n",
            "X0Y500000D01*\n",
            "X0Y0D01*\n",
            "M02*\n"
        ),
    )
    .expect("gerber file should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-outline",
        root.to_str().unwrap(),
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("outline compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "compare_gerber_outline");
    assert_eq!(report["expected_outline_count"], 1);
    assert_eq!(report["actual_geometry_count"], 1);
    assert_eq!(report["matched_count"], 1);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 0);
    assert_eq!(report["matched"][0]["kind"], "outline");

    std::fs::write(
        &gerber_path,
        concat!(
            "G04 semantic outline drift fixture*\n",
            "%FSLAX46Y46*%\n",
            "%MOMM*%\n",
            "%LPD*%\n",
            "%ADD10C,0.150000*%\n",
            "D10*\n",
            "X0Y0D02*\n",
            "X1000000Y0D01*\n",
            "X1000000Y500000D01*\n",
            "X0Y500000D01*\n",
            "X0Y0D01*\n",
            "M02*\n"
        ),
    )
    .expect("drift gerber should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-outline",
        root.to_str().unwrap(),
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("outline compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["matched_count"], 0);
    assert_eq!(report["missing_count"], 1);
    assert_eq!(report["extra_count"], 1);
    assert_eq!(report["missing"][0]["kind"], "outline");
    assert_eq!(report["extra"][0]["kind"], "outline");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_compare_gerber_outline_uses_resolver_materialized_board_state() {
    let root = unique_project_root("datum-eda-cli-project-gerber-outline-resolved-compare");
    create_native_project(
        &root,
        Some("Gerber Outline Resolved Compare Demo".to_string()),
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
    std::fs::write(
        &gerber_path,
        concat!(
            "G04 semantic resolved outline fixture*\n",
            "%MOMM*%\n",
            "%FSLAX46Y46*%\n",
            "%LPD*%\n",
            "%ADD11C,0.100000*%\n",
            "D11*\n",
            "X0Y0D02*\n",
            "X2000000Y0D01*\n",
            "X2000000Y1000000D01*\n",
            "X0Y1000000D01*\n",
            "X0Y0D01*\n",
            "M02*\n"
        ),
    )
    .expect("gerber file should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-outline",
        root.to_str().unwrap(),
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("outline compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["matched_count"], 1);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 0);
    assert_eq!(report["matched"][0]["kind"], "outline");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_compare_gerber_outline_matches_equivalent_closed_loop_with_shifted_start() {
    let root = unique_project_root("datum-eda-cli-project-gerber-outline-compare-rotated");
    create_native_project(
        &root,
        Some("Gerber Outline Compare Rotated Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Outline Compare Rotated Demo Board",
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

    let gerber_path = root.join("outline-rotated.gbr");
    std::fs::write(
        &gerber_path,
        concat!(
            "G04 rotated-start outline fixture*\n",
            "%FSLAX46Y46*%\n",
            "%MOMM*%\n",
            "%LPD*%\n",
            "%ADD17C,0.100000*%\n",
            "D17*\n",
            "X1000000Y500000D02*\n",
            "X0Y500000D01*\n",
            "X0Y0D01*\n",
            "X1000000Y0D01*\n",
            "X1000000Y500000D01*\n",
            "M02*\n"
        ),
    )
    .expect("gerber file should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-outline",
        root.to_str().unwrap(),
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("outline compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["matched_count"], 1);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
