use super::*;

#[test]
fn execute_plan_compare_scoped_replacement_manifest_artifact_reports_match_and_drift() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-compare-scoped-replacement-manifest-artifact-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-compare-scoped-replacement-manifest-artifact.json",
        Uuid::new_v4()
    ));
    let artifact_path = std::env::temp_dir().join(format!(
        "{}-cli-compare-scoped-replacement-manifest-artifact-other.json",
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

    let current_manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&manifest_path).expect("manifest should read"),
    )
    .expect("manifest JSON should parse");
    let legacy_manifest = serde_json::json!({
        "board_path": current_manifest["board_path"],
        "board_source_hash": current_manifest["board_source_hash"],
        "libraries": current_manifest["libraries"],
        "plan": current_manifest["plan"],
    });
    std::fs::write(
        &artifact_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&legacy_manifest)
                .expect("legacy manifest serialization should succeed")
        ),
    )
    .expect("legacy artifact should write");

    let ok = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "plan",
            "compare-scoped-replacement-manifest-artifact",
            manifest_path.to_str().unwrap(),
            "--artifact",
            artifact_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact compare should succeed");
    let ok_payload: serde_json::Value =
        serde_json::from_str(&ok.0).expect("artifact compare JSON should parse");
    assert_eq!(ok.1, 0);
    assert_eq!(ok_payload["matches_artifact"], true);
    assert_eq!(ok_payload["manifest_source_version"], 1);
    assert_eq!(ok_payload["artifact_source_version"], 0);
    assert_eq!(ok_payload["drift_fields"], serde_json::json!([]));

    let mut drifted_artifact = legacy_manifest;
    drifted_artifact["board_path"] = serde_json::json!("drifted-board.kicad_pcb");
    std::fs::write(
        &artifact_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&drifted_artifact)
                .expect("drifted manifest serialization should succeed")
        ),
    )
    .expect("drifted artifact should write");

    let drift = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "plan",
            "compare-scoped-replacement-manifest-artifact",
            manifest_path.to_str().unwrap(),
            "--artifact",
            artifact_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact compare should succeed");
    let drift_payload: serde_json::Value =
        serde_json::from_str(&drift.0).expect("artifact compare JSON should parse");
    assert_eq!(drift.1, 1);
    assert_eq!(drift_payload["matches_artifact"], false);
    assert_eq!(drift_payload["drift_fields"], serde_json::json!(["board_path"]));

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
    let _ = std::fs::remove_file(&artifact_path);
}
