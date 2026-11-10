//! Saved removal discards only the selected pose and keeps other editable intent.
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
fn index(d: &LibraryDraft, id: Uuid) -> usize {
    d.electrical
        .anchors
        .keys()
        .position(|k| k.parse::<Uuid>().ok() == Some(id))
        .unwrap()
}
fn fixture() -> (LibraryDraft, [Uuid; 3], Value) {
    let [unit, symbol, b, c] = std::array::from_fn(|_| Uuid::new_v4());
    let a: Uuid = "00000000-0000-1000-8000-000000000001".parse().unwrap();
    let raw = json!({"uuid":symbol,"name":"Saved","unit":unit,"drawings":[{"Circle":{"center":{"x":0,"y":0},"radius":1000000,"width":100000}}],"pin_anchors":[{"pin":a.simple().to_string(),"position":{"x":-1000000,"y":0,"unknown":true},"orientation":"Right","length_nm":500000,"unknown":[3,1]},{"pin":b,"position":{"x":1000000,"y":0},"style":{"orientation":"Left","length_nm":500000,"decoration":"none"},"custom":true}],"custom":[3,1,3]});
    let browser = LibraryBrowser {
        entries: vec![entry(
            "units",
            unit,
            json!({"uuid":unit,"name":"Named Unit","pins":{a.to_string():{"uuid":a,"name":"Input"},b.to_string():{"uuid":b,"name":"Output"},c.to_string():{"uuid":c,"name":"Supply"}}}),
        )],
        ..Default::default()
    };
    let d = LibraryDraft::begin(
        &browser,
        Some(&entry("symbols", symbol, raw.clone())),
        "symbols",
        "revision".into(),
        None,
    )
    .unwrap();
    (d, [a, b, c], raw)
}
#[test]
fn selected_removal_discards_invalid_pose_preserves_other_pose_and_distinct_addition_and_metadata()
{
    let (mut d, [a, b, c], raw) = fixture();
    d.switch_anchor(Some(index(&d, b)), false).unwrap();
    set(&mut d, "anchor_x", "3 mm");
    d.switch_anchor(Some(index(&d, a)), false).unwrap();
    set(&mut d, "anchor_x", "invalid selected pose");
    set(&mut d, "name", "Edited metadata");
    assert!(d.request().is_err());
    d.switch_anchor(None, true).unwrap();
    assert_eq!(d.electrical.anchor_id, None);
    assert!(!d.electrical.new_anchor);
    assert!(
        !d.electrical
            .anchors
            .keys()
            .any(|k| k.parse::<Uuid>().ok() == Some(a))
    );
    let mut posed = raw["pin_anchors"].clone();
    posed[1]["position"]["x"] = json!(3000000);
    let expected = json!({"name":"Edited metadata","anchor_removals":[a],"pin_anchors":posed});
    assert_eq!(json!(d.request().unwrap().fields), expected);
    d.switch_anchor(None, false).unwrap();
    for (key, value) in [
        ("anchor_pin", a.to_string().as_str()),
        ("anchor_x", "0"),
        ("anchor_y", "-2 mm"),
        ("anchor_orientation", "Down"),
        ("anchor_length", "1 mm"),
    ] {
        set(&mut d, key, value);
    }
    let anchors = d.electrical.anchors.clone();
    assert!(d.request().unwrap_err().contains("removed in this draft"));
    assert!(d.switch_anchor(Some(index(&d, b)), false).is_err());
    assert_eq!(d.value("anchor_pin"), a.to_string());
    assert_eq!(d.electrical.anchors, anchors);
    set(&mut d, "anchor_pin", &c.to_string());
    let request = d.request().unwrap();
    assert_eq!(request.fields["anchor_removals"], json!([a]));
    assert_eq!(request.fields["pin_anchors"], posed);
    assert_eq!(
        request.fields["anchor_additions"],
        json!([{"pin":c,"position":{"x":0,"y":-2000000},"style":{"orientation":"Down","length_nm":1000000,"decoration":"none"}}])
    );
    d.switch_anchor(None, true).unwrap();
    assert_eq!(json!(d.request().unwrap().fields), expected);
    assert_eq!(d.electrical.original, Some(raw));
}
#[test]
fn removing_last_saved_anchor_retains_original_ordered_removals_without_new_minimum() {
    let (mut d, [a, b, _], raw) = fixture();
    d.switch_anchor(Some(index(&d, b)), false).unwrap();
    d.switch_anchor(None, true).unwrap();
    d.switch_anchor(Some(index(&d, a)), false).unwrap();
    d.switch_anchor(None, true).unwrap();
    assert!(d.electrical.anchors.is_empty());
    assert_eq!(
        json!(d.request().unwrap().fields),
        json!({"anchor_removals":[a,b]})
    );
    assert!(d.switch_anchor(None, true).is_err());
    assert_eq!(d.electrical.original, Some(raw));
}
#[test]
fn absent_anchor_source_and_foreign_kind_controls_refuse_without_draft_mutation() {
    let (d, _, mut raw) = fixture();
    raw.as_object_mut().unwrap().remove("pin_anchors");
    let unit: Uuid = serde_json::from_value(raw["unit"].clone()).unwrap();
    let browser = LibraryBrowser {
        entries: vec![entry(
            "units",
            unit,
            json!({"uuid":unit,"name":"Unit","pins":{}}),
        )],
        ..Default::default()
    };
    let mut empty = LibraryDraft::begin(
        &browser,
        Some(&entry("symbols", d.id, raw)),
        "symbols",
        "revision".into(),
        None,
    )
    .unwrap();
    assert!(empty.request().unwrap().fields.is_empty());
    assert!(empty.switch_anchor(None, true).is_err());
    empty.switch_anchor(None, false).unwrap();
    empty.switch_anchor(None, true).unwrap();
    assert!(empty.request().unwrap().fields.is_empty());
    let mut unit = LibraryDraft::begin(
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
    let before = unit.electrical.clone();
    assert!(unit.switch_anchor(None, true).is_err());
    assert_eq!(unit.electrical, before);
}
