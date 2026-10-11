//! Dispatch-level tests for the daemon's native mutation surface:
//! `native.describe` and `native.write` (verb-registry writes through the
//! engine's native write facade against a real bootstrapped project).

use super::*;

#[path = "main_tests_native_write/library_reference_controls.rs"]
mod library_reference_controls;

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
fn native_library_authoring_requires_snapshot_guards_and_never_partially_writes() {
    let mut engine = Engine::new().unwrap();
    let root = temp_native_project("library_guards");
    let before = describe(&mut engine, 1, &root);
    let id = uuid::Uuid::new_v4();
    let base = json!({"project_root":root,"verb":"datum.library.create","params":{"kind":"padstacks","object_id":id,"pool_path":"pool","fields":{"name":"Named copper","aperture":{"circle":{"diameter_nm":1000000}}}},"actor":"datum-gui","source":"manual","reason":"Library guard proof"});
    for revision in [false, true] {
        let mut params = base.clone();
        if revision {
            params["expected_model_revision"] = before["model_revision"].clone();
        }
        let response = dispatch_request(&mut engine, native_request(2, "native.write", params));
        assert!(
            response.error.is_some(),
            "both snapshot guards are mandatory"
        );
        assert_eq!(describe(&mut engine, 3, &root)["journal_len"], 0);
        assert!(!root.join(format!("pool/padstacks/{id}.json")).exists());
    }
    let mut missing_revision = base.clone();
    missing_revision["expected_transaction_tip"] = before["transaction_tip"].clone();
    assert!(
        dispatch_request(
            &mut engine,
            native_request(3, "native.write", missing_revision)
        )
        .error
        .is_some(),
        "an explicit empty tip cannot replace the mandatory revision guard"
    );
    assert_eq!(describe(&mut engine, 3, &root)["journal_len"], 0);
    assert!(!root.join(format!("pool/padstacks/{id}.json")).exists());
    let mut guarded = base.clone();
    guarded["expected_model_revision"] = before["model_revision"].clone();
    guarded["expected_transaction_tip"] = before["transaction_tip"].clone();
    let mut wrong = guarded.clone();
    wrong["params"]["pool_path"] = json!("other");
    assert!(
        dispatch_request(&mut engine, native_request(4, "native.write", wrong))
            .error
            .is_some()
    );
    assert_eq!(describe(&mut engine, 5, &root)["journal_len"], 0);
    let created = dispatch_request(
        &mut engine,
        native_request(6, "native.write", guarded.clone()),
    );
    assert!(created.error.is_none(), "{created:?}");
    let after = describe(&mut engine, 7, &root);
    assert_eq!(after["journal_len"], 1);
    let path = root.join(format!("pool/padstacks/{id}.json"));
    let raw = std::fs::read(&path).unwrap();
    let journal_path = eda_engine::substrate::transaction_journal_path(&root);
    let journal = std::fs::read(&journal_path).unwrap();
    guarded["verb"] = json!("datum.library.edit");
    guarded["params"]["fields"] = json!({"name":"Stale edit"});
    assert!(
        dispatch_request(
            &mut engine,
            native_request(8, "native.write", guarded.clone())
        )
        .error
        .is_some()
    );
    guarded["expected_model_revision"] = after["model_revision"].clone();
    // Isolate the tip guard after advancing the model guard to the current revision.
    assert!(
        dispatch_request(
            &mut engine,
            native_request(8, "native.write", guarded.clone())
        )
        .error
        .is_some()
    );
    assert_eq!(std::fs::read(&path).unwrap(), raw);
    assert_eq!(std::fs::read(&journal_path).unwrap(), journal);
    guarded["expected_transaction_tip"] = after["transaction_tip"].clone();
    guarded["params"]["fields"] = json!({"aperture":{"rect":{"width_nm":0,"height_nm":500000}}});
    assert!(
        dispatch_request(&mut engine, native_request(9, "native.write", guarded))
            .error
            .is_some()
    );
    assert_eq!(std::fs::read(&path).unwrap(), raw);
    assert_eq!(std::fs::read(&journal_path).unwrap(), journal);
    let reopened = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    assert_eq!(reopened.journal.len(), 1);
    library_reference_controls::verify(&mut engine, &root, id);
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
fn native_write_set_display_units_commits_one_guarded_settings_transaction() {
    use eda_engine::api::native_write::genesis::bootstrap_native_project_with_units;
    use eda_engine::api::native_write::project::project_display_units;
    use eda_engine::ir::units::{
        LengthPrecisionChoice, LengthUnit, LengthUnitChoice, MeasurementSystem,
    };
    use eda_engine::substrate::ProjectResolver;

    let mut engine = Engine::new().expect("engine should initialize");
    let root = std::env::temp_dir().join(format!(
        "datum_daemon_native_write_display_units_{}",
        uuid::Uuid::new_v4()
    ));
    let seed = eda_engine::preferences::factory_units_seed();
    bootstrap_native_project_with_units(
        &root,
        GenesisSpec {
            project_name: "Daemon Units Fixture".to_owned(),
            existing_ids: None,
        },
        seed.profile,
        seed.receipt,
    )
    .unwrap();
    let board_before = std::fs::read(root.join("board/board.json")).unwrap();
    let schematic_before = std::fs::read(root.join("schematic/schematic.json")).unwrap();
    let before = describe(&mut engine, 1, &root);
    let before_revision = before["model_revision"].as_str().unwrap().to_owned();

    let response = dispatch_request(
        &mut engine,
        native_request(
            2,
            "native.write",
            json!({
                "project_root": root.display().to_string(),
                "verb": "datum.project.set_display_units",
                "params": {"profile": {
                    "system": "metric",
                    "board_length": "mil",
                    "board_length_precision": "automatic",
                    "drill_hole": "follow_system",
                    "drill_hole_precision": "decimal_3",
                    "schematic_geometry": "follow_system",
                    "schematic_geometry_precision": "exact_nm",
                    "angle_precision": "decimal_2"
                }},
                "reason": "set Project Working Units through MCP",
                "source": "tool",
                "actor": "datum-mcp",
                "expected_model_revision": before_revision,
            }),
        ),
    );
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.unwrap();
    assert_eq!(result["verb"], "datum.project.set_display_units");
    assert_eq!(result["status"], "applied");
    assert_eq!(result["journal_len"], 1);

    let model = ProjectResolver::new(&root).resolve().unwrap();
    let profile = project_display_units(&model).unwrap();
    assert_eq!(profile.system, MeasurementSystem::Metric);
    assert_eq!(
        profile.board.unit,
        LengthUnitChoice::Explicit(LengthUnit::Mil)
    );
    assert_eq!(
        profile.drill.precision,
        LengthPrecisionChoice::DecimalPlaces(3)
    );
    assert_eq!(
        std::fs::read(root.join("board/board.json")).unwrap(),
        board_before
    );
    assert_eq!(
        std::fs::read(root.join("schematic/schematic.json")).unwrap(),
        schematic_before
    );
    let _ = std::fs::remove_dir_all(root);
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

    for source in ["cli", "test", "robot"] {
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
                .contains("source must be \"tool\", \"assistant\" or \"manual\""),
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

#[test]
fn native_describe_complete_model_roundtrip_matches_its_durable_description() {
    let root = temp_native_project("snapshot");
    let mut engine = Engine::new().unwrap();
    let response = dispatch_request(
        &mut engine,
        native_request(
            1,
            "native.describe",
            json!({"project_root":root,"confirm_durable":true,"include_model":true}),
        ),
    );
    assert!(response.error.is_none(), "{response:?}");
    let description = response.result.unwrap();
    let model: eda_engine::substrate::DesignModel =
        serde_json::from_value(description["model"].clone()).unwrap();
    assert_eq!(
        model,
        eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap()
    );
    assert_eq!(description["project_id"], json!(model.project.project_id));
    assert_eq!(description["model_revision"], json!(model.model_revision.0));
    assert_eq!(
        description["transaction_tip"],
        json!(eda_engine::revision::transaction_tip(&model.journal))
    );
    assert_eq!(description["durability_confirmed"], json!(true));
    assert!(
        describe(&mut engine, 2, &root).get("model").is_none(),
        "default response remains metadata-only"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_write_preserves_manual_gui_and_default_tool_provenance() {
    let mut engine = Engine::new().unwrap();
    let root = temp_native_project("native_manual_origin");
    for (source, expected) in [
        (Some("manual"), eda_engine::substrate::CommitSource::Manual),
        (None, eda_engine::substrate::CommitSource::Tool),
    ] {
        let mut params = json!({"project_root":root,"verb":"datum.project.set_name","params":{"name":format!("Origin {expected:?}")},"reason":"native editor origin"});
        if let Some(source) = source {
            params["source"] = json!(source);
        }
        let response = dispatch_request(&mut engine, native_request(1, "native.write", params));
        assert!(response.error.is_none(), "{:?}", response.error);
        let model = eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap();
        assert_eq!(model.journal.last().unwrap().provenance.source, expected);
    }
}
