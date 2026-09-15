use std::path::Path;

use super::main_tests_project_board_zone::place_zone_fixture_with_thermal;
use super::main_tests_project_check::unique_project_root;
use super::*;
use eda_engine::substrate::ProjectResolver;

#[test]
fn project_generate_standards_repair_proposals_links_unfilled_zone_fill_findings() {
    let root = unique_project_root("datum-eda-cli-project-zone-fill-standards-repair");
    create_native_project(&root, Some("Zone Fill Standards Repair Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_zone_fixture_with_thermal(&root, false);

    let report = generate_standards_repair_report(&root);
    let proposal = report["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["repair_kind"] == "zone_fill")
        .expect("zone-fill repair proposal should be reported");
    assert_eq!(proposal["affected_zone"], zone_uuid);
    assert_eq!(proposal["operations"], 1);
    assert_eq!(proposal["prepared_against"], report["model_revision"]);
    assert_eq!(proposal["prepared_against_current_model"], true);
    assert_eq!(proposal["can_apply"], false);
    assert_eq!(
        proposal["blocker_codes"],
        serde_json::json!(["missing_acceptance"])
    );
    assert!(
        proposal["codes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|code| code == "zone_fill_unfilled")
    );

    let linked = query_check_run(&root);
    let linked_zone_findings = linked["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["code"] == "zone_fill_unfilled")
        .collect::<Vec<_>>();
    assert_eq!(linked_zone_findings.len(), 1);
    assert_eq!(linked_zone_findings[0]["primary_target"]["id"], zone_uuid);
    assert_eq!(linked["proposal_refs"].as_array().unwrap().len(), 1);
    assert!(
        !linked_zone_findings[0]["proposal_refs"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        linked_zone_findings[0]["proposal_links"]
            .as_array()
            .unwrap()
            .iter()
            .any(|link| {
                link["matched_fingerprint"] == linked_zone_findings[0]["fingerprint"]
                    && link["finding_fingerprints"]
                        .as_array()
                        .unwrap()
                        .contains(&linked_zone_findings[0]["fingerprint"])
                    && link["checks_run"]
                        .as_array()
                        .unwrap()
                        .contains(&linked["check_run_id"])
            })
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with zone-fill repair proposal");
    let proposal_id = Uuid::parse_str(proposal["proposal_id"].as_str().unwrap()).unwrap();
    let proposal = model.proposals.get(&proposal_id).unwrap();
    assert_eq!(
        proposal.source,
        eda_engine::substrate::ProposalSource::Check
    );
    assert_eq!(proposal.finding_fingerprints.len(), 1);
    let eda_engine::substrate::Operation::SetZoneFill {
        zone_id,
        previous_zone_fill,
        zone_fill,
    } = &proposal.batch.operations[0]
    else {
        panic!("zone-fill standards repair should use SetZoneFill");
    };
    assert_eq!(*zone_id, Uuid::parse_str(&zone_uuid).unwrap());
    assert!(previous_zone_fill.is_none());
    assert_eq!(zone_fill["schema_version"], 1);
    assert_eq!(zone_fill["zone_id"], zone_uuid);
    assert_eq!(zone_fill["state"], "filled");
    assert_eq!(zone_fill["model_revision"], report["model_revision"]);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_generate_standards_repair_proposals_links_stale_zone_fill_findings() {
    let root = unique_project_root("datum-eda-cli-project-zone-fill-stale-repair");
    create_native_project(&root, Some("Stale Zone Fill Repair Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_zone_fixture_with_thermal(&root, false);

    fill_zone(&root, &zone_uuid);
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "edit-board-zone",
            root.to_str().unwrap(),
            "--zone",
            &zone_uuid,
            "--priority",
            "7",
        ])
        .expect("CLI should parse"),
    )
    .expect("zone edit should make existing fill stale");

    let report = generate_standards_repair_report(&root);
    let proposal = report["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["repair_kind"] == "zone_fill")
        .expect("stale zone-fill repair proposal should be reported");
    assert_eq!(proposal["affected_zone"], zone_uuid);
    assert!(
        proposal["codes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|code| code == "zone_fill_stale")
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with stale zone-fill repair proposal");
    let proposal_id = Uuid::parse_str(proposal["proposal_id"].as_str().unwrap()).unwrap();
    let proposal = model.proposals.get(&proposal_id).unwrap();
    let eda_engine::substrate::Operation::SetZoneFill {
        previous_zone_fill,
        zone_fill,
        ..
    } = &proposal.batch.operations[0]
    else {
        panic!("stale zone-fill repair should use SetZoneFill");
    };
    assert_eq!(
        previous_zone_fill
            .as_ref()
            .expect("stale repair should preserve previous fill")["state"],
        "stale"
    );
    assert_eq!(zone_fill["schema_version"], 1);
    assert_eq!(zone_fill["state"], "filled");
    assert_eq!(zone_fill["model_revision"], report["model_revision"]);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_generate_standards_repair_proposals_skips_unsupported_zone_fill() {
    let root = unique_project_root("datum-eda-cli-project-zone-fill-unsupported-repair");
    create_native_project(&root, Some("Unsupported Zone Fill Repair Demo".to_string()))
        .expect("initial scaffold should succeed");
    place_zone_fixture_with_thermal(&root, true);

    let report = generate_standards_repair_report(&root);
    assert!(
        report["proposals"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["repair_kind"] != "zone_fill")
    );

    let _ = std::fs::remove_dir_all(&root);
}

fn generate_standards_repair_report(root: &Path) -> serde_json::Value {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "generate-standards-repair-proposals",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("standards repair proposals should generate");
    serde_json::from_str(&output).expect("repair report JSON should parse")
}

fn query_check_run(root: &Path) -> serde_json::Value {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "check-run",
        ])
        .expect("CLI should parse"),
    )
    .expect("check-run should succeed");
    serde_json::from_str(&output).expect("check-run JSON should parse")
}

fn fill_zone(root: &Path, zone_uuid: &str) {
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "fill-zones",
            root.to_str().unwrap(),
            "--zone",
            zone_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("fill-zones should succeed");
}
