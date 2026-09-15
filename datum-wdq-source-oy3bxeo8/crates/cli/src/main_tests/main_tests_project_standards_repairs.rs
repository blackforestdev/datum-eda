use std::path::Path;

use super::main_tests_project_check::{seed_board_process_aperture_fixture, unique_project_root};
use super::*;
use eda_engine::substrate::ProjectResolver;

#[test]
fn project_generate_standards_repair_proposals_links_process_aperture_findings() {
    let root = unique_project_root("datum-eda-cli-project-check-run-standards-repair");
    create_native_project(&root, Some("Standards Repair Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_process_aperture_fixture(&root);

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
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("repair report JSON should parse");
    assert_eq!(report["action"], "generate_standards_repair_proposals");
    assert_eq!(report["proposal_count"], 2);

    let proposals = report["proposals"].as_array().unwrap();
    assert!(proposals.iter().all(|entry| entry["operations"] == 1));
    assert!(proposals.iter().all(|entry| {
        entry["prepared_against"] == report["model_revision"]
            && entry["prepared_against_current_model"] == true
            && entry["can_apply"] == false
            && entry["blocker_codes"] == serde_json::json!(["missing_acceptance"])
    }));
    assert!(proposals.iter().all(|entry| {
        let codes = entry["codes"].as_array().unwrap();
        codes
            .iter()
            .any(|code| code == "pad_process_aperture_inherited_from_copper")
            && codes
                .iter()
                .any(|code| code == "pad_mask_expansion_missing")
            && codes
                .iter()
                .any(|code| code == "pad_paste_reduction_missing")
    }));

    let linked_output = execute(
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
    .expect("linked check-run should succeed");
    let linked: serde_json::Value =
        serde_json::from_str(&linked_output).expect("linked check-run JSON should parse");
    assert_eq!(linked["proposal_refs"].as_array().unwrap().len(), 2);
    let linked_process_findings = linked["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| {
            entry["code"] == "pad_process_aperture_inherited_from_copper"
                || entry["code"] == "pad_mask_expansion_missing"
                || entry["code"] == "pad_paste_reduction_missing"
        })
        .collect::<Vec<_>>();
    assert_eq!(linked_process_findings.len(), 6);
    assert!(
        linked_process_findings
            .iter()
            .all(|entry| { !entry["proposal_refs"].as_array().unwrap().is_empty() })
    );
    assert!(linked_process_findings.iter().all(|entry| {
        entry["proposal_links"]
            .as_array()
            .unwrap()
            .iter()
            .all(|link| {
                link["matched_fingerprint"] == entry["fingerprint"]
                    && link["finding_fingerprints"]
                        .as_array()
                        .unwrap()
                        .contains(&entry["fingerprint"])
                    && link["checks_run"]
                        .as_array()
                        .unwrap()
                        .contains(&linked["check_run_id"])
            })
    }));
    assert!(linked_process_findings.iter().all(|entry| {
        entry["primary_target"]["kind"] == "object_uuid"
            && entry["domain"] == "standards"
            && entry["standards_basis"] == "datum.process_aperture_and_geometry.current"
            && entry["rule_revision"] == "v1"
            && entry["primary_target"]["id"].as_str().unwrap().len() > 20
            && entry["related_targets"].as_array().unwrap().is_empty()
            && entry["suggested_next_action"]
                .as_str()
                .unwrap()
                .contains("repair-standards")
    }));

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with proposals");
    for proposal_id in linked["proposal_refs"].as_array().unwrap() {
        let proposal_id = Uuid::parse_str(proposal_id.as_str().unwrap()).unwrap();
        let proposal = model.proposals.get(&proposal_id).unwrap();
        assert_eq!(
            proposal.status,
            eda_engine::substrate::ProposalStatus::Draft
        );
        assert_eq!(
            proposal.source,
            eda_engine::substrate::ProposalSource::Check
        );
        assert_eq!(proposal.finding_fingerprints.len(), 3);
        assert!(
            proposal
                .finding_fingerprints
                .iter()
                .all(|fingerprint| fingerprint.starts_with("sha256:"))
        );
        let linked_fingerprints = linked_process_findings
            .iter()
            .filter(|entry| {
                entry["proposal_refs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|value| value == &proposal_id.to_string())
            })
            .map(|entry| entry["fingerprint"].as_str().unwrap().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            proposal
                .finding_fingerprints
                .iter()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>(),
            linked_fingerprints
        );
        assert_eq!(proposal.batch.operations.len(), 1);
        let eda_engine::substrate::Operation::SetBoardPad { pad, .. } =
            &proposal.batch.operations[0]
        else {
            panic!("standards repair should use SetBoardPad");
        };
        assert_eq!(pad["solder_mask_margin_nm"], 127000);
        assert_eq!(pad["solder_paste_margin_nm"], -127000);
    }

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_generate_standards_repair_proposals_is_idempotent_across_reruns() {
    let root = unique_project_root("datum-eda-cli-project-check-run-standards-repair-idempotent");
    create_native_project(&root, Some("Standards Repair Idempotence Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_process_aperture_fixture(&root);

    let first = generate_standards_repair_report(&root);
    let second = generate_standards_repair_report(&root);
    let first_ids = repair_proposal_ids(&first);
    let second_ids = repair_proposal_ids(&second);
    assert_eq!(first_ids, second_ids);
    assert_eq!(first_ids.len(), 2);

    let linked_output = execute(
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
    .expect("linked check-run should succeed");
    let linked: serde_json::Value =
        serde_json::from_str(&linked_output).expect("linked check-run JSON should parse");
    let proposal_refs = linked["proposal_refs"].as_array().unwrap();
    assert_eq!(proposal_refs.len(), 2);
    let unique_refs = proposal_refs
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique_refs, first_ids);

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with proposals");
    let repair_proposals = model
        .proposals
        .values()
        .filter(|proposal| proposal.source == eda_engine::substrate::ProposalSource::Check)
        .count();
    assert_eq!(repair_proposals, 2);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_waive_finding_authors_standards_domain_waiver() {
    let root = unique_project_root("datum-eda-cli-project-standards-waiver");
    create_native_project(&root, Some("Standards Waiver Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_process_aperture_fixture(&root);
    let fingerprint = standards_finding_fingerprint(&root, "pad_mask_expansion_missing");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "waive-finding",
            root.to_str().unwrap(),
            "--fingerprint",
            &fingerprint,
            "--rationale",
            "Intentional process aperture exception",
            "--created-by",
            "cli-test",
        ])
        .expect("CLI should parse"),
    )
    .expect("standards waiver should be authored");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["domain"], "standards");
    assert_standards_finding_status(&root, &fingerprint, "waived", "waiver_refs");

    let schematic = read_schematic_root(&root);
    assert!(
        schematic["waivers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|waiver| {
                waiver["domain"] == "Standards" && waiver["target"]["Fingerprint"] == fingerprint
            })
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_accept_deviation_authors_standards_domain_deviation() {
    let root = unique_project_root("datum-eda-cli-project-standards-deviation");
    create_native_project(&root, Some("Standards Deviation Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_process_aperture_fixture(&root);
    let fingerprint = standards_finding_fingerprint(&root, "pad_paste_reduction_missing");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "accept-deviation",
            root.to_str().unwrap(),
            "--fingerprint",
            &fingerprint,
            "--rationale",
            "Approved process aperture exception",
            "--accepted-by",
            "cli-test",
        ])
        .expect("CLI should parse"),
    )
    .expect("standards deviation should be authored");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["domain"], "standards");
    assert_standards_finding_status(&root, &fingerprint, "accepted_deviation", "deviation_refs");

    let schematic = read_schematic_root(&root);
    assert!(
        schematic["deviations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|deviation| {
                deviation["domain"] == "Standards"
                    && deviation["target"]["Fingerprint"] == fingerprint
            })
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

fn standards_finding_fingerprint(root: &Path, code: &str) -> String {
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
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["domain"] == "standards" && entry["code"] == code)
        .unwrap_or_else(|| panic!("standards finding {code} should exist"))["fingerprint"]
        .as_str()
        .unwrap()
        .to_string()
}

fn assert_standards_finding_status(root: &Path, fingerprint: &str, status: &str, refs_key: &str) {
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
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let finding = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["fingerprint"].as_str() == Some(fingerprint))
        .expect("fingerprint should remain visible");
    assert_eq!(finding["domain"], "standards");
    assert_eq!(finding["status"], status);
    assert!(!finding[refs_key].as_array().unwrap().is_empty());
}

fn read_schematic_root(root: &Path) -> serde_json::Value {
    serde_json::from_str(
        &std::fs::read_to_string(root.join("schematic/schematic.json"))
            .expect("schematic root should read"),
    )
    .expect("schematic root should parse")
}

fn repair_proposal_ids(report: &serde_json::Value) -> std::collections::BTreeSet<String> {
    report["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["proposal_id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn project_generate_standards_repair_proposals_links_dimension_rule_findings() {
    let root = unique_project_root("datum-eda-cli-project-check-run-dimension-repair");
    create_native_project(&root, Some("Dimension Repair Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_dimension_rule_fixture(&root);

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
    .expect("standards dimension repair proposals should generate");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("repair report JSON should parse");
    let track_proposal = report["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["repair_kind"] == "track_geometry")
        .expect("track geometry repair proposal should be reported");
    assert_eq!(track_proposal["operations"], 1);
    assert_eq!(track_proposal["prepared_against"], report["model_revision"]);
    assert_eq!(track_proposal["prepared_against_current_model"], true);
    assert_eq!(track_proposal["can_apply"], false);
    assert_eq!(
        track_proposal["blocker_codes"],
        serde_json::json!(["missing_acceptance"])
    );
    assert_eq!(
        track_proposal["affected_net_class"],
        dimension_net_class_uuid().to_string()
    );
    let codes = track_proposal["codes"].as_array().unwrap();
    assert!(codes.iter().any(|code| code == "track_width_below_min"));

    let via_proposal = report["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["repair_kind"] == "via_geometry")
        .expect("via geometry repair proposal should be reported");
    assert_eq!(via_proposal["operations"], 1);
    assert_eq!(via_proposal["prepared_against"], report["model_revision"]);
    assert_eq!(via_proposal["prepared_against_current_model"], true);
    assert_eq!(via_proposal["can_apply"], false);
    assert_eq!(
        via_proposal["blocker_codes"],
        serde_json::json!(["missing_acceptance"])
    );
    assert_eq!(
        via_proposal["affected_net_class"],
        dimension_net_class_uuid().to_string()
    );
    let codes = via_proposal["codes"].as_array().unwrap();
    assert!(codes.iter().any(|code| code == "via_hole_out_of_range"));
    assert!(codes.iter().any(|code| code == "via_annular_below_min"));

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with dimension repair proposals");
    let track_proposal_id =
        Uuid::parse_str(track_proposal["proposal_id"].as_str().unwrap()).unwrap();
    let proposal = model.proposals.get(&track_proposal_id).unwrap();
    assert_eq!(
        proposal.status,
        eda_engine::substrate::ProposalStatus::Draft
    );
    assert_eq!(
        proposal.source,
        eda_engine::substrate::ProposalSource::Check
    );
    assert_eq!(proposal.finding_fingerprints.len(), 1);
    let eda_engine::substrate::Operation::SetBoardTrack { track, .. } =
        &proposal.batch.operations[0]
    else {
        panic!("track standards repair should use SetBoardTrack");
    };
    assert_eq!(track["width"], 200000);

    let via_proposal_id = Uuid::parse_str(via_proposal["proposal_id"].as_str().unwrap()).unwrap();
    let proposal = model.proposals.get(&via_proposal_id).unwrap();
    assert_eq!(
        proposal.status,
        eda_engine::substrate::ProposalStatus::Draft
    );
    assert_eq!(
        proposal.source,
        eda_engine::substrate::ProposalSource::Check
    );
    assert_eq!(proposal.finding_fingerprints.len(), 2);
    let eda_engine::substrate::Operation::SetBoardVia { via, .. } = &proposal.batch.operations[0]
    else {
        panic!("via standards repair should use SetBoardVia");
    };
    assert_eq!(via["drill"], 300000);
    assert_eq!(via["diameter"], 600000);

    let linked_output = execute(
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
    .expect("linked check-run should succeed");
    let linked: serde_json::Value =
        serde_json::from_str(&linked_output).expect("linked check-run JSON should parse");
    assert_eq!(
        linked["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| {
                matches!(entry["code"].as_str(), Some("track_width_below_min"))
                    && entry["domain"] == "standards"
                    && entry["standards_basis"] == "datum.process_aperture_and_geometry.current"
                    && entry["proposal_refs"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|value| value == &track_proposal_id.to_string())
            })
            .count(),
        1
    );
    assert_eq!(
        linked["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| {
                matches!(
                    entry["code"].as_str(),
                    Some("via_hole_out_of_range") | Some("via_annular_below_min")
                ) && entry["domain"] == "standards"
                    && entry["standards_basis"] == "datum.process_aperture_and_geometry.current"
                    && entry["proposal_refs"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|value| value == &via_proposal_id.to_string())
            })
            .count(),
        2
    );

    let _ = std::fs::remove_dir_all(&root);
}

fn dimension_net_class_uuid() -> Uuid {
    Uuid::parse_str("77777777-7777-4777-8777-777777777777").unwrap()
}

pub(super) fn seed_board_dimension_rule_fixture(root: &Path) {
    seed_board_process_aperture_fixture(root);
    let net_class_uuid = dimension_net_class_uuid();
    let net_uuid = Uuid::parse_str("88888888-8888-4888-8888-888888888888").unwrap();
    let track_uuid = Uuid::parse_str("99999999-9999-4999-8999-999999999999").unwrap();
    let via_uuid = Uuid::parse_str("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa").unwrap();
    let board_json = root.join("board/board.json");
    let mut board: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&board_json).expect("board file should read"),
    )
    .expect("board JSON should parse");
    board["nets"] = serde_json::json!({
        net_uuid.to_string(): {
            "uuid": net_uuid,
            "name": "SIG",
            "class": net_class_uuid
        }
    });
    board["net_classes"] = serde_json::json!({
        net_class_uuid.to_string(): {
            "uuid": net_class_uuid,
            "name": "Default",
            "clearance": 150000,
            "track_width": 200000,
            "via_drill": 300000,
            "via_diameter": 600000,
            "diffpair_width": 0,
            "diffpair_gap": 0
        }
    });
    for pad in board["pads"].as_object_mut().unwrap().values_mut() {
        pad["net"] = serde_json::json!(net_uuid);
    }
    board["tracks"] = serde_json::json!({
        track_uuid.to_string(): {
            "uuid": track_uuid,
            "net": net_uuid,
            "from": { "x": 0, "y": 0 },
            "to": { "x": 1000000, "y": 0 },
            "width": 100000,
            "layer": 1
        }
    });
    board["vias"] = serde_json::json!({
        via_uuid.to_string(): {
            "uuid": via_uuid,
            "net": net_uuid,
            "position": { "x": 500000, "y": 500000 },
            "drill": 200000,
            "diameter": 300000,
            "from_layer": 1,
            "to_layer": 1
        }
    });
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            eda_engine::ir::serialization::to_json_deterministic(&board)
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");
}
