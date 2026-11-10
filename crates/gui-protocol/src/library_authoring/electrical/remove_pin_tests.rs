//! Original raw identity drives sparse removals; ordinary new-row discard stays separate.
use super::*;

fn set(draft: &mut LibraryDraft, key: &str, text: &str) {
    draft.focused = draft
        .fields
        .iter()
        .position(|field| field.key == key)
        .unwrap();
    draft.change(text.into());
}
fn draft() -> (LibraryDraft, Value, Uuid, Uuid) {
    let (unit, b) = (Uuid::new_v4(), Uuid::new_v4());
    let a: Uuid = "00000000-0000-1000-8000-000000000001".parse().unwrap();
    let raw = json!({"uuid":unit,"name":"Saved","pins":{
        a.simple().to_string():{"uuid":a,"name":"Spare","direction":"Input","swap_group":7,"custom":[3,1]},
        b.simple().to_string().to_uppercase():{"uuid":b,"name":"Retained","direction":"Output","custom":{"keep":true}}},"custom":true});
    let entry = LibraryEntry {
        id: unit,
        kind: "units".into(),
        name: "Saved".into(),
        source_path: format!("pool/units/{unit}.json"),
        revision: Some(0),
        raw: raw.clone(),
        warning: None,
    };
    let draft = LibraryDraft::begin(
        &LibraryBrowser::default(),
        Some(&entry),
        "units",
        "revision".into(),
        None,
    )
    .unwrap();
    (draft, raw, a, b)
}
fn select(draft: &mut LibraryDraft, id: Uuid) {
    let index = draft
        .electrical
        .pins
        .keys()
        .position(|key| key.parse::<Uuid>().ok() == Some(id))
        .unwrap();
    draft.switch_pin(Some(index), false).unwrap();
}

#[test]
fn saved_pin_removal_discards_selected_invalid_text_and_composes_other_intent() {
    let (mut d, raw, a, b) = draft();
    select(&mut d, b);
    set(&mut d, "pin_name", "Retained edit");
    select(&mut d, a);
    set(&mut d, "pin_name", "");
    set(&mut d, "pin_type", "");
    set(&mut d, "name", "Edited Unit");
    d.switch_pin(None, true).unwrap();
    assert!(d.electrical.pin_id.is_none());
    assert_eq!(d.value("pin_name"), "");
    assert!(
        !d.electrical
            .pins
            .keys()
            .any(|key| key.parse::<Uuid>().ok() == Some(a))
    );
    let mut expected = raw["pins"].clone();
    expected[b.simple().to_string().to_uppercase()]["name"] = json!("Retained edit");
    let request = d.request().unwrap();
    assert_eq!(
        json!(request.fields),
        json!({"name":"Edited Unit","pins":expected,"pin_removals":[a]})
    );
    d.switch_pin(None, false).unwrap();
    let fresh = d.electrical.pin_id.unwrap();
    assert_ne!(fresh, a);
    set(&mut d, "pin_name", "Spare");
    set(&mut d, "pin_type", "Input");
    let request = d.request().unwrap();
    assert_eq!(request.fields["pin_removals"], json!([a]));
    assert_eq!(
        request.fields["pin_additions"],
        json!({fresh.to_string():{"name":"Spare","electrical_type":"Input"}})
    );
    set(&mut d, "pin_type", "");
    assert!(d.request().is_err());
    d.switch_pin(None, true).unwrap();
    assert_eq!(d.request().unwrap().fields["pin_removals"], json!([a]));
    assert!(!d.request().unwrap().fields.contains_key("pin_additions"));
    assert_eq!(d.electrical.original, Some(raw));
}

#[test]
fn original_all_pin_removal_keeps_no_new_minimum_and_cancel_basis_intact() {
    let (mut d, raw, a, b) = draft();
    select(&mut d, a);
    d.switch_pin(None, true).unwrap();
    select(&mut d, b);
    d.switch_pin(None, true).unwrap();
    assert!(d.electrical.pins.is_empty());
    let request = d.request().unwrap();
    let ids: std::collections::HashSet<Uuid> =
        serde_json::from_value(request.fields["pin_removals"].clone()).unwrap();
    assert_eq!(ids, std::collections::HashSet::from([a, b]));
    assert!(!request.fields.contains_key("pins"));
    assert_eq!(d.electrical.original, Some(raw));
}

#[test]
fn removal_requires_selection_and_preserves_other_invalid_metadata() {
    let (mut d, _, a, _) = draft();
    let before = d.clone();
    assert!(d.switch_pin(None, true).is_err());
    assert_eq!(d.electrical.pins, before.electrical.pins);
    assert_eq!(d.electrical.pin_id, before.electrical.pin_id);
    select(&mut d, a);
    set(&mut d, "name", "");
    d.switch_pin(None, true).unwrap();
    assert_eq!(d.value("name"), "");
    // Saved name validation belongs to the native producer. Removal must retain
    // the invalid intent for that refusal rather than silently dropping it.
    assert_eq!(d.request().unwrap().fields["name"], json!(""));
}
