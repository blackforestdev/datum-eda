//! Shared logical-definition review dispatch for the Library pane and inspector.
use super::*;
use datum_gui_protocol::library_authoring::LibraryDraft;
pub(super) fn review(p: &mut Painter<'_>, d: &LibraryDraft) {
    let Some(raw) = &d.preview else {
        return;
    };
    match d.kind.as_str() {
        "units" | "symbols" => electrical_form::review(p, d),
        "entities" => entity_form::review(p, raw),
        "parts" => part_form::review(p, raw),
        "pin_pad_maps" => pin_pad_map_form::review(p, d, raw),
        _ => {}
    }
}
pub(super) fn inspector(p: &mut Painter<'_>, d: &LibraryDraft) -> bool {
    if matches!(d.kind.as_str(), "entities" | "parts" | "pin_pad_maps") {
        review(p, d);
        true
    } else {
        false
    }
}
