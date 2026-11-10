//! Saved definition fields and the default reference prefix in the Library form.
use super::*;

pub(super) fn table(p: &mut Painter<'_>, d: &LibraryDraft) {
    if d.create {
        return;
    }
    p.line("Symbol fields");
    for (&index, row) in &d.electrical.symbol_fields.rows {
        let active = d.electrical.symbol_fields.selected == Some(index);
        let (key, value) = if active {
            (d.value("symbol_field_key"), d.value("symbol_field_value"))
        } else {
            (literal(row, "key"), literal(row, "value"))
        };
        let selected = if active { " · Selected" } else { "" };
        p.button(
            &format!("{key:?} = {value:?}{selected}"),
            C::SymbolField(index),
        );
    }
    p.button("Add field", C::AddSymbolField);
    if d.electrical.symbol_fields.selected.is_some() {
        p.button("Remove field", C::RemoveSymbolField);
        p.line("Leave X and Y blank for no field position");
    }
    p.line(&prefix_label(d.symbol_prefix()));
    p.button("Clear prefix (use U)", C::ClearSymbolPrefix);
}

pub(super) fn review(p: &mut Painter<'_>, raw: &Value) {
    p.line(&prefix_label(raw["default_refdes_prefix"].as_str()));
    let rows = raw["fields"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    p.line(&format!("Symbol fields · {}", rows.len()));
    for row in rows {
        let visibility = if row["visible"].as_bool().unwrap_or(false) {
            "Visible"
        } else {
            "Hidden"
        };
        let position = row
            .pointer("/position/x")
            .and_then(Value::as_i64)
            .zip(row.pointer("/position/y").and_then(Value::as_i64))
            .map_or_else(
                || "No position".into(),
                |(x, y)| format!("({}, {})", super::physical(x), super::physical(y)),
            );
        p.line(&format!(
            "{:?} = {:?} · {visibility} · {position}",
            literal(row, "key"),
            literal(row, "value")
        ));
    }
}

fn literal<'a>(row: &'a Value, key: &str) -> &'a str {
    row[key].as_str().unwrap_or("")
}

fn prefix_label(prefix: Option<&str>) -> String {
    match prefix {
        None => "Reference prefix · Default U".into(),
        Some("") => "Reference prefix · Empty (numeric references)".into(),
        Some(prefix) => format!("Reference prefix · {prefix:?}"),
    }
}
