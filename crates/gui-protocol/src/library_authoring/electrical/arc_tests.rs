//! Arc rows keep native tenths-of-degree values and sparse captured intent.
use super::*;
fn arc() -> Value {
    json!({"Arc":{"arc":{"center":{"x":0,"y":0,"vendor":[3,1]},"radius":1000000,"start_angle":3500,"end_angle":100,"vendor":"range"},"width":100000,"unknown":true}})
}
fn fixture(count: usize) -> (LibraryDraft, Value) {
    let (unit, symbol) = (Uuid::new_v4(), Uuid::new_v4());
    let mut drawings = vec![json!({"Line":{"from":{"x":0,"y":0},"to":{"x":1,"y":2},"width":1}})];
    drawings.extend((0..count).map(|_| arc()));
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
fn captured_arc_rows_preserve_metadata_identity_and_signed_angles() {
    let (mut d, raw) = fixture(2);
    assert!(d.request().unwrap().fields.is_empty());
    assert!(
        d.electrical
            .graphics
            .controls()
            .contains(&crate::library_browser::LibraryControl::Arc(2))
    );
    d.switch_arc(Some(2), false).unwrap();
    set(&mut d, "arc_x", "-2.5mm");
    set(&mut d, "arc_start_angle", "-2147483648");
    set(&mut d, "arc_end_angle", "2147483647");
    d.switch_arc(Some(1), false).unwrap();
    set(&mut d, "arc_width", "invalid");
    d.switch_arc(None, true).unwrap();
    d.switch_arc(None, false).unwrap();
    set(&mut d, "arc_radius", "0.75mm");
    set(&mut d, "arc_width", "0.12mm");
    set(&mut d, "arc_start_angle", "7200");
    set(&mut d, "arc_end_angle", "0");
    d.switch_text(Some(3), false).unwrap();
    assert!(!d.fields.iter().any(|f| f.key.starts_with("arc_")));
    set(&mut d, "text_literal", "READY");
    let fields = d.request().unwrap().fields;
    let edited = &fields["arc_edits"][0];
    assert_eq!(edited["index"], 2);
    assert_eq!(edited["arc"]["Arc"]["arc"]["center"]["x"], -2500000);
    assert_eq!(
        edited["arc"]["Arc"]["arc"]["center"]["vendor"],
        json!([3, 1])
    );
    assert_eq!(edited["arc"]["Arc"]["arc"]["vendor"], "range");
    assert_eq!(edited["arc"]["Arc"]["arc"]["start_angle"], i32::MIN);
    assert_eq!(edited["arc"]["Arc"]["arc"]["end_angle"], i32::MAX);
    assert_eq!(
        fields["arc_removals"],
        json!([{"index":1,"previous":raw["drawings"][1]}])
    );
    assert_eq!(
        fields["graphic_additions"][0]["Arc"]["arc"]["radius"],
        750000
    );
    assert_eq!(
        fields["graphic_additions"][0]["Arc"]["arc"]["start_angle"],
        7200
    );
    assert_eq!(fields["graphic_additions"][0]["Arc"]["arc"]["end_angle"], 0);
    assert_eq!(fields["text_edits"][0]["text"]["Text"]["text"], "READY");
    assert!(!fields.contains_key("drawings"));
}
#[test]
fn new_arc_fields_refuse_invalid_ranges_and_selected_discard_stays_available() {
    for (key, text) in [
        ("arc_radius", "0"),
        ("arc_radius", "-1nm"),
        ("arc_width", "0"),
        ("arc_width", "-1nm"),
        ("arc_x", "9223372036854775807nm"),
        ("arc_y", "-9223372036854775808nm"),
        ("arc_radius", "NaN"),
        ("arc_start_angle", "2147483648"),
        ("arc_end_angle", "-2147483649"),
        ("arc_start_angle", "90deg"),
        ("arc_end_angle", "90.5"),
    ] {
        let (mut d, _) = fixture(1);
        d.switch_arc(None, false).unwrap();
        set(&mut d, key, text);
        assert!(d.request().is_err(), "{key}={text}");
        d.switch_arc(None, true).unwrap();
        assert!(
            !d.request()
                .unwrap()
                .fields
                .contains_key("graphic_additions")
        );
    }
}
#[test]
fn unchanged_legacy_arc_retains_raw_values_and_creation_draft_refuses_graphics() {
    let (mut d, mut raw) = fixture(1);
    raw["drawings"][1]["Arc"]["arc"]["radius"] = json!(0);
    raw["drawings"][1]["Arc"]["width"] = json!(-1);
    d.electrical.original = Some(raw.clone());
    d.hydrate_graphics();
    d.switch_arc(Some(1), false).unwrap();
    assert!(d.request().unwrap().fields.is_empty());
    set(&mut d, "arc_x", "1nm");
    assert!(d.request().is_err());
    d.switch_arc(None, true).unwrap();
    assert_eq!(
        d.request().unwrap().fields["arc_removals"],
        json!([{"index":1,"previous":raw["drawings"][1]}])
    );
    d.create = true;
    assert!(d.switch_arc(None, false).is_err());
}
