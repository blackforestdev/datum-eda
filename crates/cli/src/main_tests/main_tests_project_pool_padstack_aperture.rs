//! Copper aperture edits retain raw process data and canonical history.
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
fn payload(root: &Path, kind: &str, id: Uuid) -> serde_json::Value {
    super::main_tests_project_pool_library::query_pool_object_payload(root, kind, id)
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
struct Fixture {
    root: PathBuf,
    stack: Uuid,
    package: Uuid,
}
impl Fixture {
    fn new(label: &str) -> Self {
        let root = crate::main_tests::development_workspace::path("footprint-reference-audit")
            .join(format!("{label}-{}", Uuid::new_v4()));
        create_native_project(&root, Some(label.to_string())).unwrap();
        let stack = Uuid::new_v4();
        let package = Uuid::new_v4();
        command(
            &root,
            "create-pool-padstack",
            &[
                "--padstack",
                &stack.to_string(),
                "--name",
                "Manual copper",
                "--aperture",
                "circle",
                "--diameter-nm",
                "1000",
                "--drill-nm",
                "500",
            ],
        )
        .unwrap();
        command(
            &root,
            "create-pool-package",
            &["--package", &package.to_string(), "--name", "Manual body"],
        )
        .unwrap();
        let mut object = payload(&root, "padstacks", stack);
        object["plated"] = serde_json::json!(true);
        object["layer_span"] = serde_json::json!({"blind":{"start_layer":1,"end_layer":2}});
        object["mask_policy"] = serde_json::json!({"expansion_nm":10});
        object["paste_policy"] = serde_json::json!("none");
        object["annular_ring_nm"] = serde_json::json!(25);
        object["thermal"] = serde_json::json!({"spoke_count":4,"extension":{"kept":true}});
        object["antipad"] = serde_json::json!({"clearance_nm":7,"aperture":{"rect":{"width_nm":2000,"height_nm":3000,"extension":{"kept":true}}}});
        object["extension"] = serde_json::json!({"uuid":Uuid::new_v4(),"retained":[1,2]});
        write(&root, stack, "padstacks", None, &object).unwrap();
        Self {
            root,
            stack,
            package,
        }
    }
    fn cleanup(self) {
        std::fs::remove_dir_all(&self.root).unwrap();
        std::fs::remove_file(self.root.with_extension("input.json")).unwrap();
    }
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
fn padstack_aperture_resize_change_clear_restore_history_and_new_pad() {
    let f = Fixture::new("padstack-aperture-history");
    let restored = serde_json::json!({"circle":{"diameter_nm":1500,"extension":{"uuid":Uuid::new_v4(),"kept":true}}});
    for aperture in [
        serde_json::json!({"circle":{"diameter_nm":2000,"extension":{"uuid":Uuid::new_v4(),"kept":true}}}),
        serde_json::json!({"rect":{"width_nm":2000,"height_nm":3000,"extension":{"kept":true}}}),
        serde_json::json!(null),
        restored.clone(),
    ] {
        let before = payload(&f.root, "padstacks", f.stack);
        let model = ProjectResolver::new(&f.root).resolve().unwrap();
        let view: serde_json::Value = serde_json::from_str(
            &write(&f.root, f.stack, "padstacks", Some("aperture"), &aperture).unwrap(),
        )
        .unwrap();
        assert_eq!(view["action"], "set");
        let mut expected = before.clone();
        expected["aperture"] = aperture;
        assert_eq!(payload(&f.root, "padstacks", f.stack), expected);
        assert!(expected["thermal"].get("gap_nm").is_none());
        let after = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(after.journal.len(), model.journal.len() + 1);
        assert_eq!(
            after.objects[&f.stack].object_revision.0,
            model.objects[&f.stack].object_revision.0 + 1
        );
        assert_eq!(
            after.objects[&f.package].object_revision,
            model.objects[&f.package].object_revision
        );
        assert!(matches!(
            after.journal.last().unwrap().operations.as_slice(),
            [Operation::SetPoolLibraryObject { .. }]
        ));
        command(&f.root, "undo", &[]).unwrap();
        assert_eq!(payload(&f.root, "padstacks", f.stack), before);
        command(&f.root, "redo", &[]).unwrap();
        assert_eq!(payload(&f.root, "padstacks", f.stack), expected);
    }
    let footprint = Uuid::new_v4();
    let pad = Uuid::new_v4();
    command(
        &f.root,
        "create-pool-footprint",
        &[
            "--footprint",
            &footprint.to_string(),
            "--package",
            &f.package.to_string(),
            "--name",
            "Edited copper user",
        ],
    )
    .unwrap();
    command(
        &f.root,
        "set-pool-footprint-pad",
        &[
            "--footprint",
            &footprint.to_string(),
            "--pad",
            &pad.to_string(),
            "--padstack",
            &f.stack.to_string(),
        ],
    )
    .unwrap();
    let object = payload(&f.root, "footprints", footprint);
    assert_eq!(
        object["pads"][pad.to_string()]["padstack"],
        serde_json::json!(f.stack)
    );
    assert_eq!(payload(&f.root, "padstacks", f.stack)["aperture"], restored);
    command(&f.root, "query", &["resolve-debug"]).unwrap();
    command(&f.root, "validate", &[]).unwrap();
    f.cleanup();
}
#[test]
fn padstack_aperture_preserves_whole_json_numeric_and_null_admission() {
    let f = Fixture::new("padstack-aperture-schema");
    for aperture in [
        serde_json::json!({"circle":{"diameter_nm":0}}),
        serde_json::json!({"rect":{"width_nm":-1,"height_nm":0}}),
        serde_json::json!(null),
    ] {
        let original = payload(&f.root, "padstacks", f.stack);
        let mut expected = original.clone();
        expected["aperture"] = aperture.clone();
        write(&f.root, f.stack, "padstacks", None, &expected).unwrap();
        assert_eq!(payload(&f.root, "padstacks", f.stack), expected);
        command(&f.root, "undo", &[]).unwrap();
        assert_eq!(payload(&f.root, "padstacks", f.stack), original);
        write(&f.root, f.stack, "padstacks", Some("aperture"), &aperture).unwrap();
        assert_eq!(payload(&f.root, "padstacks", f.stack), expected);
        command(&f.root, "undo", &[]).unwrap();
        assert_eq!(payload(&f.root, "padstacks", f.stack), original);
    }
    command(&f.root, "validate", &[]).unwrap();
    f.cleanup();
}
#[test]
fn padstack_aperture_refusals_preserve_authority_and_section_routing() {
    let f = Fixture::new("padstack-aperture-refusals");
    let before = authoritative(&f.root);
    for aperture in [
        serde_json::json!(1),
        serde_json::json!([]),
        serde_json::json!({}),
        serde_json::json!({"unknown":{}}),
        serde_json::json!({"circle":{}}),
        serde_json::json!({"rect":{"width_nm":1}}),
        serde_json::json!({"circle":{"diameter_nm":0.5}}),
        serde_json::json!({"circle":{"diameter_nm":9223372036854775808_u64}}),
        serde_json::json!({"circle":{"diameter_nm":1},"rect":{"width_nm":1,"height_nm":1}}),
    ] {
        let error = format!(
            "{:#}",
            write(&f.root, f.stack, "padstacks", Some("aperture"), &aperture).unwrap_err()
        );
        assert!(error.starts_with("invalid padstack aperture:"), "{error}");
        assert_eq!(authoritative(&f.root), before);
    }
    let aperture = serde_json::json!({"circle":{"diameter_nm":1}});
    for kind in ["symbols", "footprints", "packages"] {
        assert_eq!(
            format!(
                "{:#}",
                write(&f.root, f.stack, kind, Some("aperture"), &aperture).unwrap_err()
            ),
            "--section aperture requires --kind padstacks"
        );
        assert_eq!(authoritative(&f.root), before);
    }
    for (section, kind) in [
        ("fields", "symbols"),
        ("drawings", "symbols"),
        ("silkscreen", "footprints"),
        ("fab", "footprints"),
    ] {
        assert_eq!(
            format!(
                "{:#}",
                write(&f.root, f.stack, "padstacks", Some(section), &aperture).unwrap_err()
            ),
            format!("--section {section} requires --kind {kind}")
        );
        assert_eq!(authoritative(&f.root), before);
    }
    assert!(
        write(
            &f.root,
            Uuid::new_v4(),
            "padstacks",
            Some("aperture"),
            &aperture
        )
        .is_err()
    );
    assert!(write(&f.root, f.stack, "padstacks", Some("invalid"), &aperture).is_err());
    assert_eq!(authoritative(&f.root), before);
    let collision = serde_json::json!({"circle":{"diameter_nm":1,"extension":{"uuid":f.package}}});
    assert_eq!(
        format!(
            "{:#}",
            write(&f.root, f.stack, "padstacks", Some("aperture"), &collision).unwrap_err()
        ),
        "validation error: pool source objects: foreign source UUID collision"
    );
    assert_eq!(authoritative(&f.root), before);
    f.cleanup();
}
