use super::*;

fn set(draft: &mut LibraryDraft, key: &str, text: &str) {
    draft.focused = draft.fields.iter().position(|f| f.key == key).unwrap();
    draft.change(text.into());
}

#[test]
fn package_draft_retains_terminal_identities_and_exact_optional_intent() {
    let mut d = LibraryDraft::begin(
        &LibraryBrowser::default(),
        None,
        "packages",
        "revision".into(),
        None,
    )
    .unwrap();
    set(&mut d, "name", "My body");
    set(&mut d, "body_x", "5 mm");
    set(&mut d, "terminal_name", "1");
    let first = d.terminal_id.unwrap();
    set(&mut d, "terminal_role", "power");
    d.switch_terminal(None, false).unwrap();
    set(&mut d, "terminal_name", "2");
    let second = d.terminal_id.unwrap();
    assert_ne!(first, second);
    let a = d.request().unwrap();
    assert_eq!(
        serde_json::to_value(&a).unwrap(),
        serde_json::to_value(d.request().unwrap()).unwrap()
    );
    assert_eq!(a.object_id, d.id);
    assert_eq!(a.fields["body_dimensions"], json!({"x_nm":5000000}));
    assert_eq!(
        a.fields["terminals"][first.to_string()],
        json!({"uuid":first,"name":"1","role":"power"})
    );
    assert_eq!(
        a.fields["terminals"][second.to_string()],
        json!({"uuid":second,"name":"2"})
    );
    assert!(!a.fields.contains_key("mounting_type"));
    d.switch_terminal(None, true).unwrap();
    let b = d.request().unwrap();
    assert_eq!(b.fields["terminals"].as_object().unwrap().len(), 1);
    assert_eq!(b.fields["terminals"][first.to_string()]["name"], json!("1"));
    set(&mut d, "body_y", "not a length");
    assert!(d.request().is_err());
    d.switch_terminal(Some(0), false).unwrap();
    assert_eq!(
        d.terminal_id,
        Some(first),
        "malformed body does not trap terminal navigation"
    );
}

#[test]
fn empty_package_draft_does_not_invent_terminals_dimensions_or_mounting() {
    let mut d = LibraryDraft::begin(
        &LibraryBrowser::default(),
        None,
        "packages",
        "revision".into(),
        None,
    )
    .unwrap();
    set(&mut d, "name", "Named body");
    assert_eq!(
        d.request().unwrap().fields,
        json!({"name":"Named body"}).as_object().unwrap().clone()
    );
    set(&mut d, "terminal_role", "dangling role");
    assert!(d.request().is_err());
}

#[test]
fn saved_package_hydrates_only_metadata_and_sends_only_dirty_text() {
    let id = Uuid::new_v4();
    let terminal = Uuid::new_v4();
    let raw = json!({"uuid":id,"name":"Saved body","package_family":null,
        "package_code":"Original code","mounting_type":"As authored",
        "body_dimensions":{"x_nm":5000000},"body_height_nm":null,
        "terminals":{terminal.to_string():{"uuid":terminal,"name":"1"}},
        "vendor":{"ordered":[3,1,3]}});
    let entry = LibraryEntry {
        id,
        kind: "packages".into(),
        name: "Saved body".into(),
        source_path: format!("pool/packages/{id}.json"),
        revision: Some(7),
        raw: raw.clone(),
        warning: None,
    };
    assert!(entry.editable());
    let mut d = LibraryDraft::begin(
        &LibraryBrowser::default(),
        Some(&entry),
        "packages",
        "revision".into(),
        None,
    )
    .unwrap();
    assert!(!d.create);
    assert_eq!(d.id, id);
    assert_eq!(
        d.fields.iter().map(|f| f.key).collect::<Vec<_>>(),
        ["name", "package_family", "package_code", "mounting_type"]
    );
    assert_eq!(d.value("name"), "Saved body");
    assert_eq!(d.value("package_family"), "");
    assert_eq!(d.value("package_code"), "Original code");
    assert_eq!(d.value("mounting_type"), "As authored");
    assert!(d.request().unwrap().fields.is_empty());
    for (index, remove) in [(None, false), (None, true), (Some(0), false)] {
        assert!(d.switch_terminal(index, remove).is_err());
        assert!(d.terminals.is_empty());
        assert!(d.terminal_id.is_none());
        assert!(d.request().unwrap().fields.is_empty());
    }
    set(&mut d, "package_code", "");
    set(&mut d, "mounting_type", "Corrected mounting");
    assert_eq!(
        d.request().unwrap().fields,
        json!({"package_code":"", "mounting_type":"Corrected mounting"})
            .as_object()
            .unwrap()
            .clone()
    );
    assert_eq!(
        entry.raw, raw,
        "draft correction and Cancel leave source untouched"
    );
    let mut foreign = entry.clone();
    foreign.source_path = format!("external/packages/{id}.json");
    assert!(!foreign.editable());
    assert!(
        LibraryDraft::begin(
            &LibraryBrowser::default(),
            Some(&foreign),
            "packages",
            "revision".into(),
            None
        )
        .is_err()
    );
}
