use super::super::*;

#[test]
fn get_part_returns_pool_part_details() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&eagle_fixture_path("bjt-sot23.lbr"))
        .expect("fixture should import");
    let part_uuid = engine
        .search_pool("sot23")
        .expect("search should succeed")
        .first()
        .expect("part should exist")
        .uuid;
    let part = engine
        .get_part(&part_uuid)
        .expect("part query should succeed");
    assert_eq!(part.uuid, part_uuid);
    assert!(!part.package.name.is_empty());
    assert!(part.package.pads > 0);
    assert!(!part.entity.gates.is_empty());
}

#[test]
fn get_package_returns_pool_package_details() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&eagle_fixture_path("bjt-sot23.lbr"))
        .expect("fixture should import");
    let part_uuid = engine
        .search_pool("sot23")
        .expect("search should succeed")
        .first()
        .expect("part should exist")
        .uuid;
    let part = engine
        .get_part(&part_uuid)
        .expect("part query should succeed");
    let package_uuid = engine
        .pool
        .parts
        .get(&part_uuid)
        .expect("part should exist in pool")
        .package;
    let package = engine
        .get_package(&package_uuid)
        .expect("package query should succeed");
    assert_eq!(package.uuid, package_uuid);
    assert_eq!(package.name, part.package.name);
    assert!(!package.pads.is_empty());
}

#[test]
fn get_package_change_candidates_reports_unique_compatible_packages() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture should import");
    let lmv321_part_uuid = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .expect("LMV321 part should exist")
        .uuid;
    engine
        .assign_part(AssignPartInput {
            uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            part_uuid: lmv321_part_uuid,
        })
        .expect("assign_part should succeed");

    let report = engine
        .get_package_change_candidates(
            &uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        )
        .expect("candidate query should succeed");
    assert_eq!(
        report.status,
        PackageChangeCompatibilityStatus::CandidatesAvailable
    );
    assert_eq!(report.current_part_uuid, Some(lmv321_part_uuid));
    assert_eq!(report.ambiguous_package_count, 0);
    assert_eq!(report.candidates.len(), 1);
    assert_eq!(report.candidates[0].package_name, "ALT-3");
    assert_eq!(report.candidates[0].compatible_part_value, "ALTAMP");
}

#[test]
fn get_part_change_candidates_reports_compatible_parts() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture should import");
    let lmv321_part_uuid = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .expect("LMV321 part should exist")
        .uuid;
    engine
        .assign_part(AssignPartInput {
            uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            part_uuid: lmv321_part_uuid,
        })
        .expect("assign_part should succeed");

    let report = engine
        .get_part_change_candidates(
            &uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        )
        .expect("candidate query should succeed");
    assert_eq!(
        report.status,
        PartChangeCompatibilityStatus::CandidatesAvailable
    );
    assert_eq!(report.current_part_uuid, Some(lmv321_part_uuid));
    assert!(!report.candidates.is_empty());
    assert!(
        report
            .candidates
            .iter()
            .any(|candidate| candidate.package_name == "ALT-3" && candidate.value == "ALTAMP")
    );
}

#[test]
fn get_component_replacement_plan_combines_part_and_package_candidates() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture should import");
    let lmv321_part_uuid = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .expect("LMV321 part should exist")
        .uuid;
    let component_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    engine
        .assign_part(AssignPartInput {
            uuid: component_uuid,
            part_uuid: lmv321_part_uuid,
        })
        .expect("assign_part should succeed");

    let plan = engine
        .get_component_replacement_plan(&component_uuid)
        .expect("replacement plan query should succeed");
    assert_eq!(plan.component_uuid, component_uuid);
    assert_eq!(plan.current_reference, "R1");
    assert_eq!(plan.current_part_uuid, Some(lmv321_part_uuid));
    assert_eq!(
        plan.package_change.status,
        PackageChangeCompatibilityStatus::CandidatesAvailable
    );
    assert_eq!(
        plan.part_change.status,
        PartChangeCompatibilityStatus::CandidatesAvailable
    );
    assert!(
        plan.package_change
            .candidates
            .iter()
            .any(|candidate| candidate.package_name == "ALT-3")
    );
    assert!(
        plan.part_change
            .candidates
            .iter()
            .any(|candidate| candidate.package_name == "ALT-3" && candidate.value == "ALTAMP")
    );
}

#[test]
fn get_netlist_returns_board_nets_for_board_project() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("simple-demo.kicad_pcb"))
        .expect("fixture should import");
    let nets = engine.get_netlist().expect("netlist query should succeed");
    assert_eq!(nets.len(), 2);
    let gnd = nets
        .iter()
        .find(|net| net.name == "GND")
        .expect("GND net should exist");
    assert_eq!(gnd.class.as_deref(), Some("Default"));
    assert!(gnd.routed_pct.is_some());
    assert!(gnd.labels.is_none());
    assert!(gnd.ports.is_none());
}

#[test]
fn explain_violation_returns_erc_explanation_for_valid_index() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("simple-demo.kicad_sch"))
        .expect("fixture should import");
    let explanation = engine
        .explain_violation(ViolationDomain::Erc, 0)
        .expect("explanation should succeed");
    assert!(!explanation.explanation.is_empty());
    assert!(explanation.rule_detail.starts_with("erc "));
    assert!(!explanation.suggestion.is_empty());
}

#[test]
fn explain_violation_returns_drc_explanation_for_valid_index() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture should import");
    let explanation = engine
        .explain_violation(ViolationDomain::Drc, 0)
        .expect("explanation should succeed");
    assert!(!explanation.explanation.is_empty());
    assert!(explanation.rule_detail.starts_with("drc "));
    assert!(!explanation.suggestion.is_empty());
}

#[test]
fn get_netlist_returns_schematic_nets_for_schematic_project() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("simple-demo.kicad_sch"))
        .expect("fixture should import");
    let nets = engine.get_netlist().expect("netlist query should succeed");
    assert_eq!(nets.len(), 4);
    let vcc = nets
        .iter()
        .find(|net| net.name == "VCC")
        .expect("VCC net should exist");
    assert_eq!(vcc.semantic_class.as_deref(), Some("power"));
    assert!(vcc.routed_pct.is_none());
    assert_eq!(vcc.labels, Some(1));
}
