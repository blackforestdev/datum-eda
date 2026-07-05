use super::*;

#[test]
fn execute_plan_inspect_scoped_replacement_manifest_artifact_reports_artifact_only_fields() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-inspect-scoped-replacement-manifest-artifact-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-inspect-scoped-replacement-manifest-artifact.json",
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

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "plan",
            "inspect-scoped-replacement-manifest-artifact",
            manifest_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact inspection should succeed");
    let payload: serde_json::Value =
        serde_json::from_str(&output).expect("artifact inspection JSON should parse");
    assert_eq!(
        payload["kind"].as_str(),
        Some("scoped_component_replacement_plan_manifest")
    );
    assert_eq!(payload["source_version"].as_u64(), Some(1));
    assert_eq!(payload["version"].as_u64(), Some(1));
    assert_eq!(payload["migration_applied"].as_bool(), Some(false));
    assert_eq!(payload["replacements"].as_u64(), Some(1));
    assert_eq!(payload["libraries"].as_u64(), Some(1));
    assert_eq!(
        payload["board_path"].as_str(),
        Some(seeded.to_str().unwrap())
    );
    assert!(payload.get("all_inputs_match").is_none());

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
}
