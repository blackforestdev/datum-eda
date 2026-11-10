//! A single captured collection composes Rect lifecycle with existing Line intent.
use super::*;
fn rect() -> Value {
    json!({"Rect":{"min":{"x":-1000000,"y":-1000000,"vendor":[3,1]},"max":{"x":1000000,"y":1000000},"width":100000,"unknown":true}})
}
fn fixture(count: usize) -> (LibraryDraft, Value) {
    let (unit, symbol) = (Uuid::new_v4(), Uuid::new_v4());
    let mut drawings = vec![json!({"Line":{"from":{"x":0,"y":0},"to":{"x":1,"y":2},"width":1}})];
    drawings.extend((0..count).map(|_| rect()));
    let raw = json!({"uuid":symbol,"unit":unit,"name":"Saved","drawings":drawings,"pin_anchors":[],"unknown":[3,1]});
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
        entries: vec![
            LibraryEntry {
                id: unit,
                kind: "units".into(),
                name: "Logical".into(),
                source_path: format!("pool/units/{unit}.json"),
                revision: Some(0),
                raw: json!({"uuid":unit,"name":"Logical","pins":{}}),
                warning: None,
            },
            entry.clone(),
        ],
        ..Default::default()
    };
    (
        LibraryDraft::begin(&browser, Some(&entry), "symbols", "captured".into(), None).unwrap(),
        raw,
    )
}
fn set(d: &mut LibraryDraft, key: &str, text: &str) {
    d.focused = d.fields.iter().position(|f| f.key == key).unwrap();
    d.change(text.into());
}
#[test]
fn duplicate_rects_compose_selected_correction_original_removal_and_ordered_mixed_append() {
    let (mut d, raw) = fixture(2);
    assert_eq!(d.electrical.graphics.selected, None);
    assert!(d.request().unwrap().fields.is_empty());
    d.switch_rect(Some(2), false).unwrap();
    set(&mut d, "rect_min_x", "-3mm");
    d.switch_line(Some(0), false).unwrap();
    assert!(!d.fields.iter().any(|f| f.key.starts_with("rect_")));
    set(&mut d, "line_width", "-1");
    d.switch_rect(Some(1), false).unwrap();
    set(&mut d, "rect_width", "invalid");
    d.switch_rect(None, true).unwrap();
    d.switch_rect(None, false).unwrap();
    let new_rect = d.electrical.graphics.selected.unwrap();
    set(&mut d, "rect_max_y", "4mm");
    d.switch_line(None, false).unwrap();
    set(&mut d, "line_to_y", "2mm");
    d.switch_rect(None, false).unwrap();
    let f = d.request().unwrap().fields;
    assert_eq!(f["rect_edits"][0]["index"], 2);
    assert_eq!(f["rect_edits"][0]["rect"]["Rect"]["min"]["x"], -3000000);
    assert_eq!(
        f["rect_edits"][0]["rect"]["Rect"]["min"]["vendor"],
        json!([3, 1])
    );
    assert_eq!(
        f["rect_removals"],
        json!([{"index":1,"previous":raw["drawings"][1]}])
    );
    assert_eq!(f["line_edits"][0]["line"]["Line"]["width"], -1);
    assert_eq!(f["graphic_additions"][0]["Rect"]["max"]["y"], 4000000);
    assert_eq!(f["graphic_additions"][1]["Line"]["to"]["y"], 2000000);
    assert!(f["graphic_additions"][2].get("Rect").is_some());
    assert!(new_rect >= raw["drawings"].as_array().unwrap().len());
    assert!(!f.contains_key("drawings"));
    assert!(!f.contains_key("line_additions"));
    assert_eq!(d.electrical.original, Some(raw));
}
#[test]
fn selected_invalid_rect_blocks_switch_but_remove_preserves_other_intent() {
    let (mut d, _) = fixture(1);
    set(&mut d, "rect_width", "100um");
    assert!(d.request().unwrap().fields.is_empty());
    set(&mut d, "rect_min_x", "-2mm");
    d.switch_rect(None, false).unwrap();
    set(&mut d, "rect_min_x", "1mm");
    let before = d.clone();
    assert!(d.switch_line(Some(0), false).is_err());
    assert_eq!(d, before);
    assert!(d.request().is_err());
    set(&mut d, "anchor_x", "invalid");
    d.switch_rect(None, true).unwrap();
    assert_eq!(d.value("anchor_x"), "invalid");
    assert!(d.request().is_err());
    set(&mut d, "anchor_x", "");
    let f = d.request().unwrap().fields;
    assert_eq!(f["drawings"][1]["Rect"]["min"]["x"], -2000000);
    assert!(!f.contains_key("graphic_additions"));
    assert!(!d.fields.iter().any(|f| f.key.starts_with("rect_")));
}
#[test]
fn wrong_row_kind_is_atomic_and_discarding_new_rect_does_not_emit_a_removal() {
    let (mut d, _) = fixture(0);
    let before = d.clone();
    assert!(d.switch_rect(Some(0), false).is_err());
    assert_eq!(d, before);
    d.switch_rect(None, false).unwrap();
    set(&mut d, "rect_width", "invalid");
    let before = d.clone();
    assert!(d.switch_line(None, true).is_err());
    assert_eq!(d, before);
    d.switch_rect(None, true).unwrap();
    assert!(d.request().unwrap().fields.is_empty());
}
