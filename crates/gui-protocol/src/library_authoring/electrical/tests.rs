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

fn saved_entry(kind: &str, id: Uuid, raw: Value) -> crate::library_browser::LibraryEntry {
    crate::library_browser::LibraryEntry {
        id,
        kind: kind.into(),
        name: raw["name"].as_str().unwrap().into(),
        source_path: format!("pool/{kind}/{id}.json"),
        revision: Some(3),
        raw,
        warning: None,
    }
}

#[test]
fn saved_unit_form_edits_existing_metadata_without_defaults_or_raw_row_loss() {
    let (unit, first, second) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let original = json!({"uuid":unit,"name":"Saved Unit","pins":{
        first.to_string():{"uuid":first,"name":"A","direction":"Input","electrical_type":null,"swap_group":2,"custom":[7]},
        second.to_string():{"uuid":second,"name":"B","direction":"Passive","custom":{"keep":true}}
    },"extension":"exact"});
    let entry = saved_entry("units", unit, original.clone());
    let browser = LibraryBrowser {
        entries: vec![entry.clone()],
        ..Default::default()
    };
    let mut draft = LibraryDraft::begin(
        &browser,
        Some(&entry),
        "units",
        "saved-revision".into(),
        None,
    )
    .unwrap();
    assert!(entry.editable());
    assert!(draft.request().unwrap().fields.is_empty());
    let index = draft
        .electrical
        .pins
        .keys()
        .position(|k| k == &first.to_string())
        .unwrap();
    draft.switch_pin(Some(index), false).unwrap();
    assert_eq!(draft.value("pin_type"), "Input");
    set(&mut draft, "pin_name", "Renamed A");
    let index = draft
        .electrical
        .pins
        .keys()
        .position(|k| k == &second.to_string())
        .unwrap();
    draft.switch_pin(Some(index), false).unwrap();
    set(&mut draft, "pin_type", "PowerIn");
    let request = draft.request().unwrap();
    let mut expected = original["pins"].clone();
    expected[first.to_string()]["name"] = json!("Renamed A");
    expected[second.to_string()]["direction"] = json!("PowerIn");
    assert_eq!(
        request.fields,
        json!({"pins":expected}).as_object().unwrap().clone()
    );
    assert_eq!(draft.electrical.original, Some(original));
    let mut adding = draft.clone();
    adding.switch_pin(None, false).unwrap();
    assert!(adding.request().is_err());
    adding.switch_pin(None, true).unwrap();
    assert_eq!(adding.request().unwrap().fields, request.fields);
    let mut removing = draft.clone();
    removing.switch_pin(None, true).unwrap();
    assert_eq!(
        removing.request().unwrap().fields["pin_removals"],
        json!([second])
    );
    assert_eq!(draft.request().unwrap().fields, request.fields);
}

#[test]
fn saved_symbol_form_retains_order_graphics_and_legacy_anchor_representation() {
    let (unit, first, second, symbol) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let logical = saved_entry(
        "units",
        unit,
        json!({"name":"Named Unit","pins":{
            first.to_string():{"name":"A"},second.to_string():{"name":"B"}
        }}),
    );
    let original = json!({"uuid":symbol,"name":"Saved graphic","unit":unit,
        "drawings":[{"Circle":{"center":{"x":0,"y":0},"radius":500000,"width":1}}],
        "fields":[{"key":"note","value":"keep","custom":2}],
        "pin_anchors":[
            {"pin":second,"position":{"x":1000000,"y":0},"style":{"orientation":"Left","length_nm":500000,"decoration":"clock","extra":true}},
            {"pin":first,"position":{"x":-1000000,"y":0,"extra":9},"orientation":"Right","length_nm":500000,"number":"1"}
        ],"extension":[1,2]});
    let entry = saved_entry("symbols", symbol, original.clone());
    let browser = LibraryBrowser {
        entries: vec![logical, entry.clone()],
        ..Default::default()
    };
    let mut draft = LibraryDraft::begin(
        &browser,
        Some(&entry),
        "symbols",
        "saved-revision".into(),
        None,
    )
    .unwrap();
    assert!(entry.editable());
    assert!(draft.request().unwrap().fields.is_empty());
    assert_eq!(draft.electrical_unit(), unit.to_string());
    assert!(
        draft
            .fields
            .iter()
            .all(|f| f.key != "unit" && f.key != "anchor_pin" && !f.key.starts_with("rect_"))
    );
    let index = draft
        .electrical
        .anchors
        .keys()
        .position(|k| k == &first.to_string())
        .unwrap();
    draft.switch_anchor(Some(index), false).unwrap();
    assert_eq!(draft.value("anchor_orientation"), "Right");
    set(&mut draft, "anchor_x", "-1.250001mm");
    set(&mut draft, "anchor_orientation", "Down");
    let index = draft
        .electrical
        .anchors
        .keys()
        .position(|k| k == &second.to_string())
        .unwrap();
    draft.switch_anchor(Some(index), false).unwrap();
    set(&mut draft, "anchor_length", "0.75mm");
    let request = draft.request().unwrap();
    let mut expected = original["pin_anchors"].clone();
    expected[1]["position"]["x"] = json!(-1250001);
    expected[1]["orientation"] = json!("Down");
    expected[0]["style"]["length_nm"] = json!(750000);
    assert_eq!(
        request.fields,
        json!({"pin_anchors":expected}).as_object().unwrap().clone()
    );
    assert_eq!(draft.electrical.original, Some(original));
    let mut adding = draft.clone();
    adding.switch_anchor(None, false).unwrap();
    assert!(adding.request().is_err());
    adding.switch_anchor(None, true).unwrap();
    assert_eq!(adding.request().unwrap().fields, request.fields);
    let mut removing = draft.clone();
    removing.switch_anchor(None, true).unwrap();
    assert_eq!(
        removing.request().unwrap().fields["anchor_removals"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    set(&mut draft, "anchor_x", "NaN");
    assert!(draft.request().is_err());
}

#[test]
fn saved_symbol_name_only_does_not_materialize_absent_anchor_or_style_defaults() {
    let unit = Uuid::new_v4();
    let symbol = Uuid::new_v4();
    let logical = saved_entry("units", unit, json!({"name":"Unit","pins":{}}));
    let original = json!({"uuid":symbol,"name":"Before","unit":unit,"extension":null});
    let entry = saved_entry("symbols", symbol, original.clone());
    let browser = LibraryBrowser {
        entries: vec![logical, entry.clone()],
        ..Default::default()
    };
    let mut draft =
        LibraryDraft::begin(&browser, Some(&entry), "symbols", "revision".into(), None).unwrap();
    set(&mut draft, "name", "After");
    assert_eq!(
        draft.request().unwrap().fields,
        json!({"name":"After"}).as_object().unwrap().clone()
    );
    assert_eq!(draft.electrical.original, Some(original));
}

#[test]
fn saved_electrical_forms_refuse_alias_identities_and_preserve_unique_spelling() {
    let unit = Uuid::new_v4();
    let symbol = Uuid::new_v4();
    let pin = Uuid::parse_str("abcdefab-cdef-4abc-8def-abcdefabcdef").unwrap();
    let lower = pin.to_string();
    let upper = lower.to_ascii_uppercase();
    let pins = json!({lower.clone():{"uuid":pin,"name":"Lower","direction":"Input"},upper.clone():{"uuid":pin,"name":"Upper","direction":"Passive"}});
    let aliased = saved_entry(
        "units",
        unit,
        json!({"uuid":unit,"name":"Aliases","pins":pins}),
    );
    let browser = LibraryBrowser {
        entries: vec![aliased.clone()],
        ..Default::default()
    };
    assert!(LibraryDraft::begin(&browser, Some(&aliased), "units", "r".into(), None).is_err());
    let unique = saved_entry(
        "units",
        unit,
        json!({"uuid":unit,"name":"Unique","pins":{
            upper.clone():{"uuid":pin,"name":"A","direction":"Input","metadata":"exact"}
        }}),
    );
    let browser = LibraryBrowser {
        entries: vec![unique.clone()],
        ..Default::default()
    };
    let mut draft =
        LibraryDraft::begin(&browser, Some(&unique), "units", "r".into(), None).unwrap();
    draft.switch_pin(Some(0), false).unwrap();
    set(&mut draft, "pin_name", "Changed");
    let request = draft.request().unwrap();
    assert_eq!(request.fields["pins"][&upper]["name"], json!("Changed"));
    assert!(request.fields["pins"].get(&lower).is_none());
    let graphic = saved_entry(
        "symbols",
        symbol,
        json!({"name":"Aliases","unit":unit,"pin_anchors":[
            {"pin":lower,"position":{"x":0,"y":0}},{"pin":upper,"position":{"x":1,"y":0}}
        ]}),
    );
    let browser = LibraryBrowser {
        entries: vec![unique, graphic.clone()],
        ..Default::default()
    };
    assert!(LibraryDraft::begin(&browser, Some(&graphic), "symbols", "r".into(), None).is_err());
}

#[path = "rectangle_tests.rs"]
mod rectangle_tests;
