//! Saved removal retains raw identity and composes with replacement gates.
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
#[test]
fn removal_resolves_raw_alias_and_empty_draft_requires_explicit_replacement() {
    let [entity, gate, unit, symbol] = std::array::from_fn(|_| Uuid::new_v4());
    let alias = gate.simple().to_string().to_uppercase();
    let raw = json!({"uuid":entity,"name":"Saved","gates":{alias.clone():{"uuid":gate,"name":"Original","unit":unit,"symbol":symbol,"unknown":[3,1]}}});
    let browser = LibraryBrowser {
        entries: vec![
            entry("units", unit, json!({})),
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
    assert!(d.switch_gate(None, true).is_err());
    d.switch_gate(Some(0), false).unwrap();
    d.switch_gate(None, true).unwrap();
    assert!(d.entity.gates.is_empty());
    assert_eq!(json!(d.entity.saved_gates), raw["gates"]);
    assert!(d.request().unwrap_err().contains("one or more"));
    d.switch_gate(None, false).unwrap();
    let added = d.entity.gate_id.unwrap();
    for (key, value) in [
        ("gate_name", "Replacement".into()),
        ("gate_unit", unit.to_string()),
        ("gate_symbol", symbol.to_string()),
    ] {
        d.focused = d.fields.iter().position(|f| f.key == key).unwrap();
        d.change(value);
    }
    assert_eq!(
        json!(d.request().unwrap().fields),
        json!({"gate_removals":[gate],"gate_additions":{added.to_string():{"uuid":added,"name":"Replacement","unit":unit,"symbol":symbol}}})
    );
    d.switch_gate(None, true).unwrap();
    assert!(d.request().is_err());
    assert_eq!(json!(d.entity.saved_gates), raw["gates"]);
}
#[test]
fn removing_selected_row_discards_only_its_pending_fields_and_preserves_other_edits() {
    let [entity, a, b, unit, symbol] = std::array::from_fn(|_| Uuid::new_v4());
    let raw = json!({"uuid":entity,"name":"Saved","gates":{
        a.to_string():{"uuid":a,"name":"A","unit":unit,"symbol":symbol},
        b.to_string():{"uuid":b,"name":"B","unit":unit,"symbol":symbol,"unknown":true}}});
    let browser = LibraryBrowser {
        entries: vec![
            entry("units", unit, json!({})),
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
    let index = d
        .entity
        .gates
        .keys()
        .position(|k| k == &a.to_string())
        .unwrap();
    d.switch_gate(Some(index), false).unwrap();
    d.focused = d.fields.iter().position(|f| f.key == "gate_name").unwrap();
    d.change("Discard this name".into());
    d.switch_gate(None, true).unwrap();
    assert_eq!(
        json!(d.request().unwrap().fields),
        json!({"gate_removals":[a]})
    );
    assert_eq!(d.entity.gates[&b.to_string()], raw["gates"][b.to_string()]);
    assert!(d.entity.gate_id.is_none());
    assert!(d.dirty.is_empty());
}
