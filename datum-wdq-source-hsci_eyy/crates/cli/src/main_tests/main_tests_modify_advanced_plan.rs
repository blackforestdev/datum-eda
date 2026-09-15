use super::*;

#[test]
fn modify_board_with_plan_resolves_package_and_part_selectors() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-partial-route-apply-replacement-plan.kicad_pcb",
        Uuid::new_v4()
    ));
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    let lmv321 = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .cloned()
        .expect("LMV321 part should exist");
    let altamp = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .cloned()
        .expect("ALTAMP part should exist");

    let report = modify_board_with_plan(
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
                part_uuid: lmv321.uuid,
            },
            AssignPartInput {
                uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                part_uuid: lmv321.uuid,
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
        &[
            PlannedComponentReplacementInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                package_uuid: Some(altamp.package_uuid),
                part_uuid: None,
            },
            PlannedComponentReplacementInput {
                uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                package_uuid: None,
                part_uuid: Some(altamp.uuid),
            },
        ],
        &[],
        &[],
        &[],
    )
    .expect("modify apply_replacement_plan save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));

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
fn modify_board_with_plan_resolves_best_policy_candidates() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-partial-route-apply-replacement-policy.kicad_pcb",
        Uuid::new_v4()
    ));
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    let lmv321 = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .cloned()
        .expect("LMV321 part should exist");

    let report = modify_board_with_plan(
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
                part_uuid: lmv321.uuid,
            },
            AssignPartInput {
                uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                part_uuid: lmv321.uuid,
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
        &[],
        &[
            PolicyDrivenComponentReplacementInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                policy: ComponentReplacementPolicy::BestCompatiblePackage,
            },
            PolicyDrivenComponentReplacementInput {
                uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                policy: ComponentReplacementPolicy::BestCompatiblePart,
            },
        ],
        &[],
        &[],
    )
    .expect("modify apply_replacement_policy save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));

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
fn modify_board_with_plan_applies_scoped_replacement_policy() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-partial-route-apply-scoped-replacement-policy.kicad_pcb",
        Uuid::new_v4()
    ));
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    let lmv321 = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .cloned()
        .expect("LMV321 part should exist");

    let report = modify_board_with_plan(
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
                part_uuid: lmv321.uuid,
            },
            AssignPartInput {
                uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                part_uuid: lmv321.uuid,
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
        &[],
        &[],
        &[ScopedComponentReplacementPolicyInput {
            scope: ComponentReplacementScope {
                reference_prefix: Some("R".to_string()),
                value_equals: Some("LMV321".to_string()),
                current_package_uuid: None,
                current_part_uuid: None,
            },
            policy: ComponentReplacementPolicy::BestCompatiblePackage,
        }],
        &[],
    )
    .expect("modify apply_scoped_replacement_policy save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));

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
