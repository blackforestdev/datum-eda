//! Focused real JSON-RPC dispatch probe, not a new optional-guard policy.
use super::*;
use std::collections::BTreeMap;

fn copy_tree(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let dest = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &dest);
        } else {
            std::fs::copy(entry.path(), dest).unwrap();
        }
    }
}

fn preimage(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, path: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().into(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn native_parametric_fields_dispatch_preserves_guards_and_proposal_policy() {
    let root = std::env::temp_dir().join(format!("datum_field_transport_{}", uuid::Uuid::new_v4()));
    copy_tree(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../test-harness/testdata/library/native_authored_baseline_v1"),
        &root,
    );
    let mut engine = Engine::new().unwrap();
    let before = describe(&mut engine, 1, &root);
    let placed = dispatch_request(
        &mut engine,
        native_request(
            2,
            "native.write",
            json!({
                "project_root":root, "verb":"datum.schematic.place_symbol",
                "params":{"sheet":"10000000-0000-4000-8000-000000000004",
                "library_symbol":"30000000-0000-4000-8000-000000000001",
                "reference":"U1", "value":"initial", "x_nm":0, "y_nm":0},
                "source":"manual", "reason":"Place field caller fixture",
                "expected_model_revision":before["model_revision"],
                "expected_transaction_tip":before["transaction_tip"]
            }),
        ),
    );
    assert!(placed.error.is_none(), "{placed:?}");
    let symbol = placed.result.unwrap()["primary_object_id"].clone();
    let id: uuid::Uuid = serde_json::from_value(symbol.clone()).unwrap();
    let initial = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    assert!(
        initial
            .component_instances
            .values()
            .any(|c| c.placed_symbol_refs.contains(&id))
    );
    for patch in [
        json!({"field":"reference","value":"U9"}),
        json!({"field":"value","value":"updated"}),
        json!({"field":"value","clear":true}),
    ] {
        let current = describe(&mut engine, 3, &root);
        let mut params = patch.clone();
        params["symbol"] = symbol.clone();
        params["instance_path"] = json!([]);
        let request = json!({"project_root":root, "verb":"datum.schematic.set_symbol_field",
            "params":params, "source":"manual", "reason":"Edit field through native caller",
            "expected_model_revision":current["model_revision"],
            "expected_transaction_tip":current["transaction_tip"]});
        let original = preimage(&root);
        for source in [Value::Null, json!("tool"), json!("assistant")] {
            let mut automated = request.clone();
            automated["source"] = source;
            let response =
                dispatch_request(&mut engine, native_request(4, "native.write", automated));
            let error = response
                .error
                .expect("automated bound edits require Proposal");
            assert_eq!(error.code, -32060);
            assert!(error.message.contains("proposal_required"), "{error:?}");
            assert_eq!(preimage(&root), original);
        }
        let mut stale = request.clone();
        stale["expected_model_revision"] = json!("stale-revision");
        let response = dispatch_request(&mut engine, native_request(5, "native.write", stale));
        assert_eq!(response.error.unwrap().code, -32061);
        assert_eq!(preimage(&root), original);
        let mut invalid = request.clone();
        invalid["params"] =
            json!({"symbol":symbol,"instance_path":[],"field":"reference","clear":true});
        let response = dispatch_request(&mut engine, native_request(6, "native.write", invalid));
        assert!(response.error.unwrap().message.contains("nonempty"));
        assert_eq!(preimage(&root), original);
        let response = dispatch_request(&mut engine, native_request(7, "native.write", request));
        assert!(response.error.is_none(), "{response:?}");
        let reopened = eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap();
        assert_eq!(
            reopened.journal.len() as u64,
            current["journal_len"].as_u64().unwrap() + 1
        );
        let shard = reopened
            .source_shards
            .iter()
            .find(|s| s.shard_id == reopened.objects[&id].source_shard_id)
            .unwrap();
        let raw = reopened
            .materialized_source_shard_value_by_relative_path(&shard.relative_path)
            .unwrap();
        assert_eq!(
            raw["symbols"][id.to_string()][patch["field"].as_str().unwrap()],
            patch.get("value").cloned().unwrap_or(json!(""))
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "finite direct-daemon contract probe; inspect observations separately from required assertions"]
fn symbol_move_transport_contract_probe() {
    // Existing synthetic fixture supplies native pool/sheet identities only;
    // neither this setup nor its metadata approves production library content.
    let root =
        std::env::temp_dir().join(format!("datum_symbol_transport_{}", uuid::Uuid::new_v4()));
    copy_tree(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../test-harness/testdata/library/native_authored_baseline_v1"),
        &root,
    );
    let mut engine = Engine::new().unwrap();
    let before = describe(&mut engine, 1, &root);
    let placed = dispatch_request(
        &mut engine,
        native_request(
            2,
            "native.write",
            json!({
                "project_root": root, "verb": "datum.schematic.place_symbol",
                "params": {"sheet":"10000000-0000-4000-8000-000000000004",
                           "library_symbol":"30000000-0000-4000-8000-000000000001",
                           "reference":"U1", "value":"probe", "x_nm":0, "y_nm":0},
                "source":"manual", "actor":"transport-probe", "reason":"Probe fixture placement",
                "expected_model_revision":before["model_revision"],
                "expected_transaction_tip":before["transaction_tip"]
            }),
        ),
    );
    assert!(placed.error.is_none(), "{placed:?}");
    let symbol = placed.result.unwrap()["primary_object_id"].clone();
    assert!(symbol.is_string());
    let older_tip = describe(&mut engine, 3, &root)["transaction_tip"].clone();
    assert!(
        !older_tip.is_null(),
        "tip tests need an actual nonempty journal"
    );
    for (index, case) in [
        "missing_revision",
        "stale_revision",
        "missing_tip",
        "null_tip",
        "stale_tip",
        "valid",
    ]
    .into_iter()
    .enumerate()
    {
        let current = describe(&mut engine, 10, &root);
        let initial = preimage(&root);
        let mut params = json!({
            "project_root":root, "verb":"datum.schematic.move_symbol",
            "params":{"symbol":symbol,"instance_path":[],"x_nm":(index as i64+1)*1000000,"y_nm":0},
            "source":"manual", "actor":"transport-probe", "reason":format!("Symbol probe {case}"),
            "expected_model_revision":current["model_revision"],
            "expected_transaction_tip":current["transaction_tip"]
        });
        match case {
            "missing_revision" => {
                params
                    .as_object_mut()
                    .unwrap()
                    .remove("expected_model_revision");
            }
            "stale_revision" => {
                params["expected_model_revision"] = json!("not-the-current-revision");
            }
            "missing_tip" => {
                params
                    .as_object_mut()
                    .unwrap()
                    .remove("expected_transaction_tip");
            }
            "null_tip" => {
                params["expected_transaction_tip"] = Value::Null;
            }
            "stale_tip" => {
                // Older real tip if another probe committed; otherwise a
                // mismatched UUID, always with the CURRENT model revision.
                params["expected_transaction_tip"] = if older_tip == current["transaction_tip"] {
                    json!(uuid::Uuid::new_v4())
                } else {
                    older_tip.clone()
                };
            }
            _ => {}
        }
        println!("SYMBOL_TRANSPORT_REQUEST {case} {params}");
        let response = dispatch_request(&mut engine, native_request(11, "native.write", params));
        if case == "stale_revision" {
            assert_eq!(response.error.as_ref().map(|e| e.code), Some(-32061));
        } else if case == "valid" {
            assert!(response.error.is_none(), "{response:?}");
        }
        let after = describe(&mut engine, 12, &root);
        let delta =
            after["journal_len"].as_u64().unwrap() - current["journal_len"].as_u64().unwrap();
        if response.error.is_some() {
            assert_eq!(
                preimage(&root),
                initial,
                "refused {case} changed source/history"
            );
            assert_eq!(delta, 0);
        } else {
            assert_eq!(delta, 1, "accepted request must commit exactly once");
            let reopened = eda_engine::substrate::ProjectResolver::new(&root)
                .resolve()
                .unwrap();
            assert_eq!(
                reopened.journal.len() as u64,
                after["journal_len"].as_u64().unwrap()
            );
        }
        // Optional-guard outcomes are observations. No assertion requires a
        // missing/stale guard to be accepted or certifies that behavior safe.
        println!(
            "SYMBOL_TRANSPORT_OBSERVED {case} error={:?} journal_delta={delta}",
            response.error
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

/// Exercise the actual outer JSON decoder and native.write parameter dispatcher.
fn symbol_move_rpc(engine: &mut Engine, id: u64, params: Value) -> JsonRpcResponse {
    let raw = json!({"jsonrpc":"2.0", "id":id, "method":"native.write", "params":params});
    println!("SYMBOL_TIP_REQUEST {raw}");
    let decoded: JsonRpcRequest = serde_json::from_str(&raw.to_string()).unwrap();
    let response = dispatch_request(engine, decoded);
    println!(
        "SYMBOL_TIP_RESPONSE {}",
        serde_json::to_string(&response).unwrap()
    );
    response
}

#[test]
fn symbol_move_supplied_nonempty_tip_refuses_stale_intent_and_preserves_history() {
    let root = std::env::temp_dir().join(format!("datum_symbol_tip_{}", uuid::Uuid::new_v4()));
    copy_tree(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../test-harness/testdata/library/native_authored_baseline_v1"),
        &root,
    );
    let mut engine = Engine::new().unwrap();
    let before = describe(&mut engine, 1, &root);
    let placed = symbol_move_rpc(
        &mut engine,
        2,
        json!({
            "project_root":root, "verb":"datum.schematic.place_symbol",
            "params":{"sheet":"10000000-0000-4000-8000-000000000004",
            "library_symbol":"30000000-0000-4000-8000-000000000001",
            "reference":"U1", "value":"tip guard", "x_nm":0, "y_nm":0},
            "source":"manual", "reason":"Place supplied-tip fixture",
            "expected_model_revision":before["model_revision"],
            "expected_transaction_tip":before["transaction_tip"]
        }),
    );
    assert!(placed.error.is_none(), "{placed:?}");
    let symbol = placed.result.unwrap()["primary_object_id"].clone();
    let make_move = |current: &Value, x: i64| {
        json!({
            "project_root":root, "verb":"datum.schematic.move_symbol",
            "params":{"symbol":symbol,"instance_path":[],"x_nm":x,"y_nm":0},
            "source":"manual", "reason":"Move with supplied transaction tip",
            "expected_model_revision":current["model_revision"],
            "expected_transaction_tip":current["transaction_tip"]
        })
    };
    let placed_state = describe(&mut engine, 3, &root);
    let older_tip = placed_state["transaction_tip"].clone();
    assert!(older_tip.is_string());
    let seeded = symbol_move_rpc(&mut engine, 4, make_move(&placed_state, 1_000_000));
    assert!(seeded.error.is_none(), "{seeded:?}");
    let current = describe(&mut engine, 5, &root);
    assert_ne!(current["transaction_tip"], older_tip);
    println!("SYMBOL_TIP_GUARDS older_actual_tip={older_tip} current={current}");
    let original = preimage(&root);
    for case in [
        "stale_tip",
        "dry_run_stale_tip",
        "stale_revision_precedence",
    ] {
        let mut request = make_move(&current, 2_000_000);
        request["expected_transaction_tip"] = older_tip.clone();
        if case == "dry_run_stale_tip" {
            request["dry_run"] = json!(true);
        }
        if case == "stale_revision_precedence" {
            request["expected_model_revision"] = placed_state["model_revision"].clone();
        }
        let response = symbol_move_rpc(&mut engine, 6, request);
        let after = describe(&mut engine, 7, &root);
        println!(
            "SYMBOL_TIP_OBSERVED {case} before={} after={} files={}",
            current["journal_len"],
            after["journal_len"],
            original.len()
        );
        let error = response
            .error
            .expect("supplied stale intent must be refused");
        assert_eq!(error.code, -32061);
        assert!(
            error
                .message
                .contains(if case == "stale_revision_precedence" {
                    "expected_model_revision"
                } else {
                    "transaction tip"
                }),
            "{error:?}"
        );
        assert_eq!(after, current);
        assert_eq!(
            preimage(&root),
            original,
            "{case} changed authored/history preimage"
        );
    }
    let valid = symbol_move_rpc(&mut engine, 8, make_move(&current, 2_000_000));
    assert!(valid.error.is_none(), "{valid:?}");
    let moved = describe(&mut engine, 9, &root);
    assert_eq!(
        moved["journal_len"].as_u64().unwrap(),
        current["journal_len"].as_u64().unwrap() + 1
    );
    assert_ne!(moved["transaction_tip"], current["transaction_tip"]);
    let stable = preimage(&root);
    let mut repeated = make_move(&moved, 3_000_000);
    repeated["expected_transaction_tip"] = current["transaction_tip"].clone();
    let repeated = symbol_move_rpc(&mut engine, 10, repeated);
    assert_eq!(repeated.error.unwrap().code, -32061);
    assert_eq!(
        preimage(&root),
        stable,
        "repeat with fresh revision must still check supplied old tip"
    );
    let symbol_id: uuid::Uuid = serde_json::from_value(symbol).unwrap();
    for (verb, expected_x) in [
        (None, 2_000_000),
        (Some("datum.journal.undo"), 1_000_000),
        (Some("datum.journal.redo"), 2_000_000),
    ] {
        if let Some(verb) = verb {
            let state = describe(&mut engine, 11, &root);
            let response = symbol_move_rpc(
                &mut engine,
                12,
                json!({
                    "project_root":root, "verb":verb, "params":{"expected_tip":state["transaction_tip"]},
                    "source":"manual", "reason":"Supplied-tip move history proof",
                    "expected_model_revision":state["model_revision"],
                    "expected_transaction_tip":state["transaction_tip"]
                }),
            );
            assert!(response.error.is_none(), "{response:?}");
        }
        let reopened = eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap();
        let source =
            eda_engine::substrate::ElectricalSelectionSnapshot::capture(&reopened).unwrap();
        let schematic = source.schematic_source().unwrap();
        let placed = &schematic
            .sheets
            .values()
            .find_map(|s| s.symbols.get(&symbol_id))
            .unwrap();
        assert_eq!(
            placed.position,
            eda_engine::ir::geometry::Point::new(expected_x, 0)
        );
        assert_eq!(
            reopened.journal.len() as u64,
            describe(&mut engine, 13, &root)["journal_len"]
                .as_u64()
                .unwrap()
        );
        println!(
            "SYMBOL_TIP_COLD verb={verb:?} x={expected_x} journal_len={}",
            reopened.journal.len()
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
