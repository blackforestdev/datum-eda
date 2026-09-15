use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_report_drill_hole_classes_groups_through_blind_and_buried() {
    let root = unique_project_root("datum-eda-cli-project-drill-hole-classes");
    create_native_project(&root, Some("Drill Hole Classes Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let net_uuid = Uuid::new_v4();
    let through_uuid = Uuid::new_v4();
    let blind_uuid = Uuid::new_v4();
    let buried_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Drill Hole Classes Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Inner 1", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 3, "name": "Inner 2", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 31, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "pads": {},
                "tracks": {},
                "vias": {
                    through_uuid.to_string(): {
                        "uuid": through_uuid,
                        "net": net_uuid,
                        "position": { "x": 1000000, "y": 1000000 },
                        "drill": 300000,
                        "diameter": 600000,
                        "from_layer": 1,
                        "to_layer": 31
                    },
                    blind_uuid.to_string(): {
                        "uuid": blind_uuid,
                        "net": net_uuid,
                        "position": { "x": 2000000, "y": 2000000 },
                        "drill": 250000,
                        "diameter": 500000,
                        "from_layer": 1,
                        "to_layer": 2
                    },
                    buried_uuid.to_string(): {
                        "uuid": buried_uuid,
                        "net": net_uuid,
                        "position": { "x": 3000000, "y": 3000000 },
                        "drill": 200000,
                        "diameter": 450000,
                        "from_layer": 2,
                        "to_layer": 3
                    }
                },
                "zones": {},
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "N$1",
                        "class": Uuid::new_v4()
                    }
                },
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
        "eda",
        "--format",
        "json",
        "project",
        "report-drill-hole-classes",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("hole-class report should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "report_drill_hole_classes");
    assert_eq!(report["copper_layer_count"], 4);
    assert_eq!(report["via_count"], 3);
    assert_eq!(report["class_count"], 3);
    assert_eq!(report["classes"][0]["class"], "blind");
    assert_eq!(report["classes"][0]["tool_count"], 1);
    assert_eq!(report["classes"][1]["class"], "buried");
    assert_eq!(report["classes"][2]["class"], "through");

    let _ = std::fs::remove_dir_all(&root);
}
