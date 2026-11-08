use super::*;
fn set(d: &mut LibraryDraft, key: &str, value: &str) {
    d.focused = d.fields.iter().position(|f| f.key == key).unwrap();
    d.change(value.into());
}
fn entry(kind: &str, id: Uuid, name: &str, raw: Value) -> LibraryEntry {
    LibraryEntry {
        kind: kind.into(),
        id,
        name: name.into(),
        source_path: format!("pool/{kind}/{id}.json"),
        raw,
        revision: Some(0),
        warning: None,
    }
}
struct Data {
    b: LibraryBrowser,
    part: Uuid,
    other: Uuid,
    foot: Uuid,
    pad: Uuid,
    pad2: Uuid,
    a: Uuid,
    bgate: Uuid,
    pin: Uuid,
}
fn data() -> Data {
    let (part, other, entity, unit, package, foot, pad, pad2, a, bgate, pin) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let entity2 = Uuid::new_v4();
    let c = Uuid::new_v4();
    let b = LibraryBrowser {
        entries: vec![
            entry("packages", package, "Body", json!({})),
            entry(
                "units",
                unit,
                "Logic",
                json!({"pins":{pin.to_string():{"uuid":pin,"name":"Input"}}}),
            ),
            entry(
                "entities",
                entity,
                "Dual",
                json!({"gates":{a.to_string():{"uuid":a,"name":"A","unit":unit},bgate.to_string():{"uuid":bgate,"name":"B","unit":unit}}}),
            ),
            entry(
                "entities",
                entity2,
                "Other",
                json!({"gates":{c.to_string():{"uuid":c,"name":"C","unit":unit}}}),
            ),
            entry(
                "parts",
                part,
                "PN-1",
                json!({"entity":entity,"package":package,"default_footprint":foot}),
            ),
            entry(
                "parts",
                other,
                "PN-2",
                json!({"entity":entity2,"package":package}),
            ),
            entry(
                "footprints",
                foot,
                "Land",
                json!({"package":package,"pads":{pad.to_string():{"uuid":pad,"name":"1"},pad2.to_string():{"uuid":pad2,"name":"2"}}}),
            ),
        ],
        ..Default::default()
    };
    Data {
        b,
        part,
        other,
        foot,
        pad,
        pad2,
        a,
        bgate,
        pin,
    }
}
fn draft(x: &Data) -> LibraryDraft {
    let mut d = LibraryDraft::begin(&x.b, None, "pin_pad_maps", "rev".into(), None).unwrap();
    assert_eq!(d.value("map_part"), "");
    assert_eq!(d.value("map_footprint"), "");
    assert_eq!(d.value("map_default"), "false");
    assert!(!d.fields.iter().any(|f| f.key == "name"));
    set(&mut d, "map_part", &x.part.to_string());
    set(&mut d, "map_footprint", &x.foot.to_string());
    d
}
#[test]
fn map_named_rows_partial_duplicate_gate_pin_and_explicit_default_intent() {
    let x = data();
    let mut d = draft(&x);
    let id = d.id;
    assert!(d.request().is_err());
    set(&mut d, "map_pad", &x.pad.to_string());
    set(&mut d, "map_gate", &x.a.to_string());
    set(&mut d, "map_pin", &x.pin.to_string());
    let one = d.request().unwrap();
    assert_eq!(one.fields["mappings"].as_object().unwrap().len(), 1);
    assert_eq!(one.fields["assign_default"], json!(false));
    set(&mut d, "map_pad", &x.pad2.to_string());
    set(&mut d, "map_gate", &x.bgate.to_string());
    set(&mut d, "map_pin", &x.pin.to_string());
    let two = d.request().unwrap();
    assert_eq!(two.fields["mappings"].as_object().unwrap().len(), 2);
    assert_eq!(
        two.fields["mappings"][x.pad.to_string()],
        json!({"gate":x.a,"pin":x.pin})
    );
    assert_eq!(
        two.fields["mappings"][x.pad2.to_string()],
        json!({"gate":x.bgate,"pin":x.pin})
    );
    set(&mut d, "map_gate", &x.a.to_string());
    assert!(
        d.request().is_err(),
        "same gate/pin pair cannot be assigned twice"
    );
    set(&mut d, "map_gate", &x.bgate.to_string());
    set(&mut d, "map_default", "true");
    assert_eq!(d.request().unwrap().fields["assign_default"], json!(true));
    assert_eq!(d.id, id);
    let index = d
        .pin_pad_map
        .pads
        .iter()
        .position(|p| p.id == Some(x.pad))
        .unwrap();
    d.select_mapping_pad(Some(index), false).unwrap();
    assert_eq!(d.value("map_gate"), x.a.to_string());
    d.select_mapping_pad(None, true).unwrap();
    assert_eq!(
        d.request().unwrap().fields["mappings"]
            .as_object()
            .unwrap()
            .len(),
        1
    );
    set(&mut d, "map_part", &x.other.to_string());
    assert!(d.pin_pad_map.mappings.is_empty());
    assert_eq!(d.value("map_default"), "false");
    assert!(d.request().is_err());
}
#[test]
fn map_cursor_has_no_dependency_trap_and_same_choice_preserves_pending_fields() {
    let x = data();
    let mut d = draft(&x);
    set(&mut d, "map_gate", &x.a.to_string());
    set(&mut d, "map_pin", &x.pin.to_string());
    set(&mut d, "map_pad", &x.pad.to_string());
    assert_eq!(d.value("map_pad"), x.pad.to_string());
    set(&mut d, "map_gate", &x.a.to_string());
    set(&mut d, "map_part", &x.part.to_string());
    assert_eq!(
        d.value("map_gate"),
        x.a.to_string(),
        "no-op source selection must not discard partial row"
    );
    set(&mut d, "map_pad", &x.pad.to_string());
    assert_eq!(d.value("map_gate"), x.a.to_string());
    set(&mut d, "map_pin", &x.pin.to_string());
    assert!(d.request().is_ok());
}
#[test]
fn map_ambiguous_source_rows_are_unavailable_and_duplicate_names_are_qualified() {
    let mut x = data();
    let f = x.b.entries.iter_mut().find(|e| e.id == x.foot).unwrap();
    f.raw["pads"][x.pad.simple().to_string()] = f.raw["pads"][x.pad.to_string()].clone();
    f.raw["pads"][x.pad2.to_string()]["name"] = json!("1");
    let mut d = draft(&x);
    assert_eq!(d.pin_pad_map.pads.len(), 3);
    assert_eq!(
        d.pin_pad_map.pads.iter().filter(|p| p.id.is_none()).count(),
        2
    );
    assert!(d.pin_pad_map.pads.iter().all(|p| p.label.contains('·')));
    set(&mut d, "map_pad", &x.pad.to_string());
    assert!(d.refusal.is_some());
    assert!(d.value("map_pad").is_empty());
    set(&mut d, "map_pad", &x.pad2.to_string());
    assert_eq!(d.value("map_pad"), x.pad2.to_string());
}
