use super::*;

#[test]
fn journaled_board_package_position_moves_owned_pads_and_undoes() {
    let root = temp_project_root("journaled_package_position_move");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let mut board = read_json_value(&root.join("board/board.json")).expect("board should read");
    board["pads"] = serde_json::json!({
        pad_id.to_string(): {
            "uuid": pad_id,
            "package": package_id,
            "name": "1",
            "net": null,
            "position": { "x": -100, "y": 0 },
            "layer": 0,
            "shape": "circle",
            "diameter": 100,
            "width": 0,
            "height": 0
        }
    });
    write_json(&root.join("board/board.json"), board);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before = model.model_revision.clone();
    let move_report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"move-package-position"),
                expected_model_revision: Some(before),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "move package through substrate".to_string(),
                },
                operations: vec![Operation::SetBoardPackagePosition {
                    package_id,
                    x: 1000,
                    y: 2000,
                }],
            },
        )
        .expect("journaled move should succeed");

    assert_eq!(
        move_report.transaction.inverse_operations,
        vec![Operation::SetBoardPackagePosition {
            package_id,
            x: 0,
            y: 0,
        }]
    );
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(
        board["packages"][package_id.to_string()]["position"]["x"],
        1000
    );
    assert_eq!(
        board["packages"][package_id.to_string()]["position"]["y"],
        2000
    );
    assert_eq!(board["pads"][pad_id.to_string()]["position"]["x"], 900);
    assert_eq!(board["pads"][pad_id.to_string()]["position"]["y"], 2000);

    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo package move".to_string(),
            },
        )
        .expect("undo should succeed");
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(
        board["packages"][package_id.to_string()]["position"]["x"],
        0
    );
    assert_eq!(
        board["packages"][package_id.to_string()]["position"]["y"],
        0
    );
    assert_eq!(board["pads"][pad_id.to_string()]["position"]["x"], -100);
    assert_eq!(board["pads"][pad_id.to_string()]["position"]["y"], 0);
}

#[test]
fn journaled_board_dimension_create_set_delete_and_undoes() {
    let root = temp_project_root("journaled_board_dimension");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let dimension_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut board = read_json_value(&root.join("board/board.json")).expect("board should read");
    board["dimensions"] = serde_json::json!([]);
    write_json(&root.join("board/board.json"), board);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let original = serde_json::json!({
        "uuid": dimension_id,
        "from": { "x": 0, "y": 0 },
        "to": { "x": 1000, "y": 500 },
        "layer": 41,
        "text": "1000x500"
    });
    let revised = serde_json::json!({
        "uuid": dimension_id,
        "from": { "x": 10, "y": 20 },
        "to": { "x": 1010, "y": 520 },
        "layer": 42,
        "text": null
    });

    let create_report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"create-board-dimension"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create board dimension through substrate".to_string(),
                },
                operations: vec![Operation::CreateBoardDimension {
                    dimension_id,
                    dimension: original.clone(),
                }],
            },
        )
        .expect("journaled dimension create should succeed");
    assert_eq!(create_report.transaction.diff.created, vec![dimension_id]);
    assert_eq!(
        create_report.transaction.inverse_operations,
        vec![Operation::DeleteBoardDimension {
            dimension_id,
            dimension: original.clone(),
        }]
    );

    let set_report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-board-dimension"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "set board dimension through substrate".to_string(),
                },
                operations: vec![Operation::SetBoardDimension {
                    dimension_id,
                    dimension: revised.clone(),
                }],
            },
        )
        .expect("journaled dimension set should succeed");
    assert_eq!(set_report.transaction.diff.modified, vec![dimension_id]);
    assert_eq!(
        set_report.transaction.inverse_operations,
        vec![Operation::SetBoardDimension {
            dimension_id,
            dimension: original.clone(),
        }]
    );
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(board["dimensions"][0]["from"]["x"], 10);
    assert_eq!(board["dimensions"][0]["text"], serde_json::Value::Null);

    let delete_report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"delete-board-dimension"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete board dimension through substrate".to_string(),
                },
                operations: vec![Operation::DeleteBoardDimension {
                    dimension_id,
                    dimension: revised.clone(),
                }],
            },
        )
        .expect("journaled dimension delete should succeed");
    assert_eq!(delete_report.transaction.diff.deleted, vec![dimension_id]);
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert!(board["dimensions"].as_array().unwrap().is_empty());

    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo dimension delete".to_string(),
            },
        )
        .expect("undo should succeed");
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(board["dimensions"][0]["uuid"], dimension_id.to_string());
    assert_eq!(board["dimensions"][0]["from"]["x"], 10);
}

#[test]
fn journaled_board_text_create_set_delete_and_undoes() {
    let root = temp_project_root("journaled_board_text");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let text_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut board = read_json_value(&root.join("board/board.json")).expect("board should read");
    board["texts"] = serde_json::json!([]);
    write_json(&root.join("board/board.json"), board);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let original = serde_json::json!({
        "uuid": text_id,
        "text": "PCB TOP",
        "position": { "x": 1000, "y": 2000 },
        "rotation": 90,
        "height_nm": 1000000,
        "stroke_width_nm": 100000,
        "layer": 1
    });
    let revised = serde_json::json!({
        "uuid": text_id,
        "text": "PCB BOT",
        "position": { "x": 3000, "y": 4000 },
        "rotation": 180,
        "height_nm": 1200000,
        "stroke_width_nm": 150000,
        "layer": 2
    });

    let create_report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"create-board-text"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create board text through substrate".to_string(),
                },
                operations: vec![Operation::CreateBoardText {
                    text_id,
                    text: original.clone(),
                }],
            },
        )
        .expect("journaled text create should succeed");
    assert_eq!(create_report.transaction.diff.created, vec![text_id]);

    let set_report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-board-text"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "set board text through substrate".to_string(),
                },
                operations: vec![Operation::SetBoardText {
                    text_id,
                    text: revised.clone(),
                }],
            },
        )
        .expect("journaled text set should succeed");
    assert_eq!(set_report.transaction.diff.modified, vec![text_id]);
    assert_eq!(
        set_report.transaction.inverse_operations,
        vec![Operation::SetBoardText {
            text_id,
            text: original.clone(),
        }]
    );
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(board["texts"][0]["text"], "PCB BOT");
    assert_eq!(board["texts"][0]["position"]["x"], 3000);

    let delete_report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"delete-board-text"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete board text through substrate".to_string(),
                },
                operations: vec![Operation::DeleteBoardText {
                    text_id,
                    text: revised.clone(),
                }],
            },
        )
        .expect("journaled text delete should succeed");
    assert_eq!(delete_report.transaction.diff.deleted, vec![text_id]);
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert!(board["texts"].as_array().unwrap().is_empty());

    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo text delete".to_string(),
            },
        )
        .expect("undo should succeed");
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(board["texts"][0]["uuid"], text_id.to_string());
    assert_eq!(board["texts"][0]["text"], "PCB BOT");
}

#[test]
fn journaled_board_keepout_create_set_delete_and_undoes() {
    let root = temp_project_root("journaled_board_keepout");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let keepout_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut board = read_json_value(&root.join("board/board.json")).expect("board should read");
    board["keepouts"] = serde_json::json!([]);
    write_json(&root.join("board/board.json"), board);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let original = serde_json::json!({
        "uuid": keepout_id,
        "polygon": {
            "vertices": [
                { "x": 0, "y": 0 },
                { "x": 1000, "y": 0 },
                { "x": 1000, "y": 500 },
                { "x": 0, "y": 500 }
            ],
            "closed": true
        },
        "layers": [1, 16],
        "kind": "copper"
    });
    let revised = serde_json::json!({
        "uuid": keepout_id,
        "polygon": {
            "vertices": [
                { "x": 10, "y": 10 },
                { "x": 1010, "y": 10 },
                { "x": 1010, "y": 510 },
                { "x": 10, "y": 510 }
            ],
            "closed": true
        },
        "layers": [2],
        "kind": "mixed"
    });

    let create_report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"create-board-keepout"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create board keepout through substrate".to_string(),
                },
                operations: vec![Operation::CreateBoardKeepout {
                    keepout_id,
                    keepout: original.clone(),
                }],
            },
        )
        .expect("journaled keepout create should succeed");
    assert_eq!(create_report.transaction.diff.created, vec![keepout_id]);

    let set_report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-board-keepout"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "set board keepout through substrate".to_string(),
                },
                operations: vec![Operation::SetBoardKeepout {
                    keepout_id,
                    keepout: revised.clone(),
                }],
            },
        )
        .expect("journaled keepout set should succeed");
    assert_eq!(set_report.transaction.diff.modified, vec![keepout_id]);
    assert_eq!(
        set_report.transaction.inverse_operations,
        vec![Operation::SetBoardKeepout {
            keepout_id,
            keepout: original.clone(),
        }]
    );
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(board["keepouts"][0]["kind"], "mixed");
    assert_eq!(board["keepouts"][0]["layers"][0], 2);

    let delete_report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"delete-board-keepout"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete board keepout through substrate".to_string(),
                },
                operations: vec![Operation::DeleteBoardKeepout {
                    keepout_id,
                    keepout: revised.clone(),
                }],
            },
        )
        .expect("journaled keepout delete should succeed");
    assert_eq!(delete_report.transaction.diff.deleted, vec![keepout_id]);
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert!(board["keepouts"].as_array().unwrap().is_empty());

    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo keepout delete".to_string(),
            },
        )
        .expect("undo should succeed");
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(board["keepouts"][0]["uuid"], keepout_id.to_string());
    assert_eq!(board["keepouts"][0]["kind"], "mixed");
}

#[test]
fn journaled_board_outline_set_and_undoes() {
    let root = temp_project_root("journaled_board_outline");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut board = read_json_value(&root.join("board/board.json")).expect("board should read");
    board["outline"] = serde_json::json!({
        "vertices": [],
        "closed": true
    });
    write_json(&root.join("board/board.json"), board);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before_outline = read_json_value(&root.join("board/board.json"))
        .expect("board should read")
        .get("outline")
        .cloned()
        .expect("board outline should exist");
    let outline = serde_json::json!({
        "vertices": [
            { "x": 0, "y": 0 },
            { "x": 2000, "y": 0 },
            { "x": 1500, "y": 1000 },
            { "x": 0, "y": 1000 }
        ],
        "closed": true
    });

    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-board-outline"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "set board outline through substrate".to_string(),
                },
                operations: vec![Operation::SetBoardOutline {
                    board_id,
                    outline: outline.clone(),
                }],
            },
        )
        .expect("journaled outline set should succeed");
    assert_eq!(report.transaction.diff.modified, vec![board_id]);
    assert_eq!(
        report.transaction.inverse_operations,
        vec![Operation::SetBoardOutline {
            board_id,
            outline: before_outline,
        }]
    );
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(board["outline"]["vertices"][1]["x"], 2000);
    assert_eq!(board["outline"]["vertices"][2]["y"], 1000);

    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo outline set".to_string(),
            },
        )
        .expect("undo should succeed");
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(board["outline"]["vertices"], serde_json::json!([]));
    assert_eq!(board["outline"]["closed"], true);
}

#[test]
fn journaled_board_stackup_set_and_undoes() {
    let root = temp_project_root("journaled_board_stackup");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut board = read_json_value(&root.join("board/board.json")).expect("board should read");
    board["stackup"] = serde_json::json!({
        "layers": [
            { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 }
        ]
    });
    write_json(&root.join("board/board.json"), board);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let before_stackup = read_json_value(&root.join("board/board.json"))
        .expect("board should read")
        .get("stackup")
        .cloned()
        .expect("board stackup should exist");
    let stackup = serde_json::json!({
        "layers": [
            { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
            { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 },
            { "id": 3, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
        ]
    });

    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-board-stackup"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "set board stackup through substrate".to_string(),
                },
                operations: vec![Operation::SetBoardStackup {
                    board_id,
                    stackup: stackup.clone(),
                }],
            },
        )
        .expect("journaled stackup set should succeed");
    assert_eq!(report.transaction.diff.modified, vec![board_id]);
    assert_eq!(
        report.transaction.inverse_operations,
        vec![Operation::SetBoardStackup {
            board_id,
            stackup: before_stackup,
        }]
    );
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(board["stackup"]["layers"].as_array().unwrap().len(), 3);
    assert_eq!(board["stackup"]["layers"][1]["layer_type"], "Dielectric");

    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo stackup set".to_string(),
            },
        )
        .expect("undo should succeed");
    let board = read_json_value(&root.join("board/board.json")).expect("board should read");
    assert_eq!(board["stackup"]["layers"].as_array().unwrap().len(), 1);
    assert_eq!(board["stackup"]["layers"][0]["name"], "Top Copper");
}
