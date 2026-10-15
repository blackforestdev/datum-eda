//! Ordinary map authoring shares existing typed validation and canonical history.
use super::main_tests_project_pool_pin_pad_map::{PinnedMapFixture, create_fixture};
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
fn fixture(label: &str) -> (PathBuf, PinnedMapFixture, Uuid) {
    let root = crate::main_tests::development_workspace::path("footprint-reference-audit")
        .join(format!("{label}-{}", Uuid::new_v4()));
    create_native_project(&root, Some(label.to_string())).unwrap();
    let fixture = create_fixture(&root, &["IN", "OUT"], &["1", "2"]);
    let id = Uuid::new_v4();
    command(
        &root,
        "create-pool-pin-pad-map",
        &[
            "--map",
            &id.to_string(),
            "--part",
            &fixture.part_id.to_string(),
            "--footprint",
            &fixture.footprint_id.to_string(),
            "--entry",
            &format!(
                "{}:{}:{}",
                fixture.pad_ids[0], fixture.gate_id, fixture.pin_ids[0]
            ),
        ],
    )
    .unwrap();
    (root, fixture, id)
}
fn payload(root: &Path, kind: &str, id: Uuid) -> serde_json::Value {
    super::main_tests_project_pool_library::query_pool_object_payload(root, kind, id)
}
fn write(
    root: &Path,
    verb: &str,
    kind: &str,
    id: Uuid,
    value: &serde_json::Value,
) -> Result<String> {
    let file = root.join("map-input.json");
    std::fs::write(&file, serde_json::to_vec(value).unwrap()).unwrap();
    command(
        root,
        verb,
        &[
            "--kind",
            kind,
            "--object",
            &id.to_string(),
            "--from-json",
            file.to_str().unwrap(),
        ],
    )
}
fn journal(root: &Path) -> Vec<u8> {
    std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap_or_default()
}

#[test]
fn library_definition_pin_pad_map_refusals_preserve_state_and_precedence() {
    let (root, f, id) = fixture("map-refusals");
    let original = payload(&root, "pin_pad_maps", id);
    let part = payload(&root, "parts", f.part_id);
    let entity = part["entity"].as_str().unwrap();
    let symbol = payload(&root, "symbols", f.symbol_id);
    let unit = symbol["unit"].as_str().unwrap();
    let before = ProjectResolver::new(&root).resolve().unwrap();
    let before_journal = journal(&root);
    let manifest = std::fs::read(root.join("project.json")).unwrap();
    let map_path = root.join(format!("pool/pin_pad_maps/{id}.json"));
    let map_bytes = std::fs::read(&map_path).unwrap();
    let missing = Uuid::new_v4();
    let created = Uuid::new_v4();
    let mut cases = Vec::new();
    for (field, reference, expected) in [
        ("part", missing, format!("missing pool parts {missing}")),
        (
            "part",
            f.footprint_id,
            format!("missing pool parts {}", f.footprint_id),
        ),
        (
            "footprint",
            missing,
            format!("missing pool footprints {missing}"),
        ),
        (
            "footprint",
            f.part_id,
            format!("missing pool footprints {}", f.part_id),
        ),
    ] {
        let mut value = original.clone();
        value[field] = serde_json::json!(reference);
        cases.push((value, expected));
    }
    let mut value = original.clone();
    value["mappings"] = serde_json::json!({});
    cases.push((value, "PinPadMap requires at least one mapping".to_string()));
    for (pad, gate, pin, expected) in [
        (
            missing,
            f.gate_id,
            f.pin_ids[0],
            format!("PinPadMap target has no pad {missing}"),
        ),
        (
            missing,
            missing,
            f.pin_ids[0],
            format!("pool entity {entity} has no gate {missing}"),
        ),
        (
            missing,
            f.gate_id,
            missing,
            format!("pool unit {unit} has no pin {missing}"),
        ),
    ] {
        let mut value = original.clone();
        value["mappings"] = serde_json::json!({pad.to_string(): {"gate": gate, "pin": pin}});
        cases.push((value, expected));
    }
    let mut aliases = original.clone();
    let pad_key = f.pad_ids[0].to_string();
    let uppercase_key = pad_key.to_uppercase();
    assert_ne!(pad_key, uppercase_key);
    aliases["mappings"][&uppercase_key] = aliases["mappings"][&pad_key].clone();
    cases.push((
        aliases,
        format!("duplicate PinPadMap mapping for pad {}", f.pad_ids[0]),
    ));
    let mut omitted_mappings = original.clone();
    omitted_mappings.as_object_mut().unwrap().remove("mappings");
    cases.push((
        omitted_mappings,
        "PinPadMap requires at least one mapping".to_string(),
    ));
    let mut duplicate = original.clone();
    duplicate["mappings"][f.pad_ids[1].to_string()] =
        serde_json::json!({"gate": f.gate_id, "pin": f.pin_ids[0]});
    cases.push((
        duplicate,
        format!(
            "duplicate PinPadMap mapping for gate {} pin {}",
            f.gate_id, f.pin_ids[0]
        ),
    ));
    let alternate_package = Uuid::new_v4();
    let alternate_footprint = Uuid::new_v4();
    command(
        &root,
        "create-pool-package",
        &[
            "--package",
            &alternate_package.to_string(),
            "--name",
            "Alternate",
        ],
    )
    .unwrap();
    command(
        &root,
        "create-pool-footprint",
        &[
            "--footprint",
            &alternate_footprint.to_string(),
            "--package",
            &alternate_package.to_string(),
            "--name",
            "Alternate",
        ],
    )
    .unwrap();
    let mut mismatch = original.clone();
    mismatch["footprint"] = serde_json::json!(alternate_footprint);
    cases.push((mismatch, format!("pool footprint {alternate_footprint} belongs to package {alternate_package}, not part package {}", part["package"].as_str().unwrap())));
    // These two setup commands are ordinary canonical writes; capture their new basis.
    let baseline = ProjectResolver::new(&root).resolve().unwrap();
    let baseline_journal = journal(&root);
    assert_eq!(baseline.journal.len(), before.journal.len() + 2);
    assert_ne!(baseline_journal, before_journal);
    for verb in ["create-pool-library-object", "set-pool-library-object"] {
        let target = if verb.starts_with("create") {
            created
        } else {
            id
        };
        for (base, expected) in &cases {
            let mut value = base.clone();
            value["uuid"] = serde_json::json!(target);
            assert_eq!(
                format!(
                    "{:#}",
                    write(&root, verb, "pin_pad_maps", target, &value).unwrap_err()
                ),
                *expected
            );
            assert_eq!(journal(&root), baseline_journal);
            assert_eq!(std::fs::read(&map_path).unwrap(), map_bytes);
            assert_eq!(std::fs::read(root.join("project.json")).unwrap(), manifest);
            assert!(
                !root
                    .join(format!("pool/pin_pad_maps/{created}.json"))
                    .exists()
            );
            let after = ProjectResolver::new(&root).resolve().unwrap();
            assert_eq!(after.model_revision, baseline.model_revision);
            assert_eq!(after.objects.len(), baseline.objects.len());
            for (id, object) in &baseline.objects {
                assert_eq!(after.objects[id].object_revision, object.object_revision);
            }
        }
        for fault in ["uuid", "schema", "shape", "foreign"] {
            let mut value = original.clone();
            value["uuid"] = serde_json::json!(target);
            value["mappings"] =
                serde_json::json!({missing.to_string(): {"gate": f.gate_id, "pin": f.pin_ids[0]}});
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
                    value["tags"] = serde_json::json!(1);
                    "invalid pool library pin_pad_maps object"
                }
                "foreign" => {
                    value["extension"] = serde_json::json!({"uuid": f.part_id});
                    if verb.starts_with("create") {
                        "source UUID already exists"
                    } else {
                        "foreign source UUID collision"
                    }
                }
                _ => unreachable!(),
            };
            assert!(
                format!(
                    "{:#}",
                    write(&root, verb, "pin_pad_maps", target, &value).unwrap_err()
                )
                .contains(expected)
            );
            assert_eq!(journal(&root), baseline_journal);
            assert_eq!(std::fs::read(&map_path).unwrap(), map_bytes);
        }
    }
    let mut collision = original.clone();
    collision["mappings"] = serde_json::json!({});
    assert!(
        format!(
            "{:#}",
            write(
                &root,
                "create-pool-library-object",
                "pin_pad_maps",
                id,
                &collision
            )
            .unwrap_err()
        )
        .contains("source UUID already exists")
    );
    assert_eq!(journal(&root), baseline_journal);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn library_definition_pin_pad_map_valid_history_preserves_defaults_and_extensions() {
    let (root, f, id) = fixture("map-history");
    let original = payload(&root, "pin_pad_maps", id);
    let mut edited = original.clone();
    let leaf = Uuid::new_v4();
    edited["extension"] = serde_json::json!({"uuid": leaf, "retained": [1,2]});
    edited["mappings"] =
        serde_json::json!({f.pad_ids[1].to_string(): {"gate": f.gate_id, "pin": f.pin_ids[1]}});
    let before = ProjectResolver::new(&root).resolve().unwrap();
    write(
        &root,
        "set-pool-library-object",
        "pin_pad_maps",
        id,
        &edited,
    )
    .unwrap();
    let after = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(after.journal.len(), before.journal.len() + 1);
    assert_eq!(
        after.objects[&id].object_revision.0,
        before.objects[&id].object_revision.0 + 1
    );
    assert!(after.objects.contains_key(&leaf));
    assert!(matches!(
        after.journal.last().unwrap().operations.as_slice(),
        [Operation::SetPoolLibraryObject { .. }]
    ));
    for source in [
        f.part_id,
        f.footprint_id,
        f.gate_id,
        f.pin_ids[0],
        f.pin_ids[1],
        f.pad_ids[0],
        f.pad_ids[1],
    ] {
        assert_eq!(
            after.objects[&source].object_revision,
            before.objects[&source].object_revision
        );
    }
    assert_eq!(payload(&root, "pin_pad_maps", id), edited);
    command(&root, "undo", &[]).unwrap();
    assert_eq!(payload(&root, "pin_pad_maps", id), original);
    assert!(
        !ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .objects
            .contains_key(&leaf)
    );
    command(&root, "redo", &[]).unwrap();
    assert_eq!(payload(&root, "pin_pad_maps", id), edited);
    // Footprint omitted or null uses Package pads, exactly as typed admission.
    let part = payload(&root, "parts", f.part_id);
    let package_id = Uuid::parse_str(part["package"].as_str().unwrap()).unwrap();
    let mut package = payload(&root, "packages", package_id);
    let footprint = payload(&root, "footprints", f.footprint_id);
    let package_pad = Uuid::new_v4();
    let mut pad = footprint["pads"][f.pad_ids[0].to_string()].clone();
    pad["uuid"] = serde_json::json!(package_pad);
    package["pads"][package_pad.to_string()] = pad;
    write(
        &root,
        "set-pool-library-object",
        "packages",
        package_id,
        &package,
    )
    .unwrap();
    let new = Uuid::new_v4();
    let new_leaf = Uuid::new_v4();
    let mut created = original.clone();
    created["uuid"] = serde_json::json!(new);
    created.as_object_mut().unwrap().remove("footprint");
    created.as_object_mut().unwrap().remove("tags");
    created["extension"] = serde_json::json!({"uuid": new_leaf, "retained": "manual metadata"});
    created["mappings"] =
        serde_json::json!({package_pad.to_string(): {"gate": f.gate_id, "pin": f.pin_ids[0]}});
    let before_create = ProjectResolver::new(&root).resolve().unwrap();
    write(
        &root,
        "create-pool-library-object",
        "pin_pad_maps",
        new,
        &created,
    )
    .unwrap();
    let after_create = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(after_create.journal.len(), before_create.journal.len() + 1);
    assert_eq!(after_create.objects[&new].object_revision.0, 0);
    assert!(matches!(
        after_create.journal.last().unwrap().operations.as_slice(),
        [Operation::CreatePoolLibraryObject { .. }]
    ));
    assert_eq!(payload(&root, "pin_pad_maps", new), created);
    command(&root, "undo", &[]).unwrap();
    let undone = ProjectResolver::new(&root).resolve().unwrap();
    assert!(!undone.objects.contains_key(&new) && !undone.objects.contains_key(&new_leaf));
    assert!(!root.join(format!("pool/pin_pad_maps/{new}.json")).exists());
    command(&root, "redo", &[]).unwrap();
    assert_eq!(payload(&root, "pin_pad_maps", new), created);
    created["footprint"] = serde_json::Value::Null;
    write(
        &root,
        "set-pool-library-object",
        "pin_pad_maps",
        new,
        &created,
    )
    .unwrap();
    assert_eq!(payload(&root, "pin_pad_maps", new), created);
    let validation: serde_json::Value =
        serde_json::from_str(&command(&root, "validate", &[]).unwrap()).unwrap();
    assert_eq!(validation["valid"], true);
    std::fs::remove_dir_all(root).unwrap();
}
