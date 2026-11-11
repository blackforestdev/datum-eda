//! Original indices distinguish identical graphics and survive disposable row changes.
use super::*;
fn line() -> Value {
    json!({"Line":{"from":{"x":0,"y":0,"vendor":[3,1]},"to":{"x":1000000,"y":0},"width":100000,"unknown":true}})
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
    let raw = json!({"uuid":symbol,"unit":unit,"name":"Saved","drawings":[{"Circle":{"center":{"x":0,"y":0},"radius":1000000,"width":100000}},line(),{"Rect":{"min":{"x":-2000000,"y":-1000000},"max":{"x":2000000,"y":1000000},"width":100000}},line()],"pin_anchors":[],"unknown":[3,1]});
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
    let d =
        LibraryDraft::begin(&browser, Some(&entry), "symbols", "captured".into(), None).unwrap();
    (d, raw, browser)
}
fn set(d: &mut LibraryDraft, key: &str, text: &str) {
    d.focused = d.fields.iter().position(|f| f.key == key).unwrap();
    d.change(text.into());
}
#[test]
fn duplicate_lines_use_original_indices_and_compose_raw_edits_removals_additions_and_body() {
    let (mut d, raw, _) = fixture();
    assert_eq!(
        d.electrical
            .graphics
            .rows
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
    assert!(
        d.electrical
            .graphics
            .controls()
            .contains(&crate::library_browser::LibraryControl::Circle(0))
    );
    assert!(d.request().unwrap().fields.is_empty());
    d.switch_line(Some(3), false).unwrap();
    set(&mut d, "line_to_x", "7mm");
    d.switch_line(Some(1), false).unwrap();
    set(&mut d, "line_width", "150um");
    let fields = d.request().unwrap().fields;
    assert_eq!(fields["line_edits"].as_array().unwrap().len(), 2);
    assert_eq!(fields["line_edits"][0]["index"], 1);
    assert_eq!(fields["line_edits"][1]["index"], 3);
    assert_eq!(fields["line_edits"][1]["line"]["Line"]["to"]["x"], 7000000);
    d.switch_line(None, false).unwrap();
    let new = d.electrical.graphics.selected.unwrap();
    assert_eq!(new, 4);
    set(&mut d, "line_to_y", "2mm");
    d.switch_line(Some(3), false).unwrap();
    set(&mut d, "line_from_x", "invalid");
    d.switch_line(None, true).unwrap();
    assert!(d.electrical.graphics.rows.contains_key(&1));
    assert!(d.electrical.graphics.rows.contains_key(&new));
    assert!(!d.electrical.graphics.rows.contains_key(&3));
    d.switch_rect(Some(2), false).unwrap();
    set(&mut d, "rect_min_x", "-3mm");
    set(&mut d, "name", "Revised");
    let fields = d.request().unwrap().fields;
    assert_eq!(
        fields["line_removals"],
        json!([{"index":3,"previous":raw["drawings"][3]}])
    );
    assert_eq!(fields["line_edits"][0]["index"], 1);
    assert_eq!(
        fields["line_edits"][0]["line"]["Line"]["from"]["vendor"],
        json!([3, 1])
    );
    assert_eq!(fields["line_additions"][0]["Line"]["to"]["y"], 2000000);
    assert_eq!(fields["drawings"][1], raw["drawings"][1]);
    assert_eq!(fields["drawings"][3], raw["drawings"][3]);
    assert_eq!(fields["drawings"][2]["Rect"]["min"]["x"], -3000000);
    assert_eq!(fields["name"], "Revised");
    assert_eq!(d.electrical.original, Some(raw));
}
#[test]
fn new_invalid_discard_preserves_other_buffered_intent_and_same_input_is_noop() {
    let (mut d, raw, _) = fixture();
    d.switch_line(Some(1), false).unwrap();
    set(&mut d, "line_to_x", "1mm");
    assert!(d.request().unwrap().fields.is_empty());
    set(&mut d, "line_width", "-1nm");
    set(&mut d, "line_to_x", "0");
    assert!(
        d.request().is_ok(),
        "saved Line retains existing schema-only edit admission"
    );
    d.switch_line(None, false).unwrap();
    let new = d.electrical.graphics.selected.unwrap();
    set(&mut d, "line_to_x", "0");
    assert!(d.request().unwrap_err().contains("distinct endpoints"));
    set(&mut d, "anchor_x", "invalid");
    d.switch_line(None, true).unwrap();
    assert!(!d.electrical.graphics.rows.contains_key(&new));
    assert!(d.focused < d.fields.len());
    assert_eq!(d.value("anchor_x"), "invalid");
    assert!(d.request().is_err());
    set(&mut d, "anchor_x", "");
    let f = d.request().unwrap().fields;
    assert!(!f.contains_key("line_additions"));
    assert_eq!(f["line_edits"][0]["index"], 1);
    assert_eq!(d.electrical.original, Some(raw));
}
#[test]
fn invalid_switch_and_wrong_kind_controls_leave_the_form_unchanged() {
    let (mut d, _, browser) = fixture();
    d.switch_line(Some(1), false).unwrap();
    set(&mut d, "line_from_x", "invalid");
    let before = d.clone();
    assert!(d.switch_line(Some(3), false).is_err());
    assert_eq!(d, before);
    assert!(d.switch_line(Some(99), false).is_err());
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
        for (index, remove) in [(None, false), (Some(1), false), (None, true)] {
            assert!(draft.switch_line(index, remove).is_err());
            assert_eq!(draft, before);
        }
    }
}
