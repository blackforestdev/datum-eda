use super::*;

#[test]
fn saved_part_exposes_metadata_and_default_footprint_with_changed_fields_only() {
    let (id, entity, package) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let mut raw = eda_engine::api::native_write::library::pool_part_payload(
        id,
        entity,
        package,
        "PN-1",
        "Designer",
        "Original value",
        "Description",
        "",
        "Unknown",
    );
    raw["tags"] = json!(["z", "a", "z"]);
    raw["custom"] = json!([3, 1]);
    let e = LibraryEntry {
        id,
        kind: "parts".into(),
        name: "Designer PN-1".into(),
        source_path: format!("pool/parts/{id}.json"),
        revision: Some(0),
        raw: raw.clone(),
        warning: None,
    };
    let mut d = LibraryDraft::begin(
        &LibraryBrowser::default(),
        Some(&e),
        "parts",
        "basis".into(),
        None,
    )
    .unwrap();
    assert_eq!(
        d.fields.iter().map(|f| f.key).collect::<Vec<_>>(),
        [
            "manufacturer",
            "mpn",
            "value",
            "description",
            "datasheet",
            "tags",
            "part_footprint"
        ]
    );
    assert!(d.fields.iter().take(6).all(|f| !f.selector));
    assert!(d.fields[6].selector);
    assert_eq!(d.part.original.as_ref(), Some(&raw));
    assert_eq!(d.value("tags"), "z, a, z");
    assert!(d.request().unwrap().fields.is_empty());
    d.focused = d.fields.iter().position(|f| f.key == "value").unwrap();
    d.change("Corrected value".into());
    assert_eq!(
        d.request().unwrap().fields,
        *json!({"value":"Corrected value"}).as_object().unwrap()
    );
    assert_eq!(d.part.original.as_ref(), Some(&raw));
    d.focused = d
        .fields
        .iter()
        .position(|f| f.key == "manufacturer")
        .unwrap();
    d.change("".into());
    assert!(d.request().is_err());
}

#[test]
fn saved_default_footprint_uses_fixed_package_and_preserves_untouched_optional_states() {
    let (id, entity, package, first, second, wrong) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let entry = |kind: &str, id: Uuid, name: &str, raw: Value| LibraryEntry {
        id,
        kind: kind.into(),
        name: name.into(),
        source_path: format!("pool/{kind}/{id}.json"),
        revision: Some(0),
        warning: None,
        raw,
    };
    let browser = LibraryBrowser {
        entries: vec![
            entry("footprints", first, "Land A", json!({"package":package})),
            entry("footprints", second, "Land B", json!({"package":package})),
            entry(
                "footprints",
                wrong,
                "Other Package land",
                json!({"package":Uuid::new_v4()}),
            ),
        ],
        ..Default::default()
    };
    for state in 0..4 {
        let mut raw = eda_engine::api::native_write::library::pool_part_payload(
            id, entity, package, "PN-1", "Designer", "Value", "", "", "Active",
        );
        raw["default_footprint"] = if state == 0 {
            json!(first)
        } else {
            Value::Null
        };
        if state == 3 {
            raw["default_footprint"] = json!(first.simple().to_string());
        }
        if state == 2 {
            raw.as_object_mut().unwrap().remove("default_footprint");
        }
        let saved = entry("parts", id, "PN-1", raw.clone());
        let mut d =
            LibraryDraft::begin(&browser, Some(&saved), "parts", "basis".into(), None).unwrap();
        assert!(d.request().unwrap().fields.is_empty());
        assert!(
            !d.fields
                .iter()
                .any(|f| matches!(f.key, "part_entity" | "part_package"))
        );
        let i = d
            .fields
            .iter()
            .position(|f| f.key == "part_footprint")
            .unwrap();
        assert_eq!(
            d.fields[i].text,
            if matches!(state, 0 | 3) {
                first.to_string()
            } else {
                String::new()
            }
        );
        assert_eq!(d.fields[i].choices.len(), 3);
        assert!(
            !d.fields[i]
                .choices
                .iter()
                .any(|(key, _)| key == &wrong.to_string())
        );
        d.focused = i;
        d.change(second.to_string());
        assert_eq!(
            d.request().unwrap().fields,
            *json!({"default_footprint":second}).as_object().unwrap()
        );
        d.change(String::new());
        assert_eq!(
            d.request().unwrap().fields,
            *json!({"default_footprint":null}).as_object().unwrap()
        );
        d.change(wrong.to_string());
        assert!(d.request().is_err());
        assert_eq!(d.part.original, Some(raw));
    }
}

#[test]
fn saved_unavailable_or_ambiguous_declared_footprint_stays_visible_and_requires_explicit_choice() {
    let (id, entity, package, footprint) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let mut raw = eda_engine::api::native_write::library::pool_part_payload(
        id, entity, package, "PN-1", "Designer", "Value", "", "", "Active",
    );
    raw["default_footprint"] = json!(footprint);
    let saved = LibraryEntry {
        id,
        kind: "parts".into(),
        name: "Saved".into(),
        source_path: format!("pool/parts/{id}.json"),
        revision: Some(0),
        warning: None,
        raw: raw.clone(),
    };
    let foot = LibraryEntry {
        id: footprint,
        kind: "footprints".into(),
        name: "Duplicate land".into(),
        source_path: format!("pool/footprints/{footprint}.json"),
        revision: Some(0),
        warning: None,
        raw: json!({"package":package}),
    };
    for entries in [vec![], vec![foot.clone(), foot.clone()]] {
        let browser = LibraryBrowser {
            entries,
            ..Default::default()
        };
        let mut d =
            LibraryDraft::begin(&browser, Some(&saved), "parts", "basis".into(), None).unwrap();
        let i = d
            .fields
            .iter()
            .position(|f| f.key == "part_footprint")
            .unwrap();
        assert_eq!(d.value("part_footprint"), footprint.to_string());
        assert!(
            d.fields[i]
                .choices
                .iter()
                .any(|(key, label)| key == &footprint.to_string() && label.contains("Unavailable"))
        );
        assert!(d.request().unwrap().fields.is_empty());
        d.refresh_part_footprints();
        assert_eq!(d.value("part_footprint"), footprint.to_string());
        d.focused = i;
        d.change(footprint.to_string());
        assert!(d.request().is_err());
        d.change(String::new());
        assert_eq!(
            d.request().unwrap().fields["default_footprint"],
            Value::Null
        );
        assert_eq!(d.part.original, Some(raw.clone()));
    }
}
