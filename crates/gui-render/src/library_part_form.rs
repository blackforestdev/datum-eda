//! Readable orderable identity and explicit logical/physical Part link review.
use super::*;
fn named(p: &Painter<'_>, kind: &str, id: &Value) -> String {
    let rows: Vec<_> = p
        .state
        .library
        .entries
        .iter()
        .filter(|e| e.kind == kind && id.as_str() == Some(e.id.to_string().as_str()))
        .collect();
    if rows.len() == 1 && rows[0].warning.is_none() {
        rows[0].name.clone()
    } else {
        "Unavailable / ambiguous".into()
    }
}
pub(super) fn review(p: &mut Painter<'_>, raw: &Value) {
    p.line(&format!(
        "Review Part {}",
        raw["mpn"].as_str().unwrap_or("Unavailable")
    ));
    for (key, label) in [
        ("manufacturer", "Manufacturer"),
        ("mpn", "MPN"),
        ("value", "Value"),
        ("description", "Description"),
        ("datasheet", "Datasheet reference"),
    ] {
        let value = raw[key]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or("Not entered");
        p.line(&format!("{label}: {value}"));
    }
    p.line(&format!("Entity: {}", named(p, "entities", &raw["entity"])));
    p.line(&format!(
        "Package: {}",
        named(p, "packages", &raw["package"])
    ));
    p.line(&format!(
        "Default Footprint: {}",
        if raw["default_footprint"].is_null() {
            "None chosen".into()
        } else {
            named(p, "footprints", &raw["default_footprint"])
        }
    ));
    p.line(&format!(
        "Lifecycle: {} · read-only",
        raw["lifecycle"].as_str().unwrap_or("Unavailable")
    ));
    p.line(&format!("Tags: {}", raw["tags"]));
    p.line(&format!(
        "Pin-pad map: {}",
        if raw["default_pin_pad_map"].is_null() {
            "Not assigned".into()
        } else {
            named(p, "pin_pad_maps", &raw["default_pin_pad_map"])
        }
    ));
}

pub(super) fn saved_links(p: &mut Painter<'_>, raw: &Value) {
    for (key, kind, label) in [
        ("entity", "entities", "Entity"),
        ("package", "packages", "Package"),
    ] {
        let value = if raw[key].is_null() {
            "Not assigned".into()
        } else {
            named(p, kind, &raw[key])
        };
        p.line(&format!("{label}: {value} · read-only"));
    }
    p.line(&format!(
        "Lifecycle: {} · read-only",
        raw["lifecycle"].as_str().unwrap_or("Unavailable")
    ));
}
