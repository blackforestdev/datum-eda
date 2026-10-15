//! Manual Symbol field replacement preserves raw content and canonical history.
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
        let root = crate::main_tests::development_workspace::path("footprint-reference-audit")
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
        original["drawings"] =
            serde_json::json!([{"Line":{"from":{"x":1,"y":2},"to":{"x":3,"y":4},"width":1}}]);
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

#[test]
fn symbol_fields_add_change_remove_clear_preserve_payload_and_history() {
    let f = Fixture::new("symbol-fields-history");
    let original = payload(&f.root, f.symbol);
    assert!(original.get("fields").is_none());
    let row = serde_json::json!({"key":"Value","value":"first","extension":{"uuid":Uuid::new_v4(),"kept":true}});
    let duplicate = serde_json::json!({"key":"Value","value":"second","position":{"x":-1,"y":2},"visible":true});
    let mut changed = row.clone();
    changed["value"] = serde_json::json!("changed");
    let arrays = [
        serde_json::json!([row.clone(), duplicate.clone()]),
        serde_json::json!([duplicate, changed.clone()]),
        serde_json::json!([changed]),
        serde_json::json!([]),
    ];
    for fields in arrays {
        let before = payload(&f.root, f.symbol);
        let model = ProjectResolver::new(&f.root).resolve().unwrap();
        let view: serde_json::Value = serde_json::from_str(
            &write(&f.root, f.symbol, &fields, Some("fields"), "symbols").unwrap(),
        )
        .unwrap();
        assert_eq!(view["action"], "set");
        let mut expected = before.clone();
        expected["fields"] = fields;
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
    let mut replacement = payload(&f.root, f.symbol);
    replacement["name"] = serde_json::json!("Whole replacement");
    replacement.as_object_mut().unwrap().remove("fields");
    write(&f.root, f.symbol, &replacement, None, "symbols").unwrap();
    assert_eq!(payload(&f.root, f.symbol), replacement);
    std::fs::remove_dir_all(&f.root).unwrap();
    std::fs::remove_file(f.root.with_extension("input.json")).unwrap();
}

#[test]
fn symbol_fields_refusals_preserve_authority_and_reference_priority() {
    let f = Fixture::new("symbol-fields-refusals");
    let before = authoritative(&f.root);
    for fields in [
        serde_json::json!(null),
        serde_json::json!({}),
        serde_json::json!(["row"]),
        serde_json::json!([{"value":"a"}]),
        serde_json::json!([{"key":"k","value":1}]),
        serde_json::json!([{"key":"k","value":"v","visible":null}]),
        serde_json::json!([{"key":"k","value":"v","position":{"x":0.5,"y":0}}]),
    ] {
        let error = format!(
            "{:#}",
            write(&f.root, f.symbol, &fields, Some("fields"), "symbols").unwrap_err()
        );
        assert!(error.starts_with("invalid symbol fields:"), "{error}");
        assert_eq!(authoritative(&f.root), before);
    }
    let fields = serde_json::json!([]);
    assert_eq!(
        format!(
            "{:#}",
            write(&f.root, f.symbol, &fields, Some("fields"), "packages").unwrap_err()
        ),
        "--section fields requires --kind symbols"
    );
    assert!(write(&f.root, f.symbol, &fields, Some("anchors"), "symbols").is_err());
    assert!(write(&f.root, Uuid::new_v4(), &fields, Some("fields"), "symbols").is_err());
    assert_eq!(authoritative(&f.root), before);
    let collision = serde_json::json!([{"key":"x","value":"y","extension":{"uuid":f.unit}}]);
    let error = format!(
        "{:#}",
        write(&f.root, f.symbol, &collision, Some("fields"), "symbols").unwrap_err()
    );
    assert_eq!(
        error,
        "validation error: pool source objects: foreign source UUID collision"
    );
    assert_eq!(authoritative(&f.root), before);
    std::fs::remove_dir_all(&f.root).unwrap();
    std::fs::remove_file(f.root.with_extension("input.json")).unwrap();
}
