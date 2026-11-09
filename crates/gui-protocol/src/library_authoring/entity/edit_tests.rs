use super::*;

#[test]
fn saved_fields_patch_only_dirty_metadata_and_topology_controls_refuse() {
    let (id, gate, unit, symbol) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let raw = json!({"schema_version":1,"uuid":id,"name":"Saved Entity","prefix":"U",
        "tags":["z","a","z"],"gates":{gate.to_string():{"uuid":gate,"name":"B","unit":unit,"symbol":symbol,"custom":[3,1]}},
        "custom":{"untouched":true}});
    let entry = LibraryEntry {
        id,
        kind: "entities".into(),
        name: "Saved Entity".into(),
        source_path: format!("pool/entities/{id}.json"),
        revision: Some(0),
        raw: raw.clone(),
        warning: None,
    };
    let mut d = LibraryDraft::begin(
        &LibraryBrowser::default(),
        Some(&entry),
        "entities",
        "revision".into(),
        None,
    )
    .unwrap();
    assert_eq!(
        d.fields.iter().map(|f| f.key).collect::<Vec<_>>(),
        [
            "name",
            "prefix",
            "manufacturer",
            "tags",
            "gate_name",
            "gate_unit",
            "gate_symbol"
        ]
    );
    assert_eq!(d.value("manufacturer"), "");
    assert_eq!(d.value("tags"), "z, a, z");
    assert!(d.request().unwrap().fields.is_empty());
    assert_eq!(d.entity.gates, *raw["gates"].as_object().unwrap());
    d.switch_gate(Some(0), false).unwrap();
    assert_eq!(d.entity.gate_id, Some(gate));
    assert_eq!(d.value("gate_symbol"), symbol.to_string());
    assert!(d.fields.iter().any(|f| f.key == "gate_unit"));
    for (index, remove) in [(None, false), (None, true)] {
        assert!(d.switch_gate(index, remove).is_err());
    }
    assert!(d.dirty.is_empty());
    d.focused = d.fields.iter().position(|f| f.key == "prefix").unwrap();
    d.change("IC".into());
    assert_eq!(
        d.request().unwrap().fields,
        *json!({"prefix":"IC"}).as_object().unwrap()
    );
    d.focused = d.fields.iter().position(|f| f.key == "tags").unwrap();
    d.change("logic, analog, logic".into());
    assert_eq!(
        d.request().unwrap().fields["tags"],
        json!(["analog", "logic"])
    );
    assert_eq!(d.entity.gates, *raw["gates"].as_object().unwrap());
    let mut external = entry.clone();
    external.source_path = format!("other/entities/{id}.json");
    assert!(
        LibraryDraft::begin(
            &LibraryBrowser::default(),
            Some(&external),
            "entities",
            "revision".into(),
            None
        )
        .is_err()
    );
}

#[test]
fn saved_gate_symbol_changes_are_compatible_dirty_only_and_survive_row_navigation() {
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
    fn set(d: &mut LibraryDraft, key: &str, value: &str) {
        d.focused = d.fields.iter().position(|f| f.key == key).unwrap();
        d.change(value.into());
    }
    let (unit, other, a, b, old, corrected, wrong, entity) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let raw = json!({"uuid":entity,"name":"Dual","gates":{
        a.to_string():{"uuid":a,"name":"A","unit":unit,"symbol":old,"custom":[3,1]},
        b.to_string():{"uuid":b,"name":"B","unit":unit,"symbol":old}},"tags":["z","a","z"]});
    let browser = LibraryBrowser {
        entries: vec![
            entry("units", unit, "Fixed Unit", json!({})),
            entry("symbols", old, "Original", json!({"unit":unit})),
            entry("symbols", corrected, "Corrected", json!({"unit":unit})),
            entry("symbols", wrong, "Other Unit body", json!({"unit":other})),
        ],
        ..Default::default()
    };
    let mut d = LibraryDraft::begin(
        &browser,
        Some(&entry("entities", entity, "Dual", raw.clone())),
        "entities",
        "revision".into(),
        None,
    )
    .unwrap();
    assert!(d.request().unwrap().fields.is_empty());
    let index = d
        .entity
        .gates
        .keys()
        .position(|id| *id == a.to_string())
        .unwrap();
    d.switch_gate(Some(index), false).unwrap();
    let choices = &d
        .fields
        .iter()
        .find(|f| f.key == "gate_symbol")
        .unwrap()
        .choices;
    assert_eq!(choices.len(), 2);
    assert!(!choices.iter().any(|(id, _)| *id == wrong.to_string()));
    set(&mut d, "gate_symbol", &wrong.to_string());
    assert!(d.request().is_err());
    set(&mut d, "gate_symbol", &corrected.to_string());
    assert_eq!(
        d.request().unwrap().fields,
        *json!({"gate_symbols":{a.to_string():corrected}})
            .as_object()
            .unwrap()
    );
    let index = d
        .entity
        .gates
        .keys()
        .position(|id| *id == b.to_string())
        .unwrap();
    d.switch_gate(Some(index), false).unwrap();
    assert_eq!(
        d.request().unwrap().fields,
        *json!({"gate_symbols":{a.to_string():corrected}})
            .as_object()
            .unwrap()
    );
    let mut expected = raw["gates"].clone();
    expected[a.to_string()]["symbol"] = json!(corrected);
    assert_eq!(Value::Object(d.entity_gates().unwrap()), expected);
    assert_eq!(Value::Object(d.entity.saved_gates.clone()), raw["gates"]);
    assert!(d.switch_gate(None, false).is_err());
    assert!(d.switch_gate(Some(index), true).is_err());
}

#[test]
fn saved_gate_key_spelling_is_preserved_and_duplicate_aliases_refuse() {
    let (id, gate, unit, symbol, replacement) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let key = gate.simple().to_string().to_uppercase();
    let raw = json!({"uuid":id,"name":"Saved","tags":[],"gates":{key.clone():{
        "uuid":gate,"name":"A","unit":unit,"symbol":symbol,"custom":[3,1]}}});
    let entry = LibraryEntry {
        id,
        kind: "entities".into(),
        name: "Saved".into(),
        source_path: format!("pool/entities/{id}.json"),
        revision: Some(0),
        raw: raw.clone(),
        warning: None,
    };
    let browser = LibraryBrowser {
        entries: [(symbol, "Old"), (replacement, "New")]
            .into_iter()
            .map(|(id, name)| LibraryEntry {
                id,
                kind: "symbols".into(),
                name: name.into(),
                source_path: format!("pool/symbols/{id}.json"),
                revision: Some(0),
                raw: json!({"unit":unit}),
                warning: None,
            })
            .collect(),
        ..Default::default()
    };
    let mut d =
        LibraryDraft::begin(&browser, Some(&entry), "entities", "revision".into(), None).unwrap();
    d.switch_gate(Some(0), false).unwrap();
    assert_eq!(d.entity.selected_gate().unwrap(), &raw["gates"][&key]);
    assert_eq!(
        d.fields
            .iter()
            .find(|f| f.key == "gate_symbol")
            .unwrap()
            .choices
            .len(),
        2
    );
    d.focused = d
        .fields
        .iter()
        .position(|f| f.key == "gate_symbol")
        .unwrap();
    d.change(replacement.to_string());
    assert_eq!(
        d.request().unwrap().fields,
        *json!({"gate_symbols":{key.clone():replacement}})
            .as_object()
            .unwrap()
    );
    assert!(d.entity_gates().unwrap().contains_key(&key));
    let mut ambiguous = entry.clone();
    ambiguous.raw["gates"][gate.to_string()] = raw["gates"][&key].clone();
    let mut d = LibraryDraft::begin(
        &browser,
        Some(&ambiguous),
        "entities",
        "revision".into(),
        None,
    )
    .unwrap();
    assert!(d.switch_gate(Some(0), false).is_err());
    assert!(d.entity.gate_id.is_none());
}

fn name_draft() -> (LibraryDraft, Uuid, Uuid, Value) {
    let (id, a, b, unit, symbol) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let alias = a.simple().to_string().to_uppercase();
    let raw = json!({"schema_version":1,"uuid":id,"name":"Dual","prefix":"U","tags":["z","a","z"],
        "gates":{alias:{"uuid":a,"name":"A","unit":unit,"symbol":symbol,"custom":[3,1,3]},
        b.to_string():{"uuid":b,"name":"B","unit":unit,"symbol":symbol}},"custom":{"intact":true}});
    let entry = LibraryEntry {
        id,
        kind: "entities".into(),
        name: "Dual".into(),
        source_path: format!("pool/entities/{id}.json"),
        revision: Some(0),
        warning: None,
        raw: raw.clone(),
    };
    (
        LibraryDraft::begin(
            &LibraryBrowser::default(),
            Some(&entry),
            "entities",
            "basis".into(),
            None,
        )
        .unwrap(),
        a,
        b,
        raw,
    )
}
fn select_name_gate(d: &mut LibraryDraft, gate: Uuid) {
    let index = d
        .entity
        .gates
        .keys()
        .position(|k| k.parse::<Uuid>().ok() == Some(gate))
        .unwrap();
    d.switch_gate(Some(index), false).unwrap();
}
fn change_name(d: &mut LibraryDraft, name: &str) {
    d.focused = d.fields.iter().position(|f| f.key == "gate_name").unwrap();
    d.change(name.into());
}
#[test]
fn saved_gate_names_keep_alias_raw_and_edits_across_rows_with_sparse_request() {
    let (mut d, a, b, raw) = name_draft();
    assert!(d.request().unwrap().fields.is_empty());
    select_name_gate(&mut d, a);
    assert_eq!(d.value("gate_name"), "A");
    change_name(&mut d, "  Amplifier  ");
    assert_eq!(
        d.request().unwrap().fields,
        *json!({"gate_names":{a.simple().to_string().to_uppercase():"  Amplifier  "}})
            .as_object()
            .unwrap()
    );
    select_name_gate(&mut d, b);
    assert_eq!(d.value("gate_name"), "B");
    change_name(&mut d, "  Amplifier  ");
    select_name_gate(&mut d, a);
    assert_eq!(d.value("gate_name"), "  Amplifier  ");
    let request = d.request().unwrap();
    assert_eq!(request.fields,*json!({"gate_names":{a.simple().to_string().to_uppercase():"  Amplifier  ",b.to_string():"  Amplifier  "}}).as_object().unwrap());
    let mut expected = raw["gates"].clone();
    expected[a.simple().to_string().to_uppercase()]["name"] = json!("  Amplifier  ");
    expected[b.to_string()]["name"] = json!("  Amplifier  ");
    assert_eq!(Value::Object(d.entity_gates().unwrap()), expected);
    assert_eq!(Value::Object(d.entity.saved_gates.clone()), raw["gates"]);
    change_name(&mut d, "A");
    select_name_gate(&mut d, b);
    change_name(&mut d, "B");
    assert!(
        d.request().unwrap().fields.is_empty(),
        "reverting names does not author unrelated fields"
    );
}
#[test]
fn saved_gate_name_requires_existing_selection_and_nonempty_text_without_new_gate_identity() {
    let (mut d, _, _, _) = name_draft();
    change_name(&mut d, "No selection");
    assert!(d.entity.gate_id.is_none());
    assert!(d.request().is_err());
    let (mut d, _, _, _) = name_draft();
    let gate = d.entity.gates.keys().next().unwrap().parse().unwrap();
    select_name_gate(&mut d, gate);
    change_name(&mut d, "  ");
    assert!(d.request().is_err());
    assert!(
        d.switch_gate(Some(1), false).is_err(),
        "invalid name is not silently lost during navigation"
    );
    change_name(&mut d, "Recovered");
    assert!(d.request().is_ok());
    assert_eq!(d.entity.gate_id, Some(gate));
    assert_eq!(d.entity.gates.len(), 2);
    let (mut d, _, _, _) = name_draft();
    let raw = d.entity.gates.values().next().unwrap().clone();
    let gate: Uuid = serde_json::from_value(raw["uuid"].clone()).unwrap();
    d.entity.gates.insert(gate.to_string(), raw.clone());
    d.entity.gates.insert(gate.simple().to_string(), raw);
    let index = d
        .entity
        .gates
        .keys()
        .position(|key| key.parse::<Uuid>().ok() == Some(gate))
        .unwrap();
    assert!(d.switch_gate(Some(index), false).is_err());
}
