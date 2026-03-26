use super::*;

#[test]
fn execute_plan_validate_scoped_replacement_manifest_reports_match_exit_zero() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-validate-scoped-replacement-manifest-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-validate-scoped-replacement-manifest.json",
        Uuid::new_v4()
    ));
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    let lmv321_part_uuid = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .map(|part| part.uuid)
        .expect("LMV321 part should exist");
    modify_board(
        &source,
        &[],
        &[],
        &[],
        &[eagle_fixture_path("simple-opamp.lbr")],
        &[],
        &[],
        &[],
        &[AssignPartInput {
            uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            part_uuid: lmv321_part_uuid,
        }],
        &[],
        &[],
        &[],
        &[],
        &[],
        None,
        0,
        0,
        Some(&seeded),
        false,
    )
    .expect("modify assign_part save should succeed");

    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "plan",
        "export-scoped-replacement-manifest",
        seeded.to_str().unwrap(),
        "--out",
        manifest_path.to_str().unwrap(),
        "package",
        "--ref-prefix",
        "R",
        "--value",
        "LMV321",
        "--library",
        eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
    ])
    .expect("CLI should parse");
    execute(export_cli).expect("manifest export should succeed");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "plan",
        "validate-scoped-replacement-manifest",
        manifest_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let (output, exit_code) =
        execute_with_exit_code(cli).expect("manifest validation should succeed");
    let payload: serde_json::Value =
        serde_json::from_str(&output).expect("manifest validation JSON should parse");
    assert_eq!(exit_code, 0);
    assert_eq!(payload["manifests_checked"].as_u64(), Some(1));
    assert_eq!(payload["manifests_passing"].as_u64(), Some(1));
    assert_eq!(payload["manifests_failing"].as_u64(), Some(0));
    assert_eq!(payload["reports"][0]["all_inputs_match"].as_bool(), Some(true));
    assert_eq!(payload["reports"][0]["board_status"].as_str(), Some("match"));
    assert_eq!(payload["reports"][0]["drifted_libraries"].as_u64(), Some(0));
    assert_eq!(payload["reports"][0]["missing_libraries"].as_u64(), Some(0));

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
}

#[test]
fn execute_plan_validate_scoped_replacement_manifest_reports_drift_exit_one() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-validate-scoped-replacement-manifest-drifted.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-validate-scoped-replacement-manifest-drifted.json",
        Uuid::new_v4()
    ));
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    let lmv321_part_uuid = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .map(|part| part.uuid)
        .expect("LMV321 part should exist");
    modify_board(
        &source,
        &[],
        &[],
        &[],
        &[eagle_fixture_path("simple-opamp.lbr")],
        &[],
        &[],
        &[],
        &[AssignPartInput {
            uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            part_uuid: lmv321_part_uuid,
        }],
        &[],
        &[],
        &[],
        &[],
        &[],
        None,
        0,
        0,
        Some(&seeded),
        false,
    )
    .expect("modify assign_part save should succeed");

    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "plan",
        "export-scoped-replacement-manifest",
        seeded.to_str().unwrap(),
        "--out",
        manifest_path.to_str().unwrap(),
        "package",
        "--ref-prefix",
        "R",
        "--value",
        "LMV321",
        "--library",
        eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
    ])
    .expect("CLI should parse");
    execute(export_cli).expect("manifest export should succeed");

    std::fs::write(&seeded, "(kicad_pcb drifted)\n").expect("board drift should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "text",
        "plan",
        "validate-scoped-replacement-manifest",
        manifest_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let (output, exit_code) =
        execute_with_exit_code(cli).expect("manifest validation should succeed");
    assert_eq!(exit_code, 1);
    assert!(output.contains("manifests_checked: 1"));
    assert!(output.contains("manifests_failing: 1"));
    assert!(output.contains("all_inputs_match: false"));
    assert!(output.contains("board_status: drifted"));
    assert!(output.contains("drifted_libraries: 0"));
    assert!(output.contains("missing_libraries: 0"));

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
}

#[test]
fn execute_plan_validate_scoped_replacement_manifest_batches_results() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded_ok = std::env::temp_dir().join(format!(
        "{}-cli-validate-scoped-replacement-manifest-batch-ok.kicad_pcb",
        Uuid::new_v4()
    ));
    let seeded_drifted = std::env::temp_dir().join(format!(
        "{}-cli-validate-scoped-replacement-manifest-batch-drifted.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_ok = std::env::temp_dir().join(format!(
        "{}-cli-validate-scoped-replacement-manifest-batch-ok.json",
        Uuid::new_v4()
    ));
    let manifest_drifted = std::env::temp_dir().join(format!(
        "{}-cli-validate-scoped-replacement-manifest-batch-drifted.json",
        Uuid::new_v4()
    ));
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    let lmv321_part_uuid = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .map(|part| part.uuid)
        .expect("LMV321 part should exist");

    for seeded in [&seeded_ok, &seeded_drifted] {
        modify_board(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[AssignPartInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                part_uuid: lmv321_part_uuid,
            }],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(seeded),
            false,
        )
        .expect("modify assign_part save should succeed");
    }

    for (seeded, manifest_path) in [(&seeded_ok, &manifest_ok), (&seeded_drifted, &manifest_drifted)] {
        let export_cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "plan",
            "export-scoped-replacement-manifest",
            seeded.to_str().unwrap(),
            "--out",
            manifest_path.to_str().unwrap(),
            "package",
            "--ref-prefix",
            "R",
            "--value",
            "LMV321",
            "--library",
            eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
        ])
        .expect("CLI should parse");
        execute(export_cli).expect("manifest export should succeed");
    }

    std::fs::write(&seeded_drifted, "(kicad_pcb drifted)\n").expect("board drift should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "plan",
        "validate-scoped-replacement-manifest",
        manifest_ok.to_str().unwrap(),
        manifest_drifted.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let (output, exit_code) =
        execute_with_exit_code(cli).expect("batch manifest validation should succeed");
    let payload: serde_json::Value =
        serde_json::from_str(&output).expect("manifest validation JSON should parse");
    assert_eq!(exit_code, 1);
    assert_eq!(payload["manifests_checked"].as_u64(), Some(2));
    assert_eq!(payload["manifests_passing"].as_u64(), Some(1));
    assert_eq!(payload["manifests_failing"].as_u64(), Some(1));
    assert_eq!(payload["reports"].as_array().map(Vec::len), Some(2));

    let _ = std::fs::remove_file(&seeded_ok);
    let _ = std::fs::remove_file(seeded_ok.with_file_name(format!(
        "{}.parts.json",
        seeded_ok.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&seeded_drifted);
    let _ = std::fs::remove_file(seeded_drifted.with_file_name(format!(
        "{}.parts.json",
        seeded_drifted.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_ok);
    let _ = std::fs::remove_file(&manifest_drifted);
}
