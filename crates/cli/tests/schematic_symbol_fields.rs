//! Actual CLI and MCP callers, persisted payloads, qualifications and history.
use eda_engine::substrate::{DesignModel, ProjectResolver};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use uuid::Uuid;

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to.join(entry.file_name()));
        } else {
            std::fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
        }
    }
}
fn fixture() -> PathBuf {
    let root = std::env::temp_dir().join(format!("datum-field-callers-{}", Uuid::new_v4()));
    copy_tree(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../test-harness/testdata/library/native_authored_baseline_v1"),
        &root,
    );
    // This library fixture has a minimal engine-only board. Complete the empty
    // board shape before first resolution for the ordinary CLI read model.
    let path = root.join("board/board.json");
    let mut board: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    board["stackup"] = json!({"layers":[]});
    board["outline"] = json!({"vertices":[],"closed":true});
    board["pads"] = json!({});
    for field in ["rules", "keepouts", "dimensions", "texts"] {
        board[field] = json!([]);
    }
    std::fs::write(path, serde_json::to_vec_pretty(&board).unwrap()).unwrap();
    root
}
fn cli(args: &[String]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_datum-eda"))
        .args(["--format", "json"])
        .args(args)
        .env_remove("DATUM_COMMIT_SOURCE")
        .output()
        .unwrap()
}
fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
fn model(root: &Path) -> DesignModel {
    ProjectResolver::new(root).resolve().unwrap()
}
fn raw(model: &DesignModel, id: Uuid) -> Value {
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == model.objects[&id].source_shard_id)
        .unwrap();
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()["symbols"][id.to_string()]
    .clone()
}
fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, out);
            } else {
                out.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, &mut out);
    out
}
fn request(root: &Path, id: Uuid, field: &str, value: &str) -> Value {
    let observed = success(cli(&[
        "journal".into(),
        "list".into(),
        root.display().to_string(),
    ]));
    json!({"path":root,"symbol":id,"field":field,"value":value,
        "expected_model_revision":observed["model_revision"],
        "expected_tip_transaction":observed["transactions"].as_array().unwrap().last()
            .map(|entry|entry["transaction_id"].clone()).unwrap_or(json!("none"))})
}
fn edit(request: &Value) -> Output {
    let mut args = vec!["project".into(), "set-symbol-field".into()];
    for (key, flag) in [
        ("path", "root"),
        ("symbol", "symbol"),
        ("field", "field"),
        ("expected_model_revision", "expected-model-revision"),
        ("expected_tip_transaction", "expected-tip-transaction"),
        ("value", "value"),
    ] {
        if let Some(value) = request.get(key) {
            args.extend([format!("--{flag}"), value.as_str().unwrap().into()]);
        }
    }
    if request["clear"] == true {
        args.push("--clear".into());
    }
    cli(&args)
}
fn mcp(request: &Value) -> Output {
    Command::new("python3").args(["-c", "import json,sys; from server_runtime import EngineDaemonClient; from tool_dispatch import dispatch_tool_call; print(json.dumps(dispatch_tool_call(EngineDaemonClient(), 'datum.schematic.set_symbol_field', json.loads(sys.argv[1])).result))", &request.to_string()])
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../mcp-server"))
        .env("DATUM_CLI_BIN", env!("CARGO_BIN_EXE_datum-eda")).env_remove("DATUM_COMMIT_SOURCE").output().unwrap()
}
fn history(root: &Path, command: &str) {
    let m = model(root);
    success(cli(&[
        "project".into(),
        command.into(),
        root.display().to_string(),
        "--expected-model-revision".into(),
        m.model_revision.0,
        "--expected-tip-transaction".into(),
        eda_engine::revision::transaction_tip(&m.journal)
            .unwrap()
            .to_string(),
    ]));
}

#[test]
fn schematic_symbol_fields_public_callers_preserve_history_and_policy() {
    let root = fixture();
    let placed = success(cli(&[
        "project".into(),
        "place-symbol".into(),
        root.display().to_string(),
        "--sheet".into(),
        "10000000-0000-4000-8000-000000000004".into(),
        "--lib-id".into(),
        "30000000-0000-4000-8000-000000000001".into(),
        "--reference".into(),
        "U1".into(),
        "--value".into(),
        "initial".into(),
        "--x-nm".into(),
        "0".into(),
        "--y-nm".into(),
        "0".into(),
    ]));
    let id: Uuid = serde_json::from_value(placed["symbol_uuid"].clone()).unwrap();
    let ci = model(&root)
        .component_instances
        .values()
        .find(|c| c.placed_symbol_refs.contains(&id))
        .unwrap()
        .id;
    for (field, value) in [("reference", "U9"), ("value", "updated"), ("value", "")] {
        let before = raw(&model(&root), id);
        let mut req = request(&root, id, field, value);
        if value.is_empty() {
            req.as_object_mut().unwrap().remove("value");
            req["clear"] = json!(true);
        }
        let original = files(&root);
        let refused = mcp(&req);
        assert!(!refused.status.success());
        assert!(String::from_utf8_lossy(&refused.stderr).contains("proposal_required"));
        assert_eq!(files(&root), original);
        success(edit(&req));
        let mut expected = before.clone();
        expected[field] = json!(value);
        assert_eq!(raw(&model(&root), id), expected);
        assert_eq!(
            model(&root).component_instances[&ci].placed_symbol_refs,
            vec![id]
        );
        history(&root, "undo");
        assert_eq!(raw(&model(&root), id), before);
        history(&root, "redo");
        assert_eq!(raw(&model(&root), id), expected);
    }
    // Existing canonical component deletion creates the unbound caller case.
    success(cli(&[
        "project".into(),
        "delete-component-instance".into(),
        root.display().to_string(),
        "--component-instance".into(),
        ci.to_string(),
    ]));
    for (field, value) in [
        ("reference", "U10"),
        ("value", "MCP literal"),
        ("value", ""),
    ] {
        let before = raw(&model(&root), id);
        let mut req = request(&root, id, field, value);
        if value.is_empty() {
            req.as_object_mut().unwrap().remove("value");
            req["clear"] = json!(true);
        }
        success(mcp(&req));
        let mut expected = before.clone();
        expected[field] = json!(value);
        let after = model(&root);
        assert_eq!(raw(&after, id), expected);
        assert_eq!(
            after.journal.last().unwrap().provenance.source,
            eda_engine::substrate::CommitSource::Tool
        );
        history(&root, "undo");
        assert_eq!(raw(&model(&root), id), before);
        history(&root, "redo");
        assert_eq!(raw(&model(&root), id), expected);
    }
    let good = request(&root, id, "value", "refused");
    let original = files(&root);
    for (key, value) in [
        ("expected_model_revision", json!("stale")),
        ("expected_model_revision", json!("")),
        ("expected_tip_transaction", json!("none")),
        ("expected_tip_transaction", json!(Uuid::nil())),
        ("expected_tip_transaction", json!("")),
        ("expected_tip_transaction", json!("malformed")),
    ] {
        let mut req = good.clone();
        req[key] = value;
        assert!(!edit(&req).status.success());
        assert!(!mcp(&req).status.success());
        assert_eq!(files(&root), original);
    }
    for key in ["expected_model_revision", "expected_tip_transaction"] {
        let mut req = good.clone();
        req.as_object_mut().unwrap().remove(key);
        assert!(!edit(&req).status.success());
        assert!(!mcp(&req).status.success());
        assert_eq!(files(&root), original);
    }
    let mut invalid = good.clone();
    invalid["field"] = json!("reference");
    invalid["value"] = json!("");
    assert!(!edit(&invalid).status.success());
    assert!(!mcp(&invalid).status.success());
    assert_eq!(files(&root), original);
    let mut conflict = good.clone();
    conflict["clear"] = json!(true);
    assert!(!edit(&conflict).status.success());
    assert!(!mcp(&conflict).status.success());
    assert_eq!(files(&root), original);
    for key in ["expected_model_revision", "expected_tip_transaction"] {
        let mut null = good.clone();
        null[key] = Value::Null;
        assert!(!mcp(&null).status.success());
        assert_eq!(files(&root), original);
    }
    // Isolate the tip guard using a current revision and an older real tip.
    let before_history = raw(&model(&root), id);
    history(&root, "undo");
    history(&root, "redo");
    assert_eq!(raw(&model(&root), id), before_history);
    let mut stale_tip = request(&root, id, "value", "refused");
    stale_tip["expected_tip_transaction"] = good["expected_tip_transaction"].clone();
    let original = files(&root);
    let refused = edit(&stale_tip);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("transaction tip mismatch"));
    let refused = mcp(&stale_tip);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("transaction tip mismatch"));
    assert_eq!(files(&root), original);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn schematic_symbol_fields_explicit_empty_tip_and_opaque_payload() {
    for caller in [edit as fn(&Value) -> Output, mcp] {
        let root = fixture();
        let id = Uuid::new_v4();
        let path = root.join("schematic/sheets/main.json");
        let mut sheet: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        // Seed an authored source before its first resolution or journal write.
        sheet["symbols"][id.to_string()] = json!({"uuid":id,"reference":"R1","value":"10k",
            "part":null,"entity":null,"gate":null,"lib_id":null,"pins":[],
            "fields":[{"uuid":Uuid::new_v4(),"key":"Reference","value":"opaque","visible":false,"position":null,"extension":[1,2]}],
            "position":{"x":0,"y":0},"rotation":0,"mirrored":false,
            "unit_selection":null,"display_mode":"LibraryDefault","pin_overrides":[],
            "hidden_power_behavior":"SourceDefinedImplicit","extension":{"retained":true}});
        std::fs::write(&path, serde_json::to_vec_pretty(&sheet).unwrap()).unwrap();
        let good = request(&root, id, "value", "4k7");
        assert_eq!(good["expected_tip_transaction"], "none");
        let before = raw(&model(&root), id);
        let original = files(&root);
        let mut bad = good.clone();
        bad["expected_tip_transaction"] = json!(Uuid::new_v4());
        assert!(!caller(&bad).status.success());
        assert_eq!(files(&root), original);
        success(caller(&good));
        let mut expected = before.clone();
        expected["value"] = json!("4k7");
        assert_eq!(raw(&model(&root), id), expected);
        history(&root, "undo");
        assert_eq!(raw(&model(&root), id), before);
        history(&root, "redo");
        assert_eq!(raw(&model(&root), id), expected);
        std::fs::remove_dir_all(root).unwrap();
    }
}
