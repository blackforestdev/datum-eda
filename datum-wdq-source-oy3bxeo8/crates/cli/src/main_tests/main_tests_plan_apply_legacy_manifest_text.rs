use super::*;

#[test]
fn execute_modify_apply_scoped_replacement_manifest_renders_text_summary() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-manifest-text-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-manifest-text.json",
        Uuid::new_v4()
    ));
    let target = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-manifest-text-out.kicad_pcb",
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
        &[
            AssignPartInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                part_uuid: lmv321_part_uuid,
            },
            AssignPartInput {
                uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                part_uuid: lmv321_part_uuid,
            },
        ],
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

    let modify_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "text",
        "modify",
        seeded.to_str().unwrap(),
        "--apply-scoped-replacement-manifest",
        manifest_path.to_str().unwrap(),
        "--save",
        target.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(modify_cli).expect("manifest apply should succeed");
    assert!(output.contains("actions:"));
    assert!(output.contains("saved_path:"));
    assert!(output.contains("applied_scoped_replacement_manifests:"));
    assert!(output.contains("migration_applied=false"));
    assert!(output.contains("replacements=2"));

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_file(target.with_file_name(format!(
        "{}.parts.json",
        target.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(target.with_file_name(format!(
        "{}.packages.json",
        target.file_name().unwrap().to_string_lossy()
    )));
}

#[test]
fn execute_plan_upgrade_scoped_replacement_manifest_renders_text_summary() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-upgrade-scoped-replacement-manifest-text-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-upgrade-scoped-replacement-manifest-text.json",
        Uuid::new_v4()
    ));
    let upgraded_path = std::env::temp_dir().join(format!(
        "{}-cli-upgrade-scoped-replacement-manifest-text-out.json",
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

    let mut legacy_payload: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&manifest_path).expect("manifest should read"),
    )
    .expect("manifest JSON should parse");
    let object = legacy_payload
        .as_object_mut()
        .expect("manifest should be an object");
    object.remove("kind");
    object.remove("version");
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&legacy_payload).expect("legacy manifest should serialize"),
    )
    .expect("legacy manifest should write");

    let upgrade_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "text",
        "plan",
        "upgrade-scoped-replacement-manifest",
        manifest_path.to_str().unwrap(),
        "--out",
        upgraded_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(upgrade_cli).expect("manifest upgrade should succeed");
    assert!(output.contains("input:"));
    assert!(output.contains("output:"));
    assert!(output.contains("source_version: 0"));
    assert!(output.contains("version: 1"));
    assert!(output.contains("migration_applied: true"));

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
    let _ = std::fs::remove_file(&upgraded_path);
}
