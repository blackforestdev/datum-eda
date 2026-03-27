use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_compare_gerber_export_plan_reports_missing_and_extra_files() {
    let root = unique_project_root("datum-eda-cli-project-gerber-plan-compare");
    create_native_project(&root, Some("Gerber Compare Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Compare Demo Board",
                "stackup": {
                    "layers": [
                        {
                            "id": 1,
                            "name": "Top Copper",
                            "layer_type": "Copper",
                            "thickness_nm": 35000
                        },
                        {
                            "id": 2,
                            "name": "Top Mask",
                            "layer_type": "SolderMask",
                            "thickness_nm": 10000
                        },
                        {
                            "id": 31,
                            "name": "Bottom Copper",
                            "layer_type": "Copper",
                            "thickness_nm": 35000
                        }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 1000, "y": 0 },
                        { "x": 1000, "y": 500 },
                        { "x": 0, "y": 500 }
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

    let output_dir = root.join("gerbers");
    std::fs::create_dir_all(&output_dir).expect("output dir should exist");
    std::fs::write(output_dir.join("release-a-outline.gbr"), "").expect("outline should write");
    std::fs::write(output_dir.join("release-a-l1-top-copper-copper.gbr"), "")
        .expect("copper should write");
    std::fs::write(output_dir.join("unexpected-note.txt"), "").expect("extra should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-export-plan",
        root.to_str().unwrap(),
        "--output-dir",
        output_dir.to_str().unwrap(),
        "--prefix",
        "Release A",
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("gerber plan compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "compare_gerber_export_plan");
    assert_eq!(report["prefix"], "release-a");
    assert_eq!(report["expected_count"], 4);
    assert_eq!(report["present_count"], 3);
    assert_eq!(report["missing_count"], 2);
    assert_eq!(report["extra_count"], 1);

    let matched = report["matched"].as_array().expect("matched array");
    assert_eq!(matched.len(), 2);
    assert_eq!(matched[0], "release-a-l1-top-copper-copper.gbr");
    assert_eq!(matched[1], "release-a-outline.gbr");

    let missing = report["missing"].as_array().expect("missing array");
    assert_eq!(missing.len(), 2);
    assert_eq!(missing[0], "release-a-l2-top-mask-mask.gbr");
    assert_eq!(missing[1], "release-a-l31-bottom-copper-copper.gbr");

    let extra = report["extra"].as_array().expect("extra array");
    assert_eq!(extra.len(), 1);
    assert_eq!(extra[0], "unexpected-note.txt");

    let _ = std::fs::remove_dir_all(&root);
}
