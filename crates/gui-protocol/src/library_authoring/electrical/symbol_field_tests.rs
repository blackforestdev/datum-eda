//! Saved definition field intent and distinct prefix values.
use super::*;
use crate::library_browser::LibraryControl as C;
fn fixture() -> (LibraryDraft, Value, LibraryBrowser) {
    let (unit, symbol) = (Uuid::new_v4(), Uuid::new_v4());
    let logical = LibraryEntry {
        id: unit,
        kind: "units".into(),
        name: "Logical".into(),
        source_path: format!("pool/units/{unit}.json"),
        revision: Some(0),
        raw: json!({"uuid":unit,"name":"Logical","pins":{}}),
        warning: None,
    };
    let row =
        json!({"key":"MPN","value":" old ","position":{"x":0,"y":1,"vendor":true},"unknown":[3,1]});
    let raw = json!({"uuid":symbol,"unit":unit,"name":"Saved","drawings":[],"pin_anchors":[],
        "fields":[row,row,{"key":"Note","value":"","position":null}],"default_refdes_prefix":null});
    let entry = LibraryEntry {
        id: symbol,
        kind: "symbols".into(),
        name: "Saved".into(),
        source_path: format!("pool/symbols/{symbol}.json"),
        revision: Some(0),
        raw: raw.clone(),
        warning: None,
    };
    let browser = LibraryBrowser {
        entries: vec![logical, entry.clone()],
        ..Default::default()
    };
    let draft =
        LibraryDraft::begin(&browser, Some(&entry), "symbols", "captured".into(), None).unwrap();
    (draft, raw, browser)
}
fn set(d: &mut LibraryDraft, key: &str, value: &str) {
    d.focused = d.fields.iter().position(|f| f.key == key).unwrap();
    d.change(value.into());
}

fn activate(d: &mut LibraryDraft, control: C) -> Result<&'static str, String> {
    d.switch_symbol_field_control(&control).unwrap()
}

#[test]
fn saved_symbol_field_changes_compose_with_graphics_and_preserve_preimages() {
    let (mut d, raw, _) = fixture();
    assert!(d.request().unwrap().fields.is_empty());
    activate(&mut d, C::SymbolField(1)).unwrap();
    set(&mut d, "symbol_field_value", "  new Ω  ");
    set(&mut d, "symbol_field_x", "2mm");
    d.switch_text(None, false).unwrap();
    set(&mut d, "text_literal", "Graphic");
    activate(&mut d, C::SymbolField(0)).unwrap();
    activate(&mut d, C::RemoveSymbolField).unwrap();
    activate(&mut d, C::AddSymbolField).unwrap();
    set(&mut d, "symbol_field_key", "MPN");
    set(&mut d, "symbol_field_visible", "true");
    let fields = d.request().unwrap().fields;
    assert_eq!(fields["symbol_field_edits"][0]["index"], 1);
    assert_eq!(
        fields["symbol_field_edits"][0]["previous"],
        raw["fields"][1]
    );
    let row = &fields["symbol_field_edits"][0]["field"];
    assert_eq!(row["value"], "  new Ω  ");
    assert_eq!(row["position"], json!({"x":2000000,"y":1,"vendor":true}));
    assert_eq!(row["unknown"], json!([3, 1]));
    assert_eq!(
        fields["symbol_field_removals"],
        json!([{"index":0,"previous":raw["fields"][0]}])
    );
    assert_eq!(
        fields["symbol_field_additions"],
        json!([{"key":"MPN","value":"","visible":true}])
    );
    assert_eq!(fields["graphic_additions"][0]["Text"]["text"], "Graphic");
    assert!(!fields.contains_key("default_refdes_prefix"));
}

#[test]
fn saved_symbol_field_incomplete_position_is_discarded_only_by_removing_that_row() {
    let (mut d, raw, _) = fixture();
    activate(&mut d, C::SymbolField(0)).unwrap();
    set(&mut d, "symbol_field_x", "");
    let before = d.clone();
    assert!(d.request().is_err());
    assert!(activate(&mut d, C::SymbolField(2)).is_err());
    assert_eq!(d, before);
    activate(&mut d, C::RemoveSymbolField).unwrap();
    assert_eq!(
        d.request().unwrap().fields["symbol_field_removals"],
        json!([{"index":0,"previous":raw["fields"][0]}])
    );

    activate(&mut d, C::SymbolField(1)).unwrap();
    set(&mut d, "symbol_field_x", "");
    set(&mut d, "symbol_field_y", "");
    let fields = d.request().unwrap().fields;
    assert!(
        fields["symbol_field_edits"][0]["field"]
            .get("position")
            .is_none()
    );
}

#[test]
fn saved_symbol_field_prefix_absent_empty_literal_and_clear_are_distinct() {
    let (mut d, _, _) = fixture();
    assert_eq!(d.symbol_prefix(), None);
    assert!(
        !d.request()
            .unwrap()
            .fields
            .contains_key("default_refdes_prefix")
    );
    set(&mut d, "symbol_prefix", " Ω ");
    assert_eq!(d.symbol_prefix(), Some(" Ω "));
    assert_eq!(d.request().unwrap().fields["default_refdes_prefix"], " Ω ");
    set(&mut d, "symbol_prefix", "");
    assert_eq!(d.symbol_prefix(), Some(""));
    assert_eq!(d.request().unwrap().fields["default_refdes_prefix"], "");
    activate(&mut d, C::ClearSymbolPrefix).unwrap();
    assert_eq!(d.symbol_prefix(), None);
    assert_eq!(
        d.request().unwrap().fields.get("default_refdes_prefix"),
        Some(&Value::Null)
    );
    set(&mut d, "symbol_prefix", "R");
    assert_eq!(d.request().unwrap().fields["default_refdes_prefix"], "R");
}
