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

pub mod pad_materialization;
mod verb_registry;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub(crate) use verb_registry::VERBS;

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
    if matches!(edit, BoardPackageEdit::Position { .. }) {
        let packages = load_board_packages(model)?;
        let package = packages.get(&package_id).ok_or(EngineError::NotFound {
            object_type: "board package",
            uuid: package_id,
        })?;
        if package.locked {
            return Err(EngineError::Operation(
                "cannot move a locked Footprint".into(),
            ));
        }
    }
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
#[path = "board_components/tests.rs"]
mod tests;
