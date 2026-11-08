use super::*;
fn set(d: &mut LibraryDraft, key: &str, value: &str) {
    d.focused = d.fields.iter().position(|f| f.key == key).unwrap();
    d.change(value.into());
}
fn entry(kind: &str, id: Uuid, name: &str, raw: Value) -> LibraryEntry {
    LibraryEntry {
        id,
        kind: kind.into(),
        name: name.into(),
        source_path: format!("pool/{kind}/{id}.json"),
        revision: Some(0),
        raw,
        warning: None,
    }
}
#[test]
fn named_pairs_switch_remove_stable_identity_and_no_inferred_defaults() {
    let (u, v, s, t) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let b = LibraryBrowser {
        entries: vec![
            entry("units", u, "Unit A", json!({})),
            entry("units", v, "Unit B", json!({})),
            entry("symbols", s, "Symbol A", json!({"unit":u})),
            entry("symbols", t, "Symbol B", json!({"unit":v})),
        ],
        ..Default::default()
    };
    let mut d = LibraryDraft::begin(&b, None, "entities", "revision".into(), None).unwrap();
    assert_eq!(d.value("prefix"), "");
    assert_eq!(d.value("manufacturer"), "");
    assert_eq!(d.value("gate_unit"), "");
    assert_eq!(d.value("gate_symbol"), "");
    assert!(d.request().is_err());
    set(&mut d, "name", "My entity");
    set(&mut d, "gate_name", "A");
    let a = d.entity.gate_id.unwrap();
    set(&mut d, "gate_unit", &u.to_string());
    let choices = &d
        .fields
        .iter()
        .find(|f| f.key == "gate_symbol")
        .unwrap()
        .choices;
    assert_eq!(choices.len(), 1);
    assert!(choices[0].1.contains("Symbol A"));
    set(&mut d, "gate_symbol", &s.to_string());
    d.switch_gate(None, false).unwrap();
    let second = d.entity.gate_id.unwrap();
    assert_ne!(a, second);
    set(&mut d, "gate_name", "B");
    set(&mut d, "gate_unit", &u.to_string());
    set(&mut d, "gate_symbol", &s.to_string());
    set(&mut d, "gate_unit", &v.to_string());
    assert_eq!(d.value("gate_symbol"), "");
    assert!(d.request().is_err());
    set(&mut d, "gate_symbol", &t.to_string());
    set(&mut d, "tags", " logic, analog,logic ");
    let request = d.request().unwrap();
    assert_eq!(request.fields["gates"].as_object().unwrap().len(), 2);
    assert_eq!(request.fields["tags"], json!(["analog", "logic"]));
    assert_eq!(request.fields["gates"][a.to_string()]["symbol"], json!(s));
    d.switch_gate(None, false).unwrap();
    let i = d
        .entity
        .gates
        .keys()
        .position(|id| *id == a.to_string())
        .unwrap();
    d.switch_gate(Some(i), false).unwrap();
    assert_eq!(d.entity.gate_id, Some(a));
    assert_eq!(d.value("gate_name"), "A");
    assert_eq!(
        d.request().unwrap().fields["gates"],
        request.fields["gates"]
    );
    d.switch_gate(None, true).unwrap();
    assert_eq!(d.entity.gates.len(), 1);
    assert_eq!(
        d.request().unwrap().fields["gates"]
            .as_object()
            .unwrap()
            .len(),
        1
    );
    let saved = entry("entities", d.id, "Saved", json!({}));
    assert!(LibraryDraft::begin(&b, Some(&saved), "entities", "revision".into(), None).is_err());
}
#[test]
fn ambiguous_and_warned_reference_choices_are_unavailable() {
    let (u, s) = (Uuid::new_v4(), Uuid::new_v4());
    let mut warned = entry("symbols", s, "Warning", json!({"unit":u}));
    warned.warning = Some("Invalid source".into());
    let b = LibraryBrowser {
        entries: vec![
            entry("units", u, "First", json!({})),
            entry("units", u, "Alias", json!({})),
            warned,
        ],
        ..Default::default()
    };
    let mut d = LibraryDraft::begin(&b, None, "entities", "revision".into(), None).unwrap();
    set(&mut d, "gate_unit", &u.to_string());
    assert!(
        d.fields
            .iter()
            .find(|f| f.key == "gate_unit")
            .unwrap()
            .choices
            .is_empty()
    );
    assert!(
        d.fields
            .iter()
            .find(|f| f.key == "gate_symbol")
            .unwrap()
            .choices
            .is_empty()
    );
}
