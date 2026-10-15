//! Ordinary manual library commands retain their accepted inputs and refusal history.

use super::main_tests_project_pool_footprint::{
    FOOTPRINT_ID, FOOTPRINT_PAD_ID, PACKAGE_ID, create_footprint_fixture,
};
use super::main_tests_project_pool_library::{query_pool_object_payload, unique_project_root};
use super::*;
use eda_engine::substrate::{Operation, ProjectResolver};

fn command(root: &Path, verb: &str, args: &[&str]) -> Result<String> {
    let mut argv = vec![
        "datum-eda",
        "--format",
        "json",
        "project",
        verb,
        root.to_str().unwrap(),
    ];
    argv.extend_from_slice(args);
    execute(Cli::try_parse_from(argv).expect("manual command should parse"))
}

fn journal(root: &Path) -> Vec<u8> {
    std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap()
}

#[test]
fn library_definition_manual_replacement_and_reopen_history() {
    let root = unique_project_root("definition-manual-history");
    create_native_project(&root, Some("Manual definitions".to_string())).unwrap();
    create_footprint_fixture(&root);
    let padstack = Uuid::new_v4().to_string();
    // Null aperture, positive drill, and blank names are existing accepted input.
    command(
        &root,
        "create-pool-padstack",
        &["--padstack", &padstack, "--name", "", "--drill-nm", "1"],
    )
    .unwrap();
    let payload =
        query_pool_object_payload(&root, "padstacks", Uuid::parse_str(&padstack).unwrap());
    assert_eq!(payload["aperture"], serde_json::Value::Null);
    assert_eq!(payload["drill_nm"], 1);
    assert!(payload.get("layer_span").is_none());

    let footprint = Uuid::new_v4().to_string();
    let package = PACKAGE_ID.to_string();
    command(
        &root,
        "create-pool-footprint",
        &[
            "--footprint",
            &footprint,
            "--package",
            &package,
            "--name",
            "",
        ],
    )
    .unwrap();
    let pad = FOOTPRINT_PAD_ID.to_string();
    let args = [
        "--footprint",
        &footprint,
        "--pad",
        &pad,
        "--padstack",
        &padstack,
        "--pad-name",
        " 1 ",
        "--layer",
        "17",
        "--x-nm",
        "123",
        "--y-nm",
        "456",
    ];
    command(&root, "set-pool-footprint-pad", &args).unwrap();
    let footprint_id = Uuid::parse_str(&footprint).unwrap();
    let first = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(first["name"], "");
    assert_eq!(first["pads"][&pad]["layer"], 17);
    assert_eq!(first["pads"][&pad]["name"], " 1 ");
    let mut replacement = args;
    let name_index = args.iter().position(|arg| *arg == "--pad-name").unwrap() + 1;
    replacement[name_index] = "2";
    let before = journal(&root);
    command(&root, "set-pool-footprint-pad", &replacement).unwrap();
    assert_eq!(
        journal(&root).split(|&b| b == b'\n').count(),
        before.split(|&b| b == b'\n').count() + 1
    );
    let second = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(second["pads"].as_object().unwrap().len(), 1);
    assert_eq!(second["pads"][&pad]["name"], "2");
    command(&root, "undo", &[]).unwrap();
    assert_eq!(
        query_pool_object_payload(&root, "footprints", footprint_id),
        first
    );
    command(&root, "redo", &[]).unwrap();
    // Query resolves the persisted Project afresh; no retained model masks drift.
    assert_eq!(
        query_pool_object_payload(&root, "footprints", footprint_id),
        second
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn library_definition_manual_refusals_preserve_exact_errors_and_journal() {
    let root = unique_project_root("definition-manual-refusal");
    create_native_project(&root, Some("Manual refusals".to_string())).unwrap();
    create_footprint_fixture(&root);
    let id = Uuid::new_v4().to_string();
    let initial_journal = journal(&root);
    let initial_footprint = query_pool_object_payload(&root, "footprints", FOOTPRINT_ID);
    for (options, expected) in [
        (
            vec!["--aperture", "unknown", "--drill-nm", "0"],
            "padstack drill-nm must be positive",
        ),
        (
            vec!["--diameter-nm", "1"],
            "padstack aperture dimensions require --aperture circle or rect",
        ),
        (
            vec!["--aperture", "rect", "--diameter-nm", "1"],
            "rect padstack aperture does not accept diameter-nm",
        ),
        (
            vec!["--aperture", "circle"],
            "padstack diameter-nm is required",
        ),
        (
            vec!["--aperture", "rect", "--width-nm", "1", "--height-nm", "0"],
            "padstack height-nm must be positive",
        ),
    ] {
        let mut args = vec!["--padstack", &id, "--name", "Entered"];
        args.extend(options);
        let error = command(&root, "create-pool-padstack", &args).unwrap_err();
        assert_eq!(format!("{error:#}"), expected);
        assert_eq!(journal(&root), initial_journal);
        assert!(!root.join(format!("pool/padstacks/{id}.json")).exists());
    }
    let footprint = FOOTPRINT_ID.to_string();
    let pad = FOOTPRINT_PAD_ID.to_string();
    let wrong_kind = PACKAGE_ID.to_string();
    for (name, layer, padstack, expected) in [
        (
            " \t",
            "0",
            id.as_str(),
            "footprint pad name must be non-empty".to_string(),
        ),
        (
            "1",
            "0",
            id.as_str(),
            "footprint pad layer must be positive".to_string(),
        ),
        ("1", "1", id.as_str(), format!("missing pool padstack {id}")),
        (
            "1",
            "1",
            wrong_kind.as_str(),
            format!("missing pool padstack {wrong_kind}"),
        ),
    ] {
        let error = command(
            &root,
            "set-pool-footprint-pad",
            &[
                "--footprint",
                &footprint,
                "--pad",
                &pad,
                "--padstack",
                padstack,
                "--pad-name",
                name,
                "--layer",
                layer,
            ],
        )
        .unwrap_err();
        assert_eq!(format!("{error:#}"), expected);
        assert_eq!(journal(&root), initial_journal);
        assert_eq!(
            query_pool_object_payload(&root, "footprints", FOOTPRINT_ID),
            initial_footprint
        );
    }
    let error = command(
        &root,
        "create-pool-footprint",
        &[
            "--footprint",
            &id,
            "--package",
            &footprint,
            "--name",
            "Entered",
        ],
    )
    .unwrap_err();
    assert_eq!(
        format!("{error:#}"),
        format!("missing pool package {footprint}")
    );
    assert_eq!(journal(&root), initial_journal);
    std::fs::remove_dir_all(root).unwrap();
}

fn footprint_json_edit(root: &Path, payload: &serde_json::Value) -> Result<String> {
    let source = root.join("footprint-replacement.json");
    std::fs::write(&source, serde_json::to_vec(payload).unwrap()).unwrap();
    command(
        root,
        "set-pool-library-object",
        &[
            "--kind",
            "footprints",
            "--object",
            &FOOTPRINT_ID.to_string(),
            "--from-json",
            source.to_str().unwrap(),
        ],
    )
}

fn footprint_edit_project(label: &str) -> PathBuf {
    let root = crate::main_tests::development_workspace::path("footprint-reference-audit")
        .join(format!("{label}-{}", Uuid::new_v4()));
    create_native_project(&root, Some(label.to_string())).unwrap();
    create_footprint_fixture(&root);
    command(
        &root,
        "set-pool-footprint-pad",
        &[
            "--footprint",
            &FOOTPRINT_ID.to_string(),
            "--pad",
            &FOOTPRINT_PAD_ID.to_string(),
            "--padstack",
            &super::main_tests_project_pool_footprint::FOOTPRINT_PADSTACK_ID.to_string(),
        ],
    )
    .unwrap();
    root
}

#[test]
fn library_definition_footprint_json_refusals_preserve_shape_errors_and_history() {
    use super::main_tests_project_pool_footprint::FOOTPRINT_PADSTACK_ID;
    let root = footprint_edit_project("json-reference-refusal");
    let second_pad = Uuid::new_v4().to_string();
    command(
        &root,
        "set-pool-footprint-pad",
        &[
            "--footprint",
            &FOOTPRINT_ID.to_string(),
            "--pad",
            &second_pad,
            "--padstack",
            &FOOTPRINT_PADSTACK_ID.to_string(),
        ],
    )
    .unwrap();
    let original = query_pool_object_payload(&root, "footprints", FOOTPRINT_ID);
    let path = root.join(format!("pool/footprints/{FOOTPRINT_ID}.json"));
    let initial_bytes = std::fs::read(&path).unwrap();
    let initial_journal = journal(&root);
    let before = ProjectResolver::new(&root).resolve().unwrap();
    let missing = Uuid::new_v4();
    for (field, target, expected) in [
        (
            "package",
            missing,
            format!("missing pool package {missing}"),
        ),
        (
            "package",
            FOOTPRINT_PADSTACK_ID,
            format!("missing pool package {FOOTPRINT_PADSTACK_ID}"),
        ),
        (
            "padstack",
            missing,
            format!("missing pool padstack {missing}"),
        ),
        (
            "padstack",
            PACKAGE_ID,
            format!("missing pool padstack {PACKAGE_ID}"),
        ),
    ] {
        let mut payload = original.clone();
        if field == "package" {
            payload["package"] = serde_json::json!(target);
        } else {
            // The first pad remains valid: every pad's link must be checked.
            payload["pads"][&second_pad]["padstack"] = serde_json::json!(target);
        }
        let error = footprint_json_edit(&root, &payload).unwrap_err();
        assert_eq!(format!("{error:#}"), expected);
        assert_eq!(std::fs::read(&path).unwrap(), initial_bytes);
        assert_eq!(journal(&root), initial_journal);
        let after = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(after.model_revision, before.model_revision);
        for (id, object) in &before.objects {
            assert_eq!(after.objects[id].object_revision, object.object_revision);
        }
        assert_eq!(
            query_pool_object_payload(&root, "footprints", FOOTPRINT_ID),
            original
        );
    }
    // Old UUID/schema/shape refusals precede the newly checked broken link.
    for (fault, expected) in [
        ("uuid", "does not match --object"),
        (
            "schema_version",
            "unsupported pool library object schema_version 2; expected 1",
        ),
        ("courtyard", "invalid pool library footprints object"),
    ] {
        let mut payload = original.clone();
        payload["package"] = serde_json::json!(missing);
        payload[fault] = match fault {
            "uuid" => serde_json::json!(missing),
            "schema_version" => serde_json::json!(2),
            _ => serde_json::Value::Null,
        };
        let error = footprint_json_edit(&root, &payload).unwrap_err();
        assert!(format!("{error:#}").contains(expected), "{error:#}");
        assert_eq!(std::fs::read(&path).unwrap(), initial_bytes);
        assert_eq!(journal(&root), initial_journal);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn library_definition_footprint_json_valid_edits_relink_reopen_undo_redo() {
    let root = footprint_edit_project("json-reference-history");
    let package = Uuid::new_v4().to_string();
    let padstack = Uuid::new_v4().to_string();
    command(
        &root,
        "create-pool-package",
        &["--package", &package, "--name", "Alternate body"],
    )
    .unwrap();
    command(
        &root,
        "create-pool-padstack",
        &["--padstack", &padstack, "--name", "Alternate copper"],
    )
    .unwrap();
    let package_before = query_pool_object_payload(&root, "packages", PACKAGE_ID);
    let mut payload = query_pool_object_payload(&root, "footprints", FOOTPRINT_ID);
    payload["name"] = serde_json::json!("Renamed land pattern");
    payload["extension"] = serde_json::json!({"retained": [1, 2]});
    payload["pads"][FOOTPRINT_PAD_ID.to_string()]["extension"] = serde_json::json!("pad metadata");
    footprint_json_edit(&root, &payload).unwrap();
    assert_eq!(
        query_pool_object_payload(&root, "footprints", FOOTPRINT_ID),
        payload
    );
    // An unchanged-reference replacement still follows the existing Set lifecycle.
    footprint_json_edit(&root, &payload).unwrap();
    let before = ProjectResolver::new(&root).resolve().unwrap();
    let prior_journal = journal(&root);
    let previous = payload.clone();
    payload["package"] = serde_json::json!(package);
    payload["pads"][FOOTPRINT_PAD_ID.to_string()]["padstack"] = serde_json::json!(padstack);
    footprint_json_edit(&root, &payload).unwrap();
    let after = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(after.journal.len(), before.journal.len() + 1);
    assert_eq!(
        after.objects[&FOOTPRINT_ID].object_revision.0,
        before.objects[&FOOTPRINT_ID].object_revision.0 + 1
    );
    assert_eq!(
        after.objects[&FOOTPRINT_PAD_ID].object_revision.0,
        before.objects[&FOOTPRINT_PAD_ID].object_revision.0 + 1
    );
    assert_eq!(
        after.objects[&PACKAGE_ID].object_revision,
        before.objects[&PACKAGE_ID].object_revision
    );
    assert_ne!(journal(&root), prior_journal);
    assert_eq!(after.journal.last().unwrap().operations.len(), 1);
    assert!(matches!(
        after.journal.last().unwrap().operations[0],
        Operation::SetPoolLibraryObject { .. }
    ));
    assert_eq!(
        query_pool_object_payload(&root, "footprints", FOOTPRINT_ID),
        payload
    );
    assert_eq!(
        query_pool_object_payload(&root, "packages", PACKAGE_ID),
        package_before
    );
    command(&root, "undo", &[]).unwrap();
    assert_eq!(
        query_pool_object_payload(&root, "footprints", FOOTPRINT_ID),
        previous
    );
    let undone = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(undone.journal.len(), after.journal.len() + 1);
    command(&root, "redo", &[]).unwrap();
    assert_eq!(
        query_pool_object_payload(&root, "footprints", FOOTPRINT_ID),
        payload
    );
    let redone = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(redone.journal.len(), undone.journal.len() + 1);
    assert_eq!(
        query_pool_object_payload(&root, "packages", PACKAGE_ID),
        package_before
    );
    std::fs::remove_dir_all(root).unwrap();
}
