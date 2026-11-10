//! Line selection/actions reuse the Library form, painter and numeric editors.
use super::*;

pub(super) fn table(p: &mut Painter<'_>, d: &LibraryDraft) {
    if d.create {
        return;
    }
    p.line("Line graphics · select a row to edit endpoints and stroke");
    for index in d.electrical.lines.rows.keys() {
        p.button(
            &format!(
                "{}Line graphic {}",
                if d.electrical.lines.selected == Some(*index) {
                    "Selected · "
                } else {
                    ""
                },
                index + 1
            ),
            C::Line(*index),
        );
    }
    p.button("Add line", C::AddLine);
    if d.electrical.lines.selected.is_some() {
        p.button("Remove line", C::RemoveLine);
    }
}

pub(super) fn review(p: &mut Painter<'_>, raw: &Value) {
    for (index, row) in raw["drawings"].as_array().into_iter().flatten().enumerate() {
        if let Some(line) = row.get("Line") {
            let coordinate = |pointer: &str| {
                line.pointer(pointer)
                    .and_then(Value::as_i64)
                    .map(physical)
                    .unwrap_or_else(|| "Unavailable".into())
            };
            p.line(&format!(
                "Line {}: ({}, {}) to ({}, {}); stroke {}",
                index + 1,
                coordinate("/from/x"),
                coordinate("/from/y"),
                coordinate("/to/x"),
                coordinate("/to/y"),
                coordinate("/width")
            ));
        }
    }
}
