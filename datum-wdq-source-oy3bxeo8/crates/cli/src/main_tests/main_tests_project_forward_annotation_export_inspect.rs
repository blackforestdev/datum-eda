use super::main_tests_project_forward_annotation_support::{
    find_action_id, native_symbol, query_forward_annotation_proposal, unique_project_root,
    write_board_packages, write_component_instance_shard,
    write_component_instance_shard_with_part_ref, write_native_sheet_symbols,
    write_pool_part_object,
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
fn project_export_forward_annotation_proposal_uses_resolver_materialized_project_name() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-export-resolver");
    let _ = setup_export_fixture(
        &root,
        "Forward Annotation Stale Name",
        "Forward Annotation Stale Name Board",
    );
    let project_json = root.join("project.json");
    let stale_project = std::fs::read_to_string(&project_json).expect("project file should read");

    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "set-project-name",
            root.to_str().unwrap(),
            "--name",
            "Forward Annotation Resolved Name",
        ])
        .expect("CLI should parse"),
    )
    .expect("set project name should succeed");
    std::fs::write(&project_json, stale_project).expect("stale project file should restore");

    let artifact_path = root.join("forward-annotation-proposal-resolved.json");
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "export-forward-annotation-proposal",
            root.to_str().unwrap(),
            "--out",
            artifact_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should read resolver-materialized project name");

    let artifact: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_path).expect("artifact should read"),
    )
    .expect("artifact should parse");
    assert_eq!(artifact["project_name"], "Forward Annotation Resolved Name");

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

#[test]
fn project_forward_annotation_uses_component_instance_before_reference_match() {
    let root = unique_project_root("datum-eda-cli-forward-annotation-component-instance-first");
    create_native_project(
        &root,
        Some("Forward Annotation ComponentInstance".to_string()),
    )
    .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let symbol_r1 = Uuid::new_v4();
    let symbol_r2 = Uuid::new_v4();
    let part_r1 = Uuid::new_v4();
    let part_r2 = Uuid::new_v4();
    write_native_sheet_symbols(
        &root,
        sheet_uuid,
        "Main",
        vec![
            native_symbol(symbol_r1, "R1", "10k", "Device:R", Some(part_r1), None),
            native_symbol(symbol_r2, "R2", "1k", "Device:R", Some(part_r2), None),
        ],
    );

    let package_r1 = Uuid::new_v4();
    let package_r99 = Uuid::new_v4();
    write_board_packages(
        &root,
        "Forward Annotation ComponentInstance Board",
        vec![
            PlacedPackage {
                uuid: package_r1,
                part: part_r2,
                package: Uuid::new_v4(),
                reference: "R1".into(),
                value: "1k".into(),
                position: Point::new(0, 0),
                rotation: 0,
                layer: 1,
                locked: false,
            },
            PlacedPackage {
                uuid: package_r99,
                part: part_r1,
                package: Uuid::new_v4(),
                reference: "R99".into(),
                value: "10k".into(),
                position: Point::new(10, 0),
                rotation: 0,
                layer: 1,
                locked: false,
            },
        ],
    );
    write_component_instance_shard(&root, Uuid::new_v4(), symbol_r1, package_r99);
    write_component_instance_shard(&root, Uuid::new_v4(), symbol_r2, package_r1);

    let proposal = query_forward_annotation_proposal(&root);
    assert_eq!(proposal["total_actions"], 0);
    assert_eq!(proposal["update_component_actions"], 0);
    assert_eq!(proposal["add_component_actions"], 0);
    assert_eq!(proposal["remove_component_actions"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_forward_annotation_update_targets_component_instance_bound_package() {
    let root = unique_project_root("datum-eda-cli-forward-annotation-component-instance-target");
    create_native_project(
        &root,
        Some("Forward Annotation ComponentInstance Target".to_string()),
    )
    .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let part_uuid = Uuid::new_v4();
    write_native_sheet_symbols(
        &root,
        sheet_uuid,
        "Main",
        vec![native_symbol(
            symbol_uuid,
            "R1",
            "10k",
            "Device:R",
            Some(part_uuid),
            None,
        )],
    );

    let bound_package_uuid = Uuid::new_v4();
    write_board_packages(
        &root,
        "Forward Annotation ComponentInstance Target Board",
        vec![PlacedPackage {
            uuid: bound_package_uuid,
            part: part_uuid,
            package: Uuid::new_v4(),
            reference: "R99".into(),
            value: "22k".into(),
            position: Point::new(0, 0),
            rotation: 0,
            layer: 1,
            locked: false,
        }],
    );
    write_component_instance_shard(&root, Uuid::new_v4(), symbol_uuid, bound_package_uuid);

    let proposal = query_forward_annotation_proposal(&root);
    assert_eq!(proposal["update_component_actions"], 1);
    assert_eq!(proposal["add_component_actions"], 0);
    assert_eq!(proposal["remove_component_actions"], 0);
    let update = proposal["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "update_component")
        .expect("update action should exist");
    assert_eq!(update["reason"], "value_mismatch");
    assert_eq!(update["symbol_uuid"], symbol_uuid.to_string());
    assert_eq!(update["component_uuid"], bound_package_uuid.to_string());
    assert_eq!(update["reference"], "R1");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_forward_annotation_component_instance_part_ref_overrides_symbol_part() {
    let root = unique_project_root("datum-eda-cli-forward-annotation-component-instance-part-ref");
    create_native_project(
        &root,
        Some("Forward Annotation ComponentInstance Part".to_string()),
    )
    .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let stale_symbol_part = Uuid::new_v4();
    let component_instance_part = Uuid::new_v4();
    write_pool_part_object(&root, component_instance_part);
    write_native_sheet_symbols(
        &root,
        sheet_uuid,
        "Main",
        vec![native_symbol(
            symbol_uuid,
            "R1",
            "10k",
            "Device:R",
            Some(stale_symbol_part),
            None,
        )],
    );

    let package_uuid = Uuid::new_v4();
    write_board_packages(
        &root,
        "Forward Annotation ComponentInstance Part Board",
        vec![PlacedPackage {
            uuid: package_uuid,
            part: component_instance_part,
            package: Uuid::new_v4(),
            reference: "R1".into(),
            value: "10k".into(),
            position: Point::new(0, 0),
            rotation: 0,
            layer: 1,
            locked: false,
        }],
    );
    write_component_instance_shard_with_part_ref(
        &root,
        Uuid::new_v4(),
        symbol_uuid,
        package_uuid,
        Some(component_instance_part),
    );

    let proposal = query_forward_annotation_proposal(&root);
    assert_eq!(proposal["total_actions"], 0);
    assert_eq!(proposal["update_component_actions"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
