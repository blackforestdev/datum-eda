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
        warning: None,
        revision: Some(0),
    }
}
#[test]
fn part_metadata_optional_footprint_compatibility_and_stable_intent() {
    let (entity, a, b, fa, fb) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let browser = LibraryBrowser {
        entries: vec![
            entry("entities", entity, "Logic", json!({})),
            entry("packages", a, "Body A", json!({})),
            entry("packages", b, "Body B", json!({})),
            entry("footprints", fa, "Land A", json!({"package":a})),
            entry("footprints", fb, "Land B", json!({"package":b})),
        ],
        ..Default::default()
    };
    let mut d = LibraryDraft::begin(&browser, None, "parts", "revision".into(), None).unwrap();
    let id = d.id;
    assert!(
        !d.fields
            .iter()
            .any(|f| matches!(f.key, "name" | "lifecycle" | "pin_pad_map"))
    );
    assert!(d.request().is_err());
    set(&mut d, "manufacturer", "Designer");
    set(&mut d, "mpn", "PN-1");
    set(&mut d, "part_entity", &entity.to_string());
    set(&mut d, "part_package", &a.to_string());
    let request = d.request().unwrap();
    assert!(request.fields["default_footprint"].is_null());
    let options = &d
        .fields
        .iter()
        .find(|f| f.key == "part_footprint")
        .unwrap()
        .choices;
    assert_eq!(options.len(), 2);
    assert!(options[1].1.contains("Land A"));
    set(&mut d, "part_footprint", &fa.to_string());
    assert_eq!(d.request().unwrap().fields["default_footprint"], json!(fa));
    d.preview = Some(json!({}));
    set(&mut d, "part_package", &b.to_string());
    assert_eq!(d.value("part_footprint"), "");
    assert!(d.preview.is_none());
    set(&mut d, "part_footprint", &fa.to_string());
    assert!(d.request().is_err());
    set(&mut d, "part_footprint", &fb.to_string());
    set(&mut d, "description", "As entered");
    set(&mut d, "tags", "logic, logic,test");
    let request = d.request().unwrap();
    assert_eq!(request.object_id, id);
    assert!(!request.fields.contains_key("name"));
    assert_eq!(request.fields["description"], json!("As entered"));
    assert_eq!(request.fields["tags"], json!(["logic", "test"]));
    assert!(request.fields.get("lifecycle").is_none());
    let saved = entry("parts", id, "Saved", json!({}));
    assert!(LibraryDraft::begin(&browser, Some(&saved), "parts", "revision".into(), None).is_err());
}
#[test]
fn part_ambiguous_and_warned_references_are_not_choices() {
    let id = Uuid::new_v4();
    let mut bad = entry("packages", Uuid::new_v4(), "Bad", json!({}));
    bad.warning = Some("Unreadable".into());
    let browser = LibraryBrowser {
        entries: vec![
            entry("entities", id, "First", json!({})),
            entry("entities", id, "Alias", json!({})),
            bad,
        ],
        ..Default::default()
    };
    let d = LibraryDraft::begin(&browser, None, "parts", "revision".into(), None).unwrap();
    for key in ["part_entity", "part_package"] {
        assert!(
            d.fields
                .iter()
                .find(|f| f.key == key)
                .unwrap()
                .choices
                .is_empty()
        );
    }
}
