use super::main_tests_project_forward_annotation_support::{
    find_action_id, native_symbol, query_forward_annotation_proposal, unique_project_root,
    write_board_packages, write_native_sheet_symbols,
};
use super::*;
use eda_engine::board::PlacedPackage;
use eda_engine::ir::geometry::Point;

fn setup_export_fixture(root: &Path, project_name: &str, board_name: &str) -> String {
    create_native_project(root, Some(project_name.to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let part_uuid = Uuid::new_v4();
    write_native_sheet_symbols(
        root,
        sheet_uuid,
        "Main",
        vec![native_symbol(
            symbol_uuid,
            "C1",
            "1u",
            "Device:C",
            Some(part_uuid),
            None,
        )],
    );

    write_board_packages(
        root,
        board_name,
        vec![PlacedPackage {
            uuid: Uuid::new_v4(),
            part: Uuid::new_v4(),
            package: Uuid::new_v4(),
            reference: "U1".into(),
            value: "MCU".into(),
            position: Point::new(0, 0),
            rotation: 0,
            layer: 1,
            locked: false,
        }],
    );

    let proposal = query_forward_annotation_proposal(root);
    let add_action_id = find_action_id(&proposal, "add_component", None);
    let defer_cli = Cli::try_parse_from([
        "eda",
        "project",
        "defer-forward-annotation-action",
        root.to_str().unwrap(),
        "--action-id",
        &add_action_id,
    ])
    .expect("CLI should parse");
    let _ = execute(defer_cli).expect("defer should succeed");
    add_action_id
}

#[test]
fn project_export_forward_annotation_proposal_writes_versioned_artifact_with_reviews() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-export");
    let add_action_id = setup_export_fixture(
        &root,
        "Forward Annotation Export Demo",
        "Forward Annotation Export Demo Board",
    );

    let artifact_path = root.join("forward-annotation-proposal.json");
    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-forward-annotation-proposal",
        root.to_str().unwrap(),
        "--out",
        artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let export_output = execute(export_cli).expect("export should succeed");
    let report: serde_json::Value = serde_json::from_str(&export_output).expect("export JSON");
    assert_eq!(report["action"], "export_forward_annotation_proposal");
    assert_eq!(
        report["kind"],
        "native_forward_annotation_proposal_artifact"
    );
    assert_eq!(report["version"], 1);
    assert_eq!(report["actions"], 2);
    assert_eq!(report["reviews"], 1);

    let artifact_text = std::fs::read_to_string(&artifact_path).expect("artifact should read");
    let artifact: serde_json::Value =
        serde_json::from_str(&artifact_text).expect("artifact should parse");
    assert_eq!(
        artifact["kind"],
        "native_forward_annotation_proposal_artifact"
    );
    assert_eq!(artifact["version"], 1);
    assert_eq!(artifact["project_name"], "Forward Annotation Export Demo");
    assert_eq!(artifact["actions"].as_array().unwrap().len(), 2);
    assert_eq!(artifact["reviews"].as_array().unwrap().len(), 1);
    assert_eq!(artifact["reviews"][0]["action_id"], add_action_id);
    assert_eq!(artifact["reviews"][0]["decision"], "deferred");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_inspect_forward_annotation_proposal_artifact_reports_counts() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-inspect-artifact");
    let _ = setup_export_fixture(
        &root,
        "Forward Annotation Inspect Demo",
        "Forward Annotation Inspect Demo Board",
    );

    let artifact_path = root.join("forward-annotation-proposal.json");
    let export_cli = Cli::try_parse_from([
        "eda",
        "project",
        "export-forward-annotation-proposal",
        root.to_str().unwrap(),
        "--out",
        artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let _ = execute(export_cli).expect("export should succeed");

    let inspect_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "inspect-forward-annotation-proposal-artifact",
        artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let inspect_output = execute(inspect_cli).expect("inspect should succeed");
    let inspection: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspection JSON");
    assert_eq!(
        inspection["kind"],
        "native_forward_annotation_proposal_artifact"
    );
    assert_eq!(inspection["source_version"], 1);
    assert_eq!(inspection["version"], 1);
    assert_eq!(inspection["migration_applied"], false);
    assert_eq!(
        inspection["project_name"],
        "Forward Annotation Inspect Demo"
    );
    assert_eq!(inspection["actions"], 2);
    assert_eq!(inspection["reviews"], 1);
    assert_eq!(inspection["add_component_actions"], 1);
    assert_eq!(inspection["remove_component_actions"], 1);
    assert_eq!(inspection["update_component_actions"], 0);
    assert_eq!(inspection["deferred_reviews"], 1);
    assert_eq!(inspection["rejected_reviews"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
