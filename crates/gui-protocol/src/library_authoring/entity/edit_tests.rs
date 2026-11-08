use super::*;

#[test]
fn saved_fields_patch_only_dirty_metadata_and_gate_controls_refuse() {
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
        ["name", "prefix", "manufacturer", "tags"]
    );
    assert_eq!(d.value("manufacturer"), "");
    assert_eq!(d.value("tags"), "z, a, z");
    assert!(d.request().unwrap().fields.is_empty());
    assert_eq!(d.entity.gates, *raw["gates"].as_object().unwrap());
    for (index, remove) in [(None, false), (Some(0), false), (None, true)] {
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
