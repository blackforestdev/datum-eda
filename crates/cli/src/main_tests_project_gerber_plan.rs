use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_plan_gerber_export_reports_deterministic_artifact_set() {
    let root = unique_project_root("datum-eda-cli-project-gerber-plan");
    create_native_project(&root, Some("Gerber Plan Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Plan Demo Board",
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
                            "id": 3,
                            "name": "Top Silk",
                            "layer_type": "Silkscreen",
                            "thickness_nm": 10000
                        },
                        {
                            "id": 4,
                            "name": "Top Paste",
                            "layer_type": "Paste",
                            "thickness_nm": 10000
                        },
                        {
                            "id": 31,
                            "name": "Bottom Copper",
                            "layer_type": "Copper",
                            "thickness_nm": 35000
                        },
                        {
                            "id": 32,
                            "name": "Bottom Mask",
                            "layer_type": "SolderMask",
                            "thickness_nm": 10000
                        },
                        {
                            "id": 50,
                            "name": "Fab Notes",
                            "layer_type": "Mechanical",
                            "thickness_nm": 0
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

    let cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "plan-gerber-export",
        root.to_str().unwrap(), "--prefix", "Release A",
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("gerber plan should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "plan_gerber_export");
    assert_eq!(report["prefix"], "release-a");
    assert_eq!(report["outline_vertex_count"], 4);
    assert_eq!(report["outline_closed"], true);
    assert_eq!(report["copper_layers"], 2);
    assert_eq!(report["soldermask_layers"], 2);
    assert_eq!(report["silkscreen_layers"], 1);
    assert_eq!(report["paste_layers"], 1);
    assert_eq!(report["mechanical_layers"], 1);

    let artifacts = report["artifacts"].as_array().expect("artifacts array");
    assert_eq!(artifacts.len(), 8);
    assert_eq!(artifacts[0]["kind"], "outline");
    assert_eq!(artifacts[0]["filename"], "release-a-outline.gbr");
    assert_eq!(artifacts[1]["filename"], "release-a-l1-top-copper-copper.gbr");
    assert_eq!(artifacts[2]["filename"], "release-a-l2-top-mask-mask.gbr");
    assert_eq!(artifacts[3]["filename"], "release-a-l3-top-silk-silk.gbr");
    assert_eq!(artifacts[4]["filename"], "release-a-l4-top-paste-paste.gbr");
    assert_eq!(artifacts[5]["filename"], "release-a-l31-bottom-copper-copper.gbr");
    assert_eq!(artifacts[6]["filename"], "release-a-l32-bottom-mask-mask.gbr");
    assert_eq!(artifacts[7]["filename"], "release-a-l50-fab-notes-mech.gbr");

    let _ = std::fs::remove_dir_all(&root);
}
