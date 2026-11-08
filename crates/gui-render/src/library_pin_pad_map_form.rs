//! Full named map rows over the shared Library scroll/focus backbone.
use super::definition_table::row;
use super::*;
use datum_gui_protocol::library_authoring::LibraryDraft;

fn pair<'a>(d: &'a LibraryDraft, raw: &'a Value) -> (&'a str, &'a str) {
    (
        d.pin_pad_map
            .gate_label(raw.get("gate").and_then(Value::as_str).unwrap_or("")),
        d.pin_pad_map.pin_label(
            raw.get("gate").and_then(Value::as_str).unwrap_or(""),
            raw.get("pin").and_then(Value::as_str).unwrap_or(""),
        ),
    )
}
pub(super) fn table(p: &mut Painter<'_>, d: &LibraryDraft) {
    p.line("Pad → gate-qualified logical pin · unmapped pads stay unmapped");
    let rows = match d.pin_pad_mappings() {
        Ok(rows) => rows,
        Err(reason) => {
            p.line(&format!("Draft mapping needs attention: {reason}"));
            d.pin_pad_map.mappings.clone()
        }
    };
    p.line(&format!(
        "{} mapped / {} pads · unmapped pads have no pin assignment",
        rows.len(),
        d.pin_pad_map.pads.len()
    ));
    row(
        p,
        ["Pad", "Gate / Unit", "Logical pin", "State"],
        None,
        false,
    );
    for (index, pad) in d.pin_pad_map.pads.iter().enumerate() {
        let key = pad.id.map(|id| id.to_string());
        let active = key.as_deref() == Some(d.value("map_pad"));
        let entry = key.as_ref().and_then(|id| rows.get(id));
        let (gate, pin, state) = if pad.id.is_none() {
            ("Unavailable", "Unavailable", "Source unavailable")
        } else if active && (!d.value("map_gate").is_empty() || !d.value("map_pin").is_empty()) {
            (
                d.pin_pad_map.gate_label(d.value("map_gate")),
                d.pin_pad_map
                    .pin_label(d.value("map_gate"), d.value("map_pin")),
                if d.preview.is_some() {
                    "Validated preview"
                } else {
                    "Draft / preview required"
                },
            )
        } else if let Some(entry) = entry {
            let (gate, pin) = pair(d, entry);
            (
                gate,
                pin,
                if d.preview.is_some() {
                    "Validated preview"
                } else {
                    "Draft / preview required"
                },
            )
        } else {
            ("—", "—", "Unmapped")
        };
        row(
            p,
            [&pad.label, gate, pin, state],
            Some(C::MappingPad(index)),
            active,
        );
    }
    p.button("Unmap selected pad", C::UnmapPad);
    if d.create {
        p.line("Default assignment begins off. It requires a matching explicit Part default Footprint and no placed users. Preview checks whether assignment is available.");
    } else {
        p.line(&format!(
            "Part: {} · read-only",
            d.pin_pad_map.part_label(d.map_source("map_part"))
        ));
        p.line(&format!(
            "Footprint: {} · read-only",
            d.pin_pad_map.footprint_label(d.map_source("map_footprint"))
        ));
        p.line("Existing placed mappings stay pinned; Part default unchanged");
    }
}
pub(super) fn review(p: &mut Painter<'_>, d: &LibraryDraft, raw: &Value) {
    p.line(if d.create {
        "Review: create PinPadMap"
    } else {
        "Review: revise PinPadMap assignments"
    });
    p.line(&format!(
        "Part: {}",
        d.pin_pad_map.part_label(d.map_source("map_part"))
    ));
    p.line(&format!(
        "Footprint: {}",
        d.pin_pad_map.footprint_label(d.map_source("map_footprint"))
    ));
    let rows = raw.get("mappings").and_then(Value::as_object);
    p.line(&format!(
        "{} mapped / {} pads",
        rows.map_or(0, |m| m.len()),
        d.pin_pad_map.pads.len()
    ));
    for (pad, entry) in rows.into_iter().flatten() {
        let (gate, pin) = pair(d, entry);
        p.line(&format!(
            "{} → {} / {}",
            d.pin_pad_map.pad_label(pad),
            gate,
            pin
        ));
    }
    if d.value("map_default") == "true" {
        p.line("Also set this new map as the selected Part default in the same transaction");
        p.line(&format!(
            "Previous default: {}",
            d.pin_pad_map.previous_default(d.map_source("map_part"))
        ));
    } else {
        p.line("Map only · leave the Part default unchanged");
    }
}
