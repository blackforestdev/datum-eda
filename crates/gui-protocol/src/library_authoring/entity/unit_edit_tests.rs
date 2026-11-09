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
fn set(d: &mut LibraryDraft, key: &str, id: Uuid) {
    d.focused = d.fields.iter().position(|f| f.key == key).unwrap();
    d.change(id.to_string());
}
#[test]
fn saved_pair_retains_incompatible_choice_then_persists_sparse_rows() {
    let [entity, a, b, unit, new_unit, symbol, new_symbol] =
        std::array::from_fn(|_| Uuid::new_v4());
    let alias = a.simple().to_string().to_uppercase();
    let raw = json!({"uuid":entity,"name":"Dual","gates":{
        alias.clone():{"uuid":a,"name":"A","unit":unit,"symbol":symbol,"custom":[3,1]},
        b.to_string():{"uuid":b,"name":"B","unit":unit,"symbol":symbol}}});
    let browser = LibraryBrowser {
        entries: vec![
            entry("units", unit, json!({})),
            entry("units", new_unit, json!({})),
            entry("symbols", symbol, json!({"unit":unit})),
            entry("symbols", new_symbol, json!({"unit":new_unit})),
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
    let i = d.entity.gates.keys().position(|k| k == &alias).unwrap();
    d.switch_gate(Some(i), false).unwrap();
    set(&mut d, "gate_unit", new_unit);
    assert_eq!(d.value("gate_symbol"), symbol.to_string());
    assert!(
        d.fields
            .iter()
            .find(|f| f.key == "gate_symbol")
            .unwrap()
            .choices
            .iter()
            .all(|(id, _)| id != &symbol.to_string())
    );
    assert!(d.request().is_err());
    assert!(d.switch_gate(Some(1 - i), false).is_err());
    set(&mut d, "gate_symbol", new_symbol);
    let expected =
        json!({"gate_units":{alias.clone():new_unit},"gate_symbols":{alias.clone():new_symbol}});
    assert_eq!(json!(d.request().unwrap().fields), expected);
    d.switch_gate(Some(1 - i), false).unwrap();
    d.switch_gate(Some(i), false).unwrap();
    assert_eq!(d.value("gate_unit"), new_unit.to_string());
    assert_eq!(d.value("gate_symbol"), new_symbol.to_string());
    assert_eq!(json!(d.request().unwrap().fields), expected);
    assert_eq!(json!(d.entity.saved_gates), raw["gates"]);
    set(&mut d, "gate_unit", unit);
    assert_eq!(d.value("gate_symbol"), new_symbol.to_string());
    assert!(d.request().is_err());
    set(&mut d, "gate_symbol", symbol);
    assert!(d.request().unwrap().fields.is_empty());
}

#[test]
fn pair_validation_preserves_untouched_reference_spelling() {
    let [entity, gate, unit, symbol, next] = std::array::from_fn(|_| Uuid::new_v4());
    let unit_alias = unit.simple().to_string().to_uppercase();
    let symbol_alias = symbol.simple().to_string().to_uppercase();
    let raw = json!({"uuid":entity,"name":"Raw aliases","gates":{gate.to_string():
        {"uuid":gate,"name":"A","unit":unit_alias,"symbol":symbol_alias,"custom":[3,1]}}});
    let browser = LibraryBrowser {
        entries: vec![
            entry("units", unit, json!({})),
            entry("symbols", symbol, json!({"unit":unit})),
            entry("symbols", next, json!({"unit":unit})),
        ],
        ..Default::default()
    };
    for field in ["gate_symbol", "gate_unit"] {
        let mut d = LibraryDraft::begin(
            &browser,
            Some(&entry("entities", entity, raw.clone())),
            "entities",
            "revision".into(),
            None,
        )
        .unwrap();
        d.switch_gate(Some(0), false).unwrap();
        let chosen = if field == "gate_symbol" { next } else { unit };
        set(&mut d, field, chosen);
        let changes = d.request().unwrap().fields;
        let expected = if field == "gate_symbol" {
            json!({"gate_symbols":{gate.to_string():next}})
        } else {
            json!({"gate_units":{gate.to_string():unit}})
        };
        assert_eq!(json!(changes), expected);
        let gates = d.entity_gates().unwrap();
        let untouched = if field == "gate_symbol" {
            "unit"
        } else {
            "symbol"
        };
        assert_eq!(
            gates[&gate.to_string()][untouched],
            raw["gates"][gate.to_string()][untouched]
        );
        assert_eq!(gates[&gate.to_string()]["custom"], json!([3, 1]));
    }
}
