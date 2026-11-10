//! Saved pin rows retain fresh intent and original raw metadata across selection.
use super::*;
fn entry(kind: &str, id: Uuid, raw: Value) -> LibraryEntry {
    LibraryEntry {
        id,
        kind: kind.into(),
        name: raw["name"].as_str().unwrap().into(),
        source_path: format!("pool/{kind}/{id}.json"),
        revision: Some(0),
        raw,
        warning: None,
    }
}
fn set(d: &mut LibraryDraft, key: &str, value: &str) {
    d.focused = d.fields.iter().position(|f| f.key == key).unwrap();
    d.change(value.into());
}
#[test]
fn saved_new_pin_preserves_raw_alias_and_dirty_rows_and_discards_only_new_intent() {
    let [unit, old] = std::array::from_fn(|_| Uuid::new_v4());
    let alias = old.simple().to_string().to_uppercase();
    let raw = json!({"uuid":unit,"name":"Saved","pins":{alias.clone():{"uuid":old,"name":"Old","direction":"Input","swap_group":7,"custom":[3,1]}}});
    let mut d = LibraryDraft::begin(
        &LibraryBrowser::default(),
        Some(&entry("units", unit, raw.clone())),
        "units",
        "revision".into(),
        None,
    )
    .unwrap();
    let mut unselected = d.clone();
    set(&mut unselected, "pin_name", "Requires explicit Add");
    assert!(unselected.request().is_err());
    d.switch_pin(None, false).unwrap();
    let new = d.electrical.pin_id.unwrap();
    assert_ne!(new, old);
    assert!(d.electrical.selected_new_pin());
    set(&mut d, "pin_name", "Enable");
    assert_eq!(d.electrical.pin_id, Some(new));
    assert!(d.request().is_err());
    set(&mut d, "pin_type", "Input");
    let addition = json!({new.to_string():{"name":"Enable","electrical_type":"Input"}});
    assert_eq!(
        json!(d.request().unwrap().fields),
        json!({"pin_additions":addition})
    );
    let old_index = d.electrical.pins.keys().position(|k| k == &alias).unwrap();
    d.switch_pin(Some(old_index), false).unwrap();
    assert_eq!(d.electrical.pin_id, Some(old));
    assert!(!d.electrical.selected_new_pin());
    set(&mut d, "pin_name", "Explicit old rename");
    let index = d
        .electrical
        .pins
        .keys()
        .position(|k| k == &new.to_string())
        .unwrap();
    d.switch_pin(Some(index), false).unwrap();
    assert_eq!(d.value("pin_name"), "Enable");
    assert_eq!(d.value("pin_type"), "Input");
    let mut expected = raw["pins"].clone();
    expected[&alias]["name"] = json!("Explicit old rename");
    assert_eq!(
        json!(d.request().unwrap().fields),
        json!({"pins":expected,"pin_additions":addition})
    );
    set(&mut d, "pin_type", "");
    assert!(d.request().is_err());
    assert!(d.switch_pin(None, false).is_err());
    assert_eq!(d.electrical.pin_id, Some(new));
    d.switch_pin(None, true).unwrap();
    assert!(d.electrical.pin_id.is_none());
    assert_eq!(json!(d.request().unwrap().fields), json!({"pins":expected}));
    d.switch_pin(Some(0), false).unwrap();
    let mut removing = d.clone();
    removing.switch_pin(None, true).unwrap();
    assert_eq!(
        removing.request().unwrap().fields["pin_removals"],
        json!([old])
    );
    assert_eq!(d.electrical.original, Some(raw));
}
#[test]
fn multiple_new_rows_keep_identity_and_later_symbol_choices_use_pin_names() {
    let unit = Uuid::new_v4();
    let raw = json!({"uuid":unit,"name":"Saved","pins":{}});
    let mut d = LibraryDraft::begin(
        &LibraryBrowser::default(),
        Some(&entry("units", unit, raw.clone())),
        "units",
        "revision".into(),
        None,
    )
    .unwrap();
    let mut ids = Vec::new();
    for (name, kind) in [("Enable", "Input"), ("Supply", "PowerIn")] {
        d.switch_pin(None, false).unwrap();
        ids.push(d.electrical.pin_id.unwrap());
        set(&mut d, "pin_name", name);
        set(&mut d, "pin_type", kind);
    }
    assert_ne!(ids[0], ids[1]);
    let request = d.request().unwrap();
    assert_eq!(request.fields.len(), 1);
    assert_eq!(
        request.fields["pin_additions"].as_object().unwrap().len(),
        2
    );
    d.switch_pin(Some(0), false).unwrap();
    let first = d.electrical.pin_id.unwrap();
    d.switch_pin(None, true).unwrap();
    let remaining = d.request().unwrap().fields["pin_additions"]
        .as_object()
        .unwrap()
        .clone();
    assert_eq!(remaining.len(), 1);
    assert!(!remaining.contains_key(&first.to_string()));
    let mut later = raw;
    later["pins"] = request.fields["pin_additions"].clone();
    let browser = LibraryBrowser {
        entries: vec![entry("units", unit, later)],
        ..Default::default()
    };
    let mut symbol =
        LibraryDraft::begin(&browser, None, "symbols", "revision".into(), None).unwrap();
    set(&mut symbol, "unit", &unit.to_string());
    let choices = &symbol
        .fields
        .iter()
        .find(|f| f.key == "anchor_pin")
        .unwrap()
        .choices;
    assert!(choices.contains(&(ids[0].to_string(), "Enable".into())));
    assert!(choices.contains(&(ids[1].to_string(), "Supply".into())));
}

#[test]
fn saved_symbol_refuses_unit_pin_controls_without_mutating_the_draft() {
    let unit = Uuid::new_v4();
    let symbol = Uuid::new_v4();
    let raw = json!({"uuid":symbol,"name":"Saved body","unit":unit,"drawings":[],"pin_anchors":[]});
    let browser = LibraryBrowser {
        entries: vec![entry(
            "units",
            unit,
            json!({"uuid":unit,"name":"Named Unit","pins":{}}),
        )],
        ..Default::default()
    };
    let mut d = LibraryDraft::begin(
        &browser,
        Some(&entry("symbols", symbol, raw)),
        "symbols",
        "revision".into(),
        None,
    )
    .unwrap();
    let before = d.electrical.clone();
    let fields: Vec<_> = d.fields.iter().map(|f| (f.key, f.text.clone())).collect();
    for (index, remove) in [(None, false), (None, true), (Some(0), false)] {
        assert!(d.switch_pin(index, remove).is_err());
        assert_eq!(d.electrical, before);
        assert_eq!(
            d.fields
                .iter()
                .map(|f| (f.key, f.text.clone()))
                .collect::<Vec<_>>(),
            fields
        );
    }
}
