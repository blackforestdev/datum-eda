//! Literal Text shares the captured graphics selection and disposable field buffer.
use super::*;
fn text(literal: &str) -> Value {
    json!({"Text":{"text":literal,"position":{"x":0,"y":1000000,"vendor":[3,1]},"rotation":0,"unknown":true}})
}
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
    let raw = json!({"uuid":symbol,"unit":unit,"name":"Saved","drawings":[{"Line":{"from":{"x":0,"y":0},"to":{"x":1,"y":2},"width":1}},{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}},text(" same "),text(" same ")],"pin_anchors":[],"unknown":[3,1]});
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
#[test]
fn symbol_text_rows_compose_saved_literal_geometry_line_rect_and_ordered_mixed_append() {
    let (mut d, raw, _) = fixture();
    assert!(d.request().unwrap().fields.is_empty());
    d.switch_text(Some(3), false).unwrap();
    set(&mut d, "text_literal", "  Raw Ω  ");
    set(&mut d, "text_x", "2mm");
    set(&mut d, "text_rotation", "-721");
    d.switch_line(Some(0), false).unwrap();
    assert!(!d.fields.iter().any(|f| f.key.starts_with("text_")));
    set(&mut d, "line_width", "-1");
    d.switch_rect(Some(1), false).unwrap();
    set(&mut d, "rect_max_x", "2mm");
    d.switch_text(None, false).unwrap();
    assert!(d.request().unwrap_err().contains("must not be empty"));
    set(&mut d, "text_literal", "  NEW  ");
    set(&mut d, "text_rotation", "45");
    d.switch_line(None, false).unwrap();
    d.switch_rect(None, false).unwrap();
    d.switch_text(Some(2), false).unwrap();
    set(&mut d, "text_rotation", "2147483648");
    assert!(d.request().is_err());
    d.switch_text(None, true).unwrap();
    let f = d.request().unwrap().fields;
    assert_eq!(f["text_edits"][0]["index"], 3);
    assert_eq!(f["text_edits"][0]["text"]["Text"]["text"], "  Raw Ω  ");
    assert_eq!(f["text_edits"][0]["text"]["Text"]["rotation"], -721);
    assert_eq!(
        f["text_edits"][0]["text"]["Text"]["position"]["vendor"],
        json!([3, 1])
    );
    assert_eq!(
        f["text_removals"],
        json!([{"index":2,"previous":raw["drawings"][2]}])
    );
    assert_eq!(f["line_edits"][0]["line"]["Line"]["width"], -1);
    assert_eq!(f["drawings"][1]["Rect"]["max"]["x"], 2000000);
    assert_eq!(f["drawings"][2], raw["drawings"][2]);
    assert_eq!(f["graphic_additions"][0]["Text"]["text"], "NEW");
    assert_eq!(f["graphic_additions"][0]["Text"]["rotation"], 45);
    assert!(f["graphic_additions"][1].get("Line").is_some());
    assert!(f["graphic_additions"][2].get("Rect").is_some());
    assert!(!f.contains_key("line_additions"));
    assert_eq!(d.electrical.original, Some(raw));
}
#[test]
fn symbol_text_invalid_selected_discard_retains_other_intent_and_old_schema_semantics() {
    let (mut d, _, _) = fixture();
    d.switch_text(Some(2), false).unwrap();
    set(&mut d, "text_literal", "");
    set(&mut d, "text_rotation", "-2147483648");
    assert!(d.request().is_ok());
    d.switch_text(None, false).unwrap();
    set(&mut d, "text_literal", " \t ");
    let before = d.clone();
    assert!(d.switch_text(Some(3), false).is_err());
    assert_eq!(d, before);
    set(&mut d, "anchor_x", "invalid");
    set(&mut d, "name", "Metadata");
    d.switch_text(None, true).unwrap();
    assert_eq!(d.value("anchor_x"), "invalid");
    assert!(d.request().is_err());
    set(&mut d, "anchor_x", "");
    let f = d.request().unwrap().fields;
    assert_eq!(f["text_edits"][0]["text"]["Text"]["text"], "");
    assert_eq!(f["text_edits"][0]["text"]["Text"]["rotation"], i32::MIN);
    assert_eq!(f["name"], "Metadata");
    assert!(!f.contains_key("graphic_additions"));
    assert!(!f.contains_key("text_removals"));
}
#[test]
fn symbol_text_same_raw_literal_and_equivalent_angles_are_noop_and_wrong_controls_atomic() {
    let (mut d, _, browser) = fixture();
    d.switch_text(Some(2), false).unwrap();
    set(&mut d, "text_literal", " same ");
    set(&mut d, "text_rotation", " 0 ");
    set(&mut d, "text_y", "1mm");
    assert!(d.request().unwrap().fields.is_empty());
    let before = d.clone();
    assert!(d.switch_text(Some(0), false).is_err());
    assert_eq!(d, before);
    assert!(d.switch_text(Some(999), false).is_err());
    assert_eq!(d, before);
    assert!(d.switch_rect(None, true).is_err());
    assert_eq!(d, before);
    for mut draft in [
        LibraryDraft::begin(
            &browser,
            Some(&browser.entries[0]),
            "units",
            "captured".into(),
            None,
        )
        .unwrap(),
        LibraryDraft::begin(&browser, None, "symbols", "captured".into(), None).unwrap(),
    ] {
        let before = draft.clone();
        for (index, remove) in [(None, false), (Some(2), false), (None, true)] {
            assert!(draft.switch_text(index, remove).is_err());
            assert_eq!(draft, before);
        }
    }
}
