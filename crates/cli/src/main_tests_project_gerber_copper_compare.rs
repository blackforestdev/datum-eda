use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_compare_gerber_copper_layer_is_semantic_and_reports_drift() {
    let root = unique_project_root("datum-eda-cli-project-gerber-copper-compare");
    create_native_project(&root, Some("Gerber Copper Compare Demo".to_string()))
        .expect("initial scaffold should succeed");

    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let circle_pad_uuid = Uuid::new_v4();
    let rect_pad_uuid = Uuid::new_v4();
    let component_uuid = Uuid::new_v4();
    let track_a_uuid = Uuid::new_v4();
    let track_b_uuid = Uuid::new_v4();
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
                "name": "Gerber Copper Compare Demo Board",
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
                            "shape": "rect",
                            "diameter_nm": 0,
                            "width_nm": 900000,
                            "height_nm": 450000
                        },
                        {
                            "uuid": Uuid::new_v4(),
                            "name": "CP3",
                            "position": { "x": 2750000, "y": 250000 },
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
                    track_a_uuid.to_string(): {
                        "uuid": track_a_uuid,
                        "net": net_uuid,
                        "from": { "x": 0, "y": 0 },
                        "to": { "x": 1000000, "y": 0 },
                        "width": 200000,
                        "layer": 1
                    },
                    track_b_uuid.to_string(): {
                        "uuid": track_b_uuid,
                        "net": net_uuid,
                        "from": { "x": 0, "y": 500000 },
                        "to": { "x": 1000000, "y": 500000 },
                        "width": 300000,
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
    std::fs::write(
        &gerber_path,
        concat!(
            "G04 semantic copper compare fixture*\n",
            "%FSLAX46Y46*%\n",
            "%MOMM*%\n",
            "%LPD*%\n",
            "%ADD12C,0.300000*%\n",
            "%ADD13C,0.450000*%\n",
            "%ADD14C,0.500000*%\n",
            "%ADD15C,0.600000*%\n",
            "%ADD16C,0.200000*%\n",
            "%ADD17R,0.800000X0.400000*%\n",
            "%ADD18R,0.900000X0.450000*%\n",
            "D14*\n",
            "X1750000Y250000D03*\n",
            "D15*\n",
            "X250000Y250000D03*\n",
            "D13*\n",
            "X750000Y250000D03*\n",
            "D17*\n",
            "X1250000Y250000D03*\n",
            "D18*\n",
            "X2250000Y250000D03*\n",
            "G36*\n",
            "X0Y1000000D02*\n",
            "X1000000Y1000000D01*\n",
            "X1000000Y1500000D01*\n",
            "X0Y1000000D01*\n",
            "G37*\n",
            "D12*\n",
            "X0Y500000D02*\n",
            "X1000000Y500000D01*\n",
            "D16*\n",
            "X0Y0D02*\n",
            "X1000000Y0D01*\n",
            "M02*\n"
        ),
    )
    .expect("gerber file should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-copper-layer",
        root.to_str().unwrap(),
        "--layer",
        "1",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("copper compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "compare_gerber_copper_layer");
    assert_eq!(report["layer"], 1);
    assert_eq!(report["expected_pad_count"], 4);
    assert_eq!(report["actual_pad_count"], 4);
    assert_eq!(report["expected_track_count"], 2);
    assert_eq!(report["actual_track_count"], 2);
    assert_eq!(report["expected_zone_count"], 1);
    assert_eq!(report["actual_zone_count"], 1);
    assert_eq!(report["expected_via_count"], 1);
    assert_eq!(report["actual_via_count"], 1);
    assert_eq!(report["matched_count"], 8);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 0);

    std::fs::write(
        &gerber_path,
        concat!(
            "G04 semantic copper drift fixture*\n",
            "%FSLAX46Y46*%\n",
            "%MOMM*%\n",
            "%LPD*%\n",
            "%ADD10C,0.200000*%\n",
            "%ADD11C,0.300000*%\n",
            "%ADD12C,0.450000*%\n",
            "%ADD13C,0.700000*%\n",
            "%ADD14C,0.500000*%\n",
            "%ADD15R,0.800000X0.400000*%\n",
            "D11*\n",
            "X0Y500000D02*\n",
            "X1000000Y500000D01*\n",
            "D12*\n",
            "X750000Y250000D03*\n",
            "D13*\n",
            "X250000Y250000D03*\n",
            "D14*\n",
            "X1750000Y250000D03*\n",
            "D15*\n",
            "X1250000Y250000D03*\n",
            "G36*\n",
            "X0Y1000000D02*\n",
            "X1000000Y1000000D01*\n",
            "X1000000Y1500000D01*\n",
            "X0Y1000000D01*\n",
            "G37*\n",
            "M02*\n"
        ),
    )
    .expect("drift gerber should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-copper-layer",
        root.to_str().unwrap(),
        "--layer",
        "1",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("copper compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["actual_pad_count"], 3);
    assert_eq!(report["actual_track_count"], 1);
    assert_eq!(report["actual_zone_count"], 1);
    assert_eq!(report["actual_via_count"], 0);
    assert_eq!(report["matched_count"], 5);
    assert_eq!(report["missing_count"], 3);
    assert_eq!(report["extra_count"], 1);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_compare_gerber_copper_layer_matches_equivalent_reordered_geometry() {
    let root = unique_project_root("datum-eda-cli-project-gerber-copper-compare-reordered");
    create_native_project(
        &root,
        Some("Gerber Copper Compare Reordered Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let circle_pad_uuid = Uuid::new_v4();
    let rect_pad_uuid = Uuid::new_v4();
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
                "name": "Gerber Copper Compare Reordered Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
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

    let gerber_path = root.join("top-copper-reordered.gbr");
    std::fs::write(
        &gerber_path,
        concat!(
            "G04 reordered copper geometry fixture*\n",
            "%MOMM*%\n",
            "%FSLAX46Y46*%\n",
            "%LPD*%\n",
            "%ADD21C,0.450000*%\n",
            "%ADD22C,0.600000*%\n",
            "%ADD23C,0.200000*%\n",
            "%ADD24R,0.800000X0.400000*%\n",
            "G36*\n",
            "X1000000Y1500000D02*\n",
            "X1000000Y1000000D01*\n",
            "X0Y1000000D01*\n",
            "X1000000Y1500000D01*\n",
            "G37*\n",
            "D21*\n",
            "X750000Y250000D03*\n",
            "D22*\n",
            "X250000Y250000D03*\n",
            "D24*\n",
            "X1250000Y250000D03*\n",
            "D23*\n",
            "X1000000Y0D02*\n",
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
        "compare-gerber-copper-layer",
        root.to_str().unwrap(),
        "--layer",
        "1",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("copper compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["expected_pad_count"], 2);
    assert_eq!(report["actual_pad_count"], 2);
    assert_eq!(report["expected_track_count"], 1);
    assert_eq!(report["actual_track_count"], 1);
    assert_eq!(report["expected_zone_count"], 1);
    assert_eq!(report["actual_zone_count"], 1);
    assert_eq!(report["expected_via_count"], 1);
    assert_eq!(report["actual_via_count"], 1);
    assert_eq!(report["matched_count"], 5);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
