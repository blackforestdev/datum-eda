//! Unit pin table, Symbol anchor/body properties and readable source review.
use super::*;
use datum_gui_protocol::library_authoring::LibraryDraft;

pub(super) fn table(p: &mut Painter<'_>, d: &LibraryDraft, rect: RectPx) {
    if d.kind == "units" {
        for (column, label) in ["Pin name", "Electrical type"].into_iter().enumerate() {
            let width = (rect.width - 16.).max(1.) / 2.;
            let cell = RectPx {
                x: rect.x + 8. + column as f32 * width,
                y: p.y,
                width,
                height: 26.,
            };
            draw_text_clipped(
                label,
                cell.x + 5.,
                cell.y + 6.,
                12.,
                TEXT_PRIMARY,
                TextFace::UiStrong,
                cell,
                p.text,
            );
        }
        p.y += 26.;
        for (index, (id, pin)) in d.electrical.pins.iter().enumerate() {
            let active = d
                .electrical
                .pin_id
                .is_some_and(|pin| id.parse().ok() == Some(pin));
            let name = if active {
                d.value("pin_name")
            } else {
                pin.get("name").and_then(Value::as_str).unwrap_or("")
            };
            let kind = if active {
                d.value("pin_type")
            } else {
                pin.get("electrical_type")
                    .filter(|v| v.is_string())
                    .or_else(|| pin.get("direction"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
            };
            pin_row(p, rect, name, kind, C::Pin(index), active);
        }
        if d.electrical.pin_id.is_some_and(|id| {
            !d.electrical
                .pins
                .keys()
                .any(|key| key.parse().ok() == Some(id))
        }) {
            let field = d.fields.iter().position(|f| f.key == "pin_name").unwrap();
            pin_row(
                p,
                rect,
                d.value("pin_name"),
                d.value("pin_type"),
                C::Field(field),
                true,
            );
        }
        p.button("Add pin", C::AddPin);
        if d.create || d.electrical.selected_new_pin() {
            p.button("Remove draft pin", C::RemovePin);
        }
        if !d.create {
            p.line("Existing logical pin identities retained");
        }
    } else {
        p.line("Pin anchors · coordinates and stub lengths have physical units");
        for (index, anchor) in d.electrical.anchors.values().enumerate() {
            p.button(&anchor_label(d, anchor), C::Anchor(index));
        }
        p.button("Add anchor", C::AddAnchor);
        if d.create || d.electrical.new_anchor || d.electrical.anchor_id.is_some() {
            p.button(
                if !d.create && !d.electrical.new_anchor {
                    "Remove anchor"
                } else {
                    "Remove draft anchor"
                },
                C::RemoveAnchor,
            );
        }
        if !d.create {
            p.line("Unit and logical pins retained");
            if d.saved_rectangle_index().is_none() {
                p.line("Body correction unavailable · exactly one existing rectangle required");
            }
        }
    }
}

fn pin_row(p: &mut Painter<'_>, rect: RectPx, name: &str, kind: &str, control: C, active: bool) {
    let row = RectPx {
        x: rect.x + 8.,
        y: p.y,
        width: (rect.width - 16.).max(1.),
        height: 30.,
    };
    p.quads
        .push(Quad::from_rect(row, design_tokens::chrome::SURFACE_02));
    if active {
        push_rect_border(p.quads, row, TEXT_ACCENT, 1.);
    }
    for (column, label) in [name, kind].into_iter().enumerate() {
        let cell = RectPx {
            x: row.x + column as f32 * row.width / 2.,
            width: row.width / 2.,
            ..row
        };
        draw_text_clipped(
            if label.is_empty() {
                "Not entered"
            } else {
                label
            },
            cell.x + 5.,
            cell.y + 7.,
            12.,
            TEXT_PRIMARY,
            TextFace::Ui,
            cell,
            p.text,
        );
    }
    p.hits.push(HitRegion {
        target: HitTarget::Library(control),
        rect: row,
    });
    p.y += row.height + 4.;
}

pub(super) fn review(p: &mut Painter<'_>, d: &LibraryDraft) {
    let Some(raw) = d.preview.as_ref() else {
        return;
    };
    let name = raw.get("name").and_then(Value::as_str).unwrap_or("");
    let action = if d.create { "create" } else { "edit" };
    if d.kind == "units" {
        let pins = raw.get("pins").and_then(Value::as_object);
        p.line(&format!(
            "Review: {action} Unit {name} · {} logical pins",
            pins.map_or(0, |p| p.len())
        ));
        for pin in pins.into_iter().flat_map(|p| p.values()) {
            p.line(&format!(
                "{} · {}",
                pin.get("name").and_then(Value::as_str).unwrap_or(""),
                pin.get("electrical_type")
                    .filter(|v| v.is_string())
                    .or_else(|| pin.get("direction"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ));
        }
    } else {
        let unit = d
            .electrical
            .unit_label
            .as_deref()
            .or_else(|| {
                d.fields
                    .iter()
                    .find(|f| f.key == "unit")
                    .and_then(|f| f.choices.iter().find(|c| c.0 == f.text))
                    .map(|c| c.1.as_str())
            })
            .unwrap_or("Unavailable");
        p.line(&format!(
            "Review: {action} Symbol {name} linked to Unit {unit}"
        ));
        let coordinate = |pointer: &str| {
            raw.pointer(pointer)
                .and_then(Value::as_i64)
                .map(physical)
                .unwrap_or_else(|| "Unavailable".into())
        };
        let rectangle = if d.create {
            Some(0)
        } else {
            d.saved_rectangle_index()
        };
        if let Some(index) = rectangle {
            let base = format!("/drawings/{index}/Rect");
            p.line(&format!(
                "Rectangle: ({}, {}) to ({}, {}); stroke {}",
                coordinate(&format!("{base}/min/x")),
                coordinate(&format!("{base}/min/y")),
                coordinate(&format!("{base}/max/x")),
                coordinate(&format!("{base}/max/y")),
                coordinate(&format!("{base}/width"))
            ));
        }
        if !d.create {
            p.line(if rectangle.is_some() {
                "Other graphics and fields retained · placed users stay frozen"
            } else {
                "Body correction unavailable · exactly one existing rectangle required"
            });
        }
        for anchor in raw
            .get("pin_anchors")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            p.line(&anchor_label(d, anchor));
        }
    }
    p.line(if d.create {
        "Preview only · Commit creates one definition · Cancel discards the draft"
    } else {
        "Preview only · Commit edits this source · Cancel discards the draft"
    });
}

fn anchor_label(d: &LibraryDraft, anchor: &Value) -> String {
    let pin = anchor.get("pin").and_then(Value::as_str).unwrap_or("");
    let name = d
        .electrical
        .unit_pins
        .iter()
        .find(|(id, _)| {
            id.parse::<eda_engine::substrate::ObjectId>().ok()
                == d.electrical_unit()
                    .parse::<eda_engine::substrate::ObjectId>()
                    .ok()
        })
        .map(|(_, pins)| pins)
        .and_then(|pins| {
            pins.iter().find(|(id, _)| {
                id.parse::<eda_engine::substrate::ObjectId>()
                    .ok()
                    .is_some_and(|id| {
                        pin.parse::<eda_engine::substrate::ObjectId>().ok() == Some(id)
                    })
            })
        })
        .map(|(_, name)| name.as_str())
        .unwrap_or("Unavailable pin");
    let length = |key: &str| {
        (if key == "/style/length_nm" {
            anchor.get("length_nm").and_then(Value::as_i64)
        } else {
            None
        })
        .or_else(|| anchor.pointer(key).and_then(Value::as_i64))
        .map(physical)
        .unwrap_or_else(|| "Unavailable".into())
    };
    format!(
        "{name}: ({}, {}), {} stub {}",
        length("/position/x"),
        length("/position/y"),
        anchor
            .get("orientation")
            .and_then(Value::as_str)
            .or_else(|| anchor.pointer("/style/orientation").and_then(Value::as_str))
            .unwrap_or("Unavailable"),
        length("/style/length_nm")
    )
}

fn physical(nm: i64) -> String {
    let absolute = nm.unsigned_abs();
    let sign = if nm < 0 { "-" } else { "" };
    let raw = format!("{sign}{}.{:06}", absolute / 1_000_000, absolute % 1_000_000);
    format!("{} mm", raw.trim_end_matches('0').trim_end_matches('.'))
}
