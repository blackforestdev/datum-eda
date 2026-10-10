//! Footprint collection edits share raw primitive admission and canonical history.
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
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.datum/footprint-reference-audit")
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
        let mut object = payload(&root, footprint);
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
fn shapes() -> Vec<(&'static str, Vec<&'static str>, serde_json::Value)> {
    vec![
        (
            "line",
            vec![
                "--from-x-nm",
                "0",
                "--from-y-nm",
                "0",
                "--to-x-nm",
                "10",
                "--to-y-nm",
                "20",
                "--width-nm",
                "2",
            ],
            serde_json::json!({"Line": {"from": {"x": 0, "y": 0}, "to": {"x": 10, "y": 20}, "width": 2}}),
        ),
        (
            "rect",
            vec![
                "--min-x-nm",
                "0",
                "--min-y-nm",
                "0",
                "--max-x-nm",
                "10",
                "--max-y-nm",
                "20",
                "--width-nm",
                "2",
            ],
            serde_json::json!({"Rect": {"min": {"x": 0, "y": 0}, "max": {"x": 10, "y": 20}, "width": 2}}),
        ),
        (
            "circle",
            vec![
                "--center-x-nm",
                "10",
                "--center-y-nm",
                "20",
                "--radius-nm",
                "30",
                "--width-nm",
                "2",
            ],
            serde_json::json!({"Circle": {"center": {"x": 10, "y": 20}, "radius": 30, "width": 2}}),
        ),
        (
            "polygon",
            vec!["--vertices", "0,0;10,0;0,20", "--width-nm", "2"],
            serde_json::json!({"Polygon": {"polygon": {"vertices": [{"x": 0, "y": 0},{"x": 10, "y": 0},{"x": 0, "y": 20}], "closed": true}, "width": 2}}),
        ),
    ]
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
fn footprint_graphic_edit_both_collections_correct_remove_reorder_clear_history() {
    let f = Fixture::new("footprint-collection-history");
    for field in ["silkscreen", "fab"] {
        let before = payload(&f.root, f.footprint);
        let mut expected = before.clone();
        for (shape, args, primitive) in shapes() {
            let identity = f.footprint.to_string();
            let mut argv = vec!["--footprint", &identity, "--target", field];
            argv.extend(args);
            command(&f.root, &format!("add-pool-footprint-{shape}"), &argv).unwrap();
            expected[field].as_array_mut().unwrap().push(primitive);
            assert_eq!(payload(&f.root, f.footprint), expected);
        }
        let mut corrected = expected[field].clone();
        corrected[0]["Line"]["to"]["x"] = serde_json::json!(99);
        corrected[0]["Line"]["extension"] = serde_json::json!({"uuid":Uuid::new_v4(),"kept":true});
        let repeated = corrected[3].clone();
        corrected.as_array_mut().unwrap().push(repeated);
        let mut removed = corrected.clone();
        removed.as_array_mut().unwrap().remove(1);
        let mut reordered = removed.clone();
        reordered.as_array_mut().unwrap().reverse();
        for graphics in [corrected, removed, reordered, serde_json::json!([])] {
            let original = payload(&f.root, f.footprint);
            let model = ProjectResolver::new(&f.root).resolve().unwrap();
            let view: serde_json::Value = serde_json::from_str(
                &write(&f.root, f.footprint, "footprints", Some(field), &graphics).unwrap(),
            )
            .unwrap();
            assert_eq!(view["action"], "set");
            let mut expected = original.clone();
            expected[field] = graphics;
            assert_eq!(payload(&f.root, f.footprint), expected);
            let after = ProjectResolver::new(&f.root).resolve().unwrap();
            assert_eq!(after.journal.len(), model.journal.len() + 1);
            assert_eq!(
                after.objects[&f.footprint].object_revision.0,
                model.objects[&f.footprint].object_revision.0 + 1
            );
            assert!(matches!(
                after.journal.last().unwrap().operations.as_slice(),
                [Operation::SetPoolLibraryObject { .. }]
            ));
            for id in [f.package, f.stack] {
                assert_eq!(
                    after.objects[&id].object_revision,
                    model.objects[&id].object_revision
                );
            }
            command(&f.root, "undo", &[]).unwrap();
            assert_eq!(payload(&f.root, f.footprint), original);
            command(&f.root, "redo", &[]).unwrap();
            assert_eq!(payload(&f.root, f.footprint), expected);
        }
    }
    command(&f.root, "query", &["resolve-debug"]).unwrap();
    command(&f.root, "validate", &[]).unwrap();
    std::fs::remove_dir_all(&f.root).unwrap();
    std::fs::remove_file(f.root.with_extension("input.json")).unwrap();
}
#[test]
fn footprint_graphic_edit_preserves_six_primitive_whole_json_schema_admission() {
    let f = Fixture::new("footprint-collection-shape-only");
    let graphics = serde_json::json!([
        {"Line":{"from":{"x":0,"y":0},"to":{"x":0,"y":0},"width":0}},
        {"Rect":{"min":{"x":3,"y":2},"max":{"x":1,"y":0},"width":-1}},
        {"Circle":{"center":{"x":0,"y":0},"radius":-1,"width":0}},
        {"Polygon":{"polygon":{"vertices":[],"closed":false},"width":0}},
        {"Arc":{"arc":{"center":{"x":0,"y":0},"radius":0,"start_angle":0,"end_angle":0},"width":-1}},
        {"Text":{"text":"   ","position":{"x":0,"y":0},"rotation":-123}}
    ]);
    for field in ["silkscreen", "fab"] {
        let original = payload(&f.root, f.footprint);
        let mut expected = original.clone();
        expected[field] = graphics.clone();
        write(&f.root, f.footprint, "footprints", None, &expected).unwrap();
        assert_eq!(payload(&f.root, f.footprint), expected);
        command(&f.root, "undo", &[]).unwrap();
        assert_eq!(payload(&f.root, f.footprint), original);
        write(&f.root, f.footprint, "footprints", Some(field), &graphics).unwrap();
        assert_eq!(payload(&f.root, f.footprint), expected);
        command(&f.root, "undo", &[]).unwrap();
        assert_eq!(payload(&f.root, f.footprint), original);
    }
    command(&f.root, "validate", &[]).unwrap();
    std::fs::remove_dir_all(&f.root).unwrap();
    std::fs::remove_file(f.root.with_extension("input.json")).unwrap();
}
#[test]
fn footprint_graphic_edit_refusals_preserve_authority_and_kind_routing() {
    let f = Fixture::new("footprint-collection-refusals");
    let before = authoritative(&f.root);
    for field in ["silkscreen", "fab"] {
        for graphics in [
            serde_json::json!(null),
            serde_json::json!({}),
            serde_json::json!(["row"]),
            serde_json::json!([{"Unknown":{}}]),
            serde_json::json!([{"Circle":{"center":{"x":0,"y":0},"radius":"x","width":1}}]),
            serde_json::json!([{"Line":{"from":{"x":0,"y":0},"width":1}}]),
            serde_json::json!([{"Arc":{"arc":{"center":{"x":0,"y":0},"radius":1,"start_angle":2147483648_i64,"end_angle":0},"width":1}}]),
        ] {
            let error = format!(
                "{:#}",
                write(&f.root, f.footprint, "footprints", Some(field), &graphics).unwrap_err()
            );
            assert!(
                error.starts_with(&format!("invalid footprint {field}:")),
                "{error}"
            );
            assert_eq!(authoritative(&f.root), before);
        }
        assert_eq!(
            format!(
                "{:#}",
                write(
                    &f.root,
                    f.footprint,
                    "symbols",
                    Some(field),
                    &serde_json::json!([])
                )
                .unwrap_err()
            ),
            format!("--section {field} requires --kind footprints")
        );
        assert!(
            write(
                &f.root,
                Uuid::new_v4(),
                "footprints",
                Some(field),
                &serde_json::json!([])
            )
            .is_err()
        );
        assert_eq!(authoritative(&f.root), before);
        let mut graphic = shapes()[0].2.clone();
        graphic["Line"]["extension"] = serde_json::json!({"uuid":f.package});
        assert_eq!(
            format!(
                "{:#}",
                write(
                    &f.root,
                    f.footprint,
                    "footprints",
                    Some(field),
                    &serde_json::json!([graphic])
                )
                .unwrap_err()
            ),
            "validation error: pool source objects: foreign source UUID collision"
        );
        assert_eq!(authoritative(&f.root), before);
    }
    for section in ["fields", "drawings"] {
        assert_eq!(
            format!(
                "{:#}",
                write(
                    &f.root,
                    f.footprint,
                    "footprints",
                    Some(section),
                    &serde_json::json!([])
                )
                .unwrap_err()
            ),
            format!("--section {section} requires --kind symbols")
        );
        assert_eq!(authoritative(&f.root), before);
    }
    assert!(
        write(
            &f.root,
            f.footprint,
            "footprints",
            Some("invalid"),
            &serde_json::json!([])
        )
        .is_err()
    );
    assert_eq!(authoritative(&f.root), before);
    std::fs::remove_dir_all(&f.root).unwrap();
    std::fs::remove_file(f.root.with_extension("input.json")).unwrap();
}
