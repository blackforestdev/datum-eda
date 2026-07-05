use super::main_tests_project_check::{
    build_native_check_fixture, unique_project_root, write_native_waivers,
};
use super::*;
use eda_engine::schematic::{CheckDomain, WaiverTarget};
use eda_engine::substrate::{ProjectResolver, SourceShardKind};

#[test]
fn check_run_command_reports_native_check_run() {
    let root = unique_project_root("datum-eda-cli-check-run-command");
    create_native_project(&root, Some("Canonical Check Demo".to_string()))
        .expect("initial scaffold should succeed");
    build_native_check_fixture(&root);

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
            "--profile",
            "native-combined",
        ])
        .expect("CLI should parse"),
    )
    .expect("check run should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("check-run JSON should parse");

    assert_eq!(report["contract"], "check_run_v1");
    assert_eq!(report["persisted"], true);
    assert_eq!(report["profile_id"], "native-combined");
    assert_eq!(report["raw_report"]["domain"], "combined");
    assert_eq!(report["profile_basis"]["profile_id"], "native-combined");
    assert!(report["coverage"].as_array().unwrap().iter().any(|entry| {
        entry["domain"] == "erc"
            && entry["rule_id"] == "schematic_connectivity"
            && entry["status"] == "evaluated"
    }));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn check_waive_command_authors_fingerprint_waiver() {
    let root = unique_project_root("datum-eda-cli-check-waive-command");
    create_native_project(&root, Some("Canonical Waiver Demo".to_string()))
        .expect("initial scaffold should succeed");
    build_native_check_fixture(&root);

    let initial_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("check run should succeed");
    let initial: serde_json::Value =
        serde_json::from_str(&initial_output).expect("check-run JSON should parse");
    let fingerprint = initial["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["source"] == "erc" && entry["code"] == "unconnected_component_pin")
        .expect("target finding should exist")["fingerprint"]
        .as_str()
        .unwrap()
        .to_string();

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "waive",
            root.to_str().unwrap(),
            "--fingerprint",
            &fingerprint,
            "--rationale",
            "Intentional dangling fixture pin",
            "--created-by",
            "cli-test",
        ])
        .expect("CLI should parse"),
    )
    .expect("check waive should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("waive report JSON should parse");

    assert_eq!(report["contract"], "project_waive_finding_v1");
    assert_eq!(report["status"], "applied");
    assert_fingerprint_status(&root, &fingerprint, "waived");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_check_run_reports_resolver_keyed_persisted_findings() {
    let root = unique_project_root("datum-eda-cli-project-query-check-run");
    create_native_project(&root, Some("Check Run Demo".to_string()))
        .expect("initial scaffold should succeed");
    build_native_check_fixture(&root);

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
    .expect("project query check-run should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("check-run JSON should parse");

    assert_eq!(report["contract"], "check_run_v1");
    assert_eq!(report["persisted"], false);
    assert_eq!(report["profile_id"], "native-combined");
    assert_eq!(report["status"], "warning");
    assert!(report["check_run_id"].as_str().unwrap().len() > 20);
    assert!(report["model_revision"].as_str().unwrap().len() >= 64);
    assert_eq!(report["proposal_refs"], serde_json::json!([]));
    assert_eq!(
        report["finding_count"],
        report["findings"].as_array().unwrap().len()
    );
    assert!(report["findings"].as_array().unwrap().iter().any(|entry| {
        entry["source"] == "erc"
            && entry["code"] == "unconnected_component_pin"
            && entry["domain"] == "erc"
            && entry["rule_id"] == "unconnected_component_pin"
            && entry["status"] == "active"
            && entry["fingerprint"]
                .as_str()
                .unwrap()
                .starts_with("sha256:")
            && entry["primary_target"]["kind"].as_str().unwrap() != "unknown"
            && entry["primary_target"]["id"].as_str().is_some()
            && entry["related_targets"].as_array().is_some()
            && entry.get("affected_object_ids").is_none()
            && entry["message"].as_str().is_some()
            && entry["explanation"]
                .as_str()
                .unwrap()
                .contains("Rule unconnected_component_pin")
            && entry["suggested_next_action"]
                .as_str()
                .unwrap()
                .contains("fix, waive, or accept")
            && entry["evidence"].as_array().unwrap().len() == 2
            && entry["evidence"]
                .as_array()
                .unwrap()
                .iter()
                .any(|evidence| {
                    evidence["evidence_kind"] == "erc_pin_taxonomy"
                        && evidence["pin_taxonomy_revision"] == "LibraryPinElectricalType:v1"
                        && evidence["pins"][0]["canonical_pin_type"] == "passive"
                        && evidence["pins"][0]["lib_id"] == "Device:R"
                })
            && entry["waiver_refs"] == serde_json::json!([])
            && entry["deviation_refs"] == serde_json::json!([])
            && entry["proposal_refs"] == serde_json::json!([])
    }));

    let repeat_output = execute(
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
    .expect("project query check-run repeat should succeed");
    assert_eq!(output, repeat_output);

    let check_run_id = report["check_run_id"].as_str().unwrap();
    let persisted_path = root.join(format!(".datum/check_runs/{check_run_id}.json"));
    assert!(!persisted_path.exists());
    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("resolver should read project without persisted check run");
    assert!(
        !resolved
            .check_runs
            .contains_key(&Uuid::parse_str(check_run_id).unwrap())
    );
    assert!(
        !resolved
            .source_shards
            .iter()
            .any(|shard| shard.kind == SourceShardKind::CheckRun)
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_check_run_applies_fingerprint_waiver_state() {
    let root = unique_project_root("datum-eda-cli-project-query-check-run-fingerprint-waiver");
    create_native_project(&root, Some("Fingerprint Waiver Demo".to_string()))
        .expect("initial scaffold should succeed");
    build_native_check_fixture(&root);

    let initial_output = execute(
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
    .expect("initial check-run should succeed");
    let initial: serde_json::Value =
        serde_json::from_str(&initial_output).expect("check-run JSON should parse");
    let target = initial["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["source"] == "erc" && entry["code"] == "unconnected_component_pin")
        .expect("target finding should exist");
    let fingerprint = target["fingerprint"].as_str().unwrap().to_string();
    let target_severity = target["severity"].as_str().unwrap().to_string();
    let initial_errors = initial["summary"]["errors"].as_u64().unwrap();
    let initial_warnings = initial["summary"]["warnings"].as_u64().unwrap();
    let initial_infos = initial["summary"]["infos"].as_u64().unwrap();
    let waiver_id = Uuid::new_v4();
    write_native_waivers(
        &root,
        &[serde_json::to_value(serde_json::json!({
            "uuid": waiver_id,
            "domain": CheckDomain::ERC,
            "target": WaiverTarget::Fingerprint(fingerprint.clone()),
            "rationale": "Intentional fingerprint-scoped ERC waiver",
            "created_by": "cli-test"
        }))
        .expect("waiver should serialize")],
    );

    let waived_output = execute(
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
    .expect("waived check-run should succeed");
    let waived: serde_json::Value =
        serde_json::from_str(&waived_output).expect("check-run JSON should parse");
    let waived_finding = waived["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["fingerprint"].as_str() == Some(fingerprint.as_str()))
        .expect("waived finding should remain visible");
    assert_eq!(waived_finding["status"], "waived");
    assert_eq!(
        waived_finding["waiver_refs"],
        serde_json::json!([waiver_id.to_string()])
    );
    assert_eq!(waived["summary"]["waived"], 1);
    match target_severity.as_str() {
        "error" => assert_eq!(
            waived["summary"]["errors"].as_u64().unwrap(),
            initial_errors - 1
        ),
        "warning" => assert_eq!(
            waived["summary"]["warnings"].as_u64().unwrap(),
            initial_warnings - 1
        ),
        _ => assert_eq!(
            waived["summary"]["infos"].as_u64().unwrap(),
            initial_infos - 1
        ),
    }

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_waive_finding_commits_journaled_fingerprint_waiver() {
    let root = unique_project_root("datum-eda-cli-project-waive-finding");
    create_native_project(&root, Some("Journaled Waiver Demo".to_string()))
        .expect("initial scaffold should succeed");
    build_native_check_fixture(&root);

    let initial_output = execute(
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
    .expect("initial check-run should succeed");
    let initial: serde_json::Value =
        serde_json::from_str(&initial_output).expect("check-run JSON should parse");
    let fingerprint = initial["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["source"] == "erc" && entry["code"] == "unconnected_component_pin")
        .expect("target finding should exist")["fingerprint"]
        .as_str()
        .unwrap()
        .to_string();

    let waive_output = execute(
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
            "Intentional dangling fixture pin",
            "--created-by",
            "cli-test",
        ])
        .expect("CLI should parse"),
    )
    .expect("waive-finding should succeed");
    let waive_report: serde_json::Value =
        serde_json::from_str(&waive_output).expect("waive report JSON should parse");
    assert_eq!(waive_report["contract"], "project_waive_finding_v1");
    assert_eq!(waive_report["status"], "applied");
    assert_eq!(waive_report["fingerprint"], fingerprint);

    assert_fingerprint_status(&root, &fingerprint, "waived");

    let journal_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "journal-list",
        ])
        .expect("CLI should parse"),
    )
    .expect("journal-list should succeed");
    let journal: serde_json::Value =
        serde_json::from_str(&journal_output).expect("journal JSON should parse");
    assert_eq!(journal["count"], 1);
    assert_eq!(journal["transactions"][0]["operations"], 1);
    assert_eq!(journal["can_undo"], true);

    execute(
        Cli::try_parse_from(["eda", "project", "undo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("undo should succeed");
    assert_fingerprint_status(&root, &fingerprint, "active");

    execute(
        Cli::try_parse_from(["eda", "project", "redo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("redo should succeed");
    assert_fingerprint_status(&root, &fingerprint, "waived");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_accept_deviation_commits_journaled_fingerprint_deviation() {
    let root = unique_project_root("datum-eda-cli-project-accept-deviation");
    create_native_project(&root, Some("Journaled Deviation Demo".to_string()))
        .expect("initial scaffold should succeed");
    build_native_check_fixture(&root);

    let initial_output = execute(
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
    .expect("initial check-run should succeed");
    let initial: serde_json::Value =
        serde_json::from_str(&initial_output).expect("check-run JSON should parse");
    let fingerprint = initial["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["source"] == "erc" && entry["code"] == "unconnected_component_pin")
        .expect("target finding should exist")["fingerprint"]
        .as_str()
        .unwrap()
        .to_string();

    let deviation_output = execute(
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
            "Accepted fixture-level ERC deviation",
            "--accepted-by",
            "cli-test",
        ])
        .expect("CLI should parse"),
    )
    .expect("accept-deviation should succeed");
    let deviation_report: serde_json::Value =
        serde_json::from_str(&deviation_output).expect("deviation report JSON should parse");
    assert_eq!(deviation_report["contract"], "project_accept_deviation_v1");
    assert_eq!(deviation_report["status"], "applied");
    assert_eq!(deviation_report["fingerprint"], fingerprint);

    assert_fingerprint_status(&root, &fingerprint, "accepted_deviation");
    assert_fingerprint_ref_count(&root, &fingerprint, "deviation_refs", 1);

    execute(
        Cli::try_parse_from(["eda", "project", "undo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("undo should succeed");
    assert_fingerprint_status(&root, &fingerprint, "active");
    assert_fingerprint_ref_count(&root, &fingerprint, "deviation_refs", 0);

    execute(
        Cli::try_parse_from(["eda", "project", "redo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("redo should succeed");
    assert_fingerprint_status(&root, &fingerprint, "accepted_deviation");
    assert_fingerprint_ref_count(&root, &fingerprint, "deviation_refs", 1);

    let _ = std::fs::remove_dir_all(&root);
}

pub(super) fn assert_fingerprint_status(
    root: &std::path::Path,
    fingerprint: &str,
    expected_status: &str,
) {
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
    let report: serde_json::Value = serde_json::from_str(&output).expect("JSON should parse");
    let finding = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["fingerprint"].as_str() == Some(fingerprint))
        .expect("finding should remain visible");
    assert_eq!(finding["status"], expected_status);
}

pub(super) fn assert_fingerprint_ref_count(
    root: &std::path::Path,
    fingerprint: &str,
    ref_field: &str,
    expected_count: usize,
) {
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
    let report: serde_json::Value = serde_json::from_str(&output).expect("JSON should parse");
    let finding = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["fingerprint"].as_str() == Some(fingerprint))
        .expect("finding should remain visible");
    assert_eq!(finding[ref_field].as_array().unwrap().len(), expected_count);
}

#[test]
fn project_query_check_run_uses_journal_materialized_schematic_state() {
    let root = unique_project_root("datum-eda-cli-project-check-run-materialized-schematic");
    create_native_project(
        &root,
        Some("Check Run Materialized Schematic Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let (passive_pin_uuid, _) = build_native_check_fixture(&root);
    let schematic_root: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("schematic/schematic.json"))
            .expect("schematic root should read"),
    )
    .expect("schematic root should parse");
    let (sheet_uuid, sheet_relative_path) = schematic_root["sheets"]
        .as_object()
        .expect("schematic sheets should be an object")
        .iter()
        .next()
        .expect("fixture should contain one sheet");

    let sheet_path = root
        .join("schematic")
        .join(sheet_relative_path.as_str().expect("sheet path string"));
    let sheet: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&sheet_path).expect("fixture sheet should read"),
    )
    .expect("fixture sheet should parse");
    let passive_pin_string = passive_pin_uuid.to_string();
    let passive_symbol = sheet["symbols"]
        .as_object()
        .expect("symbols should be an object")
        .values()
        .find(|symbol| {
            symbol["pins"]
                .as_array()
                .unwrap()
                .iter()
                .any(|pin| pin["uuid"] == passive_pin_string)
        })
        .expect("fixture should contain passive pin symbol");
    let passive_symbol_uuid = passive_symbol["uuid"]
        .as_str()
        .expect("passive symbol uuid should serialize")
        .to_string();

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-noconnect",
            root.to_str().unwrap(),
            "--sheet",
            sheet_uuid,
            "--symbol",
            &passive_symbol_uuid,
            "--pin",
            &passive_pin_string,
            "--x-nm",
            "5",
            "--y-nm",
            "5",
        ])
        .expect("CLI should parse"),
    )
    .expect("journaled noconnect placement should succeed");

    let mut stale_sheet: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&sheet_path).expect("promoted sheet should read"),
    )
    .expect("promoted sheet should parse");
    stale_sheet["noconnects"] = serde_json::json!({});
    std::fs::write(
        &sheet_path,
        format!(
            "{}\n",
            eda_engine::ir::serialization::to_json_deterministic(&stale_sheet)
                .expect("stale sheet should serialize")
        ),
    )
    .expect("stale promoted sheet should write");

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
    .expect("project query check-run should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("check-run JSON should parse");
    assert!(!report["findings"].as_array().unwrap().iter().any(|entry| {
        entry["source"] == "erc"
            && entry["code"] == "unconnected_component_pin"
            && entry["payload"]["object_uuids"] == serde_json::json!([passive_pin_uuid.to_string()])
    }));
    let _ = std::fs::remove_dir_all(&root);
}
