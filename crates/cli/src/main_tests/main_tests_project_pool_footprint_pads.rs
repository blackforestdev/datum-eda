//! UUID-keyed pad collection edits preserve raw content and canonical history.
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
    execute(Cli::try_parse_from(argv)?)
}
fn payload(root: &Path, id: Uuid) -> serde_json::Value {
    super::main_tests_project_pool_library::query_pool_object_payload(root, "footprints", id)
}
struct Fixture {
    root: PathBuf,
    footprint: Uuid,
    package: Uuid,
    stack: Uuid,
}
impl Fixture {
    fn new(label: &str) -> Self {
        let root = crate::main_tests::development_workspace::path("footprint-reference-audit")
            .join(format!("{label}-{}", Uuid::new_v4()));
        create_native_project(&root, Some(label.to_string())).unwrap();
        let package = Uuid::new_v4();
        let footprint = Uuid::new_v4();
        let stack = Uuid::new_v4();
        let pad = Uuid::new_v4();
        command(
            &root,
            "create-pool-package",
            &["--package", &package.to_string(), "--name", "Manual body"],
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
                "Manual graphics",
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
                "Manual pad",
                "--aperture",
                "circle",
                "--diameter-nm",
                "1000",
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
        let second = Uuid::new_v4();
        command(
            &root,
            "set-pool-footprint-pad",
            &[
                "--footprint",
                &footprint.to_string(),
                "--pad",
                &second.to_string(),
                "--padstack",
                &stack.to_string(),
                "--pad-name",
                "2",
            ],
        )
        .unwrap();
        let mut object = payload(&root, footprint);
        object["silkscreen"] = serde_json::json!([{"Text":{"text":"  raw sibling  ","position":{"x":1,"y":2},"rotation":-123}}]);
        object["fab"] =
            serde_json::json!([{"Circle":{"center":{"x":1,"y":2},"radius":3,"width":1}}]);
        object["pads"][pad.to_string()]["extension"] =
            serde_json::json!({"uuid":Uuid::new_v4(),"retained":[1,2]});
        let primitive =
            serde_json::json!({"Line":{"from":{"x":1,"y":2},"to":{"x":3,"y":4},"width":1}});
        for field in ["assembly", "mechanical"] {
            object[field] = serde_json::json!([primitive.clone()]);
        }
        object["courtyard"] = serde_json::json!({"vertices":[{"x":0,"y":0},{"x":10,"y":0},{"x":0,"y":10}],"closed":true});
        object["process_aperture_policy"] = serde_json::json!("preserve-native-policy");
        object["models_3d"] =
            serde_json::json!([{"path":"retained.step","extension":{"kept":true}}]);
        object["extension"] = serde_json::json!({"uuid": Uuid::new_v4(), "retained": [1,2]});
        let file = root.with_extension("input.json");
        std::fs::write(&file, serde_json::to_vec(&object).unwrap()).unwrap();
        command(
            &root,
            "set-pool-library-object",
            &[
                "--kind",
                "footprints",
                "--object",
                &footprint.to_string(),
                "--from-json",
                file.to_str().unwrap(),
            ],
        )
        .unwrap();
        Self {
            root,
            footprint,
            package,
            stack,
        }
    }
}
fn write(
    root: &Path,
    id: Uuid,
    kind: &str,
    section: Option<&str>,
    value: &serde_json::Value,
) -> Result<String> {
    let file = root.with_extension("input.json");
    std::fs::write(&file, serde_json::to_vec(value).unwrap()).unwrap();
    let identity = id.to_string();
    let mut args = vec![
        "--kind",
        kind,
        "--object",
        &identity,
        "--from-json",
        file.to_str().unwrap(),
    ];
    if let Some(section) = section {
        args.extend_from_slice(&["--section", section]);
    }
    command(root, "set-pool-library-object", &args)
}
fn authoritative(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn collect(path: &Path, rows: &mut Vec<(PathBuf, Vec<u8>)>) {
        if path.is_dir() {
            for e in std::fs::read_dir(path).unwrap() {
                collect(&e.unwrap().path(), rows);
            }
        } else if path.is_file() {
            rows.push((path.to_path_buf(), std::fs::read(path).unwrap()));
        }
    }
    let mut rows = Vec::new();
    for p in [
        "pool",
        "board",
        "schematic",
        "project.json",
        ".datum/journal/transactions.jsonl",
    ] {
        collect(&root.join(p), &mut rows);
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}
#[test]
fn footprint_pads_add_edit_remove_clear_restore_history() {
    let f = Fixture::new("footprint-pads-history");
    let original = payload(&f.root, f.footprint);
    let existing = original["pads"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone();
    let added_id = Uuid::new_v4();
    let mut added = original["pads"].clone();
    added[added_id.to_string()] = serde_json::json!({"uuid":added_id,"name":"3","padstack":f.stack,"position":{"x":10,"y":20},"layer":1,"extension":{"uuid":Uuid::new_v4(),"kept":true}});
    let mut edited = added.clone();
    edited[&existing]["position"] = serde_json::json!({"x":99,"y":-100});
    edited[&existing]["name"] = serde_json::json!("  changed  ");
    let mut removed = edited.clone();
    removed
        .as_object_mut()
        .unwrap()
        .remove(&added_id.to_string());
    for pads in [
        added,
        edited,
        removed,
        serde_json::json!({}),
        original["pads"].clone(),
    ] {
        let before = payload(&f.root, f.footprint);
        let model = ProjectResolver::new(&f.root).resolve().unwrap();
        let view: serde_json::Value = serde_json::from_str(
            &write(&f.root, f.footprint, "footprints", Some("pads"), &pads).unwrap(),
        )
        .unwrap();
        assert_eq!(view["action"], "set");
        let mut expected = before.clone();
        expected["pads"] = pads;
        assert_eq!(payload(&f.root, f.footprint), expected);
        let after = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(after.journal.len(), model.journal.len() + 1);
        assert_eq!(
            after.objects[&f.footprint].object_revision.0,
            model.objects[&f.footprint].object_revision.0 + 1
        );
        for id in [f.package, f.stack] {
            assert_eq!(
                after.objects[&id].object_revision,
                model.objects[&id].object_revision
            );
        }
        assert!(matches!(
            after.journal.last().unwrap().operations.as_slice(),
            [Operation::SetPoolLibraryObject { .. }]
        ));
        command(&f.root, "undo", &[]).unwrap();
        assert_eq!(payload(&f.root, f.footprint), before);
        command(&f.root, "redo", &[]).unwrap();
        assert_eq!(payload(&f.root, f.footprint), expected);
    }
    let id = Uuid::parse_str(&existing).unwrap();
    let before = payload(&f.root, f.footprint);
    command(
        &f.root,
        "set-pool-footprint-pad",
        &[
            "--footprint",
            &f.footprint.to_string(),
            "--pad",
            &id.to_string(),
            "--padstack",
            &f.stack.to_string(),
            "--pad-name",
            "single pad continuation",
            "--x-nm",
            "12",
        ],
    )
    .unwrap();
    let mut expected = before;
    expected["pads"][&existing] = serde_json::json!({"uuid":id,"name":"single pad continuation","position":{"x":12,"y":0},"padstack":f.stack,"layer":1});
    assert_eq!(payload(&f.root, f.footprint), expected);
    command(&f.root, "query", &["resolve-debug"]).unwrap();
    command(&f.root, "validate", &[]).unwrap();
    cleanup(f);
}
fn cleanup(f: Fixture) {
    std::fs::remove_dir_all(&f.root).unwrap();
    std::fs::remove_file(f.root.with_extension("input.json")).unwrap();
}
#[test]
fn footprint_pads_preserves_whole_json_map_schema_admission() {
    let f = Fixture::new("footprint-pads-schema");
    let original = payload(&f.root, f.footprint);
    let mut pads = original["pads"].clone();
    for pad in pads.as_object_mut().unwrap().values_mut() {
        pad["name"] = serde_json::json!("   ");
        pad["layer"] = serde_json::json!(-1);
        pad["position"] = serde_json::json!({"x":-100,"y":0});
    }
    let mut mismatch = pads.clone();
    let key = mismatch.as_object().unwrap().keys().next().unwrap().clone();
    let pad = mismatch.as_object_mut().unwrap().remove(&key).unwrap();
    mismatch[Uuid::new_v4().to_string()] = pad;
    for value in [pads, mismatch, serde_json::json!({})] {
        let mut expected = original.clone();
        expected["pads"] = value.clone();
        write(&f.root, f.footprint, "footprints", None, &expected).unwrap();
        assert_eq!(payload(&f.root, f.footprint), expected);
        command(&f.root, "undo", &[]).unwrap();
        assert_eq!(payload(&f.root, f.footprint), original);
        write(&f.root, f.footprint, "footprints", Some("pads"), &value).unwrap();
        assert_eq!(payload(&f.root, f.footprint), expected);
        command(&f.root, "undo", &[]).unwrap();
        assert_eq!(payload(&f.root, f.footprint), original);
    }
    command(&f.root, "validate", &[]).unwrap();
    cleanup(f);
}
#[test]
fn footprint_pads_refusals_preserve_authority_and_references() {
    let f = Fixture::new("footprint-pads-refusals");
    let before = authoritative(&f.root);
    let id = Uuid::new_v4();
    let valid = serde_json::json!({"uuid":id,"name":"1","position":{"x":0,"y":0},"padstack":f.stack,"layer":1});
    let mut missing = valid.clone();
    missing.as_object_mut().unwrap().remove("padstack");
    let mut fractional = valid.clone();
    fractional["position"]["x"] = serde_json::json!(0.5);
    let mut overflow = valid.clone();
    overflow["layer"] = serde_json::json!(2147483648_i64);
    for pads in [
        serde_json::json!(null),
        serde_json::json!([]),
        serde_json::json!({"invalid":valid.clone()}),
        serde_json::json!({id.to_string():"pad"}),
        serde_json::json!({id.to_string():missing}),
        serde_json::json!({id.to_string():fractional}),
        serde_json::json!({id.to_string():overflow}),
    ] {
        let error = format!(
            "{:#}",
            write(&f.root, f.footprint, "footprints", Some("pads"), &pads).unwrap_err()
        );
        assert!(error.starts_with("invalid footprint pads:"), "{error}");
        assert_eq!(authoritative(&f.root), before);
    }
    for stack in [Uuid::new_v4(), f.package] {
        let mut pad = valid.clone();
        pad["padstack"] = serde_json::json!(stack);
        assert_eq!(
            format!(
                "{:#}",
                write(
                    &f.root,
                    f.footprint,
                    "footprints",
                    Some("pads"),
                    &serde_json::json!({id.to_string():pad})
                )
                .unwrap_err()
            ),
            format!("missing pool padstack {stack}")
        );
        assert_eq!(authoritative(&f.root), before);
    }
    for kind in ["symbols", "padstacks", "packages"] {
        assert_eq!(
            format!(
                "{:#}",
                write(
                    &f.root,
                    f.footprint,
                    kind,
                    Some("pads"),
                    &serde_json::json!({})
                )
                .unwrap_err()
            ),
            "--section pads requires --kind footprints"
        );
        assert_eq!(authoritative(&f.root), before);
    }
    assert!(
        write(
            &f.root,
            Uuid::new_v4(),
            "footprints",
            Some("pads"),
            &serde_json::json!({})
        )
        .is_err()
    );
    assert!(
        write(
            &f.root,
            f.footprint,
            "footprints",
            Some("invalid"),
            &serde_json::json!({})
        )
        .is_err()
    );
    assert_eq!(authoritative(&f.root), before);
    let mut pad = valid;
    pad["extension"] = serde_json::json!({"uuid":f.package});
    assert_eq!(
        format!(
            "{:#}",
            write(
                &f.root,
                f.footprint,
                "footprints",
                Some("pads"),
                &serde_json::json!({id.to_string():pad})
            )
            .unwrap_err()
        ),
        "validation error: pool source objects: foreign source UUID collision"
    );
    assert_eq!(authoritative(&f.root), before);
    cleanup(f);
}
