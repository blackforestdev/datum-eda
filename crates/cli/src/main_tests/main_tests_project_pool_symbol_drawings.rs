//! Manual Symbol drawing replacement preserves primitive schema and raw history.
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
    super::main_tests_project_pool_library::query_pool_object_payload(root, "symbols", id)
}
fn write(
    root: &Path,
    id: Uuid,
    value: &serde_json::Value,
    section: Option<&str>,
    kind: &str,
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
    unit: Uuid,
    symbol: Uuid,
}
impl Fixture {
    fn new(label: &str) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.datum/footprint-reference-audit")
            .join(format!("{label}-{}", Uuid::new_v4()));
        create_native_project(&root, Some(label.to_string())).unwrap();
        let unit = Uuid::new_v4();
        let symbol = Uuid::new_v4();
        command(
            &root,
            "create-pool-unit",
            &["--unit", &unit.to_string(), "--name", "Manual Unit"],
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
                "Manual Symbol",
            ],
        )
        .unwrap();
        let pin = Uuid::new_v4();
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
        let mut original = payload(&root, symbol);
        original["pin_anchors"] = serde_json::json!([{"pin":pin,"position":{"x":11,"y":12},"style":{"orientation":"Right"}}]);
        original["fields"] = serde_json::json!([{ "key":"Value", "value":"first", "extension":{"uuid":Uuid::new_v4()} }, {"key":"Value", "value":"second"}]);
        original["default_refdes_prefix"] = serde_json::json!("U");
        original["extension"] = serde_json::json!({"uuid":Uuid::new_v4(),"retained":[1,2]});
        write(&root, symbol, &original, None, "symbols").unwrap();
        Self { root, unit, symbol }
    }
}
fn authoritative(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn collect(path: &Path, rows: &mut Vec<(PathBuf, Vec<u8>)>) {
        if path.is_dir() {
            for entry in std::fs::read_dir(path).unwrap() {
                collect(&entry.unwrap().path(), rows);
            }
        } else if path.is_file() {
            rows.push((path.to_path_buf(), std::fs::read(path).unwrap()));
        }
    }
    let mut rows = Vec::new();
    for path in [
        "pool",
        "board",
        "schematic",
        "project.json",
        ".datum/journal/transactions.jsonl",
    ] {
        collect(&root.join(path), &mut rows);
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}

fn shapes() -> Vec<(&'static str, Vec<&'static str>, serde_json::Value)> {
    vec![
        (
            "line",
            vec![
                "--from-x-nm",
                "1",
                "--from-y-nm",
                "2",
                "--to-x-nm",
                "3",
                "--to-y-nm",
                "4",
                "--width-nm",
                "1",
            ],
            serde_json::json!({"Line":{"from":{"x":1,"y":2},"to":{"x":3,"y":4},"width":1}}),
        ),
        (
            "rect",
            vec![
                "--min-x-nm",
                "0",
                "--min-y-nm",
                "0",
                "--max-x-nm",
                "4",
                "--max-y-nm",
                "5",
                "--width-nm",
                "1",
            ],
            serde_json::json!({"Rect":{"min":{"x":0,"y":0},"max":{"x":4,"y":5},"width":1}}),
        ),
        (
            "circle",
            vec![
                "--center-x-nm",
                "1",
                "--center-y-nm",
                "2",
                "--radius-nm",
                "3",
                "--width-nm",
                "1",
            ],
            serde_json::json!({"Circle":{"center":{"x":1,"y":2},"radius":3,"width":1}}),
        ),
        (
            "polygon",
            vec!["--vertices", "0,0;4,0;4,5", "--width-nm", "1"],
            serde_json::json!({"Polygon":{"polygon":{"vertices":[{"x":0,"y":0},{"x":4,"y":0},{"x":4,"y":5}],"closed":true},"width":1}}),
        ),
        (
            "arc",
            vec![
                "--x-nm",
                "1",
                "--y-nm",
                "2",
                "--radius-nm",
                "3",
                "--start-angle",
                "0",
                "--end-angle",
                "900",
                "--width-nm",
                "1",
            ],
            serde_json::json!({"Arc":{"arc":{"center":{"x":1,"y":2},"radius":3,"start_angle":0,"end_angle":900},"width":1}}),
        ),
        (
            "text",
            vec![
                "--text",
                "  Label  ",
                "--x-nm",
                "1",
                "--y-nm",
                "2",
                "--rotation",
                "900",
            ],
            serde_json::json!({"Text":{"text":"Label","position":{"x":1,"y":2},"rotation":900}}),
        ),
    ]
}
#[test]
fn symbol_drawings_six_append_correct_remove_reorder_clear_and_history() {
    let f = Fixture::new("symbol-drawings-history");
    let original = payload(&f.root, f.symbol);
    assert!(original.get("drawings").is_none());
    let mut expected = original.clone();
    expected["drawings"] = serde_json::json!([]);
    for (shape, args, primitive) in shapes() {
        let identity = f.symbol.to_string();
        let mut argv = vec!["--symbol", &identity];
        argv.extend(args);
        command(&f.root, &format!("add-pool-symbol-{shape}"), &argv).unwrap();
        expected["drawings"].as_array_mut().unwrap().push(primitive);
        assert_eq!(payload(&f.root, f.symbol), expected);
    }
    let mut corrected = expected["drawings"].clone();
    corrected[0]["Line"]["to"]["x"] = serde_json::json!(9);
    corrected[0]["Line"]["extension"] = serde_json::json!({"uuid":Uuid::new_v4(),"retained":true});
    let repeated = corrected[5].clone();
    corrected.as_array_mut().unwrap().push(repeated);
    let mut removed = corrected.clone();
    removed.as_array_mut().unwrap().remove(1);
    let mut reordered = removed.clone();
    reordered.as_array_mut().unwrap().reverse();
    for drawings in [corrected, removed, reordered, serde_json::json!([])] {
        let before = payload(&f.root, f.symbol);
        let model = ProjectResolver::new(&f.root).resolve().unwrap();
        let result: serde_json::Value = serde_json::from_str(
            &write(&f.root, f.symbol, &drawings, Some("drawings"), "symbols").unwrap(),
        )
        .unwrap();
        assert_eq!(result["action"], "set");
        let mut expected = before.clone();
        expected["drawings"] = drawings;
        assert_eq!(payload(&f.root, f.symbol), expected);
        let after = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(after.journal.len(), model.journal.len() + 1);
        assert_eq!(
            after.objects[&f.symbol].object_revision.0,
            model.objects[&f.symbol].object_revision.0 + 1
        );
        assert_eq!(
            after.objects[&f.unit].object_revision,
            model.objects[&f.unit].object_revision
        );
        assert!(matches!(
            after.journal.last().unwrap().operations.as_slice(),
            [Operation::SetPoolLibraryObject { .. }]
        ));
        command(&f.root, "undo", &[]).unwrap();
        assert_eq!(payload(&f.root, f.symbol), before);
        command(&f.root, "redo", &[]).unwrap();
        assert_eq!(payload(&f.root, f.symbol), expected);
    }
    command(&f.root, "query", &["resolve-debug"]).unwrap();
    command(&f.root, "validate", &[]).unwrap();
    std::fs::remove_dir_all(&f.root).unwrap();
    std::fs::remove_file(f.root.with_extension("input.json")).unwrap();
}
#[test]
fn symbol_drawings_preserve_existing_full_json_shape_only_admission() {
    let f = Fixture::new("symbol-drawings-shape-only");
    let drawings = serde_json::json!([
        {"Line":{"from":{"x":0,"y":0},"to":{"x":0,"y":0},"width":0}},
        {"Rect":{"min":{"x":3,"y":2},"max":{"x":1,"y":0},"width":-1}},
        {"Circle":{"center":{"x":0,"y":0},"radius":-1,"width":0}},
        {"Polygon":{"polygon":{"vertices":[],"closed":false},"width":0}},
        {"Arc":{"arc":{"center":{"x":0,"y":0},"radius":0,"start_angle":0,"end_angle":0},"width":-1}},
        {"Text":{"text":"   ","position":{"x":0,"y":0},"rotation":-123}}
    ]);
    let original = payload(&f.root, f.symbol);
    let mut expected = original.clone();
    expected["drawings"] = drawings.clone();
    write(&f.root, f.symbol, &expected, None, "symbols").unwrap();
    assert_eq!(payload(&f.root, f.symbol), expected);
    command(&f.root, "undo", &[]).unwrap();
    assert_eq!(payload(&f.root, f.symbol), original);
    write(&f.root, f.symbol, &drawings, Some("drawings"), "symbols").unwrap();
    assert_eq!(payload(&f.root, f.symbol), expected);
    command(&f.root, "validate", &[]).unwrap();
    std::fs::remove_dir_all(&f.root).unwrap();
    std::fs::remove_file(f.root.with_extension("input.json")).unwrap();
}
#[test]
fn symbol_drawings_refusals_preserve_authority_and_raw_uuid_ownership() {
    let f = Fixture::new("symbol-drawings-refusals");
    let before = authoritative(&f.root);
    for drawings in [
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
            write(&f.root, f.symbol, &drawings, Some("drawings"), "symbols").unwrap_err()
        );
        assert!(error.starts_with("invalid symbol drawings:"), "{error}");
        assert_eq!(authoritative(&f.root), before);
    }
    assert_eq!(
        format!(
            "{:#}",
            write(
                &f.root,
                f.symbol,
                &serde_json::json!([]),
                Some("drawings"),
                "units"
            )
            .unwrap_err()
        ),
        "--section drawings requires --kind symbols"
    );
    assert!(
        write(
            &f.root,
            f.symbol,
            &serde_json::json!([]),
            Some("invalid"),
            "symbols"
        )
        .is_err()
    );
    assert!(
        write(
            &f.root,
            Uuid::new_v4(),
            &serde_json::json!([]),
            Some("drawings"),
            "symbols"
        )
        .is_err()
    );
    assert_eq!(authoritative(&f.root), before);
    let mut drawing = shapes()[0].2.clone();
    drawing["Line"]["extension"] = serde_json::json!({"uuid":f.unit});
    assert_eq!(
        format!(
            "{:#}",
            write(
                &f.root,
                f.symbol,
                &serde_json::json!([drawing]),
                Some("drawings"),
                "symbols"
            )
            .unwrap_err()
        ),
        "validation error: pool source objects: foreign source UUID collision"
    );
    assert_eq!(authoritative(&f.root), before);
    std::fs::remove_dir_all(&f.root).unwrap();
    std::fs::remove_file(f.root.with_extension("input.json")).unwrap();
}
