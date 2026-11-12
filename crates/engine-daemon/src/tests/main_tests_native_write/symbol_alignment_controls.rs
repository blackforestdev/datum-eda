//! Symbol alignment require an explicit current revision and journal tip.
use super::*;
use std::collections::BTreeMap;
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
fn symbol_alignment_transport_refuses_missing_and_stale_guards_without_writes() {
    let mut engine = Engine::new().unwrap();
    let root = temp_native_project("symbol_alignment_guards");
    let before = describe(&mut engine, 1, &root);
    let original = preimage(&root);
    for verb in ["datum.schematic.align_symbols"] {
        for case in 0..4 {
            let mut params = json!({"project_root":root,"verb":verb,"params":{},
                "source":"manual","reason":"Explicit Symbol alignment preview",
                "expected_model_revision":before["model_revision"],"expected_transaction_tip":before["transaction_tip"]});
            match case {
                0 => {
                    params
                        .as_object_mut()
                        .unwrap()
                        .remove("expected_model_revision");
                }
                1 => {
                    params
                        .as_object_mut()
                        .unwrap()
                        .remove("expected_transaction_tip");
                }
                2 => params["expected_model_revision"] = json!("stale"),
                _ => params["expected_transaction_tip"] = json!(uuid::Uuid::new_v4()),
            }
            let response = dispatch_request(&mut engine, native_request(2, "native.write", params));
            let error = response.error.as_ref().expect("guard refusal");
            let (code, message) = match case {
                0 => (-32602, "requires expected_model_revision"),
                1 => (-32602, "requires explicit expected_transaction_tip"),
                2 => (-32061, "stale expected_model_revision"),
                _ => (-32061, "stale expected transaction tip"),
            };
            assert_eq!(error.code, code, "{verb} case {case}: {response:?}");
            assert!(
                error.message.contains(message),
                "{verb} case {case}: {response:?}"
            );
            assert_eq!(
                preimage(&root),
                original,
                "{verb} case {case}: complete file preimage"
            );
            let after = describe(&mut engine, 3, &root);
            assert_eq!(after["journal_len"], before["journal_len"]);
            assert_eq!(after["model_revision"], before["model_revision"]);
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
