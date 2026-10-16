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
