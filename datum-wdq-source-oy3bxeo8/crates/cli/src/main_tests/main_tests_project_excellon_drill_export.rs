use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_export_excellon_drill_writes_narrow_excellon_from_board_vias() {
    let root = unique_project_root("datum-eda-cli-project-excellon-drill-export");
    create_native_project(&root, Some("Excellon Drill Export Demo".to_string()))
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
                "name": "Excellon Drill Export Demo Board",
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
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-excellon-drill",
        root.to_str().unwrap(),
        "--out",
        drill_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("excellon drill export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "export_excellon_drill");
    assert_eq!(report["production_classification"], "manual_debug_export");
    assert_eq!(report["via_count"], 2);
    assert_eq!(report["tool_count"], 2);
    assert_eq!(report["tools"][0]["tool"], "T01");
    assert_eq!(report["tools"][0]["diameter_mm"], "0.300000");
    assert_eq!(report["tools"][0]["hits"], 1);
    assert_eq!(report["tools"][1]["tool"], "T02");
    assert_eq!(report["tools"][1]["diameter_mm"], "0.350000");
    assert_eq!(report["tools"][1]["hits"], 1);
    assert_eq!(
        report["production_projection"]["projection_kind"],
        "excellon_drill"
    );
    assert_eq!(
        report["production_projection"]["projection_contract"],
        "datum.production_projection.excellon_drill.v1"
    );
    assert!(
        report["production_projection"]["model_revision"]
            .as_str()
            .is_some_and(|revision| revision.len() >= 64)
    );
    assert!(
        report["production_projection"]["sha256"]
            .as_str()
            .is_some_and(|hash| hash.starts_with("sha256:"))
    );

    let drill = std::fs::read_to_string(&drill_path).expect("drill should read");
    assert_eq!(
        report["production_projection"]["byte_count"],
        serde_json::json!(drill.len())
    );
    assert!(drill.contains("M48"));
    assert!(drill.contains("METRIC,TZ"));
    assert!(drill.contains("T01C0.300000"));
    assert!(drill.contains("T02C0.350000"));
    assert!(drill.contains("T01\nX1.000000Y1.500000"));
    assert!(drill.contains("T02\nX2.000000Y3.000000"));
    assert!(drill.ends_with("M30\n"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_export_excellon_drill_uses_resolver_materialized_board_state() {
    let root = unique_project_root("datum-eda-cli-project-excellon-drill-resolved-export");
    create_native_project(
        &root,
        Some("Excellon Drill Resolved Export Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let board_json = root.join("board/board.json");
    let stale_board = std::fs::read_to_string(&board_json).expect("board file should read");

    let class_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-net-class",
            root.to_str().unwrap(),
            "--name",
            "Default",
            "--clearance-nm",
            "150000",
            "--track-width-nm",
            "200000",
            "--via-drill-nm",
            "300000",
            "--via-diameter-nm",
            "600000",
        ])
        .expect("CLI should parse"),
    )
    .expect("place board net class should succeed");
    let class_report: serde_json::Value =
        serde_json::from_str(&class_output).expect("class output should parse");
    let net_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-net",
            root.to_str().unwrap(),
            "--name",
            "GND",
            "--class",
            class_report["net_class_uuid"].as_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("place board net should succeed");
    let net_report: serde_json::Value =
        serde_json::from_str(&net_output).expect("net output should parse");
    let net_uuid = net_report["net_uuid"].as_str().unwrap();

    let _ = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-via",
            root.to_str().unwrap(),
            "--net",
            net_uuid,
            "--x-nm",
            "1000000",
            "--y-nm",
            "1500000",
            "--drill-nm",
            "300000",
            "--diameter-nm",
            "600000",
            "--from-layer",
            "1",
            "--to-layer",
            "31",
        ])
        .expect("CLI should parse"),
    )
    .expect("place board via should succeed");
    std::fs::write(&board_json, stale_board).expect("stale board file should restore");

    let drill_path = root.join("drill-resolved.drl");
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-excellon-drill",
            root.to_str().unwrap(),
            "--out",
            drill_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("excellon drill export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["via_count"], 1);
    assert_eq!(report["hit_count"], 1);
    assert_eq!(report["tools"][0]["diameter_mm"], "0.300000");
    let drill = std::fs::read_to_string(&drill_path).expect("drill should read");
    assert!(drill.contains("T01C0.300000"));
    assert!(drill.contains("T01\nX1.000000Y1.500000"));

    let _ = std::fs::remove_dir_all(&root);
}
