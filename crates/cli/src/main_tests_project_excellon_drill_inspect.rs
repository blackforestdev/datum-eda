use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_inspect_excellon_drill_reports_tool_table_and_hits() {
    let root = unique_project_root("datum-eda-cli-project-excellon-drill-inspect");
    create_native_project(&root, Some("Excellon Drill Inspect Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let net_uuid = Uuid::new_v4();
    let via_b_uuid = Uuid::new_v4();
    let via_a_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Excellon Drill Inspect Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "pads": {},
                "tracks": {},
                "vias": {
                    via_b_uuid.to_string(): {
                        "uuid": via_b_uuid,
                        "net": net_uuid,
                        "position": { "x": 2000000, "y": 3000000 },
                        "drill": 350000,
                        "diameter": 700000,
                        "from_layer": 31,
                        "to_layer": 1
                    },
                    via_a_uuid.to_string(): {
                        "uuid": via_a_uuid,
                        "net": net_uuid,
                        "position": { "x": 1000000, "y": 1500000 },
                        "drill": 300000,
                        "diameter": 600000,
                        "from_layer": 1,
                        "to_layer": 31
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
    let export_cli = Cli::try_parse_from([
        "eda",
        "project",
        "export-excellon-drill",
        root.to_str().unwrap(),
        "--out",
        drill_path.to_str().unwrap(),
    ])
    .expect("export CLI should parse");
    let _ = execute(export_cli).expect("excellon drill export should succeed");

    let inspect_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "inspect-excellon-drill",
        drill_path.to_str().unwrap(),
    ])
    .expect("inspect CLI should parse");
    let output = execute(inspect_cli).expect("excellon drill inspect should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "inspect_excellon_drill");
    assert_eq!(report["metric"], true);
    assert_eq!(report["tool_count"], 2);
    assert_eq!(report["hit_count"], 2);
    assert_eq!(report["tools"][0]["tool"], "T01");
    assert_eq!(report["tools"][0]["diameter_mm"], "0.300000");
    assert_eq!(report["tools"][0]["hits"], 1);
    assert_eq!(report["tools"][1]["tool"], "T02");
    assert_eq!(report["tools"][1]["diameter_mm"], "0.350000");
    assert_eq!(report["tools"][1]["hits"], 1);

    let _ = std::fs::remove_dir_all(&root);
}
