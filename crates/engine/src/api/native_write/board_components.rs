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
use crate::substrate::{DesignModel, ObjectId, Operation};

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

#[cfg(test)]
mod tests {
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::ir::geometry::Point;
    use crate::substrate::{CommitSource, ObjectRevision};

    fn test_provenance() -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, "board components facade test")
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
        assert!(prepared.batch.operations.iter().all(|operation| matches!(
            operation,
            Operation::CreateBoardPackage { .. }
        )));
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
}
