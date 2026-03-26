use super::*;

#[test]
fn execute_plan_export_scoped_replacement_manifest_renders_text_summary() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-export-scoped-replacement-manifest-text-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-export-scoped-replacement-manifest-text.json",
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

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "text",
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
    let output = execute(cli).expect("manifest export should succeed");
    assert!(output.contains("manifest:"));
    assert!(output.contains("kind: scoped_component_replacement_plan_manifest"));
    assert!(output.contains("version: 1"));
    assert!(output.contains("replacements: 1"));

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
}

#[test]
fn execute_plan_inspect_scoped_replacement_manifest_renders_text_summary() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-inspect-scoped-replacement-manifest-text-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-inspect-scoped-replacement-manifest-text.json",
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

    let inspect_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "text",
        "plan",
        "inspect-scoped-replacement-manifest",
        manifest_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(inspect_cli).expect("manifest inspect should succeed");
    assert!(output.contains("manifest:"));
    assert!(output.contains("source_version: 1"));
    assert!(output.contains("migration_applied: false"));
    assert!(output.contains("board:"));
    assert!(output.contains("[match]"));

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
}
