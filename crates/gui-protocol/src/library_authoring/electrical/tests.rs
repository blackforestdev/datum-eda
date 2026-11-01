//! GUI intent must round-trip through the existing typed native builder.
use super::*;
use eda_engine::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
};
use eda_engine::substrate::ProjectResolver;
use std::sync::Arc;

fn set(draft: &mut LibraryDraft, key: &str, value: &str) {
    draft.focused = draft.fields.iter().position(|f| f.key == key).unwrap();
    draft.change(value.into());
}

#[test]
fn electrical_draft_to_native_builder_preserves_identity_units_and_pin_style_without_preview_writes()
 {
    let root = std::env::temp_dir().join(format!("datum-electrical-form-{}", Uuid::new_v4()));
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "GUI electrical intent".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let browser = LibraryBrowser::capture(Arc::new(model.clone()));
    let mut unit = LibraryDraft::begin(
        &browser,
        None,
        "units",
        model.model_revision.0.clone(),
        None,
    )
    .unwrap();
    set(&mut unit, "name", "Test logical pins");
    set(&mut unit, "pin_name", "A");
    set(&mut unit, "pin_type", "Passive");
    let first = unit.electrical.pin_id.unwrap();
    unit.switch_pin(None, false).unwrap();
    set(&mut unit, "pin_name", "B");
    set(&mut unit, "pin_type", "Passive");
    let second = unit.electrical.pin_id.unwrap();
    assert_ne!(first, second);
    let before = model.model_revision.clone();
    unit.validate_preview(&browser, &root).unwrap();
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .model_revision,
        before
    );
    let prepared = build_definition(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        WriteProvenance::new("test", CommitSource::Test, "GUI unit intent"),
        unit.request().unwrap(),
        true,
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let browser = LibraryBrowser::capture(Arc::new(model.clone()));
    let mut symbol = LibraryDraft::begin(
        &browser,
        None,
        "symbols",
        model.model_revision.0.clone(),
        None,
    )
    .unwrap();
    assert!(
        symbol
            .fields
            .iter()
            .find(|f| f.key == "unit")
            .unwrap()
            .choices
            .iter()
            .any(|(id, name)| id == &unit.id.to_string() && name.contains("Test logical pins"))
    );
    set(&mut symbol, "name", "Test visible symbol");
    set(&mut symbol, "unit", &unit.id.to_string());
    for (key, value) in [
        ("rect_min_x", "-1 mm"),
        ("rect_min_y", "-1 mm"),
        ("rect_max_x", "1 mm"),
        ("rect_max_y", "1 mm"),
        ("rect_width", "0.15 mm"),
        ("anchor_pin", first.to_string().as_str()),
        ("anchor_x", "-2 mm"),
        ("anchor_y", "0 mm"),
        ("anchor_orientation", "Right"),
        ("anchor_length", "1 mm"),
    ] {
        set(&mut symbol, key, value);
    }
    symbol.switch_anchor(None, false).unwrap();
    for (key, value) in [
        ("anchor_pin", second.to_string().as_str()),
        ("anchor_x", "2 mm"),
        ("anchor_y", "0 mm"),
        ("anchor_orientation", "Left"),
        ("anchor_length", "1 mm"),
    ] {
        set(&mut symbol, key, value);
    }
    let before = model.model_revision.clone();
    symbol.validate_preview(&browser, &root).unwrap();
    let preview = symbol.preview.clone().unwrap();
    assert_eq!(
        preview["drawings"][0]["Rect"]["min"]["x"],
        json!(-1_000_000)
    );
    assert_eq!(preview["pin_anchors"].as_array().unwrap().len(), 2);
    symbol.validate_preview(&browser, &root).unwrap();
    assert_eq!(symbol.preview, Some(preview));
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .model_revision,
        before
    );
    symbol.switch_anchor(None, false).unwrap();
    let anchors = symbol.electrical.anchors.clone();
    let first_index = symbol
        .electrical
        .anchors
        .keys()
        .position(|id| id == &first.to_string())
        .unwrap();
    symbol.switch_anchor(Some(first_index), false).unwrap();
    set(&mut symbol, "anchor_pin", &second.to_string());
    assert!(
        symbol
            .request()
            .unwrap_err()
            .contains("already has an anchor"),
        "editing A must not overwrite B"
    );
    assert_eq!(symbol.electrical.anchors, anchors);
    set(&mut symbol, "anchor_pin", &first.to_string());
    symbol.switch_anchor(None, false).unwrap();
    set(&mut symbol, "anchor_pin", &second.to_string());
    assert!(
        symbol
            .request()
            .unwrap_err()
            .contains("already has an anchor"),
        "Add anchor must not overwrite B"
    );
    symbol.switch_anchor(None, true).unwrap();
    assert_eq!(
        symbol.electrical.anchors, anchors,
        "discarding the conflicting new row must preserve B"
    );
    let second_index = symbol
        .electrical
        .anchors
        .keys()
        .position(|id| id == &second.to_string())
        .unwrap();
    symbol.switch_anchor(Some(second_index), false).unwrap();
    set(&mut symbol, "anchor_orientation", "left");
    assert!(symbol.preview.is_none());
    assert!(
        symbol.validate_preview(&browser, &root).is_err(),
        "existing enum is case-sensitive"
    );
    drop(symbol);
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .model_revision,
        before
    );
    drop(browser);
    drop(model);
    std::fs::remove_dir_all(root).unwrap();
}
