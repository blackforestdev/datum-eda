//! Caption projection keeps source identity and failure distinctions intact.
use super::*;
use crate::library_browser::LibraryBrowser;
use serde_json::json;

fn entry(id: u128, kind: &str, fields: Value) -> LibraryEntry {
    let id = Uuid::from_u128(id);
    let mut raw = fields;
    raw["uuid"] = json!(id);
    LibraryEntry {
        id,
        kind: kind.into(),
        name: "Original source".into(),
        source_path: format!("pool/{kind}/{id}.json"),
        revision: Some(4),
        raw,
        warning: None,
    }
}
fn sources() -> Vec<LibraryEntry> {
    vec![
        entry(
            1,
            "pin_pad_maps",
            json!({"part":Uuid::from_u128(2),"footprint":Uuid::from_u128(3),"tags":["sensor-tag"],"mappings":{}}),
        ),
        entry(
            2,
            "parts",
            json!({"manufacturer":"Maker","mpn":"FIRST","default_footprint":Uuid::from_u128(3)}),
        ),
        entry(3, "footprints", json!({"name":"Two lands"})),
        entry(
            4,
            "pin_pad_maps",
            json!({"part":Uuid::from_u128(5),"footprint":Uuid::from_u128(3)}),
        ),
        entry(5, "parts", json!({"manufacturer":"Maker","mpn":"SECOND"})),
        entry(
            6,
            "pin_pad_maps",
            json!({"part":Uuid::from_u128(2),"footprint":Uuid::from_u128(3)}),
        ),
    ]
}

#[test]
fn captions_search_preserve_rows_keys_payload_order_and_relationships() {
    let mut browser = LibraryBrowser {
        entries: sources(),
        kind: 7,
        ..Default::default()
    };
    let before = browser.clone();
    project(&mut browser.entries);
    assert_eq!(
        browser.entries[0].name,
        "Part: Maker FIRST · Footprint: Two lands"
    );
    assert_eq!(
        browser.entries[3].name,
        "Part: Maker SECOND · Footprint: Two lands"
    );
    assert_eq!(
        browser.entries[5].name, browser.entries[0].name,
        "same caption never collapses maps"
    );
    for (old, new) in before.entries.iter().zip(&browser.entries) {
        let mut restored = new.clone();
        restored.name = old.name.clone();
        assert_eq!(&restored, old);
        assert_eq!(browser.links(new), before.links(old));
        assert_eq!(
            browser
                .selected(&crate::SelectionTarget::AuthoredObject(new.key()))
                .unwrap()
                .id,
            old.id
        );
    }
    assert_eq!(browser.results(), vec![0, 3, 5]);
    for (query, expected) in [
        ("second", vec![3]),
        ("FIRST", vec![0, 5]),
        ("two lands", vec![0, 3, 5]),
        ("sensor-tag", vec![0]),
    ] {
        browser.search = query.into();
        assert_eq!(browser.results(), expected);
    }
    browser.kind = 3;
    browser.search = "SECOND".into();
    assert_eq!(browser.results(), vec![4]);
    assert_eq!(browser.entries[4].name, before.entries[4].name);
}

#[test]
fn declared_reference_failures_and_optional_absence_do_not_guess() {
    for (value, expected) in [
        (None, "Reference missing"),
        (Some(json!(null)), "Malformed reference"),
        (Some(json!(7)), "Malformed reference"),
        (Some(json!("not-a-uuid")), "Malformed reference"),
        (Some(json!(Uuid::from_u128(99))), "Missing"),
        (Some(json!(Uuid::from_u128(3))), "Wrong kind"),
    ] {
        let mut entries = sources();
        if let Some(value) = value {
            entries[0].raw["part"] = value;
        } else {
            entries[0].raw.as_object_mut().unwrap().remove("part");
        }
        project(&mut entries);
        assert!(
            entries[0].name.starts_with(&format!("Part: {expected}")),
            "{}",
            entries[0].name
        );
    }
    for absent in [false, true] {
        let mut entries = sources();
        if absent {
            entries[0].raw.as_object_mut().unwrap().remove("footprint");
        } else {
            entries[0].raw["footprint"] = Value::Null;
        }
        project(&mut entries);
        assert_eq!(
            entries[0].name, "Part: Maker FIRST · Footprint: Absent",
            "Part default must not supply an undeclared Footprint"
        );
    }
    for (value, expected) in [
        (json!("bad"), "Malformed reference"),
        (json!(Uuid::from_u128(99)), "Missing"),
        (json!(Uuid::from_u128(2)), "Wrong kind"),
    ] {
        let mut entries = sources();
        entries[0].raw["footprint"] = value;
        project(&mut entries);
        assert!(entries[0].name.contains(&format!("Footprint: {expected}")));
    }
    let mut entries = sources();
    entries[1].raw = json!({"uuid":Uuid::from_u128(2),"name":"Declared legacy identity"});
    project(&mut entries);
    assert!(entries[0].name.contains("Part: Declared legacy identity"));
    let mut entries = sources();
    entries[2].raw["name"] = json!(42);
    project(&mut entries);
    assert!(
        entries[0]
            .name
            .contains("Footprint: Display identity unavailable")
    );
}

#[test]
fn ambiguous_cross_kind_and_unreadable_sources_never_supply_a_name() {
    for duplicate_kind in ["parts", "symbols"] {
        let mut entries = sources();
        entries.push(entry(2, duplicate_kind, json!({"mpn":"Wrong candidate"})));
        entries[1].warning =
            Some("Multiple source candidates share this identity; inspection only".into());
        let before = entries.clone();
        project(&mut entries);
        assert!(entries[0].name.starts_with("Part: Ambiguous"));
        assert!(!entries[0].name.contains("FIRST"));
        assert!(!entries[0].name.contains("Wrong candidate"));
        assert_eq!(entries[1].warning, before[1].warning);
    }
    let mut entries = sources();
    entries[1].warning = Some("Source is unreadable".into());
    project(&mut entries);
    assert!(entries[0].name.starts_with("Part: Unavailable"));
    let mut entries = sources();
    entries[1].raw["uuid"] = json!(Uuid::from_u128(88));
    project(&mut entries);
    assert!(
        entries[0]
            .name
            .starts_with("Part: Malformed source identity")
    );
    let mut entries = sources();
    entries[0].raw = Value::Null;
    entries[0].warning = Some("Cannot materialize source".into());
    project(&mut entries);
    assert_eq!(entries[0].name, "PinPadMap source unavailable");
    assert!(!entries[0].name.contains("Absent"));
    let mut entries = sources();
    let mut duplicate = entries[0].clone();
    duplicate.source_path = "pool/pin_pad_maps/duplicate.json".into();
    duplicate.warning = Some("Ambiguous map source".into());
    entries[0].warning = duplicate.warning.clone();
    entries.push(duplicate);
    project(&mut entries);
    assert!(entries[0].name.starts_with("Ambiguous map source"));
    assert_eq!(entries[0].name, entries.last().unwrap().name);
    assert_ne!(entries[0].key(), entries.last().unwrap().key());
}
