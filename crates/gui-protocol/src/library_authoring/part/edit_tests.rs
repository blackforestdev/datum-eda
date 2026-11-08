use super::*;

#[test]
fn saved_part_only_exposes_metadata_and_submits_changed_fields() {
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
            "tags"
        ]
    );
    assert!(d.fields.iter().all(|f| !f.selector));
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
