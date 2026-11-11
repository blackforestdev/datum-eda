//! Saved designator text and explicit clear compose with the existing anchor draft.
use super::*;
fn set(d: &mut LibraryDraft, key: &str, value: &str) {
    d.focused = d.fields.iter().position(|f| f.key == key).unwrap();
    d.change(value.into());
}
fn select(d: &mut LibraryDraft, pin: Uuid) {
    let index = d
        .electrical
        .anchors
        .keys()
        .position(|key| key.parse::<Uuid>().ok() == Some(pin))
        .unwrap();
    d.switch_anchor(Some(index), false).unwrap();
}
fn fixture(available: bool) -> (LibraryDraft, Value, [Uuid; 3]) {
    let a: Uuid = "00000000-0000-1000-8000-000000000001".parse().unwrap();
    let [b, c, unit, symbol] = std::array::from_fn(|_| Uuid::new_v4());
    let logical = LibraryEntry {
        id: unit,
        kind: "units".into(),
        name: "Logical".into(),
        source_path: format!("pool/units/{unit}.json"),
        revision: Some(0),
        raw: json!({"uuid":unit,"pins":{a.simple().to_string():{"uuid":a,"name":if available {"Input"} else {""}},b.to_string():{"uuid":b,"name":"Output"},c.to_string():{"uuid":c,"name":"Supply"}}}),
        warning: None,
    };
    let raw = json!({"uuid":symbol,"unit":unit,"name":"Saved","drawings":[],"custom":[3,1],"pin_anchors":[
        {"pin":b,"number":null,"position":{"x":2000000,"y":0},"style":{"orientation":"Left","length_nm":1000000,"custom":true}},
        {"pin":a.simple().to_string(),"position":{"x":-2000000,"y":0,"custom":true},"orientation":"Right","length_nm":1000000,"custom":[3,1]}]});
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
    (draft, raw, [a, b, c])
}

#[test]
fn saved_anchor_designator_buffers_literal_empty_clear_and_relink() {
    let (mut d, original, [a, b, c]) = fixture(true);
    assert!(!d.has_anchor_designator_control());
    assert!(d.clear_anchor_designator().is_err());
    select(&mut d, a);
    assert!(d.has_anchor_designator_control());
    set(&mut d, "anchor_designator", " P-7 Ω ");
    set(&mut d, "anchor_x", "-3mm");
    set(&mut d, "anchor_pin", &c.to_string());
    select(&mut d, b);
    assert_eq!(d.anchor_designator(), None);
    assert!(
        !d.request().unwrap().fields["anchor_designators"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["index"] == 0)
    );
    set(&mut d, "anchor_designator", "");
    assert_eq!(d.anchor_designator(), Some(""));
    let edits = &d.request().unwrap().fields["anchor_designators"];
    assert_eq!(
        edits[0],
        json!({"index":0,"previous":original["pin_anchors"][0],"number":""})
    );
    d.clear_anchor_designator().unwrap();
    let f = d.request().unwrap().fields;
    assert_eq!(
        f["anchor_designators"],
        json!([
            {"index":0,"previous":original["pin_anchors"][0],"number":null},
            {"index":1,"previous":original["pin_anchors"][1],"number":" P-7 Ω "},
        ])
    );
    assert_eq!(f["anchor_relinks"], json!({a.to_string():c}));
    assert_eq!(f["pin_anchors"][1]["position"]["x"], -3000000);
    assert!(f["pin_anchors"][1].get("number").is_none());
    select(&mut d, a);
    assert_eq!(d.value("anchor_designator"), " P-7 Ω ");
}

#[test]
fn saved_anchor_designator_removal_discards_pending_number() {
    let (mut d, _, [a, _, _]) = fixture(true);
    select(&mut d, a);
    set(&mut d, "anchor_designator", "old draft");
    d.switch_anchor(None, true).unwrap();
    let f = d.request().unwrap().fields;
    assert!(!d.has_anchor_designator_control());
    assert!(!f.contains_key("anchor_designators"));
    assert_eq!(f["anchor_removals"], json!([a]));
}
