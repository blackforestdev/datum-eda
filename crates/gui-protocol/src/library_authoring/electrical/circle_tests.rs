//! Circle intent composes with the existing captured graphic collection.
use super::*;
fn circle() -> Value {
    json!({"Circle":{"center":{"x":0,"y":0,"vendor":[3,1]},"radius":1000000,"width":100000,"unknown":true}})
}
fn fixture(count: usize) -> (LibraryDraft, Value) {
    let (unit, symbol) = (Uuid::new_v4(), Uuid::new_v4());
    let mut drawings = vec![json!({"Line":{"from":{"x":0,"y":0},"to":{"x":1,"y":2},"width":1}})];
    drawings.extend((0..count).map(|_| circle()));
    drawings.push(json!({"Text":{"text":"KEEP","position":{"x":0,"y":0},"rotation":0}}));
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
fn captured_circle_rows_preserve_metadata_and_compose_removal_and_mixed_append() {
    let (mut d, raw) = fixture(2);
    assert_eq!(d.electrical.graphics.selected, None);
    assert!(d.request().unwrap().fields.is_empty());
    assert!(
        d.electrical
            .graphics
            .controls()
            .contains(&crate::library_browser::LibraryControl::Circle(2))
    );
    d.switch_circle(Some(2), false).unwrap();
    set(&mut d, "circle_x", "-2.5mm");
    d.switch_circle(Some(1), false).unwrap();
    set(&mut d, "circle_width", "invalid");
    d.switch_circle(None, true).unwrap();
    d.switch_circle(None, false).unwrap();
    set(&mut d, "circle_radius", "0.75mm");
    set(&mut d, "circle_width", "0.12mm");
    d.switch_text(Some(3), false).unwrap();
    assert!(!d.fields.iter().any(|f| f.key.starts_with("circle_")));
    set(&mut d, "text_literal", "READY");
    d.switch_rect(None, false).unwrap();
    let f = d.request().unwrap().fields;
    assert_eq!(f["circle_edits"][0]["index"], 2);
    assert_eq!(
        f["circle_edits"][0]["circle"]["Circle"]["center"]["x"],
        -2500000
    );
    assert_eq!(
        f["circle_edits"][0]["circle"]["Circle"]["center"]["vendor"],
        json!([3, 1])
    );
    assert_eq!(
        f["circle_removals"],
        json!([{"index":1,"previous":raw["drawings"][1]}])
    );
    assert_eq!(f["text_edits"][0]["text"]["Text"]["text"], "READY");
    assert_eq!(f["graphic_additions"][0]["Circle"]["radius"], 750000);
    assert_eq!(f["graphic_additions"][0]["Circle"]["width"], 120000);
    assert!(f["graphic_additions"][1].get("Rect").is_some());
    assert!(!f.contains_key("line_additions"));
    assert!(!f.contains_key("drawings"));
}

#[test]
fn new_circle_fields_refuse_invalid_extents_and_selected_discard_stays_available() {
    for (key, text) in [
        ("circle_radius", "0"),
        ("circle_radius", "-1nm"),
        ("circle_width", "0"),
        ("circle_width", "-1nm"),
        ("circle_x", "9223372036854775807nm"),
        ("circle_y", "-9223372036854775808nm"),
        ("circle_radius", "NaN"),
    ] {
        let (mut d, _) = fixture(1);
        d.switch_circle(None, false).unwrap();
        set(&mut d, key, text);
        assert!(d.request().is_err(), "{key}={text}");
        d.switch_circle(None, true).unwrap();
        assert!(
            !d.request()
                .unwrap()
                .fields
                .contains_key("graphic_additions")
        );
    }
}

#[test]
fn unchanged_legacy_circle_hydrates_without_new_geometry_gate() {
    let (mut d, mut raw) = fixture(1);
    raw["drawings"][1]["Circle"]["radius"] = json!(0);
    raw["drawings"][1]["Circle"]["width"] = json!(-1);
    d.electrical.original = Some(raw.clone());
    d.hydrate_graphics();
    d.switch_circle(Some(1), false).unwrap();
    assert!(d.request().unwrap().fields.is_empty());
    set(&mut d, "circle_x", "1nm");
    assert!(d.request().is_err());
    d.switch_circle(None, true).unwrap();
    assert_eq!(
        d.request().unwrap().fields["circle_removals"],
        json!([{"index":1,"previous":raw["drawings"][1]}])
    );
    d.create = true;
    assert!(d.switch_circle(None, false).is_err());
}
