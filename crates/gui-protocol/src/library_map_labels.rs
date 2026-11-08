//! Disposable map captions over declared, identity-checked browser relationships.
use super::LibraryEntry;
use serde_json::Value;
use uuid::Uuid;

fn text<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn reference(entries: &[LibraryEntry], value: Option<&Value>, kind: &str) -> String {
    if kind == "footprints" && value.is_none_or(Value::is_null) {
        return "Absent".into();
    }
    let Some(value) = value else {
        return "Reference missing".into();
    };
    let Some(id) = value.as_str().and_then(|s| Uuid::parse_str(s).ok()) else {
        return "Malformed reference".into();
    };
    let identities: Vec<_> = entries.iter().filter(|entry| entry.id == id).collect();
    let candidates: Vec<_> = identities
        .iter()
        .filter(|entry| entry.kind == kind)
        .collect();
    if candidates.is_empty() {
        return format!(
            "{} ({id})",
            if identities.is_empty() {
                "Missing"
            } else {
                "Wrong kind"
            }
        );
    }
    if identities.len() != 1 {
        return format!("Ambiguous ({id})");
    }
    let target = candidates[0];
    if target.warning.is_some() {
        return format!("Unavailable ({id})");
    }
    if target
        .raw
        .get("uuid")
        .and_then(Value::as_str)
        .and_then(|s| Uuid::parse_str(s).ok())
        != Some(id)
    {
        return format!("Malformed source identity ({id})");
    }
    let name = if kind == "parts" {
        text(&target.raw, "mpn")
            .map(|mpn| match text(&target.raw, "manufacturer") {
                Some(manufacturer) => format!("{manufacturer} {mpn}"),
                None => mpn.to_owned(),
            })
            .or_else(|| text(&target.raw, "name").map(str::to_owned))
    } else {
        text(&target.raw, "name").map(str::to_owned)
    };
    name.unwrap_or_else(|| format!("Display identity unavailable ({id})"))
}

pub(super) fn project(entries: &mut [LibraryEntry]) {
    let captions: Vec<_> = entries
        .iter()
        .map(|entry| {
            if entry.kind != "pin_pad_maps" {
                return None;
            }
            if !entry.raw.is_object() {
                return Some("PinPadMap source unavailable".into());
            }
            let prefix = if entry.warning.is_none() {
                ""
            } else if entries.iter().filter(|other| other.id == entry.id).count() > 1 {
                "Ambiguous map source · "
            } else {
                "Unavailable map source · "
            };
            Some(format!(
                "{prefix}Part: {} · Footprint: {}",
                reference(entries, entry.raw.get("part"), "parts"),
                reference(entries, entry.raw.get("footprint"), "footprints")
            ))
        })
        .collect();
    for (entry, caption) in entries.iter_mut().zip(captions) {
        if let Some(caption) = caption {
            entry.name = caption;
        }
    }
}

#[cfg(test)]
#[path = "library_map_labels_tests.rs"]
mod tests;
