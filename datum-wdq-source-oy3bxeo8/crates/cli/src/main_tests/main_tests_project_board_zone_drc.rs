use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn seed_unfilled_zone_drc_fixture(root: &Path) -> String {
    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let package_a_uuid = Uuid::new_v4();
    let package_b_uuid = Uuid::new_v4();
    let pad_a_uuid = Uuid::new_v4();
    let pad_b_uuid = Uuid::new_v4();
    let zone_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Unfilled Zone DRC Demo Board",
                "stackup": { "layers": [{
                    "id": 1,
                    "name": "F.Cu",
                    "layer_type": "Copper",
                    "thickness_nm": 35000
                }] },
                "outline": { "vertices": [
                    { "x": 0, "y": 0 },
                    { "x": 2000000, "y": 0 },
                    { "x": 2000000, "y": 2000000 },
                    { "x": 0, "y": 2000000 }
                ], "closed": true },
                "packages": {
                    package_a_uuid.to_string(): {
                        "uuid": package_a_uuid,
                        "part": Uuid::new_v4(),
                        "package": Uuid::new_v4(),
                        "reference": "U1",
                        "value": "TEST",
                        "position": { "x": 250000, "y": 250000 },
                        "rotation": 0,
                        "layer": 1,
                        "locked": false
                    },
                    package_b_uuid.to_string(): {
                        "uuid": package_b_uuid,
                        "part": Uuid::new_v4(),
                        "package": Uuid::new_v4(),
                        "reference": "U2",
                        "value": "TEST",
                        "position": { "x": 1750000, "y": 1750000 },
                        "rotation": 0,
                        "layer": 1,
                        "locked": false
                    }
                },
                "pads": {
                    pad_a_uuid.to_string(): {
                        "uuid": pad_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": net_uuid,
                        "position": { "x": 250000, "y": 250000 },
                        "layer": 1,
                        "diameter": 250000
                    },
                    pad_b_uuid.to_string(): {
                        "uuid": pad_b_uuid,
                        "package": package_b_uuid,
                        "name": "2",
                        "net": net_uuid,
                        "position": { "x": 1750000, "y": 1750000 },
                        "layer": 1,
                        "diameter": 250000
                    }
                },
                "tracks": {},
                "vias": {},
                "zones": {
                    zone_uuid.to_string(): {
                        "uuid": zone_uuid,
                        "net": net_uuid,
                        "polygon": { "vertices": [
                            { "x": 0, "y": 0 },
                            { "x": 2000000, "y": 0 },
                            { "x": 2000000, "y": 2000000 },
                            { "x": 0, "y": 2000000 }
                        ], "closed": true },
                        "layer": 1,
                        "priority": 1,
                        "thermal_relief": false,
                        "thermal_gap": 0,
                        "thermal_spoke_width": 0
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
    zone_uuid.to_string()
}

#[test]
fn project_query_drc_does_not_treat_unfilled_zone_boundary_as_copper() {
    let root = unique_project_root("datum-eda-cli-project-board-zone-drc-unfilled");
    create_native_project(&root, Some("Unfilled Zone DRC Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = seed_unfilled_zone_drc_fixture(&root);

    let drc_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "drc",
        ])
        .expect("CLI should parse"),
    )
    .expect("drc query should succeed");
    let drc: serde_json::Value = serde_json::from_str(&drc_output).expect("drc JSON");
    let no_copper_fingerprint = drc["raw_report"]["drc"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["code"] == "connectivity_no_copper")
        .and_then(|entry| entry["fingerprint"].as_str())
        .expect("connectivity_no_copper should carry an engine DRC fingerprint")
        .to_string();
    assert!(no_copper_fingerprint.starts_with("sha256:"));
    assert!(
        drc["raw_report"]["drc"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["code"] == "connectivity_no_copper"),
        "unfilled zone {zone_uuid} must not count as routed copper"
    );

    let diagnostics_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-diagnostics",
        ])
        .expect("CLI should parse"),
    )
    .expect("board diagnostics query should succeed");
    let diagnostics: serde_json::Value =
        serde_json::from_str(&diagnostics_output).expect("diagnostics JSON");
    assert!(
        diagnostics["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["kind"] == "net_without_copper"),
        "board diagnostics must not count unfilled zone {zone_uuid} as copper"
    );

    let unrouted_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-unrouted",
        ])
        .expect("CLI should parse"),
    )
    .expect("board unrouted query should succeed");
    let unrouted: serde_json::Value =
        serde_json::from_str(&unrouted_output).expect("unrouted JSON");
    assert_eq!(
        unrouted["airwires"].as_array().unwrap().len(),
        1,
        "board unrouted must not count unfilled zone {zone_uuid} as copper"
    );

    let board_check_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-check",
        ])
        .expect("CLI should parse"),
    )
    .expect("board check query should succeed");
    let board_check: serde_json::Value =
        serde_json::from_str(&board_check_output).expect("board-check JSON");
    assert!(
        board_check["summary"]["by_code"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["code"] == "net_without_copper" && entry["count"] == 1),
        "board-check must not count unfilled zone {zone_uuid} as copper"
    );

    let check_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "check-run",
        ])
        .expect("CLI should parse"),
    )
    .expect("check-run should succeed");
    let check: serde_json::Value = serde_json::from_str(&check_output).expect("check-run JSON");
    assert!(
        check["findings"].as_array().unwrap().iter().any(|entry| {
            entry["source"] == "drc"
                && entry["code"] == "connectivity_no_copper"
                && entry["fingerprint"].as_str() == Some(no_copper_fingerprint.as_str())
                && entry["payload"]["fingerprint"].as_str() == Some(no_copper_fingerprint.as_str())
        }),
        "check-run DRC findings must preserve engine DRC fingerprints"
    );
    assert!(
        check["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["code"] == "zone_fill_unfilled"
                && entry["payload"]["zone_id"] == zone_uuid)
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_drc_treats_filled_zone_evidence_as_copper() {
    let root = unique_project_root("datum-eda-cli-project-board-zone-drc-filled");
    create_native_project(&root, Some("Filled Zone DRC Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = Uuid::parse_str(&seed_unfilled_zone_drc_fixture(&root)).unwrap();
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "fill-zones",
            root.to_str().unwrap(),
            "--zone",
            &zone_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("fill-zones should persist filled evidence through journal");

    let drc_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "drc",
        ])
        .expect("CLI should parse"),
    )
    .expect("drc query should succeed");
    let drc: serde_json::Value = serde_json::from_str(&drc_output).expect("drc JSON");
    assert!(
        !drc["raw_report"]["drc"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["code"] == "connectivity_no_copper"),
        "filled zone evidence {zone_uuid} must count as routed copper"
    );

    let unrouted_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-unrouted",
        ])
        .expect("CLI should parse"),
    )
    .expect("board unrouted query should succeed");
    let unrouted: serde_json::Value =
        serde_json::from_str(&unrouted_output).expect("unrouted JSON");
    assert_eq!(
        unrouted["airwires"].as_array().unwrap().len(),
        0,
        "filled zone evidence {zone_uuid} must suppress airwires"
    );

    let _ = std::fs::remove_dir_all(&root);
}
