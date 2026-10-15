//! Ordinary generic Footprint creation retains canonical admission and history.
use super::main_tests_project_pool_footprint::{
    FOOTPRINT_ID, FOOTPRINT_PAD_ID, FOOTPRINT_PADSTACK_ID, PACKAGE_ID, create_footprint_fixture,
};
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
    execute(Cli::try_parse_from(argv).unwrap())
}
fn project(label: &str) -> PathBuf {
    let root = crate::main_tests::development_workspace::path("footprint-reference-audit")
        .join(format!("{label}-{}", Uuid::new_v4()));
    create_native_project(&root, Some(label.to_string())).unwrap();
    root
}
fn journal(root: &Path) -> Vec<u8> {
    std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap_or_default()
}
fn generic_create(
    root: &Path,
    pool: &str,
    id: Uuid,
    payload: &serde_json::Value,
) -> Result<String> {
    let file = root.join("create-footprint.json");
    std::fs::write(&file, serde_json::to_vec(payload).unwrap()).unwrap();
    command(
        root,
        "create-pool-library-object",
        &[
            "--pool",
            pool,
            "--kind",
            "footprints",
            "--object",
            &id.to_string(),
            "--from-json",
            file.to_str().unwrap(),
        ],
    )
}
fn template(root: &Path, id: Uuid, pad: Uuid) -> serde_json::Value {
    create_footprint_fixture(root);
    command(
        root,
        "set-pool-footprint-pad",
        &[
            "--footprint",
            &FOOTPRINT_ID.to_string(),
            "--pad",
            &FOOTPRINT_PAD_ID.to_string(),
            "--padstack",
            &FOOTPRINT_PADSTACK_ID.to_string(),
        ],
    )
    .unwrap();
    let mut payload = super::main_tests_project_pool_library::query_pool_object_payload(
        root,
        "footprints",
        FOOTPRINT_ID,
    );
    payload["uuid"] = serde_json::json!(id);
    let mut record = payload["pads"][FOOTPRINT_PAD_ID.to_string()].clone();
    record["uuid"] = serde_json::json!(pad);
    payload["pads"] = serde_json::json!({pad.to_string(): record});
    payload
}

#[test]
fn library_definition_footprint_create_refusals_keep_admission_precedence() {
    let root = project("create-reference-refusals");
    let id = Uuid::new_v4();
    let pad = Uuid::new_v4();
    let missing = Uuid::new_v4();
    let original = template(&root, id, pad);
    let before = ProjectResolver::new(&root).resolve().unwrap();
    let before_journal = journal(&root);
    let before_manifest = std::fs::read(root.join("project.json")).unwrap();
    let existing = root.join(format!("pool/footprints/{FOOTPRINT_ID}.json"));
    let before_existing = std::fs::read(&existing).unwrap();
    let mut cases = Vec::new();
    for (field, target, label) in [
        ("package", missing, "package"),
        ("package", FOOTPRINT_PADSTACK_ID, "package"),
        ("padstack", missing, "padstack"),
        ("padstack", PACKAGE_ID, "padstack"),
    ] {
        let mut payload = original.clone();
        if field == "package" {
            payload["package"] = serde_json::json!(target);
        } else {
            payload["pads"][pad.to_string()]["padstack"] = serde_json::json!(target);
        }
        cases.push((
            id,
            "pool",
            payload,
            format!("missing pool {label} {target}"),
            true,
        ));
    }
    // Existing canonical admission errors win over an additional broken link.
    for fault in ["root", "leaf", "nested", "uuid", "schema", "shape", "path"] {
        let mut payload = original.clone();
        payload["package"] = serde_json::json!(missing);
        let mut target = id;
        let mut pool = "pool";
        let expected = match fault {
            "root" => {
                target = FOOTPRINT_ID;
                payload["uuid"] = serde_json::json!(target);
                "pool source objects: source UUID already exists".to_string()
            }
            "leaf" => {
                payload["pads"][pad.to_string()]["uuid"] = serde_json::json!(FOOTPRINT_PAD_ID);
                "pool source objects: source UUID already exists".to_string()
            }
            "nested" => {
                payload["extension"] = serde_json::json!({"uuid": PACKAGE_ID});
                "pool source objects: source UUID already exists".to_string()
            }
            "uuid" => {
                payload["uuid"] = serde_json::json!(missing);
                "does not match --object".to_string()
            }
            "schema" => {
                payload["schema_version"] = serde_json::json!(2);
                "unsupported pool library object schema_version 2; expected 1".to_string()
            }
            "shape" => {
                payload["courtyard"] = serde_json::Value::Null;
                "invalid pool library footprints object".to_string()
            }
            "path" => {
                pool = "other_pool";
                "source shard ownership mismatch".to_string()
            }
            _ => unreachable!(),
        };
        cases.push((target, pool, payload, expected, false));
    }
    for (target, pool, payload, expected, exact) in cases {
        let error = generic_create(&root, pool, target, &payload).unwrap_err();
        if exact {
            assert_eq!(format!("{error:#}"), expected);
        } else {
            assert!(format!("{error:#}").contains(&expected), "{error:#}");
        }
        assert_eq!(journal(&root), before_journal);
        assert_eq!(
            std::fs::read(root.join("project.json")).unwrap(),
            before_manifest
        );
        assert_eq!(std::fs::read(&existing).unwrap(), before_existing);
        assert!(!root.join(format!("pool/footprints/{id}.json")).exists());
        let after = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(after.model_revision, before.model_revision);
        assert_eq!(after.objects.len(), before.objects.len());
        for (id, object) in &before.objects {
            assert_eq!(after.objects[id].object_revision, object.object_revision);
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn library_definition_footprint_create_missing_refs_do_not_register_pool() {
    let root = project("create-reference-no-pool");
    let id = Uuid::new_v4();
    let package = Uuid::new_v4();
    let payload = eda_engine::api::native_write::library_footprint::pool_footprint_payload(
        id, package, "Manual",
    );
    let before_manifest = std::fs::read(root.join("project.json")).unwrap();
    let before = ProjectResolver::new(&root).resolve().unwrap();
    let error = generic_create(&root, "pool", id, &payload).unwrap_err();
    assert_eq!(
        format!("{error:#}"),
        format!("missing pool package {package}")
    );
    assert!(journal(&root).is_empty());
    assert_eq!(
        std::fs::read(root.join("project.json")).unwrap(),
        before_manifest
    );
    assert!(!root.join(format!("pool/footprints/{id}.json")).exists());
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .model_revision,
        before.model_revision
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn library_definition_footprint_create_valid_reopen_undo_redo_keeps_payload() {
    let root = project("create-reference-history");
    let id = Uuid::new_v4();
    let pad = Uuid::new_v4();
    let mut payload = template(&root, id, pad);
    payload["extension"] = serde_json::json!({"retain": [1, 2]});
    payload["pads"][pad.to_string()]["extension"] = serde_json::json!("manual metadata");
    let before = ProjectResolver::new(&root).resolve().unwrap();
    generic_create(&root, "pool", id, &payload).unwrap();
    let created = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(created.journal.len(), before.journal.len() + 1);
    assert_eq!(created.objects[&id].object_revision.0, 0);
    assert_eq!(created.objects[&pad].object_revision.0, 0);
    assert_eq!(created.journal.last().unwrap().operations.len(), 1);
    assert!(matches!(
        created.journal.last().unwrap().operations[0],
        Operation::CreatePoolLibraryObject { .. }
    ));
    for source in [PACKAGE_ID, FOOTPRINT_PADSTACK_ID] {
        assert_eq!(
            created.objects[&source].object_revision,
            before.objects[&source].object_revision
        );
    }
    assert_eq!(
        super::main_tests_project_pool_library::query_pool_object_payload(&root, "footprints", id),
        payload
    );
    command(&root, "undo", &[]).unwrap();
    let undone = ProjectResolver::new(&root).resolve().unwrap();
    assert!(!undone.objects.contains_key(&id) && !undone.objects.contains_key(&pad));
    assert!(!root.join(format!("pool/footprints/{id}.json")).exists());
    assert_eq!(undone.journal.len(), created.journal.len() + 1);
    command(&root, "redo", &[]).unwrap();
    let redone = ProjectResolver::new(&root).resolve().unwrap();
    assert!(redone.objects.contains_key(&id) && redone.objects.contains_key(&pad));
    assert_eq!(redone.journal.len(), undone.journal.len() + 1);
    assert_eq!(
        super::main_tests_project_pool_library::query_pool_object_payload(&root, "footprints", id),
        payload
    );
    let validation: serde_json::Value =
        serde_json::from_str(&command(&root, "validate", &[]).unwrap()).unwrap();
    assert_eq!(validation["valid"], true);
    std::fs::remove_dir_all(root).unwrap();
}
