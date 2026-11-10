//! Saved anchors retain ordered raw identity while new rows use named Unit pins.
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
fn fixture() -> (LibraryDraft, Uuid, Uuid, Value) {
    let [unit, symbol, old, new, third] = std::array::from_fn(|_| Uuid::new_v4());
    let alias = old.simple().to_string().to_uppercase();
    let raw = json!({"uuid":symbol,"name":"Saved mixed body","unit":unit,"drawings":[{"Circle":{"center":{"x":0,"y":0},"radius":1000000,"width":100000}}],"pin_anchors":[{"pin":alias,"position":{"x":-1000000,"y":0,"custom":true},"orientation":"Right","length_nm":500000,"custom":[3,1]}],"custom":true});
    let browser = LibraryBrowser {
        entries: vec![entry(
            "units",
            unit,
            json!({"uuid":unit,"name":"Named Unit","pins":{old.to_string():{"uuid":old,"name":"Old"},new.simple().to_string().to_uppercase():{"uuid":new,"name":"Supply"},third.to_string():{"uuid":third,"name":"Enable"}}}),
        )],
        ..Default::default()
    };
    let draft = LibraryDraft::begin(
        &browser,
        Some(&entry("symbols", symbol, raw.clone())),
        "symbols",
        "revision".into(),
        None,
    )
    .unwrap();
    (draft, old, new, raw)
}
fn pose(d: &mut LibraryDraft, new: Uuid) {
    for (key, value) in [
        ("anchor_pin", new.to_string().as_str()),
        ("anchor_x", "2 mm"),
        ("anchor_y", "-1 mm"),
        ("anchor_orientation", "Left"),
        ("anchor_length", "1 mm"),
    ] {
        set(d, key, value);
    }
}
#[test]
fn saved_add_anchor_composes_old_pose_preserves_raw_and_discards_only_new_rows() {
    let (mut d, old, new, raw) = fixture();
    d.switch_anchor(Some(0), false).unwrap();
    assert!(!d.electrical.new_anchor);
    assert!(d.fields.iter().any(|f| f.key == "anchor_pin"));
    let mut removing = d.clone();
    removing.switch_anchor(None, true).unwrap();
    assert_eq!(
        removing.request().unwrap().fields["anchor_removals"],
        json!([old])
    );
    set(&mut d, "anchor_x", "-3 mm");
    d.switch_anchor(None, false).unwrap();
    assert!(d.electrical.new_anchor);
    assert_eq!(d.electrical.anchor_id, None);
    assert!(d.request().is_err());
    let choices = &d
        .fields
        .iter()
        .find(|f| f.key == "anchor_pin")
        .unwrap()
        .choices;
    assert!(choices.contains(&(new.to_string(), "Supply".into())));
    pose(&mut d, old);
    assert!(d.request().unwrap_err().contains("already has an anchor"));
    pose(&mut d, new);
    let request = d.request().unwrap();
    assert_eq!(request.fields.len(), 2);
    let mut expected = raw["pin_anchors"].clone();
    expected[0]["position"]["x"] = json!(-3000000);
    assert_eq!(request.fields["pin_anchors"], expected);
    assert_eq!(
        request.fields["anchor_additions"],
        json!([{"pin":new,"position":{"x":2000000,"y":-1000000},"style":{"orientation":"Left","length_nm":1000000,"decoration":"none"}}])
    );
    let old_index = d
        .electrical
        .anchors
        .keys()
        .position(|k| k.parse::<Uuid>().ok() == Some(old))
        .unwrap();
    set(&mut d, "anchor_x", "invalid");
    let before = d.electrical.clone();
    assert!(d.switch_anchor(Some(old_index), false).is_err());
    assert_eq!(d.value("anchor_x"), "invalid");
    assert_eq!(d.electrical, before);
    set(&mut d, "anchor_x", "2 mm");
    d.switch_anchor(Some(old_index), false).unwrap();
    assert!(!d.electrical.new_anchor);
    let mut removing = d.clone();
    removing.switch_anchor(None, true).unwrap();
    assert_eq!(
        removing.request().unwrap().fields["anchor_removals"],
        json!([old])
    );
    assert_eq!(
        removing.request().unwrap().fields["anchor_additions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let index = d
        .electrical
        .anchors
        .keys()
        .position(|k| k.parse::<Uuid>().ok() == Some(new))
        .unwrap();
    d.switch_anchor(Some(index), false).unwrap();
    assert!(d.electrical.new_anchor);
    assert_eq!(d.value("anchor_pin"), new.to_string());
    set(&mut d, "anchor_length", "invalid");
    d.switch_anchor(None, true).unwrap();
    assert!(!d.electrical.new_anchor);
    assert_eq!(d.electrical.anchor_id, None);
    assert_eq!(
        json!(d.request().unwrap().fields),
        json!({"pin_anchors":expected})
    );
    assert_eq!(d.electrical.original, Some(raw));
}
#[test]
fn pending_anchor_discard_restores_empty_source_and_unit_rejects_foreign_controls() {
    let (mut d, _, _, _) = fixture();
    d.switch_anchor(None, false).unwrap();
    d.switch_anchor(None, true).unwrap();
    assert!(d.request().unwrap().fields.is_empty());
    let unit = Uuid::new_v4();
    let mut d = LibraryDraft::begin(
        &LibraryBrowser::default(),
        Some(&entry(
            "units",
            unit,
            json!({"uuid":unit,"name":"Unit","pins":{}}),
        )),
        "units",
        "revision".into(),
        None,
    )
    .unwrap();
    let before = d.electrical.clone();
    let fields: Vec<_> = d.fields.iter().map(|f| (f.key, f.text.clone())).collect();
    for (index, remove) in [(None, false), (None, true), (Some(0), false)] {
        assert!(d.switch_anchor(index, remove).is_err());
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

#[test]
fn multiple_new_anchor_rows_retain_selected_pin_and_pose_across_switches() {
    let (mut d, old, first, _) = fixture();
    let second: Uuid = d
        .electrical
        .unit_pins
        .values()
        .flatten()
        .map(|(id, _)| id.parse().unwrap())
        .find(|id| *id != old && *id != first)
        .unwrap();
    d.switch_anchor(None, false).unwrap();
    pose(&mut d, first);
    d.switch_anchor(None, false).unwrap();
    pose(&mut d, second);
    set(&mut d, "anchor_x", "4 mm");
    let request = d.request().unwrap();
    assert_eq!(request.fields.len(), 1);
    assert_eq!(
        request.fields["anchor_additions"].as_array().unwrap().len(),
        2
    );
    let index = d
        .electrical
        .anchors
        .keys()
        .position(|k| k.parse::<Uuid>().ok() == Some(first))
        .unwrap();
    d.switch_anchor(Some(index), false).unwrap();
    assert_eq!(d.value("anchor_pin"), first.to_string());
    assert_eq!(d.value("anchor_x"), "2000000");
    let index = d
        .electrical
        .anchors
        .keys()
        .position(|k| k.parse::<Uuid>().ok() == Some(second))
        .unwrap();
    d.switch_anchor(Some(index), false).unwrap();
    assert_eq!(d.value("anchor_pin"), second.to_string());
    assert_eq!(d.value("anchor_x"), "4000000");
    d.switch_anchor(None, true).unwrap();
    let rows = d.request().unwrap().fields["anchor_additions"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["pin"], json!(first));
}
