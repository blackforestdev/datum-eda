use super::*;

#[test]
fn journaled_component_side_mirrors_owned_pads_and_undoes() {
    let root = temp_project_root("journaled_component_side");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let owned_pad_id = Uuid::new_v4();
    let foreign_pad_id = Uuid::new_v4();
    let component_pad_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut board = read_json_value(&root.join("board/board.json")).expect("board should read");
    board["packages"][package_id.to_string()]["position"] =
        serde_json::json!({ "x": 1000, "y": 2000 });
    board["packages"][package_id.to_string()]["layer"] = serde_json::json!(1);
    board["pads"] = serde_json::json!({
        owned_pad_id.to_string(): {
            "uuid": owned_pad_id,
            "package": package_id,
            "name": "1",
            "net": null,
            "position": { "x": 1250, "y": 2100 },
            "layer": 1,
            "copper_layers": [1],
            "shape": "circle",
            "diameter": 100,
            "width": 0,
            "height": 0,
            "rotation": 30,
            "mask_layers": [1],
            "paste_layers": [1]
        },
        foreign_pad_id.to_string(): {
            "uuid": foreign_pad_id,
            "package": Uuid::new_v4(),
            "name": "X",
            "net": null,
            "position": { "x": 1500, "y": 2100 },
            "layer": 1,
            "shape": "circle",
            "diameter": 100,
            "width": 0,
            "height": 0
        }
    });
    board["component_pads"] = serde_json::json!({
        package_id.to_string(): [{
            "uuid": component_pad_id,
            "package": package_id,
            "name": "2",
            "net": null,
            "position": { "x": 800, "y": 2300 },
            "layer": 1,
            "copper_layers": [1],
            "shape": "rect",
            "diameter": 0,
            "width": 200,
            "height": 100,
            "rotation": 90,
            "mask_layers": [1],
            "paste_layers": [1]
        }]
    });
    write_json(&root.join("board/board.json"), board);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-component-side"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "set component side through substrate".to_string(),
                },
                operations: vec![Operation::SetComponentSide {
                    package_id,
                    layer: 2,
                }],
            },
        )
        .expect("journaled side change should succeed");

    assert_eq!(
        report.transaction.inverse_operations,
        vec![Operation::SetComponentSide {
            package_id,
            layer: 1,
        }]
    );
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(board["packages"][package_id.to_string()]["layer"], 2);
    assert_eq!(
        board["pads"][owned_pad_id.to_string()]["position"]["x"],
        750
    );
    assert_eq!(
        board["pads"][owned_pad_id.to_string()]["position"]["y"],
        2100
    );
    assert_eq!(board["pads"][owned_pad_id.to_string()]["layer"], 2);
    assert_eq!(
        board["pads"][owned_pad_id.to_string()]["copper_layers"][0],
        2
    );
    assert_eq!(board["pads"][owned_pad_id.to_string()]["mask_layers"][0], 2);
    assert_eq!(
        board["pads"][owned_pad_id.to_string()]["paste_layers"][0],
        2
    );
    assert_eq!(board["pads"][owned_pad_id.to_string()]["rotation"], 150);
    assert_eq!(
        board["component_pads"][package_id.to_string()][0]["position"]["x"],
        1200
    );
    assert_eq!(
        board["component_pads"][package_id.to_string()][0]["layer"],
        2
    );
    assert_eq!(
        board["component_pads"][package_id.to_string()][0]["copper_layers"][0],
        2
    );
    assert_eq!(
        board["component_pads"][package_id.to_string()][0]["rotation"],
        90
    );
    assert_eq!(
        board["pads"][foreign_pad_id.to_string()]["position"]["x"],
        1500
    );
    assert_eq!(board["pads"][foreign_pad_id.to_string()]["layer"], 1);

    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo component side".to_string(),
            },
        )
        .expect("undo should succeed");
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(board["packages"][package_id.to_string()]["layer"], 1);
    assert_eq!(
        board["pads"][owned_pad_id.to_string()]["position"]["x"],
        1250
    );
    assert_eq!(board["pads"][owned_pad_id.to_string()]["layer"], 1);
    assert_eq!(
        board["pads"][owned_pad_id.to_string()]["copper_layers"][0],
        1
    );
    assert_eq!(board["pads"][owned_pad_id.to_string()]["rotation"], 30);
    assert_eq!(
        board["component_pads"][package_id.to_string()][0]["position"]["x"],
        800
    );
    assert_eq!(
        board["component_pads"][package_id.to_string()][0]["layer"],
        1
    );
    assert_eq!(
        board["component_pads"][package_id.to_string()][0]["copper_layers"][0],
        1
    );
    assert_eq!(
        board["pads"][foreign_pad_id.to_string()]["position"]["x"],
        1500
    );
}
