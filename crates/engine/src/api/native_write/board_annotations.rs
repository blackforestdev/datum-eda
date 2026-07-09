//! Board text/dimension/keepout builders for the native write facade.
//!
//! Family D of the native-write migration: all operation authoring for board
//! annotation objects lives here. The CLI callers in
//! `crates/cli/src/command_project_board_layout.rs` (text/keepout) and the
//! dimension half of
//! `crates/cli/src/command_project_board_netclass_dimension.rs` are thin
//! argument-parsers over this module: they parse/patch the typed board
//! structs (`BoardText`/`Dimension`/`Keepout`), call a `build_*` function,
//! and commit the returned [`PreparedWrite`] via [`super::commit_prepared`].
//!
//! Builders are build-only; they never touch disk. Payload shape (serde of
//! the board types; delete operations thread the stored raw value) and guard
//! insertion are byte-for-byte the CLI's historical behavior.

use crate::board::{BoardText, Dimension, Keepout};
use crate::error::EngineError;
use crate::substrate::{DesignModel, ObjectId, Operation};

use super::context::{BatchComposer, PreparedWrite, WriteProvenance};

macro_rules! board_annotation_builders {
    (
        $entity:ty,
        $id_field:ident,
        $payload_field:ident,
        $create_variant:ident,
        $set_variant:ident,
        $delete_variant:ident,
        $place_fn:ident,
        $set_fn:ident,
        $delete_fn:ident
    ) => {
        /// Build the unguarded creation batch for one new annotation object;
        /// the payload is the serde serialization of the typed struct.
        pub fn $place_fn(
            model: &DesignModel,
            provenance: WriteProvenance,
            object: &$entity,
        ) -> Result<PreparedWrite, EngineError> {
            BatchComposer::compose(model, provenance)
                .push_op(Operation::$create_variant {
                    $id_field: object.uuid,
                    $payload_field: serde_json::to_value(object)?,
                })
                .primary_object(object.uuid)
                .finish()
        }

        /// Build the revision-guarded rewrite batch for one existing
        /// annotation object; the payload is the serde serialization of the
        /// typed struct.
        pub fn $set_fn(
            model: &DesignModel,
            provenance: WriteProvenance,
            object: &$entity,
        ) -> Result<PreparedWrite, EngineError> {
            BatchComposer::compose(model, provenance)
                .push_op(Operation::$set_variant {
                    $id_field: object.uuid,
                    $payload_field: serde_json::to_value(object)?,
                })
                .primary_object(object.uuid)
                .finish()
        }

        /// Build the revision-guarded delete batch for one existing
        /// annotation object; `stored` is the raw persisted payload (threaded
        /// verbatim for byte-exact journal parity with the pre-migration
        /// CLI).
        pub fn $delete_fn(
            model: &DesignModel,
            provenance: WriteProvenance,
            object_id: ObjectId,
            stored: serde_json::Value,
        ) -> Result<PreparedWrite, EngineError> {
            BatchComposer::compose(model, provenance)
                .push_op(Operation::$delete_variant {
                    $id_field: object_id,
                    $payload_field: stored,
                })
                .primary_object(object_id)
                .finish()
        }
    };
}

board_annotation_builders!(
    BoardText,
    text_id,
    text,
    CreateBoardText,
    SetBoardText,
    DeleteBoardText,
    build_place_board_text,
    build_set_board_text,
    build_delete_board_text
);
board_annotation_builders!(
    Dimension,
    dimension_id,
    dimension,
    CreateBoardDimension,
    SetBoardDimension,
    DeleteBoardDimension,
    build_place_board_dimension,
    build_set_board_dimension,
    build_delete_board_dimension
);
board_annotation_builders!(
    Keepout,
    keepout_id,
    keepout,
    CreateBoardKeepout,
    SetBoardKeepout,
    DeleteBoardKeepout,
    build_place_board_keepout,
    build_set_board_keepout,
    build_delete_board_keepout
);

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use uuid::Uuid;

    use super::super::context::commit_prepared;
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::ir::geometry::{Point, Polygon};
    use crate::substrate::{CommitSource, ObjectRevision, ProjectResolver};

    fn test_provenance() -> WriteProvenance {
        WriteProvenance::new(
            "unit-test",
            CommitSource::Test,
            "board annotations facade test",
        )
    }

    fn test_dimension() -> Dimension {
        Dimension {
            uuid: Uuid::new_v4(),
            from: Point { x: 0, y: 0 },
            to: Point { x: 5_000_000, y: 0 },
            layer: 41,
            text: Some("5 mm".to_string()),
        }
    }

    fn test_keepout() -> Keepout {
        Keepout {
            uuid: Uuid::new_v4(),
            polygon: Polygon {
                vertices: vec![
                    Point { x: 0, y: 0 },
                    Point { x: 1_000_000, y: 0 },
                    Point { x: 0, y: 1_000_000 },
                ],
                closed: true,
            },
            layers: vec![1],
            kind: "no_copper".to_string(),
        }
    }

    /// The shared board fixture predates the annotation arrays; overlay the
    /// empty `dimensions` array the substrate's list operations require.
    fn add_board_dimensions_array(root: &std::path::Path) {
        let path = root.join("board/board.json");
        let mut board: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("board shard should read"))
                .expect("board shard should parse");
        board["dimensions"] = serde_json::json!([]);
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&board).expect("board shard should serialize"),
        )
        .expect("board shard should write");
    }

    /// Fixture with one committed dimension (created through the facade
    /// itself, then re-resolved from disk).
    fn resolved_model_with_dimension(name: &str) -> (PathBuf, DesignModel, Dimension) {
        let (root, _model, _board_id, _package_id) = resolved_model_with_board_package(name);
        add_board_dimensions_array(&root);
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("fixture project with dimensions array should resolve");
        let dimension = test_dimension();
        let prepared = build_place_board_dimension(&model, test_provenance(), &dimension)
            .expect("dimension placement should build");
        commit_prepared(&mut model, &root, prepared).expect("dimension placement should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("fixture project should re-resolve");
        (root, model, dimension)
    }

    #[test]
    fn place_builds_single_unguarded_create_with_serde_payload() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("board_annotations_place");
        let keepout = test_keepout();

        let prepared = build_place_board_keepout(&model, test_provenance(), &keepout)
            .expect("place should build");

        assert_eq!(prepared.primary_object_id, Some(keepout.uuid));
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreateBoardKeepout {
                keepout_id: keepout.uuid,
                keepout: serde_json::to_value(&keepout).unwrap(),
            }]
        );
    }

    #[test]
    fn set_guards_existing_annotation() {
        let (_root, model, dimension) = resolved_model_with_dimension("board_annotations_set");
        let mut edited = dimension.clone();
        edited.text = None;

        let prepared = build_set_board_dimension(&model, test_provenance(), &edited)
            .expect("set should build");

        assert_eq!(
            prepared.batch.operations,
            vec![
                Operation::GuardObjectRevision {
                    object_id: dimension.uuid,
                    expected_object_revision: ObjectRevision(0),
                },
                Operation::SetBoardDimension {
                    dimension_id: dimension.uuid,
                    dimension: serde_json::to_value(&edited).unwrap(),
                },
            ]
        );
    }

    #[test]
    fn delete_threads_stored_payload_raw_and_guards() {
        let (_root, model, dimension) = resolved_model_with_dimension("board_annotations_delete");
        let stored = serde_json::to_value(&dimension).unwrap();

        let prepared =
            build_delete_board_dimension(&model, test_provenance(), dimension.uuid, stored.clone())
                .expect("delete should build");

        assert_eq!(prepared.primary_object_id, Some(dimension.uuid));
        assert_eq!(
            prepared.batch.operations,
            vec![
                Operation::GuardObjectRevision {
                    object_id: dimension.uuid,
                    expected_object_revision: ObjectRevision(0),
                },
                Operation::DeleteBoardDimension {
                    dimension_id: dimension.uuid,
                    dimension: stored,
                },
            ]
        );
    }

    #[test]
    fn set_rejects_unknown_annotation() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("board_annotations_unknown");
        let dimension = test_dimension();
        let error = build_set_board_dimension(&model, test_provenance(), &dimension)
            .expect_err("unknown dimension should fail");
        assert!(matches!(
            error,
            EngineError::NotFound {
                object_type: "domain_object",
                uuid,
            } if uuid == dimension.uuid
        ));
    }
}
