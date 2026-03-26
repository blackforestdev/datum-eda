use super::*;

#[test]
fn execute_plan_export_scoped_replacement_manifest_writes_versioned_artifact() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-export-scoped-replacement-manifest-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-export-scoped-replacement-manifest.json",
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

    let cli = Cli::try_parse_from([
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
    let output = execute(cli).expect("manifest export should succeed");
    assert!(output.contains("\"version\": 1"));

    let manifest =
        load_scoped_replacement_manifest(&manifest_path).expect("manifest should deserialize");
    assert_eq!(manifest.kind, "scoped_component_replacement_plan_manifest");
    assert_eq!(manifest.version, 1);
    assert!(manifest.board_source_hash.starts_with("sha256:"));
    assert_eq!(manifest.libraries.len(), 1);
    assert_eq!(manifest.libraries[0].path, eagle_fixture_path("simple-opamp.lbr"));
    assert!(manifest.libraries[0].source_hash.starts_with("sha256:"));
    assert_eq!(manifest.plan.replacements.len(), 2);

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
}

#[test]
fn execute_modify_apply_scoped_replacement_plan_file_applies_preview_output() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-plan-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let plan_path = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-plan.json",
        Uuid::new_v4()
    ));
    let target = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-plan-out.kicad_pcb",
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

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        seeded.to_str().unwrap(),
        "scoped-replacement-plan",
        "package",
        "--ref-prefix",
        "R",
        "--value",
        "LMV321",
        "--library",
        eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let preview = execute(query_cli).expect("scoped replacement preview query should succeed");
    std::fs::write(&plan_path, preview).expect("preview file should write");

    let modify_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "modify",
        seeded.to_str().unwrap(),
        "--library",
        eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
        "--apply-scoped-replacement-plan-file",
        plan_path.to_str().unwrap(),
        "--save",
        target.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(modify_cli).expect("scoped replacement apply should succeed");
    assert!(output.contains("\"saved_path\""));

    let components = match query_components(&target).expect("saved components should query") {
        ComponentListView::Board { components } => components,
    };
    assert_eq!(
        components
            .iter()
            .filter(|component| component.value == "ALTAMP")
            .count(),
        2
    );

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&plan_path);
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
fn execute_modify_apply_scoped_replacement_manifest_loads_recorded_libraries() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-manifest-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-manifest.json",
        Uuid::new_v4()
    ));
    let target = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-manifest-out.kicad_pcb",
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
        "json",
        "modify",
        seeded.to_str().unwrap(),
        "--apply-scoped-replacement-manifest",
        manifest_path.to_str().unwrap(),
        "--save",
        target.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(modify_cli).expect("manifest apply should succeed");
    assert!(output.contains("\"saved_path\""));

    let components = match query_components(&target).expect("saved components should query") {
        ComponentListView::Board { components } => components,
    };
    assert_eq!(
        components
            .iter()
            .filter(|component| component.value == "ALTAMP")
            .count(),
        2
    );

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
fn execute_modify_apply_scoped_replacement_manifest_rejects_board_hash_drift() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-manifest-drifted.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-manifest-drifted.json",
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

    let modify_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "modify",
        seeded.to_str().unwrap(),
        "--apply-scoped-replacement-manifest",
        manifest_path.to_str().unwrap(),
        "--save-original",
    ])
    .expect("CLI should parse");
    let err = execute(modify_cli).expect_err("manifest apply should reject drifted board");
    let msg = format!("{err:#}");
    assert!(msg.contains("board hash drifted"), "{msg}");

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
}
