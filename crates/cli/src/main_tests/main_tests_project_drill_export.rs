use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_export_drill_writes_deterministic_csv_from_board_vias() {
    let root = unique_project_root("datum-eda-cli-project-drill-export");
    create_native_project(&root, Some("Drill Export Demo".to_string()))
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
                "name": "Drill Export Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "pads": {},
                "tracks": {},
                "vias": {
                    via_b_uuid.to_string(): {
                        "uuid": via_b_uuid,
                        "net": net_uuid,
                        "position": { "x": 2000, "y": 3000 },
                        "drill": 350000,
                        "diameter": 700000,
                        "from_layer": 31,
                        "to_layer": 1
                    },
                    via_a_uuid.to_string(): {
                        "uuid": via_a_uuid,
                        "net": net_uuid,
                        "position": { "x": 1000, "y": 1500 },
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

    let drill_path = root.join("drill.csv");
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-drill",
        root.to_str().unwrap(),
        "--out",
        drill_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("drill export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "export_drill");
    assert_eq!(report["production_classification"], "manual_debug_export");
    assert_eq!(report["rows"], 2);

    let csv = std::fs::read_to_string(&drill_path).expect("drill should read");
    let lines = csv.lines().collect::<Vec<_>>();
    assert_eq!(
        lines[0],
        "via_uuid,net_uuid,x_nm,y_nm,drill_nm,diameter_nm,from_layer,to_layer"
    );
    assert_eq!(
        lines[1],
        format!("{via_a_uuid},{net_uuid},1000,1500,300000,600000,1,31")
    );
    assert_eq!(
        lines[2],
        format!("{via_b_uuid},{net_uuid},2000,3000,350000,700000,31,1")
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_export_drill_uses_resolver_materialized_board_state() {
    let root = unique_project_root("datum-eda-cli-project-drill-resolved-export");
    create_native_project(&root, Some("Drill Resolved Export Demo".to_string()))
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

    let place_output = execute(
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
            "1000",
            "--y-nm",
            "1500",
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
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place output should parse");
    let via_uuid = placed["via_uuid"].as_str().unwrap();
    std::fs::write(&board_json, stale_board).expect("stale board file should restore");

    let drill_path = root.join("drill-resolved.csv");
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-drill",
            root.to_str().unwrap(),
            "--out",
            drill_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("drill export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["rows"], 1);
    let csv = std::fs::read_to_string(&drill_path).expect("drill should read");
    assert!(csv.contains(&format!(
        "{via_uuid},{net_uuid},1000,1500,300000,600000,1,31"
    )));

    let _ = std::fs::remove_dir_all(&root);
}
