//! Explicit Line intent addresses original ordered rows under captured source guards.
use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    index: usize,
    previous: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Edit {
    index: usize,
    previous: Value,
    line: Value,
}

fn original<'a>(rows: &'a [Value], row: &Row) -> Result<&'a Value, EngineError> {
    let raw = rows
        .get(row.index)
        .filter(|raw| **raw == row.previous)
        .ok_or_else(|| refusal("Saved Line index or captured source has changed"))?;
    if !matches!(
        serde_json::from_value::<crate::pool::Primitive>(raw.clone())?,
        crate::pool::Primitive::Line { .. }
    ) {
        return Err(refusal("Select an existing Line graphic"));
    }
    Ok(raw)
}

pub(super) fn apply(
    request: &LibraryDefinitionRequest,
    previous: &Value,
    object: &mut Value,
) -> Result<(), EngineError> {
    let empty = Vec::new();
    let rows = match previous.get("drawings") {
        None => &empty,
        Some(value) => value
            .as_array()
            .ok_or_else(|| refusal("Symbol drawings must remain an ordered array"))?,
    };
    let mut next = object
        .get("drawings")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut selected = HashSet::new();
    if let Some(value) = request.fields.get("line_edits") {
        for edit in serde_json::from_value::<Vec<Edit>>(value.clone())? {
            let row = Row {
                index: edit.index,
                previous: edit.previous,
            };
            let raw = original(rows, &row)?;
            if !selected.insert(row.index) {
                return Err(refusal("Conflicting edits address the same original Line"));
            }
            let crate::pool::Primitive::Line { from, to, width } =
                serde_json::from_value(edit.line.clone())?
            else {
                return Err(refusal("Line correction retains the existing graphic kind"));
            };
            let mut permitted = raw.clone();
            for (point, value) in [("from", from), ("to", to)] {
                permitted["Line"][point]["x"] = json!(value.x);
                permitted["Line"][point]["y"] = json!(value.y);
            }
            permitted["Line"]["width"] = json!(width);
            if permitted != edit.line {
                return Err(refusal(
                    "Line correction preserves unrelated raw row and point metadata",
                ));
            }
            // Existing manual drawing edits admit the Primitive schema; append
            // has its separate, established positive-width/distinct-end policy.
            next[row.index] = permitted;
        }
    }
    let mut removed = HashSet::new();
    if let Some(value) = request.fields.get("line_removals") {
        for row in serde_json::from_value::<Vec<Row>>(value.clone())? {
            original(rows, &row)?;
            if !selected.insert(row.index) {
                return Err(refusal(
                    "Removal conflicts with another edit of the same Line",
                ));
            }
            removed.insert(row.index);
        }
    }
    next = next
        .into_iter()
        .enumerate()
        .filter_map(|(i, raw)| (!removed.contains(&i)).then_some(raw))
        .collect();
    if let Some(value) = request.fields.get("line_additions") {
        for raw in value
            .as_array()
            .ok_or_else(|| refusal("New Lines must be an ordered array"))?
        {
            let crate::pool::Primitive::Line { from, to, width } =
                serde_json::from_value(raw.clone())?
            else {
                return Err(refusal("Add Line accepts only native Line graphics"));
            };
            if from == to || width <= 0 {
                return Err(refusal(
                    "New Line requires distinct endpoints and positive stroke width",
                ));
            }
            next.push(serde_json::to_value(crate::pool::Primitive::Line {
                from,
                to,
                width,
            })?);
        }
    }
    if previous.get("drawings").is_some() || !next.is_empty() {
        object["drawings"] = Value::Array(next);
    }
    Ok(())
}
