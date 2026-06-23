use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_validate_gerber_copper_layer_reports_match_and_mismatch() {
    let root = unique_project_root("datum-eda-cli-project-gerber-copper-validate");
    create_native_project(&root, Some("Gerber Copper Validate Demo".to_string()))
        .expect("initial scaffold should succeed");

    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let circle_pad_uuid = Uuid::new_v4();
    let rect_pad_uuid = Uuid::new_v4();
    let component_uuid = Uuid::new_v4();
    let track_uuid = Uuid::new_v4();
    let zone_uuid = Uuid::new_v4();
    let via_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Copper Validate Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "component_pads": {
                    component_uuid.to_string(): [
                        {
                            "uuid": Uuid::new_v4(),
                            "name": "CP1",
                            "position": { "x": 1750000, "y": 250000 },
                            "padstack": Uuid::new_v4(),
                            "layer": 1,
                            "shape": "circle",
                            "diameter_nm": 500000,
                            "width_nm": 0,
                            "height_nm": 0
                        },
                        {
                            "uuid": Uuid::new_v4(),
                            "name": "CP2",
                            "position": { "x": 2250000, "y": 250000 },
                            "padstack": Uuid::new_v4(),
                            "layer": 1,
                            "shape": null,
                            "diameter_nm": 0,
                            "width_nm": 0,
                            "height_nm": 0
                        }
                    ]
                },
                "pads": {
                    circle_pad_uuid.to_string(): {
                        "uuid": circle_pad_uuid,
                        "package": Uuid::new_v4(),
                        "name": "1",
                        "net": net_uuid,
                        "position": { "x": 750000, "y": 250000 },
                        "layer": 1,
                        "diameter": 450000
                    },
                    rect_pad_uuid.to_string(): {
                        "uuid": rect_pad_uuid,
                        "package": Uuid::new_v4(),
                        "name": "2",
                        "net": net_uuid,
                        "position": { "x": 1250000, "y": 250000 },
                        "layer": 1,
                        "shape": "rect",
                        "diameter": 0,
                        "width": 800000,
                        "height": 400000
                    }
                },
                "tracks": {
                    track_uuid.to_string(): {
                        "uuid": track_uuid,
                        "net": net_uuid,
                        "from": { "x": 0, "y": 0 },
                        "to": { "x": 1000000, "y": 0 },
                        "width": 200000,
                        "layer": 1
                    }
                },
                "vias": {
                    via_uuid.to_string(): {
                        "uuid": via_uuid,
                        "net": net_uuid,
                        "position": { "x": 250000, "y": 250000 },
                        "drill": 300000,
                        "diameter": 600000,
                        "from_layer": 1,
                        "to_layer": 2
                    }
                },
                "zones": {
                    zone_uuid.to_string(): {
                        "uuid": zone_uuid,
                        "net": net_uuid,
                        "polygon": {
                            "vertices": [
                                { "x": 0, "y": 1000000 },
                                { "x": 1000000, "y": 1000000 },
                                { "x": 1000000, "y": 1500000 }
                            ],
                            "closed": true
                        },
                        "layer": 1,
                        "priority": 1,
                        "thermal_relief": true,
                        "thermal_gap": 250000,
                        "thermal_spoke_width": 200000
                    }
                },
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "GND",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "rules": [],
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let gerber_path = root.join("top-copper.gbr");
    let export_cli = Cli::try_parse_from([
        "eda",
        "project",
        "export-gerber-copper-layer",
        root.to_str().unwrap(),
        "--layer",
        "1",
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("export CLI should parse");
    let _ = execute(export_cli).expect("gerber copper export should succeed");

    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-copper-layer",
        root.to_str().unwrap(),
        "--layer",
        "1",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("validate CLI should parse");
    let (output, exit_code) = execute_with_exit_code(validate_cli).expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 0);
    assert_eq!(report["action"], "validate_gerber_copper_layer");
    assert_eq!(report["matches_expected"], true);
    assert_eq!(report["layer"], 1);
    assert_eq!(report["pad_count"], 3);
    assert_eq!(report["track_count"], 1);
    assert_eq!(report["zone_count"], 0);
    assert_eq!(report["unfilled_zone_count"], 1);
    assert_eq!(report["unfilled_zone_ids"][0], zone_uuid.to_string());
    assert_eq!(report["via_count"], 1);
    assert_eq!(
        report["production_projection"]["projection_contract"],
        "datum.production_projection.gerber_copper_layer.v1"
    );
    assert_eq!(
        report["production_projection"]["byte_count"],
        report["expected_bytes"]
    );
    assert!(
        report["production_projection"]["sha256"]
            .as_str()
            .is_some_and(|hash| hash.starts_with("sha256:"))
    );

    std::fs::write(&gerber_path, "corrupted\n").expect("gerber overwrite should succeed");
    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-copper-layer",
        root.to_str().unwrap(),
        "--layer",
        "1",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("validate CLI should parse");
    let (output, exit_code) = execute_with_exit_code(validate_cli).expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 1);
    assert_eq!(report["matches_expected"], false);
    assert_eq!(report["pad_count"], 3);
    assert_eq!(report["zone_count"], 0);
    assert_eq!(report["unfilled_zone_count"], 1);
    assert_eq!(report["unfilled_zone_ids"][0], zone_uuid.to_string());
    assert_eq!(report["via_count"], 1);
    assert_eq!(
        report["production_projection"]["byte_count"],
        report["expected_bytes"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_validate_gerber_copper_layer_uses_resolver_materialized_board_state() {
    let root = unique_project_root("datum-eda-cli-project-gerber-copper-resolved-validate");
    create_native_project(
        &root,
        Some("Gerber Copper Resolved Validate Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let board_json = root.join("board/board.json");
    let stale_board = std::fs::read_to_string(&board_json).expect("board file should read");

    let class_cli = Cli::try_parse_from([
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
    .expect("CLI should parse");
    let class_output = execute(class_cli).expect("place board net class should succeed");
    let class_report: serde_json::Value =
        serde_json::from_str(&class_output).expect("class output should parse");
    let net_cli = Cli::try_parse_from([
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
    .expect("CLI should parse");
    let net_output = execute(net_cli).expect("place board net should succeed");
    let net_report: serde_json::Value =
        serde_json::from_str(&net_output).expect("net output should parse");
    let draw_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "draw-board-track",
        root.to_str().unwrap(),
        "--net",
        net_report["net_uuid"].as_str().unwrap(),
        "--from-x-nm",
        "100000",
        "--from-y-nm",
        "200000",
        "--to-x-nm",
        "900000",
        "--to-y-nm",
        "200000",
        "--width-nm",
        "250000",
        "--layer",
        "1",
    ])
    .expect("CLI should parse");
    let _ = execute(draw_cli).expect("draw board track should succeed");
    std::fs::write(&board_json, stale_board).expect("stale board file should restore");

    let gerber_path = root.join("top-copper-resolved.gbr");
    std::fs::write(
        &gerber_path,
        concat!(
            "G04 datum-eda native copper layer 1*\n",
            "%FSLAX46Y46*%\n",
            "%MOMM*%\n",
            "%LPD*%\n",
            "%ADD10C,0.250000*%\n",
            "D10*\n",
            "X100000Y200000D02*\n",
            "X900000Y200000D01*\n",
            "M02*\n"
        ),
    )
    .expect("gerber file should write");

    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-copper-layer",
        root.to_str().unwrap(),
        "--layer",
        "1",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("validate CLI should parse");
    let (output, exit_code) = execute_with_exit_code(validate_cli).expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 0);
    assert_eq!(report["matches_expected"], true);
    assert_eq!(report["track_count"], 1);

    let _ = std::fs::remove_dir_all(&root);
}
