//! Board component (placed package) builders for the native write facade.
//!
//! Family D of the native-write migration: all operation authoring for placed
//! board packages lives here. The CLI callers in
//! `crates/cli/src/command_project_board_component_mutations.rs`,
//! `crates/cli/src/command_project_board_component_value.rs`,
//! `crates/cli/src/command_project_board_component_reference.rs`,
//! `crates/cli/src/command_project_board_component_layer.rs`, and the
//! schematic→board handoff in
//! `crates/cli/src/command_project_board_handoff.rs` are thin
//! argument-parsers over this module: they assemble a typed
//! [`crate::board::PlacedPackage`] (plus its derived materialization
//! payload), call a `build_*` function, and commit the returned
//! [`PreparedWrite`] via [`super::commit_prepared`].
//!
//! Builders are build-only; they never touch disk. Payload shape (serde of
//! the board types), id derivation
//! (see [`derive_board_package_from_symbol_id`]), guard insertion, and
//! journal provenance are byte-for-byte the CLI's historical behavior.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::PlacedPackage;
use crate::error::EngineError;
use crate::substrate::{DesignModel, ObjectId, Operation, SourceShardKind};

use super::context::{BatchComposer, PreparedWrite, WriteProvenance};
use super::ids;

/// A placed package together with its derived pool-graphics materialization
/// payload (silkscreen/mechanical/pads/3D shards keyed off the package).
///
/// The materialization payload is derived by the caller (it depends on the
/// loaded native-project graphics maps); this facade threads it verbatim into
/// the `CreateBoardPackage` operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardPackagePlacement {
    pub package: PlacedPackage,
    pub materialized: serde_json::Value,
}

/// A single-field edit of an existing placed board package. Each variant maps
/// onto exactly one typed substrate operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardPackageEdit {
    Position {
        x: i64,
        y: i64,
    },
    Rotation {
        rotation: i32,
    },
    Locked {
        locked: bool,
    },
    Part {
        part_id: ObjectId,
    },
    /// Swap the pool package reference; the caller supplies the pre- and
    /// post-swap materialization payloads.
    Package {
        package_ref_id: ObjectId,
        previous_materialized: serde_json::Value,
        materialized: serde_json::Value,
    },
    Reference {
        reference: String,
    },
    Value {
        value: String,
    },
    /// Which board side (layer) the component sits on.
    Side {
        layer: i32,
    },
}

/// A single align/distribute mode for placed board packages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BoardPackageAlignMode {
    Left,
    Right,
    Top,
    Bottom,
    HCenter,
    VCenter,
    DistributeH,
    DistributeV,
}

/// Summary of the package ids affected or skipped by an align/distribute
/// batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardPackageAlignPlan {
    pub aligned: Vec<ObjectId>,
    pub skipped_locked: Vec<ObjectId>,
    pub unchanged: Vec<ObjectId>,
}

/// Derive the deterministic id for a board package generated from a schematic
/// symbol during handoff.
///
/// Matches the historical CLI derivation in
/// `crates/cli/src/command_project_board_handoff.rs` exactly: the v5 seed is
/// `datum-eda:board-package-from-symbol:<symbol id>` namespaced by the
/// project id.
pub fn derive_board_package_from_symbol_id(project_id: &Uuid, symbol_id: Uuid) -> ObjectId {
    ids::derive_object_id(
        project_id,
        "board-package-from-symbol",
        &[symbol_id.to_string()],
    )
}

/// Build the batch that places one new board package.
pub fn build_place_board_package(
    model: &DesignModel,
    provenance: WriteProvenance,
    placement: &BoardPackagePlacement,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(create_board_package_operation(placement)?)
        .primary_object(placement.package.uuid)
        .finish()
}

/// Build one atomic batch that places every package in `placements` (the
/// schematic→board handoff path).
pub fn build_place_board_packages(
    model: &DesignModel,
    provenance: WriteProvenance,
    placements: &[BoardPackagePlacement],
) -> Result<PreparedWrite, EngineError> {
    let operations = placements
        .iter()
        .map(create_board_package_operation)
        .collect::<Result<Vec<_>, _>>()?;
    BatchComposer::compose(model, provenance)
        .push_ops(operations)
        .finish()
}

/// Build the batch that applies one [`BoardPackageEdit`] to an existing
/// placed package (revision-guarded).
pub fn build_edit_board_package(
    model: &DesignModel,
    provenance: WriteProvenance,
    package_id: ObjectId,
    edit: BoardPackageEdit,
) -> Result<PreparedWrite, EngineError> {
    let operation = match edit {
        BoardPackageEdit::Position { x, y } => {
            Operation::SetBoardPackagePosition { package_id, x, y }
        }
        BoardPackageEdit::Rotation { rotation } => Operation::SetBoardPackageRotation {
            package_id,
            rotation,
        },
        BoardPackageEdit::Locked { locked } => {
            Operation::SetBoardPackageLocked { package_id, locked }
        }
        BoardPackageEdit::Part { part_id } => Operation::SetBoardPackagePart {
            package_id,
            part_id,
        },
        BoardPackageEdit::Package {
            package_ref_id,
            previous_materialized,
            materialized,
        } => Operation::SetBoardPackagePackage {
            package_id,
            package_ref_id,
            previous_materialized,
            materialized,
        },
        BoardPackageEdit::Reference { reference } => Operation::SetBoardPackageReference {
            package_id,
            reference,
        },
        BoardPackageEdit::Value { value } => Operation::SetBoardPackageValue { package_id, value },
        BoardPackageEdit::Side { layer } => Operation::SetComponentSide { package_id, layer },
    };
    BatchComposer::compose(model, provenance)
        .push_op(operation)
        .primary_object(package_id)
        .finish()
}

/// Build one atomic align/distribute batch for placed board packages.
///
/// Locked packages are skipped rather than moved. All movable package moves
/// are emitted as `SetBoardPackagePosition` operations in one guarded batch,
/// so the whole alignment has one journal entry and one undo step.
pub fn build_align_board_packages(
    model: &DesignModel,
    provenance: WriteProvenance,
    package_ids: &[ObjectId],
    mode: BoardPackageAlignMode,
) -> Result<(PreparedWrite, BoardPackageAlignPlan), EngineError> {
    let packages = load_board_packages(model)?;
    if package_ids.len() < 2 {
        return Err(EngineError::Validation(
            "align/distribute requires at least two component ids".to_string(),
        ));
    }

    let mut selected = Vec::new();
    for package_id in package_ids {
        let package = packages.get(package_id).ok_or(EngineError::NotFound {
            object_type: "board_package",
            uuid: *package_id,
        })?;
        selected.push(package.clone());
    }

    let movable = selected
        .iter()
        .filter(|package| !package.locked)
        .cloned()
        .collect::<Vec<_>>();
    let skipped_locked = selected
        .iter()
        .filter(|package| package.locked)
        .map(|package| package.uuid)
        .collect::<Vec<_>>();
    if movable.len() < 2 {
        return Err(EngineError::Validation(
            "align/distribute requires at least two unlocked components".to_string(),
        ));
    }

    let target_positions = align_target_positions(&movable, mode)?;
    let mut operations = Vec::new();
    let mut aligned = Vec::new();
    let mut unchanged = Vec::new();
    for package in &movable {
        let (x, y) = target_positions
            .iter()
            .find(|(id, _, _)| *id == package.uuid)
            .map(|(_, x, y)| (*x, *y))
            .expect("target exists for every movable package");
        if package.position.x == x && package.position.y == y {
            unchanged.push(package.uuid);
            continue;
        }
        aligned.push(package.uuid);
        operations.push(Operation::SetBoardPackagePosition {
            package_id: package.uuid,
            x,
            y,
        });
    }

    let prepared = BatchComposer::compose(model, provenance)
        .push_ops(operations)
        .finish()?;
    Ok((
        prepared,
        BoardPackageAlignPlan {
            aligned,
            skipped_locked,
            unchanged,
        },
    ))
}

/// Build the batch that deletes an existing placed package. `package` and
/// `materialized` are the stored payloads (threaded raw for byte-exact
/// journal parity with the pre-migration CLI).
pub fn build_delete_board_package(
    model: &DesignModel,
    provenance: WriteProvenance,
    package_id: ObjectId,
    package: serde_json::Value,
    materialized: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteBoardPackage {
            package_id,
            package,
            materialized,
        })
        .primary_object(package_id)
        .finish()
}

fn create_board_package_operation(
    placement: &BoardPackagePlacement,
) -> Result<Operation, EngineError> {
    Ok(Operation::CreateBoardPackage {
        package_id: placement.package.uuid,
        package: serde_json::to_value(&placement.package)?,
        materialized: placement.materialized.clone(),
    })
}

fn load_board_packages(
    model: &DesignModel,
) -> Result<std::collections::BTreeMap<ObjectId, PlacedPackage>, EngineError> {
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let packages = board
        .get("packages")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| EngineError::Validation("board shard missing packages map".to_string()))?;
    packages
        .values()
        .map(|value| {
            let package = serde_json::from_value::<PlacedPackage>(value.clone())?;
            Ok((package.uuid, package))
        })
        .collect()
}

fn align_target_positions(
    packages: &[PlacedPackage],
    mode: BoardPackageAlignMode,
) -> Result<Vec<(ObjectId, i64, i64)>, EngineError> {
    let min_x = packages
        .iter()
        .map(|package| package.position.x)
        .min()
        .expect("nonempty packages");
    let max_x = packages
        .iter()
        .map(|package| package.position.x)
        .max()
        .expect("nonempty packages");
    let min_y = packages
        .iter()
        .map(|package| package.position.y)
        .min()
        .expect("nonempty packages");
    let max_y = packages
        .iter()
        .map(|package| package.position.y)
        .max()
        .expect("nonempty packages");
    let h_center = min_x + (max_x - min_x) / 2;
    let v_center = min_y + (max_y - min_y) / 2;

    let mut sorted = packages.to_vec();
    match mode {
        BoardPackageAlignMode::DistributeH => {
            sorted.sort_by_key(|package| (package.position.x, package.uuid));
            distribute_targets(&sorted, true)
        }
        BoardPackageAlignMode::DistributeV => {
            sorted.sort_by_key(|package| (package.position.y, package.uuid));
            distribute_targets(&sorted, false)
        }
        _ => Ok(packages
            .iter()
            .map(|package| {
                let (x, y) = match mode {
                    BoardPackageAlignMode::Left => (min_x, package.position.y),
                    BoardPackageAlignMode::Right => (max_x, package.position.y),
                    BoardPackageAlignMode::Top => (package.position.x, min_y),
                    BoardPackageAlignMode::Bottom => (package.position.x, max_y),
                    BoardPackageAlignMode::HCenter => (h_center, package.position.y),
                    BoardPackageAlignMode::VCenter => (package.position.x, v_center),
                    BoardPackageAlignMode::DistributeH | BoardPackageAlignMode::DistributeV => {
                        unreachable!("distribution handled above")
                    }
                };
                (package.uuid, x, y)
            })
            .collect()),
    }
}

fn distribute_targets(
    sorted: &[PlacedPackage],
    horizontal: bool,
) -> Result<Vec<(ObjectId, i64, i64)>, EngineError> {
    if sorted.len() < 3 {
        return Err(EngineError::Validation(
            "distribute requires at least three unlocked components".to_string(),
        ));
    }
    let first = sorted.first().expect("nonempty packages");
    let last = sorted.last().expect("nonempty packages");
    let start = if horizontal {
        first.position.x
    } else {
        first.position.y
    };
    let end = if horizontal {
        last.position.x
    } else {
        last.position.y
    };
    let slots = (sorted.len() - 1) as i64;
    Ok(sorted
        .iter()
        .enumerate()
        .map(|(index, package)| {
            let coordinate = start + ((end - start) * index as i64) / slots;
            if horizontal {
                (package.uuid, coordinate, package.position.y)
            } else {
                (package.uuid, package.position.x, coordinate)
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
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
}
