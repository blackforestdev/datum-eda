//! Saved numeric body intent preserves the captured mixed raw array.
use super::*;
use eda_engine::api::native_write::library::{
    self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec,
};
use std::path::{Path, PathBuf};
fn drawings(count: usize) -> Value {
    let rectangle = json!({"Rect":{"min":{"x":-2000000,"y":-1000000,"extra":[3,1,3]},"max":{"x":2000000,"y":1000000,"optional":null},"width":100000,"vendor":{"keep":true}}});
    let mut graphics =
        vec![json!({"Line":{"from":{"x":1,"y":2},"to":{"x":3,"y":4},"width":1,"extension":17}})];
    graphics.extend((0..count).map(|_| rectangle.clone()));
    graphics.push(json!({"Circle":{"center":{"x":0,"y":0},"radius":500000,"width":10000}}));
    json!(graphics)
}
fn fixture(count: usize) -> (PathBuf, eda_engine::substrate::DesignModel, Uuid, Value) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.datum/scratch/codex-delivery")
        .join(format!("iltr-form-{}", Uuid::new_v4()));
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Saved body form".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let unit = Uuid::new_v4();
    let symbol = Uuid::new_v4();
    let mut raw = library::pool_symbol_payload(symbol, unit, "Mixed body");
    raw["uuid"] = json!(symbol.simple().to_string());
    raw["drawings"] = drawings(count);
    raw["extension"] = json!({"ordered":["z","a","z"]});
    raw["fields"] =
        json!([{"key":"same","value":"first","extra":9},{"key":"same","value":"second"}]);
    let p = library::build_pool_library_write(
        &model,
        WriteProvenance::new("test", CommitSource::Manual, "Saved form sources"),
        Some("pool"),
        vec![
            Spec::Create {
                target: Target::new("pool", "units", unit),
                object: library::pool_unit_payload(unit, "Body unit", ""),
            },
            Spec::Create {
                target: Target::new("pool", "symbols", symbol),
                object: raw.clone(),
            },
        ],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    (root, model, symbol, raw)
}
fn draft(
    model: &eda_engine::substrate::DesignModel,
    symbol: Uuid,
) -> (LibraryBrowser, LibraryDraft) {
    let browser = LibraryBrowser::capture(Arc::new(model.clone()));
    let entry = browser
        .entries
        .iter()
        .find(|e| e.kind == "symbols" && e.id == symbol)
        .unwrap();
    let draft = LibraryDraft::begin(
        &browser,
        Some(entry),
        "symbols",
        model.model_revision.0.clone(),
        None,
    )
    .unwrap();
    (browser, draft)
}
fn raw(root: &Path, symbol: Uuid) -> Value {
    serde_json::from_slice(
        &std::fs::read(root.join(format!("pool/symbols/{symbol}.json"))).unwrap(),
    )
    .unwrap()
}
#[test]
fn saved_rectangle_mixed_raw_dirty_only_preview_commit_history_and_cold() {
    let (root, mut model, symbol, original) = fixture(1);
    let (browser, mut d) = draft(&model, symbol);
    assert_eq!(d.saved_rectangle_index(), Some(1));
    for (key, value) in [
        ("rect_min_x", "-2000000"),
        ("rect_min_y", "-1000000"),
        ("rect_max_x", "2000000"),
        ("rect_max_y", "1000000"),
        ("rect_width", "100000"),
    ] {
        assert_eq!(d.value(key), value);
    }
    assert!(!d.fields.iter().any(|f| f.key == "unit"));
    assert!(!d.request().unwrap().fields.contains_key("drawings"));
    set(&mut d, "rect_min_x", "-3 mm");
    set(&mut d, "rect_max_y", "4 mm");
    set(&mut d, "rect_width", "0.2 mm");
    let mut expected = original.clone();
    expected["drawings"][1]["Rect"]["min"]["x"] = json!(-3000000);
    expected["drawings"][1]["Rect"]["max"]["y"] = json!(4000000);
    expected["drawings"][1]["Rect"]["width"] = json!(200000);
    let request = d.request().unwrap();
    assert_eq!(request.fields["drawings"], expected["drawings"]);
    assert!(!request.fields.contains_key("pin_anchors"));
    assert!(!request.fields.contains_key("fields"));
    let journal = eda_engine::substrate::transaction_journal_path(&root);
    let before = std::fs::read(&journal).unwrap();
    d.validate_preview(&browser, &root).unwrap();
    assert_eq!(d.preview, Some(expected.clone()));
    assert_eq!(d.electrical.original, Some(original.clone()));
    assert_eq!(raw(&root, symbol), original);
    assert_eq!(std::fs::read(&journal).unwrap(), before);
    let p = build_definition(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        WriteProvenance::new("test", CommitSource::Manual, "Correct body"),
        request,
        false,
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    assert_eq!(raw(&root, symbol), expected);
    model
        .commit_journal_undo(
            &root,
            WriteProvenance::new("test", CommitSource::Manual, "Undo body").into(),
        )
        .unwrap();
    assert_eq!(raw(&root, symbol), original);
    model
        .commit_journal_redo(
            &root,
            WriteProvenance::new("test", CommitSource::Manual, "Redo body").into(),
        )
        .unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    let (_, d) = draft(&cold, symbol);
    assert_eq!(d.value("rect_min_x"), "-3000000");
    assert_eq!(d.electrical.original, Some(expected));
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn saved_rectangle_unavailable_zero_multiple_and_numeric_refusals_do_not_rebuild_graphics() {
    for count in [0, 2] {
        let (root, model, symbol, original) = fixture(count);
        let (browser, mut d) = draft(&model, symbol);
        assert_eq!(d.saved_rectangle_index(), None);
        assert!(!d.fields.iter().any(|f| f.key.starts_with("rect_")));
        set(&mut d, "name", "Metadata remains available");
        assert!(!d.request().unwrap().fields.contains_key("drawings"));
        d.validate_preview(&browser, &root).unwrap();
        assert_eq!(raw(&root, symbol), original);
        std::fs::remove_dir_all(root).unwrap();
    }
    let (root, model, symbol, original) = fixture(1);
    for (key, value) in [
        ("rect_min_x", "2 mm"),
        ("rect_max_y", "-1 mm"),
        ("rect_width", "0"),
        ("rect_width", "-1"),
        ("rect_min_x", "not a length"),
    ] {
        let (_, mut d) = draft(&model, symbol);
        set(&mut d, key, value);
        assert!(d.request().is_err(), "{key}={value}");
        assert_eq!(raw(&root, symbol), original);
    }
    let (_, mut d) = draft(&model, symbol);
    set(&mut d, "rect_min_x", "-2 mm");
    assert!(
        !d.request().unwrap().fields.contains_key("drawings"),
        "equivalent numeric text must not rewrite raw graphics"
    );
    std::fs::remove_dir_all(root).unwrap();
}
