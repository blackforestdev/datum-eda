use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::substrate::{ProjectResolver, ProposalStatus};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn execute(cli: Cli) -> Result<String> {
    let (output, _) = execute_with_exit_code(cli)?;
    Ok(output)
}

fn seed_native_sheet(root: &Path) -> Uuid {
    let sheet_uuid = Uuid::new_v4();
    let sheet_path = root
        .join("schematic/sheets")
        .join(format!("{sheet_uuid}.json"));
    std::fs::write(
        &sheet_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": sheet_uuid,
                "name": "Main",
                "frame": null,
                "symbols": {},
                "wires": {},
                "junctions": {},
                "labels": {},
                "buses": {},
                "bus_entries": {},
                "ports": {},
                "noconnects": {},
                "texts": {},
                "drawings": {}
            }))
            .expect("sheet JSON should serialize")
        ),
    )
    .expect("sheet file should write");

    let schematic_json = root.join("schematic/schematic.json");
    let mut schematic_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schematic_json).expect("schematic.json should read"),
    )
    .expect("schematic.json should parse");
    schematic_value["sheets"] = serde_json::json!({
        sheet_uuid.to_string(): format!("sheets/{sheet_uuid}.json")
    });
    std::fs::write(
        &schematic_json,
        format!(
            "{}\n",
            to_json_deterministic(&schematic_value)
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("schematic.json should write");

    sheet_uuid
}

#[test]
fn proposal_create_draw_wire_is_non_mutating_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-draw-wire");
    create_native_project(&root, Some("Wire Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"draw-wire-proposal");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-draw-wire",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--from-x-nm",
            "10",
            "--from-y-nm",
            "20",
            "--to-x-nm",
            "30",
            "--to-y-nm",
            "40",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review schematic connectivity edit",
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal create-draw-wire should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["contract"], "proposal_create_v1");
    assert_eq!(report["action"], "propose_draw_wire");
    assert_eq!(report["proposal_id"], proposal_id.to_string());
    assert_eq!(report["proposal"]["status"], "draft");
    assert_eq!(
        report["proposal"]["batch"]["operations"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(report["proposal"]["batch"]["expected_model_revision"].is_string());

    let wires_before = query_wire_count(&root);
    assert_eq!(wires_before, 0);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal accept-apply should succeed");

    assert_eq!(query_wire_count(&root), 1);
    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    assert_eq!(
        reopened.proposals.get(&proposal_id).unwrap().status,
        ProposalStatus::Applied
    );
}

#[test]
fn proposal_create_draw_wire_rejects_stale_accept_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-draw-wire-stale");
    create_native_project(&root, Some("Stale Wire Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"draw-wire-stale-proposal");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-draw-wire",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--from-x-nm",
            "10",
            "--from-y-nm",
            "20",
            "--to-x-nm",
            "30",
            "--to-y-nm",
            "40",
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal create-draw-wire should succeed");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-project-name",
            root.to_str().unwrap(),
            "--name",
            "Changed Before Wire Proposal Apply",
        ])
        .expect("CLI should parse"),
    )
    .expect("project name should change");

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("stale proposal accept-apply should fail");
    assert!(err.to_string().contains("cannot be accepted"));
    assert_eq!(query_wire_count(&root), 0);
}

#[test]
fn proposal_create_label_is_non_mutating_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-label");
    create_native_project(&root, Some("Label Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"label-proposal");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-place-label",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--name",
            "SENSE_IN",
            "--kind",
            "global",
            "--x-nm",
            "100",
            "--y-nm",
            "200",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review schematic net label",
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal create-place-label should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["contract"], "proposal_create_v1");
    assert_eq!(report["action"], "propose_place_label");
    assert_eq!(report["proposal_id"], proposal_id.to_string());
    assert_eq!(report["proposal"]["status"], "draft");
    assert_eq!(report["name"], "SENSE_IN");
    assert_eq!(report["kind"], "global");
    assert!(report["proposal"]["batch"]["expected_model_revision"].is_string());
    assert_eq!(query_label_count(&root), 0);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal accept-apply should succeed");

    assert_eq!(query_label_count(&root), 1);
    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    assert_eq!(
        reopened.proposals.get(&proposal_id).unwrap().status,
        ProposalStatus::Applied
    );
}

#[test]
fn proposal_create_label_rejects_stale_accept_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-label-stale");
    create_native_project(&root, Some("Stale Label Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"label-stale-proposal");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-place-label",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--name",
            "SENSE_OUT",
            "--x-nm",
            "300",
            "--y-nm",
            "400",
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal create-place-label should succeed");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-project-name",
            root.to_str().unwrap(),
            "--name",
            "Changed Before Label Proposal Apply",
        ])
        .expect("CLI should parse"),
    )
    .expect("project name should change");

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("stale proposal accept-apply should fail");
    assert!(err.to_string().contains("cannot be accepted"));
    assert_eq!(query_label_count(&root), 0);
}

#[test]
fn proposal_create_place_symbol_is_non_mutating_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-symbol");
    create_native_project(&root, Some("Symbol Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"symbol-proposal");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-place-symbol",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--reference",
            "U1",
            "--value",
            "LM358",
            "--lib-id",
            "device:opamp",
            "--x-nm",
            "500",
            "--y-nm",
            "600",
            "--rotation-deg",
            "90",
            "--mirrored",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review schematic symbol placement",
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal create-place-symbol should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["contract"], "proposal_create_v1");
    assert_eq!(report["action"], "propose_place_symbol");
    assert_eq!(report["proposal_id"], proposal_id.to_string());
    assert_eq!(report["proposal"]["status"], "draft");
    assert_eq!(report["reference"], "U1");
    assert_eq!(report["value"], "LM358");
    assert_eq!(report["lib_id"], "device:opamp");
    assert_eq!(report["rotation_deg"], 90);
    assert_eq!(report["mirrored"], true);
    assert!(report["proposal"]["batch"]["expected_model_revision"].is_string());
    assert_eq!(query_symbol_count(&root), 0);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal accept-apply should succeed");

    assert_eq!(query_symbol_count(&root), 1);
    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    assert_eq!(
        reopened.proposals.get(&proposal_id).unwrap().status,
        ProposalStatus::Applied
    );
}

#[test]
fn proposal_create_place_symbol_rejects_stale_accept_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-symbol-stale");
    create_native_project(&root, Some("Stale Symbol Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"symbol-stale-proposal");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-place-symbol",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--reference",
            "R1",
            "--value",
            "10k",
            "--x-nm",
            "700",
            "--y-nm",
            "800",
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal create-place-symbol should succeed");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-project-name",
            root.to_str().unwrap(),
            "--name",
            "Changed Before Symbol Proposal Apply",
        ])
        .expect("CLI should parse"),
    )
    .expect("project name should change");

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("stale proposal accept-apply should fail");
    assert!(err.to_string().contains("cannot be accepted"));
    assert_eq!(query_symbol_count(&root), 0);
}

fn query_wire_count(root: &Path) -> usize {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "wires",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query wires should succeed");
    let wires: serde_json::Value = serde_json::from_str(&output).unwrap();
    wires.as_array().unwrap().len()
}

fn query_label_count(root: &Path) -> usize {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "labels",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query labels should succeed");
    let labels: serde_json::Value = serde_json::from_str(&output).unwrap();
    labels.as_array().unwrap().len()
}

fn query_symbol_count(root: &Path) -> usize {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "symbols",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query symbols should succeed");
    let symbols: serde_json::Value = serde_json::from_str(&output).unwrap();
    symbols.as_array().unwrap().len()
}
