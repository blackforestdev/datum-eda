use super::pool_library::write_project_with_pool;
use super::*;

#[test]
fn journaled_pool_library_object_rejects_invalid_path_and_identity() {
    let root = temp_project_root("pool_library_symbol_validation");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    write_project_with_pool(&root, project_id, board_id);

    let invalid_cases = [
        (
            "../symbols/escape.json".to_string(),
            "symbols".to_string(),
            serde_json::json!({ "schema_version": 1, "uuid": symbol_id, "name": "Escape", "unit": Uuid::new_v4() }),
            "invalid pool library object path",
        ),
        (
            format!("pool/units/{symbol_id}.json"),
            "symbols".to_string(),
            serde_json::json!({ "schema_version": 1, "uuid": symbol_id, "name": "WrongKind", "unit": Uuid::new_v4() }),
            "does not match path",
        ),
        (
            format!("pool/symbols/{symbol_id}.json"),
            "symbols".to_string(),
            serde_json::json!({ "schema_version": 1, "uuid": Uuid::new_v4(), "name": "WrongUuid", "unit": Uuid::new_v4() }),
            "does not match object id",
        ),
    ];

    for (relative_path, object_kind, object, expected_error) in invalid_cases {
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should resolve");
        let error = model
            .commit_journaled(
                &root,
                OperationBatch {
                    batch_id: Uuid::new_v4(),
                    expected_model_revision: Some(model.model_revision.clone()),
                    provenance: CommitProvenance {
                        actor: "unit-test".to_string(),
                        source: CommitSource::Test,
                        reason: "reject invalid native pool object".to_string(),
                    },
                    operations: vec![Operation::CreatePoolLibraryObject {
                        object_id: symbol_id,
                        relative_path,
                        object_kind,
                        object,
                    }],
                },
            )
            .expect_err("invalid pool library object should be rejected");
        assert!(
            matches!(&error, EngineError::Validation(message) if message.contains(expected_error)),
            "unexpected error: {error:?}"
        );
    }
}

#[test]
fn journaled_pool_library_object_rejects_invalid_typed_payload() {
    let root = temp_project_root("pool_library_symbol_schema_validation");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    write_project_with_pool(&root, project_id, board_id);

    let invalid_cases = [
        (
            serde_json::json!({
                "uuid": symbol_id,
                "name": "MissingSchema",
                "unit": Uuid::new_v4()
            }),
            "missing schema_version",
        ),
        (
            serde_json::json!({
                "schema_version": 2,
                "uuid": symbol_id,
                "name": "WrongSchema",
                "unit": Uuid::new_v4()
            }),
            "unsupported pool library object schema_version",
        ),
        (
            serde_json::json!({
                "schema_version": 1,
                "uuid": symbol_id,
                "name": "MissingUnit"
            }),
            "invalid pool library symbols object",
        ),
    ];

    for (object, expected_error) in invalid_cases {
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should resolve");
        let error = model
            .commit_journaled(
                &root,
                OperationBatch {
                    batch_id: Uuid::new_v4(),
                    expected_model_revision: Some(model.model_revision.clone()),
                    provenance: CommitProvenance {
                        actor: "unit-test".to_string(),
                        source: CommitSource::Test,
                        reason: "reject invalid native pool schema".to_string(),
                    },
                    operations: vec![Operation::CreatePoolLibraryObject {
                        object_id: symbol_id,
                        relative_path: format!("pool/symbols/{symbol_id}.json"),
                        object_kind: "symbols".to_string(),
                        object,
                    }],
                },
            )
            .expect_err("invalid pool library schema should be rejected");
        assert!(
            matches!(&error, EngineError::Validation(message) if message.contains(expected_error)),
            "unexpected error: {error:?}"
        );
    }
}

#[test]
fn journaled_pool_library_object_rejects_invalid_footprint_payload() {
    let root = temp_project_root("pool_library_footprint_schema_validation");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let footprint_id = Uuid::new_v4();
    write_project_with_pool(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "reject invalid native pool footprint".to_string(),
                },
                operations: vec![Operation::CreatePoolLibraryObject {
                    object_id: footprint_id,
                    relative_path: format!("pool/footprints/{footprint_id}.json"),
                    object_kind: "footprints".to_string(),
                    object: serde_json::json!({
                        "schema_version": 1,
                        "uuid": footprint_id,
                        "name": "MissingGeometry"
                    }),
                }],
            },
        )
        .expect_err("invalid footprint schema should be rejected");
    assert!(
        matches!(&error, EngineError::Validation(message) if message.contains("invalid pool library footprints object")),
        "unexpected error: {error:?}"
    );
}
