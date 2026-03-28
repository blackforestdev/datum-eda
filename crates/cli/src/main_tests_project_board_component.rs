use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn board_components_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-components",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_board_component_place_move_reassign_rotate_and_lock_round_trip_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-component");
    create_native_project(&root, Some("Board Component Demo".to_string()))
        .expect("initial scaffold should succeed");

    let part_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-component",
        root.to_str().unwrap(),
        "--part",
        &part_uuid.to_string(),
        "--package",
        &package_uuid.to_string(),
        "--reference",
        "U1",
        "--value",
        "MCU",
        "--x-nm",
        "1000",
        "--y-nm",
        "2000",
        "--layer",
        "1",
    ])
    .expect("CLI should parse");

    let placed_output = execute(place_cli).expect("place board component should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&placed_output).expect("place output should parse");
    let component_uuid = placed["component_uuid"].as_str().unwrap().to_string();
    assert_eq!(placed["has_persisted_component_silkscreen"], false);
    assert_eq!(placed["has_persisted_component_mechanical"], false);
    assert_eq!(placed["has_persisted_component_pads"], false);
    assert_eq!(placed["has_persisted_component_models_3d"], false);
    assert_eq!(placed["persisted_component_silkscreen_line_count"], 0);
    assert_eq!(placed["persisted_component_mechanical_line_count"], 0);
    assert_eq!(placed["persisted_component_pad_count"], 0);
    assert_eq!(placed["persisted_component_model_3d_count"], 0);

    let components_output =
        execute(board_components_query_cli(&root)).expect("board components query should succeed");
    let components: Vec<serde_json::Value> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0]["uuid"], component_uuid);
    assert_eq!(components[0]["part"], part_uuid.to_string());
    assert_eq!(components[0]["package"], package_uuid.to_string());
    assert_eq!(components[0]["reference"], "U1");
    assert_eq!(components[0]["value"], "MCU");
    assert_eq!(components[0]["position"]["x"], 1000);
    assert_eq!(components[0]["position"]["y"], 2000);
    assert_eq!(components[0]["rotation"], 0);
    assert_eq!(components[0]["layer"], 1);
    assert_eq!(components[0]["locked"], false);
    assert_eq!(components[0]["has_persisted_component_silkscreen"], false);
    assert_eq!(
        components[0]["persisted_component_silkscreen_text_count"],
        0
    );
    assert_eq!(
        components[0]["persisted_component_silkscreen_line_count"],
        0
    );
    assert_eq!(components[0]["persisted_component_silkscreen_arc_count"], 0);
    assert_eq!(
        components[0]["persisted_component_silkscreen_circle_count"],
        0
    );
    assert_eq!(
        components[0]["persisted_component_silkscreen_polygon_count"],
        0
    );
    assert_eq!(
        components[0]["persisted_component_silkscreen_polyline_count"],
        0
    );
    assert_eq!(components[0]["has_persisted_component_mechanical"], false);
    assert_eq!(
        components[0]["persisted_component_mechanical_text_count"],
        0
    );
    assert_eq!(
        components[0]["persisted_component_mechanical_line_count"],
        0
    );
    assert_eq!(components[0]["persisted_component_mechanical_arc_count"], 0);
    assert_eq!(
        components[0]["persisted_component_mechanical_circle_count"],
        0
    );
    assert_eq!(
        components[0]["persisted_component_mechanical_polygon_count"],
        0
    );
    assert_eq!(
        components[0]["persisted_component_mechanical_polyline_count"],
        0
    );
    assert_eq!(components[0]["has_persisted_component_pads"], false);
    assert_eq!(components[0]["persisted_component_pad_count"], 0);
    assert_eq!(components[0]["has_persisted_component_models_3d"], false);
    assert_eq!(components[0]["persisted_component_model_3d_count"], 0);

    let move_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "move-board-component",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
        "--x-nm",
        "3000",
        "--y-nm",
        "4000",
    ])
    .expect("CLI should parse");
    let move_output = execute(move_cli).expect("move board component should succeed");
    let move_report: serde_json::Value =
        serde_json::from_str(&move_output).expect("move output should parse");
    assert_eq!(move_report["has_persisted_component_silkscreen"], false);
    assert_eq!(move_report["has_persisted_component_mechanical"], false);
    assert_eq!(move_report["persisted_component_silkscreen_line_count"], 0);
    assert_eq!(move_report["persisted_component_mechanical_line_count"], 0);

    let components_output =
        execute(board_components_query_cli(&root)).expect("board components query should succeed");
    let components: Vec<serde_json::Value> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0]["position"]["x"], 3000);
    assert_eq!(components[0]["position"]["y"], 4000);
    assert_eq!(components[0]["rotation"], 0);
    assert_eq!(components[0]["layer"], 1);
    assert_eq!(components[0]["locked"], false);

    let replacement_part_uuid = Uuid::new_v4();
    let set_part_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-board-component-part",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
        "--part",
        &replacement_part_uuid.to_string(),
    ])
    .expect("CLI should parse");
    let set_part_output = execute(set_part_cli).expect("set board component part should succeed");
    let set_part_report: serde_json::Value =
        serde_json::from_str(&set_part_output).expect("set part output should parse");
    assert_eq!(set_part_report["has_persisted_component_silkscreen"], false);
    assert_eq!(set_part_report["has_persisted_component_mechanical"], false);
    assert_eq!(
        set_part_report["persisted_component_silkscreen_line_count"],
        0
    );
    assert_eq!(
        set_part_report["persisted_component_mechanical_line_count"],
        0
    );

    let replacement_package_uuid = Uuid::new_v4();
    let set_package_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-board-component-package",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
        "--package",
        &replacement_package_uuid.to_string(),
    ])
    .expect("CLI should parse");
    let set_package_output =
        execute(set_package_cli).expect("set board component package should succeed");
    let set_package_report: serde_json::Value =
        serde_json::from_str(&set_package_output).expect("set package output should parse");
    assert_eq!(
        set_package_report["has_persisted_component_silkscreen"],
        false
    );
    assert_eq!(
        set_package_report["has_persisted_component_mechanical"],
        false
    );
    assert_eq!(
        set_package_report["persisted_component_silkscreen_line_count"],
        0
    );
    assert_eq!(
        set_package_report["persisted_component_mechanical_line_count"],
        0
    );

    let components_output =
        execute(board_components_query_cli(&root)).expect("board components query should succeed");
    let components: Vec<serde_json::Value> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0]["part"], replacement_part_uuid.to_string());
    assert_eq!(
        components[0]["package"],
        replacement_package_uuid.to_string()
    );
    assert_eq!(components[0]["value"], "MCU");
    assert_eq!(components[0]["reference"], "U1");
    assert_eq!(components[0]["position"]["x"], 3000);
    assert_eq!(components[0]["position"]["y"], 4000);

    let set_reference_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-board-component-reference",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
        "--reference",
        "U42",
    ])
    .expect("CLI should parse");
    let set_reference_output =
        execute(set_reference_cli).expect("set board component reference should succeed");
    let set_reference_report: serde_json::Value =
        serde_json::from_str(&set_reference_output).expect("set reference output should parse");
    assert_eq!(set_reference_report["reference"], "U42");
    assert_eq!(
        set_reference_report["has_persisted_component_silkscreen"],
        false
    );
    assert_eq!(
        set_reference_report["has_persisted_component_mechanical"],
        false
    );
    assert_eq!(
        set_reference_report["persisted_component_silkscreen_line_count"],
        0
    );
    assert_eq!(
        set_reference_report["persisted_component_mechanical_line_count"],
        0
    );

    let set_value_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-board-component-value",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
        "--value",
        "MCU-REV2",
    ])
    .expect("CLI should parse");
    let set_value_output =
        execute(set_value_cli).expect("set board component value should succeed");
    let set_value_report: serde_json::Value =
        serde_json::from_str(&set_value_output).expect("set value output should parse");
    assert_eq!(set_value_report["value"], "MCU-REV2");
    assert_eq!(
        set_value_report["has_persisted_component_silkscreen"],
        false
    );
    assert_eq!(
        set_value_report["has_persisted_component_mechanical"],
        false
    );
    assert_eq!(
        set_value_report["persisted_component_silkscreen_line_count"],
        0
    );
    assert_eq!(
        set_value_report["persisted_component_mechanical_line_count"],
        0
    );

    let components_output =
        execute(board_components_query_cli(&root)).expect("board components query should succeed");
    let components: Vec<serde_json::Value> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0]["reference"], "U42");
    assert_eq!(components[0]["value"], "MCU-REV2");
    assert_eq!(components[0]["part"], replacement_part_uuid.to_string());
    assert_eq!(
        components[0]["package"],
        replacement_package_uuid.to_string()
    );
    assert_eq!(components[0]["layer"], 1);

    let set_layer_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-board-component-layer",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
        "--layer",
        "2",
    ])
    .expect("CLI should parse");
    let set_layer_output =
        execute(set_layer_cli).expect("set board component layer should succeed");
    let set_layer_report: serde_json::Value =
        serde_json::from_str(&set_layer_output).expect("set layer output should parse");
    assert_eq!(set_layer_report["layer"], 2);
    assert_eq!(
        set_layer_report["has_persisted_component_silkscreen"],
        false
    );
    assert_eq!(
        set_layer_report["has_persisted_component_mechanical"],
        false
    );
    assert_eq!(
        set_layer_report["persisted_component_silkscreen_line_count"],
        0
    );
    assert_eq!(
        set_layer_report["persisted_component_mechanical_line_count"],
        0
    );

    let rotate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "rotate-board-component",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
        "--rotation-deg",
        "180",
    ])
    .expect("CLI should parse");
    let rotate_output = execute(rotate_cli).expect("rotate board component should succeed");
    let rotate_report: serde_json::Value =
        serde_json::from_str(&rotate_output).expect("rotate output should parse");
    assert_eq!(rotate_report["has_persisted_component_silkscreen"], false);
    assert_eq!(rotate_report["has_persisted_component_mechanical"], false);
    assert_eq!(
        rotate_report["persisted_component_silkscreen_line_count"],
        0
    );
    assert_eq!(
        rotate_report["persisted_component_mechanical_line_count"],
        0
    );

    let components_output =
        execute(board_components_query_cli(&root)).expect("board components query should succeed");
    let components: Vec<serde_json::Value> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0]["layer"], 2);
    assert_eq!(components[0]["rotation"], 180);
    assert_eq!(components[0]["locked"], false);

    let lock_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-board-component-locked",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
    ])
    .expect("CLI should parse");
    let lock_output = execute(lock_cli).expect("lock board component should succeed");
    let lock_report: serde_json::Value =
        serde_json::from_str(&lock_output).expect("lock output should parse");
    assert_eq!(lock_report["has_persisted_component_silkscreen"], false);
    assert_eq!(lock_report["has_persisted_component_mechanical"], false);
    assert_eq!(lock_report["persisted_component_silkscreen_line_count"], 0);
    assert_eq!(lock_report["persisted_component_mechanical_line_count"], 0);

    let components_output =
        execute(board_components_query_cli(&root)).expect("board components query should succeed");
    let components: Vec<serde_json::Value> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0]["rotation"], 180);
    assert_eq!(components[0]["locked"], true);

    let unlock_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "clear-board-component-locked",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
    ])
    .expect("CLI should parse");
    let unlock_output = execute(unlock_cli).expect("unlock board component should succeed");
    let unlock_report: serde_json::Value =
        serde_json::from_str(&unlock_output).expect("unlock output should parse");
    assert_eq!(unlock_report["has_persisted_component_silkscreen"], false);
    assert_eq!(unlock_report["has_persisted_component_mechanical"], false);
    assert_eq!(
        unlock_report["persisted_component_silkscreen_line_count"],
        0
    );
    assert_eq!(
        unlock_report["persisted_component_mechanical_line_count"],
        0
    );

    let components_output =
        execute(board_components_query_cli(&root)).expect("board components query should succeed");
    let components: Vec<serde_json::Value> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0]["rotation"], 180);
    assert_eq!(components[0]["locked"], false);

    let delete_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "delete-board-component",
        root.to_str().unwrap(),
        "--component",
        &component_uuid,
    ])
    .expect("CLI should parse");
    let deleted_output = execute(delete_cli).expect("delete board component should succeed");
    let deleted: serde_json::Value =
        serde_json::from_str(&deleted_output).expect("delete output should parse");
    assert_eq!(deleted["action"].as_str(), Some("delete_board_component"));
    assert_eq!(
        deleted["component_uuid"].as_str(),
        Some(component_uuid.as_str())
    );
    assert_eq!(deleted["has_persisted_component_silkscreen"], false);
    assert_eq!(deleted["has_persisted_component_mechanical"], false);
    assert_eq!(deleted["persisted_component_silkscreen_line_count"], 0);
    assert_eq!(deleted["persisted_component_mechanical_line_count"], 0);

    let components_output =
        execute(board_components_query_cli(&root)).expect("board components query should succeed");
    let components: Vec<serde_json::Value> =
        serde_json::from_str(&components_output).expect("query output should parse");
    assert!(components.is_empty());

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_components: 0"));

    let _ = std::fs::remove_dir_all(&root);
}
