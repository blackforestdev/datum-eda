//! Alternate-row and swap-group buffering over the saved Unit form.
use super::*;
use crate::library_browser::LibraryControl as C;

fn fixture() -> (LibraryDraft, String, Value) {
    let (unit, pin, sibling) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let key = pin.simple().to_string().to_uppercase();
    let row = json!({"name":" SAME ","kind":"alias","vendor":true});
    let raw = json!({"uuid":unit,"name":"Saved Unit","pins":{
        key.clone():{"uuid":pin,"name":"In","direction":"Input","swap_group":7,"alternates":[row,row]},
        sibling.to_string():{"uuid":sibling,"name":"Out","direction":"Output","swap_group":0,"alternates":[]}
    }});
    let entry = LibraryEntry {
        id: unit,
        kind: "units".into(),
        name: "Saved Unit".into(),
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
    (draft, key, raw)
}

fn set(d: &mut LibraryDraft, key: &str, value: &str) {
    d.focused = d.fields.iter().position(|f| f.key == key).unwrap();
    d.change(value.into());
}

fn activate(d: &mut LibraryDraft, action: C) -> Result<&'static str, String> {
    d.switch_pin_alternate_control(&action).unwrap()
}

#[test]
fn saved_unit_pin_metadata_composes_duplicate_rows_and_other_pin_selection() {
    let (mut d, key, raw) = fixture();
    let index = d.electrical.pins.keys().position(|k| k == &key).unwrap();
    d.switch_pin(Some(index), false).unwrap();
    assert!(d.request().unwrap().fields.is_empty());
    set(&mut d, "pin_swap_group", "0");
    activate(&mut d, C::PinAlternate(1)).unwrap();
    set(&mut d, "pin_alternate_name", " Ω ");
    activate(&mut d, C::PinAlternate(0)).unwrap();
    activate(&mut d, C::RemovePinAlternate).unwrap();
    activate(&mut d, C::AddPinAlternate).unwrap();
    set(&mut d, "pin_alternate_kind", " custom ");
    assert_eq!(d.value("pin_swap_group"), "0");
    set(&mut d, "pin_name", "Renamed");
    d.switch_pin(Some(1 - index), false).unwrap();
    let fields = d.request().unwrap().fields;
    let edits = &fields["pin_metadata"][&key];
    assert_eq!(edits["previous"], raw["pins"][&key]);
    assert_eq!(edits["swap_group"], 0);
    assert_eq!(
        edits["alternate_edits"],
        json!([{"index":1,"previous":raw["pins"][&key]["alternates"][1],"name":" Ω ","kind":"alias"}])
    );
    assert_eq!(edits["alternate_removals"][0]["index"], 0);
    assert_eq!(
        edits["alternate_additions"],
        json!([{"name":"","kind":" custom "}])
    );
    assert_eq!(fields["pins"][&key]["name"], "Renamed");
    assert_eq!(
        fields["pins"][&key]["alternates"],
        raw["pins"][&key]["alternates"]
    );
}

#[test]
fn saved_unit_pin_metadata_invalid_group_blocks_switch_and_removal_discards_intent() {
    let (mut d, key, _) = fixture();
    let index = d.electrical.pins.keys().position(|k| k == &key).unwrap();
    assert!(activate(&mut d, C::AddPinAlternate).is_err());
    d.switch_pin(Some(index), false).unwrap();
    set(&mut d, "pin_swap_group", "4294967296");
    let before = d.clone();
    assert!(d.switch_pin(Some(1 - index), false).is_err());
    assert_eq!(d, before);
    activate(&mut d, C::AddPinAlternate).unwrap();
    activate(&mut d, C::RemovePinAlternate).unwrap();
    assert!(d.request().is_err());
    d.switch_pin(None, true).unwrap();
    let request = d.request().unwrap();
    assert!(!request.fields.contains_key("pin_metadata"));
    assert_eq!(request.fields["pin_removals"].as_array().unwrap().len(), 1);
}

#[test]
fn saved_unit_pin_metadata_malformed_or_mismatched_capture_is_unavailable() {
    for malformed_uuid in [false, true] {
        let (mut draft, key, mut raw) = fixture();
        if malformed_uuid {
            raw["pins"][&key]["uuid"] = json!(Uuid::new_v4());
        } else {
            raw["pins"][&key]["alternates"][0] = json!("malformed");
        }
        draft.electrical.original = Some(raw.clone());
        draft.electrical.pins = raw["pins"].as_object().unwrap().clone();
        let index = draft
            .electrical
            .pins
            .keys()
            .position(|value| value == &key)
            .unwrap();
        draft.switch_pin(Some(index), false).unwrap();
        assert!(draft.selected_pin_metadata().is_none());
        assert!(activate(&mut draft, C::AddPinAlternate).is_err());
        assert!(draft.request().unwrap().fields.is_empty());
    }
}
