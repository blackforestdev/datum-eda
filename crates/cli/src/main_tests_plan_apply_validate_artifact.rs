use super::*;

#[test]
fn execute_plan_validate_scoped_replacement_manifest_artifact_reports_match_and_drift() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-validate-scoped-replacement-manifest-artifact-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-validate-scoped-replacement-manifest-artifact.json",
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

    execute(
        Cli::try_parse_from([
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
        .expect("CLI should parse"),
    )
    .expect("manifest export should succeed");

    let ok = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "plan",
            "validate-scoped-replacement-manifest-artifact",
            manifest_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact validation should succeed");
    let ok_payload: serde_json::Value =
        serde_json::from_str(&ok.0).expect("artifact validation JSON should parse");
    assert_eq!(ok.1, 0);
    assert_eq!(ok_payload["matches_expected"], true);
    assert_eq!(ok_payload["canonical_bytes_match"], true);

    let drifted = std::fs::read_to_string(&manifest_path).expect("manifest should read");
    std::fs::write(&manifest_path, format!(" \n{drifted}")).expect("drifted manifest should write");

    let drift = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "plan",
            "validate-scoped-replacement-manifest-artifact",
            manifest_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact validation should succeed");
    let drift_payload: serde_json::Value =
        serde_json::from_str(&drift.0).expect("artifact validation JSON should parse");
    assert_eq!(drift.1, 1);
    assert_eq!(drift_payload["matches_expected"], false);
    assert_eq!(drift_payload["canonical_bytes_match"], false);

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
}
