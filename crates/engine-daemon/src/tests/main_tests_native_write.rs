//! Dispatch-level tests for the daemon's native mutation surface:
//! `native.describe` and `native.write` (verb-registry writes through the
//! engine's native write facade against a real bootstrapped project).

use super::*;

use eda_engine::api::native_write::genesis::{GenesisSpec, bootstrap_native_project};

fn temp_native_project(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "datum_daemon_native_write_{label}_{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).expect("temp project root should create");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Daemon Native Fixture".to_string(),
            existing_ids: None,
        },
    )
    .expect("genesis should succeed");
    root
}

fn native_request(id: u64, method: &str, params: Value) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(id),
        method: method.into(),
        params,
    }
}

fn describe(engine: &mut Engine, id: u64, root: &Path) -> Value {
    let response = dispatch_request(
        engine,
        native_request(
            id,
            "native.describe",
            json!({ "project_root": root.display().to_string() }),
        ),
    );
    assert!(response.error.is_none(), "{response:?}");
    response.result.expect("describe result should exist")
}

#[test]
fn native_describe_reports_revision_name_and_journal_len() {
    let mut engine = Engine::new().expect("engine should initialize");
    let root = temp_native_project("describe");

    let result = describe(&mut engine, 1, &root);
    assert_eq!(result["project_name"], "Daemon Native Fixture");
    assert!(result["project_id"].is_string());
    assert!(
        result["model_revision"]
            .as_str()
            .is_some_and(|revision| !revision.is_empty())
    );
    assert_eq!(result["journal_len"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn native_describe_rejects_unresolvable_project_root() {
    let mut engine = Engine::new().expect("engine should initialize");
    let response = dispatch_request(
        &mut engine,
        native_request(
            1,
            "native.describe",
            json!({ "project_root": "/nonexistent/datum-daemon-native-write" }),
        ),
    );
    let error = response.error.expect("describe should fail");
    assert_eq!(error.code, -32060);
}

#[test]
fn native_write_set_name_commits_through_journal() {
    let mut engine = Engine::new().expect("engine should initialize");
    let root = temp_native_project("write_commit");
    let before = describe(&mut engine, 1, &root);
    let before_revision = before["model_revision"].as_str().unwrap().to_string();

    let response = dispatch_request(
        &mut engine,
        native_request(
            2,
            "native.write",
            json!({
                "project_root": root.display().to_string(),
                "verb": "datum.project.set_name",
                "params": { "name": "Daemon Renamed" },
                "reason": "rename project via daemon",
                "source": "assistant",
                "actor": "test-assistant",
                "expected_model_revision": before_revision,
            }),
        ),
    );
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("write result should exist");
    assert_eq!(result["verb"], "datum.project.set_name");
    assert_eq!(result["status"], "applied");
    assert!(result["transaction_id"].is_string());
    assert_eq!(result["before_model_revision"], before_revision.as_str());
    assert_ne!(
        result["after_model_revision"],
        result["before_model_revision"]
    );
    assert_eq!(result["operation_count"], 2);
    assert_eq!(result["journal_len"], 1);
    assert!(result["primary_object_id"].is_string());

    let after = describe(&mut engine, 3, &root);
    assert_eq!(after["project_name"], "Daemon Renamed");
    assert_eq!(after["model_revision"], result["after_model_revision"]);
    assert_eq!(after["journal_len"], 1);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn native_write_waive_verb_commits_disposition() {
    let mut engine = Engine::new().expect("engine should initialize");
    let root = temp_native_project("write_waive");

    let response = dispatch_request(
        &mut engine,
        native_request(
            1,
            "native.write",
            json!({
                "project_root": root.display().to_string(),
                "verb": "datum.check.waive",
                "params": {
                    "domain": "erc",
                    "fingerprint": "erc:unconnected:daemon123",
                    "rationale": "reviewed via daemon",
                    "created_by": "reviewer"
                },
                "reason": "waive check finding erc:unconnected:daemon123",
            }),
        ),
    );
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("write result should exist");
    assert_eq!(result["status"], "applied");
    assert_eq!(result["operation_count"], 1);
    assert!(result["primary_object_id"].is_string());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn native_write_dry_run_previews_without_committing() {
    let mut engine = Engine::new().expect("engine should initialize");
    let root = temp_native_project("write_dry_run");

    let response = dispatch_request(
        &mut engine,
        native_request(
            1,
            "native.write",
            json!({
                "project_root": root.display().to_string(),
                "verb": "datum.project.set_name",
                "params": { "name": "Never Applied" },
                "reason": "preview rename",
                "dry_run": true,
            }),
        ),
    );
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("dry-run result should exist");
    assert_eq!(result["status"], "dry_run");
    assert_eq!(
        result["operation_kinds"],
        json!(["guard_object_revision", "set_project_name"])
    );
    assert_eq!(result["operation_count"], 2);
    assert!(result["primary_object_id"].is_string());
    assert!(result["expected_model_revision"].is_string());

    let after = describe(&mut engine, 2, &root);
    assert_eq!(after["project_name"], "Daemon Native Fixture");
    assert_eq!(after["journal_len"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn native_write_rejects_stale_expected_model_revision() {
    let mut engine = Engine::new().expect("engine should initialize");
    let root = temp_native_project("write_stale");

    let response = dispatch_request(
        &mut engine,
        native_request(
            1,
            "native.write",
            json!({
                "project_root": root.display().to_string(),
                "verb": "datum.project.set_name",
                "params": { "name": "Never Applied" },
                "reason": "stale write attempt",
                "expected_model_revision": "not-the-current-revision",
            }),
        ),
    );
    let error = response.error.expect("stale write should fail");
    assert_eq!(error.code, -32061);
    assert!(
        error.message.contains("stale expected_model_revision"),
        "{}",
        error.message
    );

    let after = describe(&mut engine, 2, &root);
    assert_eq!(after["journal_len"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn native_write_rejects_unknown_verb() {
    let mut engine = Engine::new().expect("engine should initialize");
    let root = temp_native_project("write_unknown_verb");

    let response = dispatch_request(
        &mut engine,
        native_request(
            1,
            "native.write",
            json!({
                "project_root": root.display().to_string(),
                "verb": "datum.project.explode",
                "params": {},
                "reason": "unknown verb attempt",
            }),
        ),
    );
    let error = response.error.expect("unknown verb should fail");
    assert_eq!(error.code, -32062);
    assert!(
        error
            .message
            .contains("unknown native write verb: datum.project.explode"),
        "{}",
        error.message
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn native_write_rejects_disallowed_source() {
    let mut engine = Engine::new().expect("engine should initialize");
    let root = temp_native_project("write_bad_source");

    for source in ["cli", "manual", "test", "robot"] {
        let response = dispatch_request(
            &mut engine,
            native_request(
                1,
                "native.write",
                json!({
                    "project_root": root.display().to_string(),
                    "verb": "datum.project.set_name",
                    "params": { "name": "Never Applied" },
                    "reason": "bad source attempt",
                    "source": source,
                }),
            ),
        );
        let error = response.error.expect("disallowed source should fail");
        assert_eq!(error.code, -32602, "source `{source}` must be rejected");
        assert!(
            error
                .message
                .contains("source must be \"tool\" or \"assistant\""),
            "{}",
            error.message
        );
    }

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn native_write_requires_non_empty_reason() {
    let mut engine = Engine::new().expect("engine should initialize");
    let root = temp_native_project("write_no_reason");

    let response = dispatch_request(
        &mut engine,
        native_request(
            1,
            "native.write",
            json!({
                "project_root": root.display().to_string(),
                "verb": "datum.project.set_name",
                "params": { "name": "Never Applied" },
                "reason": "   ",
            }),
        ),
    );
    let error = response.error.expect("empty reason should fail");
    assert_eq!(error.code, -32602);
    assert!(
        error.message.contains("non-empty reason"),
        "{}",
        error.message
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn native_write_surfaces_builder_errors() {
    let mut engine = Engine::new().expect("engine should initialize");
    let root = temp_native_project("write_build_error");

    let response = dispatch_request(
        &mut engine,
        native_request(
            1,
            "native.write",
            json!({
                "project_root": root.display().to_string(),
                "verb": "datum.project.set_name",
                "params": { "name": "   " },
                "reason": "empty name attempt",
            }),
        ),
    );
    let error = response.error.expect("empty name should fail the builder");
    assert_eq!(error.code, -32060);
    assert!(
        error.message.contains("project name must not be empty"),
        "{}",
        error.message
    );

    let _ = std::fs::remove_dir_all(&root);
}
