//! New Footprint entrypoint reference refusals over existing canonical definitions.
use super::*;
use eda_engine::api::native_write::{
    WriteProvenance, commit_prepared,
    library::{self, PoolLibraryObjectTarget, PoolLibraryOperationSpec},
};
use eda_engine::substrate::{CommitSource, ProjectResolver};
use std::{collections::BTreeMap, path::Path};

fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
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
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

pub(super) fn verify(engine: &mut Engine, root: &Path, stack: uuid::Uuid) {
    // Seed the existing named Package through its existing canonical facade.
    let package = uuid::Uuid::new_v4();
    let footprint = uuid::Uuid::new_v4();
    let pad = uuid::Uuid::new_v4();
    let mut model = ProjectResolver::new(root).resolve().unwrap();
    let prepared = library::build_pool_library_write(
        &model,
        WriteProvenance::new(
            "datum-gui",
            CommitSource::Manual,
            "Existing reference fixture",
        ),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "packages", package),
            object: library::pool_package_payload(package, "Named body", json!({})),
        }],
    )
    .unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    let mut fields = json!({"name":"Named footprint","package":package,"pads":{pad.to_string():{"uuid":pad,"name":"1","position":{"x":0,"y":0},"padstack":stack,"layer":1}}});
    for edit in [false, true] {
        let described = describe(engine, 20, root);
        let guarded = json!({"project_root":root,"verb":if edit {"datum.library.edit"} else {"datum.library.create"},"params":{"kind":"footprints","object_id":footprint,"pool_path":"pool","fields":fields},"actor":"datum-gui","source":"manual","reason":"Reference refusal control","expected_model_revision":described["model_revision"],"expected_transaction_tip":described["transaction_tip"]});
        let before = files(root);
        for (role, bad) in [
            ("package", uuid::Uuid::new_v4()),
            ("package", stack),
            ("padstack", uuid::Uuid::new_v4()),
            ("padstack", package),
        ] {
            let mut invalid = guarded.clone();
            if role == "package" {
                invalid["params"]["fields"]["package"] = json!(bad);
            } else {
                invalid["params"]["fields"]["pads"][pad.to_string()]["padstack"] = json!(bad);
            }
            let response = dispatch_request(engine, native_request(21, "native.write", invalid));
            let error = response.error.expect("invalid reference must refuse");
            assert_eq!(
                error.code, -32060,
                "reference validation must reach the canonical builder"
            );
            assert!(
                error
                    .message
                    .contains(&format!("missing pool {role} {bad}")),
                "reference-specific refusal required: {error:?}"
            );
            assert_eq!(
                files(root),
                before,
                "invalid {role} cannot change any source or journal bytes"
            );
        }
        if !edit {
            let response = dispatch_request(engine, native_request(22, "native.write", guarded));
            assert!(
                response.error.is_none(),
                "valid named Package and Padstack must create: {response:?}"
            );
            // Edits target the same retained pad identity and source definition.
            fields = json!({"package":package,"pads":{pad.to_string():{"padstack":stack}}});
        }
    }
    let reopened = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(
        reopened.journal.len(),
        3,
        "only Stack, Package and valid Footprint creation commit"
    );
}
