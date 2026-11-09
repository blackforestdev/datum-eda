use super::*;
fn entry(kind: &str, id: Uuid, raw: Value) -> LibraryEntry {
    LibraryEntry {
        id,
        kind: kind.into(),
        name: format!("Named {kind}"),
        source_path: format!("pool/{kind}/{id}.json"),
        revision: Some(0),
        raw,
        warning: None,
    }
}
fn set(d: &mut LibraryDraft, key: &str, text: String) {
    d.focused = d.fields.iter().position(|f| f.key == key).unwrap();
    d.change(text);
}
#[test]
fn saved_new_gate_keeps_stable_identity_and_sparse_row_intent() {
    let [entity, old, unit, symbol, other] = std::array::from_fn(|_| Uuid::new_v4());
    let alias = old.simple().to_string().to_uppercase();
    let raw = json!({"uuid":entity,"name":"Saved","gates":{alias.clone():{"uuid":old,"name":"A","unit":unit,"symbol":symbol,"custom":[3,1]}}});
    let browser = LibraryBrowser {
        entries: vec![
            entry("units", unit, json!({})),
            entry("units", other, json!({})),
            entry("symbols", symbol, json!({"unit":unit})),
        ],
        ..Default::default()
    };
    let mut d = LibraryDraft::begin(
        &browser,
        Some(&entry("entities", entity, raw.clone())),
        "entities",
        "revision".into(),
        None,
    )
    .unwrap();
    d.switch_gate(None, false).unwrap();
    let id = d.entity.gate_id.unwrap();
    assert_ne!(id, old);
    set(&mut d, "gate_name", "Supply".into());
    assert_eq!(d.entity.gate_id, Some(id));
    set(&mut d, "gate_unit", unit.to_string());
    assert!(d.request().is_err());
    set(&mut d, "gate_symbol", symbol.to_string());
    let expected = json!({"gate_additions":{id.to_string():{"uuid":id,"name":"Supply","unit":unit,"symbol":symbol}}});
    assert_eq!(json!(d.request().unwrap().fields), expected);
    d.switch_gate(Some(0), false).unwrap();
    let new_index = d
        .entity
        .gates
        .keys()
        .position(|key| key == &id.to_string())
        .unwrap();
    d.switch_gate(Some(new_index), false).unwrap();
    assert_eq!(d.value("gate_name"), "Supply");
    assert_eq!(d.entity.gate_id, Some(id));
    assert_eq!(json!(d.request().unwrap().fields), expected);
    assert_eq!(d.entity.gates[&alias], raw["gates"][&alias]);
    set(&mut d, "gate_unit", other.to_string());
    assert_eq!(d.value("gate_symbol"), symbol.to_string());
    assert!(d.request().is_err());
    assert!(d.switch_gate(None, false).is_err());
    assert!(d.switch_gate(Some(new_index), true).is_err());
    set(&mut d, "gate_unit", unit.to_string());
    assert_eq!(json!(d.request().unwrap().fields), expected);
    assert_eq!(json!(d.entity.saved_gates), raw["gates"]);
}
