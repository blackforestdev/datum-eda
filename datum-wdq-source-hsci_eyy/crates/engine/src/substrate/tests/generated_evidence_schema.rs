use super::*;

#[test]
fn resolver_rejects_unsupported_artifact_run_schema_version() {
    let root = temp_project_root("artifact_run_unsupported_schema_version");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let run_id = Uuid::new_v4();
    write_json(
        &root.join(format!(".datum/artifact_runs/{run_id}.json")),
        serde_json::json!({ "schema_version": 2 }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with artifact-run diagnostic");

    assert!(model.artifact_runs.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_artifact_run"
            && diagnostic
                .message
                .contains("unsupported ArtifactRun schema_version 2")
    }));
}

#[test]
fn resolver_rejects_unsupported_check_run_schema_version() {
    let root = temp_project_root("check_run_unsupported_schema_version");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let run_id = Uuid::new_v4();
    write_json(
        &root.join(format!(".datum/check_runs/{run_id}.json")),
        serde_json::json!({ "schema_version": 2 }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with check-run diagnostic");

    assert!(model.check_runs.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_check_run"
            && diagnostic
                .message
                .contains("unsupported CheckRun schema_version 2")
    }));
}
