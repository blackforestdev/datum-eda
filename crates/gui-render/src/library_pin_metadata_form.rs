//! Alternate names and swap groups of the selected saved logical pin.
use super::*;

pub(super) fn table(p: &mut Painter<'_>, d: &LibraryDraft) {
    let Some(metadata) = d.selected_pin_metadata() else {
        return;
    };
    p.line("Alternate names");
    for (&index, row) in &metadata.rows {
        let active = metadata.selected == Some(index);
        let (name, kind) = if active {
            (d.value("pin_alternate_name"), d.value("pin_alternate_kind"))
        } else {
            (
                row["name"].as_str().unwrap_or(""),
                row["kind"].as_str().unwrap_or(""),
            )
        };
        let selected = if active { " · Selected" } else { "" };
        p.button(
            &format!("{name:?} · {kind:?}{selected}"),
            C::PinAlternate(index),
        );
    }
    p.button("Add alternate name", C::AddPinAlternate);
    if metadata.selected.is_some() {
        p.button("Remove alternate name", C::RemovePinAlternate);
    }
}

pub(super) fn review(p: &mut Painter<'_>, raw: &Value) {
    let Some(group) = raw["swap_group"].as_u64() else {
        return;
    };
    let Some(alternates) = raw["alternates"].as_array() else {
        return;
    };
    let pin = raw["name"].as_str().unwrap_or("");
    if group == 0 {
        p.line(&format!("{pin} · No swap group"));
    } else {
        p.line(&format!("{pin} · Swap group {group}"));
    }
    p.line(&format!("Alternate names · {}", alternates.len()));
    for row in alternates {
        if let (Some(name), Some(kind)) = (row["name"].as_str(), row["kind"].as_str()) {
            p.line(&format!("{name:?} · {kind:?}"));
        }
    }
}
