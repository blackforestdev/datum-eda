use super::*;
use eda_engine::import::ImportKind;

#[test]
fn import_path_supports_eagle_libraries() {
    let report =
        import_path(&eagle_fixture_path("simple-opamp.lbr")).expect("fixture should import");
    assert!(matches!(report.kind, ImportKind::EagleLibrary));
    assert_eq!(report.counts.parts, 2);
    assert_eq!(
        report.metadata.get("library_name").map(String::as_str),
        Some("demo-analog")
    );
}

#[test]
fn search_pool_loads_multiple_libraries() {
    let results = search_pool(
        "SOT23",
        &[
            eagle_fixture_path("simple-opamp.lbr"),
            eagle_fixture_path("bjt-sot23.lbr"),
        ],
    )
    .expect("search should succeed");

    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|part| part.package == "SOT23"));
    assert!(results.iter().any(|part| part.package == "SOT23-5"));
}

#[test]
fn search_pool_rejects_non_lbr_inputs() {
    let err = search_pool("x", &[PathBuf::from("not-a-library.txt")])
        .expect_err("non-lbr input must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("only accepts Eagle .lbr inputs"), "{msg}");
}

#[test]
fn render_output_json_formats_structured_data() {
    let report = ImportReportView::from(
        import_path(&eagle_fixture_path("simple-opamp.lbr")).expect("fixture should import"),
    );
    let output = render_output(&OutputFormat::Json, &report);
    assert!(output.contains("\"kind\": \"eagle_library\""));
    assert!(output.contains("\"library_name\": \"demo-analog\""));
}

#[test]
fn render_output_text_joins_array_items() {
    let results = search_pool("SOT23", &[eagle_fixture_path("bjt-sot23.lbr")])
        .expect("search should succeed");
    let output = render_output(&OutputFormat::Text, &results);
    assert!(output.contains("\"package\": \"SOT23\""));
}

#[test]
fn cli_upgrade_scoped_replacement_manifest_rejects_out_and_in_place_together() {
    let cli = Cli::try_parse_from([
        "eda",
        "plan",
        "upgrade-scoped-replacement-manifest",
        "input.json",
        "--out",
        "output.json",
        "--in-place",
    ])
    .expect("CLI should parse");
    let err = execute(cli).expect_err("upgrade command should reject ambiguous output mode");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("either --out or --in-place, not both"),
        "{msg}"
    );
}

#[test]
fn clap_parses_import_command_with_global_format_before_subcommand() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "import",
        eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
    ])
    .expect("CLI should parse");

    match cli.command {
        Commands::Import { path } => assert!(path.ends_with("simple-opamp.lbr")),
        _ => panic!("expected import command"),
    }
    assert!(matches!(cli.format, OutputFormat::Json));
}

#[test]
fn execute_import_command_returns_report_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "import",
        eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("import command should succeed");
    assert!(output.contains("\"kind\": \"eagle_library\""));
    assert!(output.contains("\"parts\": 2"));
}

#[test]
fn execute_query_package_change_candidates_returns_report_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("partial-route-demo.kicad_pcb")
            .to_str()
            .unwrap(),
        "package-change-candidates",
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "--library",
        eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("candidate query should succeed");
    assert!(output.contains("\"status\": \"no_known_part\""));
}

#[test]
fn execute_query_part_change_candidates_returns_report_output() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-query-part-change-candidates.kicad_pcb",
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
        Some(&target),
        false,
    )
    .expect("modify assign_part save should succeed");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        target.to_str().unwrap(),
        "part-change-candidates",
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "--library",
        eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("part-change candidate query should succeed");
    assert!(output.contains("\"status\": \"candidates_available\""));
    assert!(output.contains("\"package_name\": \"ALT-3\""));
    assert!(output.contains("\"value\": \"ALTAMP\""));

    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_file(target.with_file_name(format!(
        "{}.parts.json",
        target.file_name().unwrap().to_string_lossy()
    )));
}

#[test]
fn execute_query_component_replacement_plan_returns_combined_report_output() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-query-component-replacement-plan.kicad_pcb",
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
        Some(&target),
        false,
    )
    .expect("modify assign_part save should succeed");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        target.to_str().unwrap(),
        "component-replacement-plan",
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "--library",
        eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("component replacement plan query should succeed");
    assert!(output.contains("\"current_reference\": \"R1\""));
    assert!(output.contains("\"package_change\""));
    assert!(output.contains("\"part_change\""));
    assert!(output.contains("\"status\": \"candidates_available\""));

    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_file(target.with_file_name(format!(
        "{}.parts.json",
        target.file_name().unwrap().to_string_lossy()
    )));
}

#[test]
fn execute_query_scoped_replacement_plan_returns_resolved_preview_output() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-query-scoped-replacement-plan.kicad_pcb",
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
        Some(&target),
        false,
    )
    .expect("modify assign_part save should succeed");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        target.to_str().unwrap(),
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

    let output = execute(cli).expect("scoped replacement preview query should succeed");
    assert!(output.contains("\"policy\": \"best_compatible_package\""));
    assert!(output.contains("\"current_reference\": \"R1\""));
    assert!(output.contains("\"target_package_name\": \"ALT-3\""));
    assert!(output.contains("\"target_value\": \"ALTAMP\""));

    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_file(target.with_file_name(format!(
        "{}.parts.json",
        target.file_name().unwrap().to_string_lossy()
    )));
}

#[test]
fn execute_query_scoped_replacement_plan_supports_exclusions_and_overrides() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-query-scoped-replacement-plan-edited.kicad_pcb",
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
    let altamp = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .cloned()
        .expect("ALTAMP part should exist");
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
        Some(&target),
        false,
    )
    .expect("modify assign_part save should succeed");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        target.to_str().unwrap(),
        "scoped-replacement-plan",
        "package",
        "--ref-prefix",
        "R",
        "--value",
        "LMV321",
        "--exclude-component",
        "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        "--override-component",
        &format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}:{}",
            altamp.package_uuid, altamp.uuid
        ),
        "--library",
        eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("edited scoped replacement preview query should succeed");
    assert!(output.contains("\"component_uuid\": \"aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa\""));
    assert!(!output.contains("\"component_uuid\": \"bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb\""));
    assert!(output.contains("\"target_package_name\": \"ALT-3\""));

    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_file(target.with_file_name(format!(
        "{}.parts.json",
        target.file_name().unwrap().to_string_lossy()
    )));
}
