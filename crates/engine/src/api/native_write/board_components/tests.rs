use super::super::test_support::resolved_model_with_board_package;
use super::*;
use crate::ir::geometry::Point;
use crate::substrate::{CommitSource, ObjectRevision};

fn test_provenance() -> WriteProvenance {
    WriteProvenance::new(
        "unit-test",
        CommitSource::Test,
        "board components facade test",
    )
}

fn test_package(uuid: Uuid) -> PlacedPackage {
    PlacedPackage {
        uuid,
        part: Uuid::new_v4(),
        package: Uuid::new_v4(),
        reference: "U9".to_string(),
        value: "TEST".to_string(),
        position: Point { x: 1, y: 2 },
        rotation: 0,
        layer: 1,
        locked: false,
    }
}

#[test]
fn place_builds_single_unguarded_create_with_serde_payload() {
    let (_root, model, _board_id, _package_id) =
        resolved_model_with_board_package("board_components_place");
    let placement = BoardPackagePlacement {
        package: test_package(Uuid::new_v4()),
        materialized: serde_json::json!({ "component_pads": [] }),
    };

    let prepared = build_place_board_package(&model, test_provenance(), &placement)
        .expect("place should build");

    assert_eq!(prepared.primary_object_id, Some(placement.package.uuid));
    assert_eq!(
        prepared.batch.operations,
        vec![Operation::CreateBoardPackage {
            package_id: placement.package.uuid,
            package: serde_json::to_value(&placement.package).unwrap(),
            materialized: placement.materialized.clone(),
        }]
    );
}

#[test]
fn multi_place_composes_one_atomic_batch() {
    let (_root, model, _board_id, _package_id) =
        resolved_model_with_board_package("board_components_multi");
    let placements = vec![
        BoardPackagePlacement {
            package: test_package(Uuid::new_v4()),
            materialized: serde_json::json!({}),
        },
        BoardPackagePlacement {
            package: test_package(Uuid::new_v4()),
            materialized: serde_json::json!({}),
        },
    ];

    let prepared = build_place_board_packages(&model, test_provenance(), &placements)
        .expect("multi place should build");

    assert_eq!(prepared.batch.operations.len(), 2);
    assert!(
        prepared
            .batch
            .operations
            .iter()
            .all(|operation| matches!(operation, Operation::CreateBoardPackage { .. }))
    );
    assert_eq!(
        prepared.batch.expected_model_revision,
        Some(model.model_revision.clone())
    );
}

#[test]
fn edit_guards_package_and_maps_every_variant() {
    let (_root, model, _board_id, package_id) =
        resolved_model_with_board_package("board_components_edit");
    let part_id = Uuid::new_v4();
    let package_ref_id = Uuid::new_v4();
    let cases: Vec<(BoardPackageEdit, Operation)> = vec![
        (
            BoardPackageEdit::Position { x: 5, y: -7 },
            Operation::SetBoardPackagePosition {
                package_id,
                x: 5,
                y: -7,
            },
        ),
        (
            BoardPackageEdit::Rotation { rotation: 90 },
            Operation::SetBoardPackageRotation {
                package_id,
                rotation: 90,
            },
        ),
        (
            BoardPackageEdit::Locked { locked: true },
            Operation::SetBoardPackageLocked {
                package_id,
                locked: true,
            },
        ),
        (
            BoardPackageEdit::Part { part_id },
            Operation::SetBoardPackagePart {
                package_id,
                part_id,
            },
        ),
        (
            BoardPackageEdit::Package {
                package_ref_id,
                previous_materialized: serde_json::json!({ "component_pads": [] }),
                materialized: serde_json::json!({}),
            },
            Operation::SetBoardPackagePackage {
                package_id,
                package_ref_id,
                previous_materialized: serde_json::json!({ "component_pads": [] }),
                materialized: serde_json::json!({}),
            },
        ),
        (
            BoardPackageEdit::Reference {
                reference: "U7".to_string(),
            },
            Operation::SetBoardPackageReference {
                package_id,
                reference: "U7".to_string(),
            },
        ),
        (
            BoardPackageEdit::Value {
                value: "NEW".to_string(),
            },
            Operation::SetBoardPackageValue {
                package_id,
                value: "NEW".to_string(),
            },
        ),
        (
            BoardPackageEdit::Side { layer: 31 },
            Operation::SetComponentSide {
                package_id,
                layer: 31,
            },
        ),
    ];

    for (edit, expected) in cases {
        let prepared = build_edit_board_package(&model, test_provenance(), package_id, edit)
            .expect("edit should build");
        assert_eq!(prepared.primary_object_id, Some(package_id));
        assert_eq!(
            prepared.batch.operations,
            vec![
                Operation::GuardObjectRevision {
                    object_id: package_id,
                    expected_object_revision: ObjectRevision(0),
                },
                expected,
            ]
        );
    }
}

#[test]
fn edit_rejects_unknown_package() {
    let (_root, model, _board_id, _package_id) =
        resolved_model_with_board_package("board_components_edit_unknown");
    let missing = Uuid::new_v4();
    let error = build_edit_board_package(
        &model,
        test_provenance(),
        missing,
        BoardPackageEdit::Value {
            value: "X".to_string(),
        },
    )
    .expect_err("unknown package should fail");
    assert!(matches!(
        error,
        EngineError::NotFound {
            object_type: "domain_object",
            uuid,
        } if uuid == missing
    ));
}

#[test]
fn first_edit_move_refuses_locked_and_coordinate_overflow_without_writes() {
    let (root, mut model, _, package_id) =
        resolved_model_with_board_package("first_edit_move_safety");
    let path = root.join("board/board.json");
    let preimage = std::fs::read(&path).unwrap();
    let prepared = build_edit_board_package(
        &model,
        test_provenance(),
        package_id,
        BoardPackageEdit::Position { x: i64::MIN, y: 0 },
    )
    .unwrap();
    // The existing fixture's origin is zero: establish a positive pose through
    // the canonical route so MIN cannot be translated without overflow.
    let positive = build_edit_board_package(
        &model,
        test_provenance(),
        package_id,
        BoardPackageEdit::Position { x: 1, y: 0 },
    )
    .unwrap();
    super::super::commit_prepared(&mut model, &root, positive).unwrap();
    let positive_bytes = std::fs::read(&path).unwrap();
    let fresh = build_edit_board_package(
        &model,
        test_provenance(),
        package_id,
        BoardPackageEdit::Position { x: i64::MIN, y: 0 },
    )
    .unwrap();
    assert!(super::super::commit_prepared(&mut model, &root, fresh).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), positive_bytes);
    assert!(
        super::super::commit_prepared(&mut model, &root, prepared).is_err(),
        "stale prepared move refused"
    );
    let lock = build_edit_board_package(
        &model,
        test_provenance(),
        package_id,
        BoardPackageEdit::Locked { locked: true },
    )
    .unwrap();
    super::super::commit_prepared(&mut model, &root, lock).unwrap();
    assert!(
        build_edit_board_package(
            &model,
            test_provenance(),
            package_id,
            BoardPackageEdit::Position { x: 5, y: 7 }
        )
        .is_err()
    );
    assert_ne!(std::fs::read(&path).unwrap(), preimage);
}

#[test]
fn delete_threads_stored_payloads_raw_and_guards() {
    let (_root, model, _board_id, package_id) =
        resolved_model_with_board_package("board_components_delete");
    let stored = serde_json::json!({ "uuid": package_id, "reference": "U1" });
    let materialized = serde_json::json!({ "component_silkscreen": [] });

    let prepared = build_delete_board_package(
        &model,
        test_provenance(),
        package_id,
        stored.clone(),
        materialized.clone(),
    )
    .expect("delete should build");

    assert_eq!(prepared.primary_object_id, Some(package_id));
    assert_eq!(
        prepared.batch.operations,
        vec![
            Operation::GuardObjectRevision {
                object_id: package_id,
                expected_object_revision: ObjectRevision(0),
            },
            Operation::DeleteBoardPackage {
                package_id,
                package: stored,
                materialized,
            },
        ]
    );
}

#[test]
fn handoff_package_id_matches_historical_cli_derivation() {
    let project_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    // Byte-exact historical CLI derivation
    // (command_project_board_handoff.rs, pre-migration).
    let expected = Uuid::new_v5(
        &project_id,
        format!("datum-eda:board-package-from-symbol:{symbol_id}").as_bytes(),
    );
    assert_eq!(
        derive_board_package_from_symbol_id(&project_id, symbol_id),
        expected
    );
}

#[test]
fn align_left_builds_one_guarded_batch_and_skips_locked() {
    let (root, _model, _board_id, first_id) =
        resolved_model_with_board_package("board_components_align_left");
    let second_id = Uuid::new_v4();
    let third_id = Uuid::new_v4();
    let mut board = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("board/board.json")).unwrap(),
    )
    .unwrap();
    board["packages"][second_id.to_string()] = serde_json::json!({
        "uuid": second_id,
        "part": Uuid::new_v4(),
        "package": Uuid::new_v4(),
        "reference": "U2",
        "value": "B",
        "position": { "x": 100, "y": 20 },
        "rotation": 0,
        "layer": 1,
        "locked": false
    });
    board["packages"][third_id.to_string()] = serde_json::json!({
        "uuid": third_id,
        "part": Uuid::new_v4(),
        "package": Uuid::new_v4(),
        "reference": "U3",
        "value": "C",
        "position": { "x": 250, "y": 30 },
        "rotation": 0,
        "layer": 1,
        "locked": true
    });
    std::fs::write(
        root.join("board/board.json"),
        serde_json::to_string_pretty(&board).unwrap(),
    )
    .unwrap();
    let model = crate::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();

    let (prepared, plan) = build_align_board_packages(
        &model,
        test_provenance(),
        &[first_id, second_id, third_id],
        BoardPackageAlignMode::Left,
    )
    .expect("align should build");

    assert_eq!(plan.skipped_locked, vec![third_id]);
    assert_eq!(plan.aligned, vec![second_id]);
    assert_eq!(plan.unchanged, vec![first_id]);
    assert_eq!(
        prepared.batch.operations,
        vec![
            Operation::GuardObjectRevision {
                object_id: second_id,
                expected_object_revision: ObjectRevision(0),
            },
            Operation::SetBoardPackagePosition {
                package_id: second_id,
                x: 0,
                y: 20,
            },
        ]
    );
}

#[test]
fn distribute_h_spaces_unlocked_components() {
    let (root, _model, _board_id, first_id) =
        resolved_model_with_board_package("board_components_distribute_h");
    let second_id = Uuid::new_v4();
    let third_id = Uuid::new_v4();
    let mut board = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("board/board.json")).unwrap(),
    )
    .unwrap();
    board["packages"][second_id.to_string()] = serde_json::json!({
        "uuid": second_id,
        "part": Uuid::new_v4(),
        "package": Uuid::new_v4(),
        "reference": "U2",
        "value": "B",
        "position": { "x": 90, "y": 20 },
        "rotation": 0,
        "layer": 1,
        "locked": false
    });
    board["packages"][third_id.to_string()] = serde_json::json!({
        "uuid": third_id,
        "part": Uuid::new_v4(),
        "package": Uuid::new_v4(),
        "reference": "U3",
        "value": "C",
        "position": { "x": 300, "y": 30 },
        "rotation": 0,
        "layer": 1,
        "locked": false
    });
    std::fs::write(
        root.join("board/board.json"),
        serde_json::to_string_pretty(&board).unwrap(),
    )
    .unwrap();
    let model = crate::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();

    let (prepared, plan) = build_align_board_packages(
        &model,
        test_provenance(),
        &[first_id, second_id, third_id],
        BoardPackageAlignMode::DistributeH,
    )
    .expect("distribution should build");

    assert_eq!(plan.aligned, vec![second_id]);
    assert_eq!(
        prepared.batch.operations,
        vec![
            Operation::GuardObjectRevision {
                object_id: second_id,
                expected_object_revision: ObjectRevision(0),
            },
            Operation::SetBoardPackagePosition {
                package_id: second_id,
                x: 150,
                y: 20,
            },
        ]
    );
}
