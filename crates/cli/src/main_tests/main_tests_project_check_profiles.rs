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
    let release = &report["profiles"].as_array().unwrap()[5];
    assert_eq!(release["name"], "Release Readiness");
    assert!(
        release["description"]
            .as_str()
            .unwrap()
            .contains("does not issue a Release or create revision authority")
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
fn drc_profile_reports_clearance_and_silkscreen_coverage_as_evaluated() {
    let root = unique_project_root("datum-eda-cli-check-run-drc-profile-coverage");
    create_native_project(&root, Some("DRC Profile Coverage Demo".to_string()))
        .expect("initial scaffold should succeed");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
            "--profile",
            "drc",
        ])
        .expect("CLI should parse"),
    )
    .expect("drc profile check should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["profile_id"], "drc");
    assert_eq!(
        report["profile_basis"]["domains"],
        serde_json::json!(["drc", "standards"])
    );
    assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
        entry["rule_id"] == "process_aperture_policy" && entry["status"] == "evaluated"
    }));
    assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
        entry["rule_id"] == "zone_fill_state" && entry["status"] == "filtered_by_profile"
    }));
    assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
        entry["domain"] == "drc"
            && entry["rule_id"] == "clearance_copper"
            && entry["status"] == "evaluated"
    }));
    assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
        entry["domain"] == "drc"
            && entry["rule_id"] == "silk_clearance_copper"
            && entry["status"] == "evaluated"
    }));
    assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
        entry["domain"] == "erc"
            && entry["rule_id"] == "schematic_connectivity"
            && entry["status"] == "filtered_by_profile"
    }));

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
    assert_eq!(
        report["profile_basis"]["standards_basis_detail"]["basis_id"],
        "datum.process_aperture_and_geometry.current"
    );
    assert_eq!(
        report["profile_basis"]["standards_basis_detail"]["basis_kind"],
        "process_aperture_geometry"
    );
    assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
        entry["domain"] == "standards"
            && entry["rule_id"] == "process_aperture_policy"
            && entry["status"] == "evaluated"
            && entry["target_scope"] == "board_pads_tracks_vias"
            && entry["standards_basis_detail"]["basis_id"]
                == "datum.process_aperture_and_geometry.current"
    }));
    assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
        entry["domain"] == "erc"
            && entry["rule_id"] == "schematic_connectivity"
            && entry["status"] == "filtered_by_profile"
    }));
    assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
        entry["domain"] == "drc"
            && entry["rule_id"] == "clearance_copper"
            && entry["status"] == "filtered_by_profile"
    }));
    assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
        entry["domain"] == "drc"
            && entry["rule_id"] == "silk_clearance_copper"
            && entry["status"] == "filtered_by_profile"
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
            && entry["standards_basis_detail"]["basis_id"]
                == "datum.process_aperture_and_geometry.current"
            && entry["standards_basis_detail"]["basis_kind"] == "process_aperture_geometry"
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

fn physical_profile_run(root: &Path, profile: &str) -> serde_json::Value {
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
        .unwrap(),
    )
    .unwrap();
    serde_json::from_str(&output).unwrap()
}

#[test]
fn physical_profile_errors_survive_normalization_disposition_and_reopen() {
    let root = unique_project_root("datum-eda-cli-physical-profile-errors");
    create_native_project(&root, Some("Physical profile errors".into())).unwrap();
    let net = super::main_tests_project_drc::seed_board_drc_fixture(&root);
    let board_path = root.join("board/board.json");
    let mut board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&board_path).unwrap()).unwrap();
    board["packages"] = serde_json::json!({});
    board["pads"] = serde_json::json!({});
    let track = Uuid::new_v4().to_string();
    let via = Uuid::new_v4().to_string();
    board["tracks"][&track] = serde_json::json!({
        "uuid": track, "net": net, "from": {"x": 0, "y": 0},
        "to": {"x": 1000000, "y": 0}, "width": 100000, "layer": 1
    });
    board["vias"][&via] = serde_json::json!({
        "uuid": via, "net": net, "position": {"x": 1000000, "y": 0},
        "drill": 100000, "diameter": 200000, "from_layer": 1, "to_layer": 1
    });
    std::fs::write(&board_path, serde_json::to_vec_pretty(&board).unwrap()).unwrap();
    let codes = [
        "track_width_below_min",
        "via_hole_out_of_range",
        "via_annular_below_min",
    ];
    let mut fingerprints = Vec::new();
    for profile in ["drc", "standards", "release"] {
        let report = physical_profile_run(&root, profile);
        let findings = report["findings"].as_array().unwrap();
        for code in codes {
            assert!(
                report["raw_report"]["drc"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|f| f["code"] == code && f["severity"] == "error"),
                "raw {profile}: {code}"
            );
            let finding = findings
                .iter()
                .find(|f| f["code"] == code)
                .unwrap_or_else(|| panic!("{profile} dropped emitted {code}"));
            assert_eq!(finding["severity"], "error");
            assert_eq!(finding["domain"], "standards");
            if profile == "drc" {
                fingerprints.push(finding["fingerprint"].as_str().unwrap().to_string());
            } else {
                assert!(
                    fingerprints.contains(&finding["fingerprint"].as_str().unwrap().to_string())
                );
            }
        }
        assert!(report["summary"]["errors"].as_u64().unwrap() >= 3);
        assert_eq!(report["status"], "error");
        let id = Uuid::parse_str(report["check_run_id"].as_str().unwrap()).unwrap();
        let cold = eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap();
        let saved = &cold.check_runs[&id];
        assert_eq!(saved.status, "error");
        assert_eq!(saved.summary, report["summary"]);
        for code in codes {
            assert!(
                saved
                    .findings
                    .iter()
                    .any(|f| f.code == code && f.severity == "error")
            );
        }
    }
    for (command, flag, fingerprint) in [
        ("waive", "--created-by", &fingerprints[0]),
        ("accept-deviation", "--accepted-by", &fingerprints[1]),
    ] {
        let output = execute(
            Cli::try_parse_from([
                "eda",
                "--format",
                "json",
                "check",
                command,
                root.to_str().unwrap(),
                "--fingerprint",
                fingerprint,
                "--rationale",
                "Intentional test disposition",
                flag,
                "cli-test",
            ])
            .unwrap(),
        )
        .unwrap();
        let draft = serde_json::from_str(&output).unwrap();
        super::main_tests_project_check_run_drc_lifecycle::accept_disposition_draft(&root, &draft);
    }
    for profile in ["drc", "standards", "release"] {
        let report = physical_profile_run(&root, profile);
        for (index, status, refs) in [
            (0, "waived", "waiver_refs"),
            (1, "accepted_deviation", "deviation_refs"),
        ] {
            let finding = report["findings"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["fingerprint"] == fingerprints[index])
                .unwrap();
            assert_eq!(finding["status"], status);
            assert_eq!(finding[refs].as_array().unwrap().len(), 1);
        }
        assert_eq!(report["summary"]["waived"], 1);
        assert!(report["summary"]["errors"].as_u64().unwrap() >= 1);
    }
    board["tracks"][&track]["width"] = serde_json::json!(200000);
    board["vias"][&via]["drill"] = serde_json::json!(300000);
    board["vias"][&via]["diameter"] = serde_json::json!(600000);
    std::fs::write(&board_path, serde_json::to_vec_pretty(&board).unwrap()).unwrap();
    for profile in ["drc", "standards", "release"] {
        let report = physical_profile_run(&root, profile);
        assert!(
            report["findings"]
                .as_array()
                .unwrap()
                .iter()
                .all(|f| !codes.contains(&f["code"].as_str().unwrap()))
        );
        assert_eq!(report["summary"]["errors"], 0, "{profile}: {report}");
    }
    std::fs::remove_dir_all(root).unwrap();
}
