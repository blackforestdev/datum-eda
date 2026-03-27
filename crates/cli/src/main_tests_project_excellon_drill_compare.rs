use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_compare_excellon_drill_reports_missing_extra_and_hit_drift() {
    let root = unique_project_root("datum-eda-cli-project-excellon-drill-compare");
    create_native_project(&root, Some("Excellon Drill Compare Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let net_uuid = Uuid::new_v4();
    let via_a_uuid = Uuid::new_v4();
    let via_b_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Excellon Drill Compare Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "pads": {},
                "tracks": {},
                "vias": {
                    via_a_uuid.to_string(): {
                        "uuid": via_a_uuid,
                        "net": net_uuid,
                        "position": { "x": 1000000, "y": 1500000 },
                        "drill": 300000,
                        "diameter": 600000,
                        "from_layer": 1,
                        "to_layer": 31
                    },
                    via_b_uuid.to_string(): {
                        "uuid": via_b_uuid,
                        "net": net_uuid,
                        "position": { "x": 2000000, "y": 3000000 },
                        "drill": 300000,
                        "diameter": 700000,
                        "from_layer": 31,
                        "to_layer": 1
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

    let drill_path = root.join("drill.drl");
    std::fs::write(
        &drill_path,
        "M48\nMETRIC,TZ\nT01C0.300000\nT02C0.350000\n%\nT01\nX1.000000Y1.500000\nT02\nX3.000000Y4.500000\nM30\n",
    )
    .expect("drill file should write");

    let cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "compare-excellon-drill",
        root.to_str().unwrap(), "--drill", drill_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("excellon compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "compare_excellon_drill");
    assert_eq!(report["expected_tool_count"], 1);
    assert_eq!(report["actual_tool_count"], 2);
    assert_eq!(report["expected_hit_count"], 2);
    assert_eq!(report["actual_hit_count"], 2);
    assert_eq!(report["matched_count"], 0);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 1);
    assert_eq!(report["hit_drift_count"], 1);
    assert_eq!(report["extra"][0], "0.350000");
    assert_eq!(report["hit_drift"][0]["diameter_mm"], "0.300000");
    assert_eq!(report["hit_drift"][0]["expected_hits"], 2);
    assert_eq!(report["hit_drift"][0]["actual_hits"], 1);

    let _ = std::fs::remove_dir_all(&root);
}
