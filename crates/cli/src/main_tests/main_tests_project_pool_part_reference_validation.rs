//! Ordinary generic Part writes enforce existing links without changing admission/history.
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
fn payload(root: &Path, id: Uuid) -> serde_json::Value {
    super::main_tests_project_pool_library::query_pool_object_payload(root, "parts", id)
}
fn write(root: &Path, verb: &str, id: Uuid, value: &serde_json::Value) -> Result<String> {
    let file = root.join("part-input.json");
    std::fs::write(&file, serde_json::to_vec(value).unwrap()).unwrap();
    command(
        root,
        verb,
        &[
            "--kind",
            "parts",
            "--object",
            &id.to_string(),
            "--from-json",
            file.to_str().unwrap(),
        ],
    )
}
struct Fixture {
    root: PathBuf,
    part: Uuid,
    entity: Uuid,
    package: Uuid,
    footprint: Uuid,
    map: Uuid,
}
impl Fixture {
    fn new(label: &str) -> Self {
        let root = project(label);
        let unit = Uuid::new_v4();
        let pin = Uuid::new_v4();
        let symbol = Uuid::new_v4();
        let entity = Uuid::new_v4();
        let gate = Uuid::new_v4();
        let package = Uuid::new_v4();
        let stack = Uuid::new_v4();
        let footprint = Uuid::new_v4();
        let pad = Uuid::new_v4();
        let part = Uuid::new_v4();
        let map = Uuid::new_v4();
        command(
            &root,
            "create-pool-unit",
            &["--unit", &unit.to_string(), "--name", "Manual unit"],
        )
        .unwrap();
        command(
            &root,
            "set-pool-unit-pin",
            &[
                "--unit",
                &unit.to_string(),
                "--pin",
                &pin.to_string(),
                "--name",
                "1",
                "--electrical-type",
                "Passive",
            ],
        )
        .unwrap();
        command(
            &root,
            "create-pool-symbol",
            &[
                "--symbol",
                &symbol.to_string(),
                "--unit",
                &unit.to_string(),
                "--name",
                "Manual symbol",
            ],
        )
        .unwrap();
        command(
            &root,
            "create-pool-entity",
            &[
                "--entity",
                &entity.to_string(),
                "--gate",
                &gate.to_string(),
                "--unit",
                &unit.to_string(),
                "--symbol",
                &symbol.to_string(),
                "--name",
                "Manual entity",
                "--prefix",
                "U",
            ],
        )
        .unwrap();
        command(
            &root,
            "create-pool-package",
            &[
                "--package",
                &package.to_string(),
                "--name",
                "Manual package",
            ],
        )
        .unwrap();
        command(
            &root,
            "create-pool-padstack",
            &[
                "--padstack",
                &stack.to_string(),
                "--name",
                "Manual padstack",
                "--aperture",
                "circle",
                "--diameter-nm",
                "1000000",
            ],
        )
        .unwrap();
        command(
            &root,
            "create-pool-footprint",
            &[
                "--footprint",
                &footprint.to_string(),
                "--package",
                &package.to_string(),
                "--name",
                "Manual footprint",
            ],
        )
        .unwrap();
        command(
            &root,
            "set-pool-footprint-pad",
            &[
                "--footprint",
                &footprint.to_string(),
                "--pad",
                &pad.to_string(),
                "--padstack",
                &stack.to_string(),
            ],
        )
        .unwrap();
        command(
            &root,
            "create-pool-part",
            &[
                "--part",
                &part.to_string(),
                "--entity",
                &entity.to_string(),
                "--package",
                &package.to_string(),
                "--mpn",
                "MANUAL-PART",
            ],
        )
        .unwrap();
        command(
            &root,
            "create-pool-pin-pad-map",
            &[
                "--map",
                &map.to_string(),
                "--part",
                &part.to_string(),
                "--footprint",
                &footprint.to_string(),
                "--entry",
                &format!("{pad}:{gate}:{pin}"),
            ],
        )
        .unwrap();
        Self {
            root,
            part,
            entity,
            package,
            footprint,
            map,
        }
    }
}

#[test]
fn library_definition_part_refusals_preserve_canonical_state_and_errors() {
    let fixture = Fixture::new("part-reference-refusals");
    let root = &fixture.root;
    let original = payload(root, fixture.part);
    let before = ProjectResolver::new(root).resolve().unwrap();
    let before_journal = journal(root);
    let before_manifest = std::fs::read(root.join("project.json")).unwrap();
    let part_path = root.join(format!("pool/parts/{}.json", fixture.part));
    let before_part = std::fs::read(&part_path).unwrap();
    let missing = Uuid::new_v4();
    let new_part = Uuid::new_v4();
    for verb in ["set-pool-library-object", "create-pool-library-object"] {
        let target = if verb.starts_with("set") {
            fixture.part
        } else {
            new_part
        };
        let mut base = original.clone();
        base["uuid"] = serde_json::json!(target);
        let mut cases = Vec::new();
        for (field, reference, label) in [
            ("entity", missing, "entity"),
            ("entity", fixture.package, "entity"),
            ("package", missing, "package"),
            ("package", fixture.entity, "package"),
            ("default_footprint", missing, "footprint"),
            ("default_footprint", fixture.package, "footprint"),
            ("default_pin_pad_map", missing, "pin_pad_map"),
            ("default_pin_pad_map", fixture.entity, "pin_pad_map"),
        ] {
            let mut value = base.clone();
            value[field] = serde_json::json!(reference);
            cases.push((
                target,
                value,
                format!("missing pool {label} {reference} for part {target}"),
                true,
            ));
        }
        for fault in ["uuid", "schema", "shape", "foreign"] {
            let mut value = base.clone();
            value["default_footprint"] = serde_json::json!(missing);
            let expected = match fault {
                "uuid" => {
                    value["uuid"] = serde_json::json!(missing);
                    "does not match --object"
                }
                "schema" => {
                    value["schema_version"] = serde_json::json!(2);
                    "unsupported pool library object schema_version 2; expected 1"
                }
                "shape" => {
                    value["lifecycle"] = serde_json::json!("InvalidLifecycle");
                    "invalid pool library parts object"
                }
                "foreign" => {
                    value["extension"] = serde_json::json!({"uuid": fixture.package});
                    if verb.starts_with("set") {
                        "pool source objects: foreign source UUID collision"
                    } else {
                        "pool source objects: source UUID already exists"
                    }
                }
                _ => unreachable!(),
            };
            cases.push((target, value, expected.to_string(), false));
        }
        if verb.starts_with("create") {
            let mut value = original.clone();
            value["default_footprint"] = serde_json::json!(missing);
            cases.push((
                fixture.part,
                value,
                "pool source objects: source UUID already exists".to_string(),
                false,
            ));
        }
        for (id, value, expected, exact) in cases {
            let error = write(root, verb, id, &value).unwrap_err();
            if exact {
                assert_eq!(format!("{error:#}"), expected);
            } else {
                assert!(format!("{error:#}").contains(&expected), "{error:#}");
            }
            assert_eq!(journal(root), before_journal);
            assert_eq!(
                std::fs::read(root.join("project.json")).unwrap(),
                before_manifest
            );
            assert_eq!(std::fs::read(&part_path).unwrap(), before_part);
            assert!(!root.join(format!("pool/parts/{new_part}.json")).exists());
            let after = ProjectResolver::new(root).resolve().unwrap();
            assert_eq!(after.model_revision, before.model_revision);
            assert_eq!(after.objects.len(), before.objects.len());
            for (id, object) in &before.objects {
                assert_eq!(after.objects[id].object_revision, object.object_revision);
            }
        }
    }
    let empty = project("part-reference-no-pool");
    let manifest = std::fs::read(empty.join("project.json")).unwrap();
    let mut value = original.clone();
    value["uuid"] = serde_json::json!(new_part);
    let error = write(&empty, "create-pool-library-object", new_part, &value).unwrap_err();
    assert_eq!(
        format!("{error:#}"),
        format!("missing pool entity {} for part {new_part}", fixture.entity)
    );
    assert_eq!(std::fs::read(empty.join("project.json")).unwrap(), manifest);
    assert!(journal(&empty).is_empty());
    assert!(!empty.join(format!("pool/parts/{new_part}.json")).exists());
    std::fs::remove_dir_all(empty).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn library_definition_part_valid_create_set_reopen_undo_redo_preserves_optionals() {
    let fixture = Fixture::new("part-reference-history");
    let root = &fixture.root;
    let original = payload(root, fixture.part);
    let before = ProjectResolver::new(root).resolve().unwrap();
    let meta = Uuid::new_v4();
    let mut edited = original.clone();
    edited["default_footprint"] = serde_json::json!(fixture.footprint);
    edited["default_pin_pad_map"] = serde_json::json!(fixture.map);
    edited["extension"] = serde_json::json!({"uuid": meta, "retained": [1,2]});
    write(root, "set-pool-library-object", fixture.part, &edited).unwrap();
    let changed = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(changed.journal.len(), before.journal.len() + 1);
    assert_eq!(
        changed.objects[&fixture.part].object_revision.0,
        before.objects[&fixture.part].object_revision.0 + 1
    );
    assert!(changed.objects.contains_key(&meta));
    assert!(matches!(
        changed.journal.last().unwrap().operations.as_slice(),
        [Operation::SetPoolLibraryObject { .. }]
    ));
    for source in [
        fixture.entity,
        fixture.package,
        fixture.footprint,
        fixture.map,
    ] {
        assert_eq!(
            changed.objects[&source].object_revision,
            before.objects[&source].object_revision
        );
    }
    assert_eq!(payload(root, fixture.part), edited);
    command(root, "undo", &[]).unwrap();
    assert_eq!(payload(root, fixture.part), original);
    assert!(
        !ProjectResolver::new(root)
            .resolve()
            .unwrap()
            .objects
            .contains_key(&meta)
    );
    command(root, "redo", &[]).unwrap();
    assert_eq!(payload(root, fixture.part), edited);
    assert!(
        ProjectResolver::new(root)
            .resolve()
            .unwrap()
            .objects
            .contains_key(&meta)
    );
    let mut cleared = edited.clone();
    cleared["default_footprint"] = serde_json::Value::Null;
    cleared["default_pin_pad_map"] = serde_json::Value::Null;
    write(root, "set-pool-library-object", fixture.part, &cleared).unwrap();
    assert_eq!(payload(root, fixture.part), cleared);
    cleared.as_object_mut().unwrap().remove("default_footprint");
    cleared
        .as_object_mut()
        .unwrap()
        .remove("default_pin_pad_map");
    write(root, "set-pool-library-object", fixture.part, &cleared).unwrap();
    assert_eq!(payload(root, fixture.part), cleared);
    let id = Uuid::new_v4();
    let leaf = Uuid::new_v4();
    let mut created = original.clone();
    created["uuid"] = serde_json::json!(id);
    created["default_footprint"] = serde_json::json!(fixture.footprint);
    created
        .as_object_mut()
        .unwrap()
        .remove("default_pin_pad_map");
    created["extension"] = serde_json::json!({"uuid": leaf, "retained": "manual metadata"});
    let prior = ProjectResolver::new(root).resolve().unwrap();
    write(root, "create-pool-library-object", id, &created).unwrap();
    let after = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(after.journal.len(), prior.journal.len() + 1);
    assert_eq!(after.objects[&id].object_revision.0, 0);
    assert!(after.objects.contains_key(&leaf));
    assert!(matches!(
        after.journal.last().unwrap().operations.as_slice(),
        [Operation::CreatePoolLibraryObject { .. }]
    ));
    assert_eq!(payload(root, id), created);
    command(root, "undo", &[]).unwrap();
    let undone = ProjectResolver::new(root).resolve().unwrap();
    assert!(!undone.objects.contains_key(&id) && !undone.objects.contains_key(&leaf));
    assert!(!root.join(format!("pool/parts/{id}.json")).exists());
    command(root, "redo", &[]).unwrap();
    assert_eq!(payload(root, id), created);
    // Relink the newly created Part to different existing body/land-pattern
    // definitions. Its optional map stays absent; existing maps remain owned by
    // the original Part and are neither remapped nor propagated by this edit.
    let alternate_package = Uuid::new_v4();
    let alternate_footprint = Uuid::new_v4();
    command(
        root,
        "create-pool-package",
        &[
            "--package",
            &alternate_package.to_string(),
            "--name",
            "Alternate body",
        ],
    )
    .unwrap();
    command(
        root,
        "create-pool-footprint",
        &[
            "--footprint",
            &alternate_footprint.to_string(),
            "--package",
            &alternate_package.to_string(),
            "--name",
            "Alternate land pattern",
        ],
    )
    .unwrap();
    let before_relink = ProjectResolver::new(root).resolve().unwrap();
    let mut relinked = created.clone();
    relinked["package"] = serde_json::json!(alternate_package);
    relinked["default_footprint"] = serde_json::json!(alternate_footprint);
    write(root, "set-pool-library-object", id, &relinked).unwrap();
    let after_relink = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(after_relink.journal.len(), before_relink.journal.len() + 1);
    assert_eq!(
        after_relink.objects[&id].object_revision.0,
        before_relink.objects[&id].object_revision.0 + 1
    );
    for source in [
        fixture.entity,
        fixture.package,
        fixture.footprint,
        fixture.map,
        alternate_package,
        alternate_footprint,
    ] {
        assert_eq!(
            after_relink.objects[&source].object_revision,
            before_relink.objects[&source].object_revision
        );
    }
    assert_eq!(payload(root, id), relinked);
    command(root, "undo", &[]).unwrap();
    assert_eq!(payload(root, id), created);
    command(root, "redo", &[]).unwrap();
    assert_eq!(payload(root, id), relinked);
    let validation: serde_json::Value =
        serde_json::from_str(&command(root, "validate", &[]).unwrap()).unwrap();
    assert_eq!(validation["valid"], true);
    std::fs::remove_dir_all(root).unwrap();
}
