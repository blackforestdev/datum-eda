use super::main_tests_project_forward_annotation_support::{
    find_action_id, native_symbol, query_forward_annotation_proposal, unique_project_root,
    write_board_packages, write_native_sheet_symbols,
};
use super::*;
use eda_engine::board::PlacedPackage;
use eda_engine::ir::geometry::Point;

fn setup_selection_fixture(root: &Path, project_name: &str, board_name: &str) -> (String, String) {
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
    let remove_action_id = find_action_id(&proposal, "remove_component", None);

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

    (add_action_id, remove_action_id)
}

#[test]
fn project_export_forward_annotation_proposal_selection_writes_only_selected_actions_and_reviews() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-export-selection");
    let (add_action_id, remove_action_id) = setup_selection_fixture(
        &root,
        "Forward Annotation Export Selection Demo",
        "Forward Annotation Export Selection Demo Board",
    );

    let artifact_path = root.join("forward-annotation-proposal-selection.json");
    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-forward-annotation-proposal-selection",
        root.to_str().unwrap(),
        "--action-id",
        &add_action_id,
        "--out",
        artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let export_output = execute(export_cli).expect("selection export should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&export_output).expect("selection export JSON");
    assert_eq!(
        report["action"],
        "export_forward_annotation_proposal_selection"
    );
    assert_eq!(report["actions"], 1);
    assert_eq!(report["reviews"], 1);

    let artifact_text = std::fs::read_to_string(&artifact_path).expect("artifact should read");
    let artifact: serde_json::Value =
        serde_json::from_str(&artifact_text).expect("artifact should parse");
    assert_eq!(artifact["actions"].as_array().unwrap().len(), 1);
    assert_eq!(artifact["actions"][0]["action_id"], add_action_id);
    assert_ne!(artifact["actions"][0]["action_id"], remove_action_id);
    assert_eq!(artifact["reviews"].as_array().unwrap().len(), 1);
    assert_eq!(artifact["reviews"][0]["action_id"], add_action_id);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_select_forward_annotation_proposal_artifact_writes_only_selected_artifact_actions_and_reviews()
 {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-select-artifact");
    let (add_action_id, remove_action_id) = setup_selection_fixture(
        &root,
        "Forward Annotation Select Artifact Demo",
        "Forward Annotation Select Artifact Demo Board",
    );

    let source_artifact_path = root.join("forward-annotation-proposal.json");
    let export_cli = Cli::try_parse_from([
        "eda",
        "project",
        "export-forward-annotation-proposal",
        root.to_str().unwrap(),
        "--out",
        source_artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let _ = execute(export_cli).expect("export should succeed");

    let selected_artifact_path = root.join("forward-annotation-proposal-selected.json");
    let select_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "select-forward-annotation-proposal-artifact",
        "--artifact",
        source_artifact_path.to_str().unwrap(),
        "--action-id",
        &remove_action_id,
        "--out",
        selected_artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let select_output = execute(select_cli).expect("artifact selection should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&select_output).expect("artifact selection JSON");
    assert_eq!(
        report["action"],
        "select_forward_annotation_proposal_artifact"
    );
    assert_eq!(report["actions"], 1);
    assert_eq!(report["reviews"], 0);

    let artifact_text =
        std::fs::read_to_string(&selected_artifact_path).expect("selected artifact should read");
    let artifact: serde_json::Value =
        serde_json::from_str(&artifact_text).expect("selected artifact should parse");
    assert_eq!(artifact["actions"].as_array().unwrap().len(), 1);
    assert_eq!(artifact["actions"][0]["action_id"], remove_action_id);
    assert_ne!(artifact["actions"][0]["action_id"], add_action_id);
    assert_eq!(artifact["reviews"].as_array().unwrap().len(), 0);

    let _ = std::fs::remove_dir_all(&root);
}
