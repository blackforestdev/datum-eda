use super::*;
use eda_engine::board::PlacedPackage;
use eda_engine::substrate::{ProjectResolver, ProposalStatus};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn execute(cli: Cli) -> Result<String> {
    let (output, _) = execute_with_exit_code(cli)?;
    Ok(output)
}

fn query_board_components(root: &Path) -> Vec<PlacedPackage> {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-components",
        ])
        .expect("CLI should parse"),
    )
    .expect("board component query should succeed");
    serde_json::from_str(&output).expect("board component query should parse")
}

fn place_board_component(
    root: &Path,
    reference: &str,
    value: &str,
    part: Uuid,
    package: Uuid,
    x_nm: &str,
) -> Uuid {
    let placed_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-component",
            root.to_str().unwrap(),
            "--part",
            &part.to_string(),
            "--package",
            &package.to_string(),
            "--reference",
            reference,
            "--value",
            value,
            "--x-nm",
            x_nm,
            "--y-nm",
            "2000",
            "--layer",
            "1",
        ])
        .expect("CLI should parse"),
    )
    .expect("place board component should succeed");
    let placed: serde_json::Value = serde_json::from_str(&placed_output).unwrap();
    Uuid::parse_str(placed["component_uuid"].as_str().unwrap()).unwrap()
}

#[test]
fn proposal_create_board_component_replacement_is_non_mutating_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-board-component-replacement-proposal");
    create_native_project(&root, Some("Replacement Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let original_part = Uuid::new_v4();
    let original_package = Uuid::new_v4();
    let replacement_part = Uuid::new_v4();
    let replacement_package = Uuid::new_v4();

    let placed_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-component",
            root.to_str().unwrap(),
            "--part",
            &original_part.to_string(),
            "--package",
            &original_package.to_string(),
            "--reference",
            "U1",
            "--value",
            "OLD",
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
            "--layer",
            "1",
        ])
        .expect("CLI should parse"),
    )
    .expect("place board component should succeed");
    let placed: serde_json::Value = serde_json::from_str(&placed_output).unwrap();
    let component_uuid = Uuid::parse_str(placed["component_uuid"].as_str().unwrap()).unwrap();
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"replace-board-component-u1");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-board-component-replacement",
            root.to_str().unwrap(),
            "--component",
            &component_uuid.to_string(),
            "--package",
            &replacement_package.to_string(),
            "--part",
            &replacement_part.to_string(),
            "--value",
            "NEW",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review explicit board component replacement",
        ])
        .expect("CLI should parse"),
    )
    .expect("replacement proposal should create");
    let report: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    assert_eq!(report["contract"], "proposal_create_v1");
    assert_eq!(report["action"], "propose_board_component_replacement");
    assert_eq!(report["proposal_id"], proposal_id.to_string());
    assert_eq!(report["proposal"]["status"], "draft");
    assert_eq!(
        report["proposal"]["batch"]["operations"]
            .as_array()
            .unwrap()
            .len(),
        4
    );

    let components_before = query_board_components(&root);
    assert_eq!(components_before[0].part, original_part);
    assert_eq!(components_before[0].package, original_package);
    assert_eq!(components_before[0].value, "OLD");

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
    .expect("replacement proposal accept-apply should succeed");

    let components_after = query_board_components(&root);
    assert_eq!(components_after[0].part, replacement_part);
    assert_eq!(components_after[0].package, replacement_package);
    assert_eq!(components_after[0].value, "NEW");
    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    assert_eq!(
        reopened.proposals.get(&proposal_id).unwrap().status,
        ProposalStatus::Applied
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_create_board_component_replacements_batches_multiple_components() {
    let root = unique_project_root("datum-eda-cli-board-component-replacements-proposal");
    create_native_project(&root, Some("Batch Replacement Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let original_part_u1 = Uuid::new_v4();
    let original_package_u1 = Uuid::new_v4();
    let replacement_part_u1 = Uuid::new_v4();
    let replacement_package_u1 = Uuid::new_v4();
    let original_part_u2 = Uuid::new_v4();
    let original_package_u2 = Uuid::new_v4();
    let replacement_part_u2 = Uuid::new_v4();

    let component_u1 = place_board_component(
        &root,
        "U1",
        "OLD1",
        original_part_u1,
        original_package_u1,
        "1000",
    );
    let component_u2 = place_board_component(
        &root,
        "U2",
        "OLD2",
        original_part_u2,
        original_package_u2,
        "3000",
    );
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"replace-board-components-u1-u2");
    let replacement_u1 = serde_json::json!({
        "component": component_u1,
        "package": replacement_package_u1,
        "part": replacement_part_u1,
        "value": "NEW1",
    })
    .to_string();
    let replacement_u2 = serde_json::json!({
        "component": component_u2,
        "part": replacement_part_u2,
        "value": "NEW2",
    })
    .to_string();

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-board-component-replacements",
            root.to_str().unwrap(),
            "--replacement",
            &replacement_u1,
            "--replacement",
            &replacement_u2,
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review explicit batch board component replacement",
        ])
        .expect("CLI should parse"),
    )
    .expect("batch replacement proposal should create");
    let report: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    assert_eq!(report["contract"], "proposal_create_v1");
    assert_eq!(report["action"], "propose_board_component_replacement");
    assert_eq!(report["proposal_id"], proposal_id.to_string());
    assert_eq!(report["proposal"]["status"], "draft");
    assert!(
        report["proposal"]["batch"]["operations"]
            .as_array()
            .unwrap()
            .len()
            >= 5
    );

    let components_before = query_board_components(&root);
    let before_u1 = components_before
        .iter()
        .find(|component| component.uuid == component_u1)
        .unwrap();
    let before_u2 = components_before
        .iter()
        .find(|component| component.uuid == component_u2)
        .unwrap();
    assert_eq!(before_u1.part, original_part_u1);
    assert_eq!(before_u1.package, original_package_u1);
    assert_eq!(before_u1.value, "OLD1");
    assert_eq!(before_u2.part, original_part_u2);
    assert_eq!(before_u2.package, original_package_u2);
    assert_eq!(before_u2.value, "OLD2");

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
    .expect("batch replacement proposal accept-apply should succeed");

    let components_after = query_board_components(&root);
    let after_u1 = components_after
        .iter()
        .find(|component| component.uuid == component_u1)
        .unwrap();
    let after_u2 = components_after
        .iter()
        .find(|component| component.uuid == component_u2)
        .unwrap();
    assert_eq!(after_u1.part, replacement_part_u1);
    assert_eq!(after_u1.package, replacement_package_u1);
    assert_eq!(after_u1.value, "NEW1");
    assert_eq!(after_u2.part, replacement_part_u2);
    assert_eq!(after_u2.package, original_package_u2);
    assert_eq!(after_u2.value, "NEW2");
    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    assert_eq!(
        reopened.proposals.get(&proposal_id).unwrap().status,
        ProposalStatus::Applied
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_create_board_component_replacement_plan_accepts_legacy_plan_shape() {
    let root = unique_project_root("datum-eda-cli-board-component-replacement-plan-proposal");
    create_native_project(&root, Some("Plan Replacement Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let original_part = Uuid::new_v4();
    let original_package = Uuid::new_v4();
    let replacement_part = Uuid::new_v4();
    let replacement_package = Uuid::new_v4();
    let component_uuid =
        place_board_component(&root, "U3", "OLD3", original_part, original_package, "5000");
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(
        &model.project.project_id,
        b"replace-board-component-plan-u3",
    );
    let selection = serde_json::json!({
        "uuid": component_uuid,
        "package_uuid": replacement_package,
        "part_uuid": replacement_part,
        "value": "NEW3",
    })
    .to_string();

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-board-component-replacement-plan",
            root.to_str().unwrap(),
            "--selection",
            &selection,
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review replacement plan selection",
        ])
        .expect("CLI should parse"),
    )
    .expect("replacement plan proposal should create");
    let report: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    assert_eq!(report["contract"], "proposal_create_v1");
    assert_eq!(report["action"], "propose_board_component_replacement");
    assert_eq!(report["proposal_id"], proposal_id.to_string());
    assert_eq!(report["proposal"]["status"], "draft");

    let components_before = query_board_components(&root);
    let before = components_before
        .iter()
        .find(|component| component.uuid == component_uuid)
        .unwrap();
    assert_eq!(before.part, original_part);
    assert_eq!(before.package, original_package);
    assert_eq!(before.value, "OLD3");

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
    .expect("replacement plan proposal accept-apply should succeed");

    let components_after = query_board_components(&root);
    let after = components_after
        .iter()
        .find(|component| component.uuid == component_uuid)
        .unwrap();
    assert_eq!(after.part, replacement_part);
    assert_eq!(after.package, replacement_package);
    assert_eq!(after.value, "NEW3");
    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    assert_eq!(
        reopened.proposals.get(&proposal_id).unwrap().status,
        ProposalStatus::Applied
    );
    let _ = std::fs::remove_dir_all(&root);
}
