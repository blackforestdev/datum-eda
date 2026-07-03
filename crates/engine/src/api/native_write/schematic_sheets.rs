//! Schematic sheet-structure and sheet-scoped text/drawing builders for the
//! native write facade.
//!
//! Family C of the native-write migration: sheet root create/delete/rename,
//! sheet-definition create, sheet-instance create/set/delete (from
//! `crates/cli/src/command_project/command_project_schematic_sheet_mutations.rs`)
//! and sheet-scoped text/drawing authoring (from
//! `crates/cli/src/command_project_schematic_text_drawing_mutations.rs`).
//!
//! Builders are build-only: they return a [`PreparedWrite`] and never commit.
//! Sheet-root and definition payloads are the CLI-owned native shard shapes
//! and are passed in as pre-serialized `serde_json::Value`s (the shard
//! persistence layer serializes deterministically, so payload key order never
//! drifts on disk); text/drawing payloads are the engine's typed schematic
//! primitives and are serialized here. Guard insertion and batch stamping are
//! byte-for-byte the CLI's historical behavior via [`super::context`].

use uuid::Uuid;

use crate::error::EngineError;
use crate::schematic::{SchematicPrimitive, SchematicText};
use crate::substrate::{DesignModel, ObjectId, Operation};

use super::context::{BatchComposer, PreparedWrite, WriteProvenance};

/// Build the batch that creates a new schematic sheet shard at
/// `relative_path` (relative to the project's `schematic/` directory).
pub fn build_create_schematic_sheet(
    model: &DesignModel,
    provenance: WriteProvenance,
    schematic_id: ObjectId,
    sheet_id: ObjectId,
    relative_path: &str,
    sheet: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateSchematicSheet {
            schematic_id,
            sheet_id,
            relative_path: relative_path.to_string(),
            sheet,
        })
        .primary_object(sheet_id)
        .finish()
}

/// Build the batch that deletes an existing schematic sheet shard (payload is
/// the materialized pre-delete sheet value for inverse replay).
pub fn build_delete_schematic_sheet(
    model: &DesignModel,
    provenance: WriteProvenance,
    schematic_id: ObjectId,
    sheet_id: ObjectId,
    relative_path: &str,
    sheet: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteSchematicSheet {
            schematic_id,
            sheet_id,
            relative_path: relative_path.to_string(),
            sheet,
        })
        .primary_object(sheet_id)
        .finish()
}

/// Build the batch that renames an existing schematic sheet (revision guard
/// stamped automatically).
pub fn build_rename_schematic_sheet(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    name: &str,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetSchematicSheetName {
            sheet_id,
            name: name.to_string(),
        })
        .primary_object(sheet_id)
        .finish()
}

/// Build the batch that creates a new schematic sheet-definition shard at
/// `relative_path` (relative to the project's `schematic/` directory).
pub fn build_create_schematic_definition(
    model: &DesignModel,
    provenance: WriteProvenance,
    schematic_id: ObjectId,
    definition_id: ObjectId,
    relative_path: &str,
    definition: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateSchematicDefinition {
            schematic_id,
            definition_id,
            relative_path: relative_path.to_string(),
            definition,
        })
        .primary_object(definition_id)
        .finish()
}

/// Build the batch that creates a new schematic sheet instance on the
/// schematic root.
pub fn build_create_schematic_sheet_instance(
    model: &DesignModel,
    provenance: WriteProvenance,
    schematic_id: ObjectId,
    instance_id: ObjectId,
    instance: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateSchematicSheetInstance {
            schematic_id,
            instance_id,
            instance,
        })
        .primary_object(instance_id)
        .finish()
}

/// Build the batch that rewrites an existing schematic sheet instance
/// (`previous_instance` is the pre-mutation payload for inverse replay).
pub fn build_set_schematic_sheet_instance(
    model: &DesignModel,
    provenance: WriteProvenance,
    schematic_id: ObjectId,
    instance_id: ObjectId,
    previous_instance: serde_json::Value,
    instance: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetSchematicSheetInstance {
            schematic_id,
            instance_id,
            previous_instance,
            instance,
        })
        .primary_object(instance_id)
        .finish()
}

/// Build the batch that deletes an existing schematic sheet instance
/// (payload is the pre-delete instance for inverse replay).
pub fn build_delete_schematic_sheet_instance(
    model: &DesignModel,
    provenance: WriteProvenance,
    schematic_id: ObjectId,
    instance_id: ObjectId,
    instance: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteSchematicSheetInstance {
            schematic_id,
            instance_id,
            instance,
        })
        .primary_object(instance_id)
        .finish()
}

/// Build the batch that places a free text object on `sheet_id`.
pub fn build_create_schematic_text(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    text: &SchematicText,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateSchematicText {
            sheet_id,
            text_id: text.uuid,
            text: serde_json::to_value(text)?,
        })
        .primary_object(text.uuid)
        .finish()
}

/// Build the batch that rewrites an existing text object (revision guard
/// stamped automatically).
pub fn build_set_schematic_text(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    text: &SchematicText,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetSchematicText {
            sheet_id,
            text_id: text.uuid,
            text: serde_json::to_value(text)?,
        })
        .primary_object(text.uuid)
        .finish()
}

/// Build the batch that deletes an existing text object (revision guard
/// stamped automatically).
pub fn build_delete_schematic_text(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    text: &SchematicText,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteSchematicText {
            sheet_id,
            text_id: text.uuid,
            text: serde_json::to_value(text)?,
        })
        .primary_object(text.uuid)
        .finish()
}

/// Build the batch that places a drawing primitive on `sheet_id`.
pub fn build_create_schematic_drawing(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    drawing: &SchematicPrimitive,
) -> Result<PreparedWrite, EngineError> {
    let drawing_id = drawing_uuid(drawing);
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateSchematicDrawing {
            sheet_id,
            drawing_id,
            drawing: serde_json::to_value(drawing)?,
        })
        .primary_object(drawing_id)
        .finish()
}

/// Build the batch that rewrites an existing drawing primitive (revision
/// guard stamped automatically).
pub fn build_set_schematic_drawing(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    drawing: &SchematicPrimitive,
) -> Result<PreparedWrite, EngineError> {
    let drawing_id = drawing_uuid(drawing);
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetSchematicDrawing {
            sheet_id,
            drawing_id,
            drawing: serde_json::to_value(drawing)?,
        })
        .primary_object(drawing_id)
        .finish()
}

/// Build the batch that deletes an existing drawing primitive (revision
/// guard stamped automatically).
pub fn build_delete_schematic_drawing(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    drawing: &SchematicPrimitive,
) -> Result<PreparedWrite, EngineError> {
    let drawing_id = drawing_uuid(drawing);
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteSchematicDrawing {
            sheet_id,
            drawing_id,
            drawing: serde_json::to_value(drawing)?,
        })
        .primary_object(drawing_id)
        .finish()
}

fn drawing_uuid(drawing: &SchematicPrimitive) -> Uuid {
    match drawing {
        SchematicPrimitive::Line { uuid, .. }
        | SchematicPrimitive::Rect { uuid, .. }
        | SchematicPrimitive::Circle { uuid, .. }
        | SchematicPrimitive::Arc { uuid, .. } => *uuid,
    }
}

#[cfg(test)]
mod tests {
    use super::super::context::commit_prepared;
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::ir::geometry::Point;
    use crate::substrate::{CommitSource, ObjectRevision, ProjectResolver};

    fn test_provenance(reason: &str) -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, reason)
    }

    fn fixture_schematic_id(model: &DesignModel) -> Uuid {
        Uuid::new_v5(&model.project.project_id, b"schematic")
    }

    fn fixture_sheet_id(model: &DesignModel) -> Uuid {
        Uuid::new_v5(&model.project.project_id, b"sheet")
    }

    fn empty_sheet_payload(sheet_id: Uuid, name: &str) -> serde_json::Value {
        serde_json::json!({
            "schema_version": 1,
            "uuid": sheet_id,
            "name": name,
            "frame": null,
            "symbols": {},
            "wires": {},
            "junctions": {},
            "labels": {},
            "buses": {},
            "bus_entries": {},
            "ports": {},
            "noconnects": {},
            "texts": {},
            "drawings": {}
        })
    }

    #[test]
    fn create_sheet_authors_unguarded_create_and_commits_shard_paths() {
        let (root, mut model, _board_id, _package_id) =
            resolved_model_with_board_package("schematic_sheets_create");
        let schematic_id = fixture_schematic_id(&model);
        let sheet_id = Uuid::new_v4();
        let relative_path = format!("sheets/{sheet_id}.json");
        let payload = empty_sheet_payload(sheet_id, "Aux");

        let prepared = build_create_schematic_sheet(
            &model,
            test_provenance("create schematic sheet"),
            schematic_id,
            sheet_id,
            &relative_path,
            payload.clone(),
        )
        .expect("create sheet should build");

        assert_eq!(prepared.primary_object_id, Some(sheet_id));
        assert_eq!(
            prepared.batch.expected_model_revision,
            Some(model.model_revision.clone())
        );
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreateSchematicSheet {
                schematic_id,
                sheet_id,
                relative_path: relative_path.clone(),
                sheet: payload,
            }]
        );

        commit_prepared(&mut model, &root, prepared).expect("create sheet should commit");
        // Shard staging paths are a persistence contract: the sheet shard
        // lands under schematic/<relative_path> and the schematic root lists
        // it.
        let shard_path = root.join("schematic").join(&relative_path);
        assert!(shard_path.exists(), "sheet shard should be staged on disk");
        let schematic_root: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("schematic/schematic.json"))
                .expect("schematic root should read"),
        )
        .expect("schematic root should parse");
        assert_eq!(
            schematic_root["sheets"][sheet_id.to_string()],
            serde_json::json!(relative_path)
        );
    }

    #[test]
    fn rename_sheet_guards_the_sheet_object() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("schematic_sheets_rename");
        let sheet_id = fixture_sheet_id(&model);

        let prepared = build_rename_schematic_sheet(
            &model,
            test_provenance("rename schematic sheet"),
            sheet_id,
            "Renamed",
        )
        .expect("rename should build");

        assert_eq!(
            prepared.batch.operations,
            vec![
                Operation::GuardObjectRevision {
                    object_id: sheet_id,
                    expected_object_revision: ObjectRevision(0),
                },
                Operation::SetSchematicSheetName {
                    sheet_id,
                    name: "Renamed".to_string(),
                },
            ]
        );
    }

    #[test]
    fn definition_and_instance_builders_author_expected_operations() {
        let (root, mut model, _board_id, _package_id) =
            resolved_model_with_board_package("schematic_sheets_definition");
        let schematic_id = fixture_schematic_id(&model);
        let root_sheet_id = fixture_sheet_id(&model);
        let definition_id = Uuid::new_v4();
        let relative_path = format!("definitions/{definition_id}.json");
        let definition_payload = serde_json::json!({
            "schema_version": 1,
            "uuid": definition_id,
            "root_sheet": root_sheet_id,
            "name": "DefA"
        });

        let prepared = build_create_schematic_definition(
            &model,
            test_provenance("create schematic sheet definition"),
            schematic_id,
            definition_id,
            &relative_path,
            definition_payload.clone(),
        )
        .expect("definition should build");
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreateSchematicDefinition {
                schematic_id,
                definition_id,
                relative_path: relative_path.clone(),
                definition: definition_payload,
            }]
        );
        commit_prepared(&mut model, &root, prepared).expect("definition should commit");
        assert!(
            root.join("schematic").join(&relative_path).exists(),
            "definition shard should be staged on disk"
        );

        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");
        let instance_id = Uuid::new_v4();
        let instance_payload = serde_json::json!({
            "uuid": instance_id,
            "definition": definition_id,
            "parent_sheet": root_sheet_id,
            "position": { "x": 10, "y": 20 },
            "name": "InstA"
        });
        let moved_payload = serde_json::json!({
            "uuid": instance_id,
            "definition": definition_id,
            "parent_sheet": root_sheet_id,
            "position": { "x": 30, "y": 40 },
            "name": "InstA"
        });

        let prepared = build_create_schematic_sheet_instance(
            &model,
            test_provenance("create schematic sheet instance"),
            schematic_id,
            instance_id,
            instance_payload.clone(),
        )
        .expect("instance create should build");
        assert_eq!(prepared.batch.operations.len(), 1);

        let prepared_set = build_set_schematic_sheet_instance(
            &model,
            test_provenance("move schematic sheet instance"),
            schematic_id,
            instance_id,
            instance_payload.clone(),
            moved_payload.clone(),
        )
        .expect("instance set should build");
        // SetSchematicSheetInstance is not an existing-object guard target
        // (matches the CLI's historical guard behavior).
        assert_eq!(
            prepared_set.batch.operations,
            vec![Operation::SetSchematicSheetInstance {
                schematic_id,
                instance_id,
                previous_instance: instance_payload.clone(),
                instance: moved_payload,
            }]
        );

        let prepared_delete = build_delete_schematic_sheet_instance(
            &model,
            test_provenance("delete schematic sheet instance"),
            schematic_id,
            instance_id,
            instance_payload.clone(),
        )
        .expect("instance delete should build");
        assert_eq!(
            prepared_delete.batch.operations,
            vec![Operation::DeleteSchematicSheetInstance {
                schematic_id,
                instance_id,
                instance: instance_payload,
            }]
        );
    }

    #[test]
    fn text_set_and_delete_guard_the_text_object() {
        let (root, mut model, _board_id, _package_id) =
            resolved_model_with_board_package("schematic_sheets_text");
        let sheet_id = fixture_sheet_id(&model);
        let text = SchematicText {
            uuid: Uuid::new_v4(),
            text: "note".to_string(),
            position: Point { x: 1, y: 2 },
            rotation: 0,
        };

        let prepared = build_create_schematic_text(
            &model,
            test_provenance("place schematic text"),
            sheet_id,
            &text,
        )
        .expect("text create should build");
        assert_eq!(prepared.batch.operations.len(), 1);
        commit_prepared(&mut model, &root, prepared).expect("text create should commit");

        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");
        let prepared = build_set_schematic_text(
            &model,
            test_provenance("edit schematic text"),
            sheet_id,
            &text,
        )
        .expect("text set should build");
        assert_eq!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision {
                object_id: text.uuid,
                expected_object_revision: ObjectRevision(0),
            }
        );
        assert!(matches!(
            &prepared.batch.operations[1],
            Operation::SetSchematicText { text_id, .. } if *text_id == text.uuid
        ));

        let prepared = build_delete_schematic_text(
            &model,
            test_provenance("delete schematic text"),
            sheet_id,
            &text,
        )
        .expect("text delete should build");
        assert!(matches!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == text.uuid
        ));
    }

    #[test]
    fn drawing_builders_derive_the_variant_uuid() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("schematic_sheets_drawing");
        let sheet_id = fixture_sheet_id(&model);
        let drawing_id = Uuid::new_v4();
        let drawing = SchematicPrimitive::Line {
            uuid: drawing_id,
            from: Point { x: 0, y: 0 },
            to: Point { x: 5, y: 5 },
        };

        let prepared = build_create_schematic_drawing(
            &model,
            test_provenance("place schematic drawing line"),
            sheet_id,
            &drawing,
        )
        .expect("drawing create should build");

        assert_eq!(prepared.primary_object_id, Some(drawing_id));
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreateSchematicDrawing {
                sheet_id,
                drawing_id,
                drawing: serde_json::to_value(&drawing).expect("drawing should serialize"),
            }]
        );
    }
}
