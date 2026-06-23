use super::main_tests_project_check::{seed_board_process_aperture_fixture, unique_project_root};
use super::*;

#[test]
fn check_profiles_reports_current_supported_profile() {
    let root = unique_project_root("datum-eda-cli-check-profiles");
    create_native_project(&root, Some("Check Profiles Demo".to_string()))
        .expect("initial scaffold should succeed");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "profiles",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("check profiles should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["contract"], "check_profiles_v1");
    assert_eq!(report["default_profile_id"], "native-combined");
    let profile_ids = report["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|profile| profile["profile_id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        profile_ids,
        vec![
            "native-combined",
            "erc",
            "drc",
            "standards",
            "manufacturing",
            "release"
        ]
    );
    assert!(
        report["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .all(|profile| profile["selection_supported"] == true)
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn check_run_accepts_bounded_non_default_profiles() {
    let root = unique_project_root("datum-eda-cli-check-run-supported-profiles");
    create_native_project(&root, Some("Supported Profiles Demo".to_string()))
        .expect("initial scaffold should succeed");

    for profile in ["erc", "drc", "standards", "manufacturing", "release"] {
        let output = execute(
            Cli::try_parse_from([
                "eda",
                "--format",
                "json",
                "check",
                "run",
                root.to_str().unwrap(),
                "--profile",
                profile,
            ])
            .expect("CLI should parse"),
        )
        .expect("supported check profile should succeed");
        let report: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(report["contract"], "check_run_v1");
        assert_eq!(report["profile_id"], profile);
        assert_eq!(report["profile_basis"]["profile_id"], profile);
        assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
            entry["status"] == "evaluated"
                || entry["status"] == "filtered_by_profile"
                || entry["status"] == "not_implemented"
        }));
    }

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn erc_profile_does_not_execute_drc_domains() {
    let root = unique_project_root("datum-eda-cli-check-run-erc-profile-inputs");
    create_native_project(&root, Some("ERC Profile Inputs Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_process_aperture_fixture(&root);

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
            "--profile",
            "erc",
        ])
        .expect("CLI should parse"),
    )
    .expect("erc profile check should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["profile_id"], "erc");
    assert_eq!(report["raw_report"]["drc"], serde_json::json!([]));
    assert_eq!(report["raw_report"]["diagnostics"], serde_json::json!([]));
    assert!(
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["domain"] == "erc")
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn standards_profile_includes_process_aperture_inherited_from_copper() {
    let root = unique_project_root("datum-eda-cli-check-run-standards-process-aperture");
    create_native_project(&root, Some("Standards Process Aperture Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_process_aperture_fixture(&root);
    add_peer_process_aperture_inconsistency(&root);

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
            "--profile",
            "standards",
        ])
        .expect("CLI should parse"),
    )
    .expect("standards profile check should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["profile_id"], "standards");
    assert_eq!(
        report["profile_basis"]["standards_basis"],
        "datum.process_aperture_and_geometry.current"
    );
    assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
        entry["domain"] == "standards"
            && entry["rule_id"] == "process_aperture_policy"
            && entry["status"] == "evaluated"
            && entry["target_scope"] == "board_pads_tracks_vias"
    }));
    assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
        entry["domain"] == "erc"
            && entry["rule_id"] == "schematic_connectivity"
            && entry["status"] == "filtered_by_profile"
    }));
    assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
        entry["rule_id"] == "clearance_solver" && entry["status"] == "not_implemented"
    }));
    assert_eq!(report["raw_report"]["erc"], serde_json::json!([]));
    assert!(
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["domain"] == "standards")
    );
    assert!(report["findings"].as_array().unwrap().iter().any(|entry| {
        entry["source"] == "drc"
            && entry["domain"] == "standards"
            && entry["code"] == "pad_process_aperture_inherited_from_copper"
            && entry["standards_basis"] == "datum.process_aperture_and_geometry.current"
            && entry["rule_revision"] == "v1"
    }));
    assert!(report["findings"].as_array().unwrap().iter().any(|entry| {
        entry["source"] == "drc"
            && entry["domain"] == "standards"
            && entry["code"] == "pad_process_aperture_inconsistent_with_peer_footprint"
    }));

    let _ = std::fs::remove_dir_all(&root);
}

fn add_peer_process_aperture_inconsistency(root: &Path) {
    let board_json = root.join("board/board.json");
    let mut board: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&board_json).unwrap()).unwrap();
    let first_pad = board["pads"].as_object().unwrap().values().next().unwrap();
    let package = first_pad["package"].clone();
    let net = first_pad["net"].clone();
    let peer_pad_uuid = Uuid::new_v4();
    board["pads"][peer_pad_uuid.to_string()] = serde_json::json!({
        "uuid": peer_pad_uuid,
        "package": package,
        "name": "2",
        "net": net,
        "position": { "x": 1000000, "y": 0 },
        "layer": 1,
        "copper_layers": [1],
        "shape": "rect",
        "width": 1000000,
        "height": 500000,
        "mask_layers": [2],
        "paste_layers": [4],
        "solder_mask_margin_nm": 127000,
        "solder_paste_margin_nm": -127000,
        "solder_paste_margin_ratio_ppm": 0
    });
    std::fs::write(
        &board_json,
        format!("{}\n", serde_json::to_string_pretty(&board).unwrap()),
    )
    .expect("board file should write");
}

#[test]
fn check_run_rejects_unsupported_profile() {
    let root = unique_project_root("datum-eda-cli-check-run-unsupported-profile");
    create_native_project(&root, Some("Unsupported Profile Demo".to_string()))
        .expect("initial scaffold should succeed");

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
            "--profile",
            "future-profile",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("unsupported profile should fail");
    assert!(
        error
            .to_string()
            .contains("unsupported check profile future-profile")
    );

    let _ = std::fs::remove_dir_all(&root);
}
