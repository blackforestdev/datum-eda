//! Shared ordered graphic rows use the Library painter and numeric editors.
use super::*;

pub(super) fn table(p: &mut Painter<'_>, d: &LibraryDraft) {
    if d.create {
        return;
    }
    p.line("Graphics · select a row to edit geometry and stroke");
    for control in d.electrical.graphics.controls() {
        let label = match control {
            C::Line(index) | C::Rect(index) | C::Text(index) | C::Circle(index) | C::Arc(index) => {
                format!(
                    "{}{} graphic {}",
                    if d.electrical.graphics.selected == Some(index) {
                        "Selected · "
                    } else {
                        ""
                    },
                    if matches!(control, C::Text(_)) {
                        "Text"
                    } else if matches!(control, C::Rect(_)) {
                        "Rectangle"
                    } else if matches!(control, C::Circle(_)) {
                        "Circle"
                    } else if matches!(control, C::Arc(_)) {
                        "Arc"
                    } else {
                        "Line"
                    },
                    index + 1
                )
            }
            C::AddLine => "Add line".into(),
            C::AddRect => "Add rectangle".into(),
            C::AddText => "Add text".into(),
            C::AddCircle => "Add circle".into(),
            C::RemoveCircle => "Remove circle".into(),
            C::AddArc => "Add arc".into(),
            C::RemoveArc => "Remove arc".into(),
            C::RemoveLine => "Remove line".into(),
            C::RemoveRect => "Remove rectangle".into(),
            C::RemoveText => "Remove text".into(),
            _ => continue,
        };
        p.button(&label, control);
    }
}

pub(super) fn review(p: &mut Painter<'_>, raw: &Value) {
    for (index, row) in raw["drawings"].as_array().into_iter().flatten().enumerate() {
        if let Some(text) = row.get("Text") {
            let coordinate = |key: &str| {
                text.pointer(key)
                    .and_then(Value::as_i64)
                    .map(physical)
                    .unwrap_or_else(|| "Unavailable".into())
            };
            p.line(&format!(
                "Text {}: {} at ({}, {}); rotation {}°",
                index + 1,
                text["text"],
                coordinate("/position/x"),
                coordinate("/position/y"),
                text["rotation"]
            ));
            continue;
        }
        if let Some(circle) = row.get("Circle") {
            let coordinate = |pointer: &str| {
                circle
                    .pointer(pointer)
                    .and_then(Value::as_i64)
                    .map(physical)
                    .unwrap_or_else(|| "Unavailable".into())
            };
            p.line(&format!(
                "Circle {}: center ({}, {}); radius {}; stroke {}",
                index + 1,
                coordinate("/center/x"),
                coordinate("/center/y"),
                coordinate("/radius"),
                coordinate("/width")
            ));
            continue;
        }
        if let Some(arc) = row.get("Arc") {
            let coordinate = |pointer: &str| {
                arc.pointer(pointer)
                    .and_then(Value::as_i64)
                    .map(physical)
                    .unwrap_or_else(|| "Unavailable".into())
            };
            p.line(&format!(
                "Arc {}: center ({}, {}); radius {}; angles {} to {} (tenths of degree); stroke {}",
                index + 1,
                coordinate("/arc/center/x"),
                coordinate("/arc/center/y"),
                coordinate("/arc/radius"),
                arc["arc"]["start_angle"],
                arc["arc"]["end_angle"],
                coordinate("/width")
            ));
            continue;
        }
        let (kind, a, b) = if row.get("Rect").is_some() {
            ("Rect", "min", "max")
        } else if row.get("Line").is_some() {
            ("Line", "from", "to")
        } else {
            continue;
        };
        let coordinate = |pointer: &str| {
            row[kind]
                .pointer(pointer)
                .and_then(Value::as_i64)
                .map(physical)
                .unwrap_or_else(|| "Unavailable".into())
        };
        p.line(&format!(
            "{kind} {}: ({}, {}) to ({}, {}); stroke {}",
            index + 1,
            coordinate(&format!("/{a}/x")),
            coordinate(&format!("/{a}/y")),
            coordinate(&format!("/{b}/x")),
            coordinate(&format!("/{b}/y")),
            coordinate("/width")
        ));
    }
}
