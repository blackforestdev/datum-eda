//! Saved reference corrections keep original row keys and never become remove/add.
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
        {"pin":b,"position":{"x":2000000,"y":0},"style":{"orientation":"Left","length_nm":1000000,"custom":true}},
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
fn selected_saved_choice_survives_switching_and_composes_pose_without_row_rekey() {
    let (mut d, original, [a, b, c]) = fixture(true);
    assert!(
        d.fields
            .iter()
            .all(|f| f.key != "unit" && f.key != "anchor_pin")
    );
    select(&mut d, a);
    assert!(!d.electrical.new_anchor);
    assert_eq!(d.value("anchor_pin"), a.to_string());
    assert!(d.request().unwrap().fields.is_empty());
    set(&mut d, "anchor_pin", &c.to_string());
    assert_eq!(
        json!(d.request().unwrap().fields),
        json!({"anchor_relinks":{a.to_string():c}})
    );
    select(&mut d, b);
    set(&mut d, "anchor_x", "3mm");
    set(&mut d, "name", "Corrected");
    select(&mut d, a);
    assert_eq!(d.electrical.anchor_id, Some(a));
    assert!(!d.electrical.new_anchor);
    assert_eq!(d.value("anchor_pin"), c.to_string());
    assert!(d.electrical.anchors.contains_key(&a.simple().to_string()));
    assert!(!d.electrical.anchors.contains_key(&c.to_string()));
    let mut poses = original["pin_anchors"].clone();
    poses[0]["position"]["x"] = json!(3000000);
    assert_eq!(
        json!(d.request().unwrap().fields),
        json!({"name":"Corrected","pin_anchors":poses,"anchor_relinks":{a.to_string():c}})
    );
    assert!(
        d.electrical.anchors[&a.simple().to_string()]
            .get("style")
            .is_none()
    );
    set(&mut d, "anchor_pin", &a.to_string());
    let r = d.request().unwrap();
    assert!(!r.fields.contains_key("anchor_relinks"));
    assert_eq!(r.fields["pin_anchors"], poses);
    assert_eq!(d.electrical.original, Some(original));
}

#[test]
fn rejected_choices_and_same_row_conflicts_retain_intent_and_new_discard_keeps_correction() {
    let (mut d, original, [a, b, c]) = fixture(true);
    select(&mut d, a);
    for target in [b, Uuid::nil(), Uuid::new_v4()] {
        set(&mut d, "anchor_pin", &target.to_string());
        assert!(d.request().is_err());
        assert_eq!(d.value("anchor_pin"), target.to_string());
        assert_eq!(d.electrical.anchor_id, Some(a));
    }
    set(&mut d, "anchor_pin", &c.to_string());
    assert!(d.switch_anchor(None, true).is_err());
    assert_eq!(d.value("anchor_pin"), c.to_string());
    assert_eq!(d.electrical.anchor_id, Some(a));
    d.switch_anchor(None, false).unwrap();
    assert!(d.electrical.new_anchor);
    set(&mut d, "anchor_pin", &a.to_string());
    assert!(d.request().unwrap_err().contains("conflicts"));
    d.switch_anchor(None, true).unwrap();
    assert_eq!(
        json!(d.request().unwrap().fields),
        json!({"anchor_relinks":{a.to_string():c}})
    );
    select(&mut d, a);
    set(&mut d, "anchor_pin", &a.to_string());
    d.switch_anchor(None, true).unwrap();
    assert_eq!(
        json!(d.request().unwrap().fields),
        json!({"anchor_removals":[a]})
    );
    assert_eq!(d.electrical.original, Some(original));
}

#[test]
fn unchanged_pin_does_not_add_a_new_membership_gate_to_existing_pose_editing() {
    let (mut d, original, [a, _, c]) = fixture(false);
    select(&mut d, a);
    assert!(d.request().unwrap().fields.is_empty());
    set(&mut d, "anchor_x", "-3mm");
    let mut poses = original["pin_anchors"].clone();
    poses[1]["position"]["x"] = json!(-3000000);
    assert_eq!(
        json!(d.request().unwrap().fields),
        json!({"pin_anchors":poses})
    );
    set(&mut d, "anchor_pin", &c.to_string());
    assert_eq!(
        d.request().unwrap().fields["anchor_relinks"],
        json!({a.to_string():c})
    );
    assert_eq!(d.value("anchor_pin"), c.to_string());
}
