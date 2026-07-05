use super::*;

#[test]
fn execute_plan_inspect_scoped_replacement_manifest_upgrades_legacy_unversioned_artifact() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-inspect-scoped-replacement-manifest-legacy-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-inspect-scoped-replacement-manifest-legacy.json",
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

    let inspect_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "plan",
        "inspect-scoped-replacement-manifest",
        manifest_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(inspect_cli).expect("legacy manifest inspect should succeed");
    let payload: serde_json::Value =
        serde_json::from_str(&output).expect("manifest inspect JSON should parse");
    assert_eq!(
        payload["kind"].as_str(),
        Some("scoped_component_replacement_plan_manifest")
    );
    assert_eq!(payload["source_version"].as_u64(), Some(0));
    assert_eq!(payload["version"].as_u64(), Some(1));
    assert_eq!(payload["migration_applied"].as_bool(), Some(true));
    assert_eq!(payload["all_inputs_match"].as_bool(), Some(true));

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
}

#[test]
fn execute_modify_apply_scoped_replacement_manifest_upgrades_legacy_unversioned_artifact() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-manifest-legacy-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-manifest-legacy.json",
        Uuid::new_v4()
    ));
    let target = std::env::temp_dir().join(format!(
        "{}-cli-apply-scoped-replacement-manifest-legacy-out.kicad_pcb",
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
    let output = execute(modify_cli).expect("legacy manifest apply should succeed");
    let payload: serde_json::Value =
        serde_json::from_str(&output).expect("legacy manifest apply JSON should parse");
    assert!(payload["saved_path"].is_string());
    assert_eq!(
        payload["applied_scoped_replacement_manifests"][0]["source_version"].as_u64(),
        Some(0)
    );
    assert_eq!(
        payload["applied_scoped_replacement_manifests"][0]["version"].as_u64(),
        Some(1)
    );
    assert_eq!(
        payload["applied_scoped_replacement_manifests"][0]["migration_applied"].as_bool(),
        Some(true)
    );
    assert_eq!(
        payload["applied_scoped_replacement_manifests"][0]["replacements"].as_u64(),
        Some(2)
    );

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
fn execute_plan_upgrade_scoped_replacement_manifest_rewrites_legacy_artifact() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-upgrade-scoped-replacement-manifest-legacy-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-upgrade-scoped-replacement-manifest-legacy.json",
        Uuid::new_v4()
    ));
    let upgraded_path = std::env::temp_dir().join(format!(
        "{}-cli-upgrade-scoped-replacement-manifest-upgraded.json",
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
        "json",
        "plan",
        "upgrade-scoped-replacement-manifest",
        manifest_path.to_str().unwrap(),
        "--out",
        upgraded_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(upgrade_cli).expect("manifest upgrade should succeed");
    let payload: serde_json::Value =
        serde_json::from_str(&output).expect("manifest upgrade JSON should parse");
    assert_eq!(payload["source_version"].as_u64(), Some(0));
    assert_eq!(payload["version"].as_u64(), Some(1));
    assert_eq!(payload["migration_applied"].as_bool(), Some(true));

    let upgraded = load_scoped_replacement_manifest(&upgraded_path)
        .expect("upgraded manifest should deserialize");
    assert_eq!(upgraded.kind, "scoped_component_replacement_plan_manifest");
    assert_eq!(upgraded.version, 1);

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
    let _ = std::fs::remove_file(&upgraded_path);
}

#[test]
fn execute_plan_upgrade_scoped_replacement_manifest_reports_noop_for_current_artifact() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-upgrade-scoped-replacement-manifest-current-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-upgrade-scoped-replacement-manifest-current.json",
        Uuid::new_v4()
    ));
    let upgraded_path = std::env::temp_dir().join(format!(
        "{}-cli-upgrade-scoped-replacement-manifest-current-out.json",
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

    let upgrade_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "plan",
        "upgrade-scoped-replacement-manifest",
        manifest_path.to_str().unwrap(),
        "--out",
        upgraded_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(upgrade_cli).expect("manifest upgrade should succeed");
    let payload: serde_json::Value =
        serde_json::from_str(&output).expect("manifest upgrade JSON should parse");
    assert_eq!(payload["source_version"].as_u64(), Some(1));
    assert_eq!(payload["version"].as_u64(), Some(1));
    assert_eq!(payload["migration_applied"].as_bool(), Some(false));

    let upgraded = load_scoped_replacement_manifest(&upgraded_path)
        .expect("upgraded manifest should deserialize");
    assert_eq!(upgraded.version, 1);

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
    let _ = std::fs::remove_file(&upgraded_path);
}

#[test]
fn execute_plan_upgrade_scoped_replacement_manifest_supports_in_place_rewrite() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let seeded = std::env::temp_dir().join(format!(
        "{}-cli-upgrade-scoped-replacement-manifest-in-place-seeded.kicad_pcb",
        Uuid::new_v4()
    ));
    let manifest_path = std::env::temp_dir().join(format!(
        "{}-cli-upgrade-scoped-replacement-manifest-in-place.json",
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
        "json",
        "plan",
        "upgrade-scoped-replacement-manifest",
        manifest_path.to_str().unwrap(),
        "--in-place",
    ])
    .expect("CLI should parse");
    let output = execute(upgrade_cli).expect("in-place manifest upgrade should succeed");
    let payload: serde_json::Value =
        serde_json::from_str(&output).expect("manifest upgrade JSON should parse");
    assert_eq!(payload["source_version"].as_u64(), Some(0));
    assert_eq!(payload["version"].as_u64(), Some(1));
    assert_eq!(payload["migration_applied"].as_bool(), Some(true));
    assert_eq!(
        payload["input_path"].as_str(),
        Some(manifest_path.to_str().unwrap())
    );
    assert_eq!(
        payload["output_path"].as_str(),
        Some(manifest_path.to_str().unwrap())
    );

    let upgraded = load_scoped_replacement_manifest(&manifest_path)
        .expect("in-place upgraded manifest should deserialize");
    assert_eq!(upgraded.version, 1);
    assert_eq!(upgraded.kind, "scoped_component_replacement_plan_manifest");

    let _ = std::fs::remove_file(&seeded);
    let _ = std::fs::remove_file(seeded.with_file_name(format!(
        "{}.parts.json",
        seeded.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(&manifest_path);
}
