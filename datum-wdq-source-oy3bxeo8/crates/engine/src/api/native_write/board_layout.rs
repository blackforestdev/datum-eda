//! Board outline/stackup/name builders for the native write facade.
//!
//! Family D of the native-write migration: all operation authoring for
//! board-level layout state lives here. The CLI callers in
//! `crates/cli/src/command_project_board_layout.rs` and
//! `crates/cli/src/command_project_default_stackup.rs` are thin
//! argument-parsers over this module: they parse/merge the typed inputs, call
//! a `build_*` function, and commit the returned [`PreparedWrite`] via
//! [`super::commit_prepared`].
//!
//! Builders are build-only; they never touch disk. Payload shapes are
//! byte-for-byte the CLI's historical behavior: the outline serializes as
//! `{ "vertices": [{"x","y"}...], "closed": bool }` and the stackup as
//! `{ "layers": [<serde StackupLayer>...] }`.

use crate::board::StackupLayer;
use crate::error::EngineError;
use crate::ir::geometry::Polygon;
use crate::substrate::{DesignModel, ObjectId, Operation};

use super::context::{BatchComposer, PreparedWrite, WriteProvenance};

/// Build the revision-guarded batch that replaces the board outline.
///
/// The payload is the serde serialization of [`Polygon`], byte-identical to
/// the CLI's historical `NativeOutline { vertices, closed }` shape.
pub fn build_set_board_outline(
    model: &DesignModel,
    provenance: WriteProvenance,
    board_id: ObjectId,
    outline: &Polygon,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetBoardOutline {
            board_id,
            outline: serde_json::to_value(outline)?,
        })
        .primary_object(board_id)
        .finish()
}

/// Build the revision-guarded batch that replaces the board stackup with
/// `layers` (payload `{ "layers": [...] }`).
pub fn build_set_board_stackup(
    model: &DesignModel,
    provenance: WriteProvenance,
    board_id: ObjectId,
    layers: &[StackupLayer],
) -> Result<PreparedWrite, EngineError> {
    let layers = layers
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetBoardStackup {
            board_id,
            stackup: serde_json::json!({ "layers": layers }),
        })
        .primary_object(board_id)
        .finish()
}

/// Build the revision-guarded batch that renames the board.
pub fn build_set_board_name(
    model: &DesignModel,
    provenance: WriteProvenance,
    board_id: ObjectId,
    name: String,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetBoardName { board_id, name })
        .primary_object(board_id)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::board::StackupLayerType;
    use crate::ir::geometry::Point;
    use crate::substrate::{CommitSource, ObjectRevision};

    fn test_provenance() -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, "board layout facade test")
    }

    #[test]
    fn outline_payload_matches_historical_native_outline_shape() {
        let (_root, model, board_id, _package_id) =
            resolved_model_with_board_package("board_layout_outline");
        let outline = Polygon {
            vertices: vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }],
            closed: true,
        };

        let prepared = build_set_board_outline(&model, test_provenance(), board_id, &outline)
            .expect("outline should build");

        assert_eq!(prepared.primary_object_id, Some(board_id));
        assert_eq!(
            prepared.batch.operations,
            vec![
                Operation::GuardObjectRevision {
                    object_id: board_id,
                    expected_object_revision: ObjectRevision(0),
                },
                Operation::SetBoardOutline {
                    board_id,
                    // Byte-identical to the CLI's historical NativeOutline
                    // serialization.
                    outline: serde_json::json!({
                        "vertices": [
                            { "x": 1, "y": 2 },
                            { "x": 3, "y": 4 },
                        ],
                        "closed": true,
                    }),
                },
            ]
        );
    }

    #[test]
    fn stackup_payload_wraps_serde_layers() {
        let (_root, model, board_id, _package_id) =
            resolved_model_with_board_package("board_layout_stackup");
        let layers = vec![
            StackupLayer::new(1, "Top Copper", StackupLayerType::Copper, 35_000),
            StackupLayer::new(2, "Top Mask", StackupLayerType::SolderMask, 10_000),
        ];

        let prepared = build_set_board_stackup(&model, test_provenance(), board_id, &layers)
            .expect("stackup should build");

        assert_eq!(
            prepared.batch.operations,
            vec![
                Operation::GuardObjectRevision {
                    object_id: board_id,
                    expected_object_revision: ObjectRevision(0),
                },
                Operation::SetBoardStackup {
                    board_id,
                    stackup: serde_json::json!({
                        "layers": [
                            serde_json::to_value(&layers[0]).unwrap(),
                            serde_json::to_value(&layers[1]).unwrap(),
                        ],
                    }),
                },
            ]
        );
    }

    #[test]
    fn name_builds_guarded_set_board_name() {
        let (_root, model, board_id, _package_id) =
            resolved_model_with_board_package("board_layout_name");

        let prepared = build_set_board_name(
            &model,
            test_provenance(),
            board_id,
            "Main Board".to_string(),
        )
        .expect("name should build");

        assert_eq!(
            prepared.batch.operations,
            vec![
                Operation::GuardObjectRevision {
                    object_id: board_id,
                    expected_object_revision: ObjectRevision(0),
                },
                Operation::SetBoardName {
                    board_id,
                    name: "Main Board".to_string(),
                },
            ]
        );
    }
}
