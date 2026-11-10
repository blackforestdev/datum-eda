//! Captured ordered-row graphics edits share identity, conflict and append admission.
use super::*;
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    index: usize,
    previous: Value,
}
fn original<'a>(rows: &'a [Value], row: &Row, kind: &str) -> Result<&'a Value, EngineError> {
    let raw = rows
        .get(row.index)
        .filter(|raw| **raw == row.previous)
        .ok_or_else(|| refusal("Saved graphic index or captured source has changed"))?;
    let _: crate::pool::Primitive = serde_json::from_value(raw.clone())?;
    if raw.get(kind).is_none() {
        return Err(refusal(format!("Select an existing {kind} graphic")));
    }
    Ok(raw)
}
fn corrected(raw: &Value, edited: &Value, kind: &str) -> Result<Value, EngineError> {
    let primitive: crate::pool::Primitive = serde_json::from_value(edited.clone())?;
    let mut permitted = raw.clone();
    if let crate::pool::Primitive::Text {
        text,
        position,
        rotation,
    } = &primitive
    {
        if kind != "Text" {
            return Err(refusal("Correction retains the captured graphic kind"));
        }
        permitted["Text"]["text"] = json!(text);
        permitted["Text"]["position"]["x"] = json!(position.x);
        permitted["Text"]["position"]["y"] = json!(position.y);
        permitted["Text"]["rotation"] = json!(rotation);
    } else {
        let (first, second, a, b, width) = match primitive {
            crate::pool::Primitive::Line { from, to, width } if kind == "Line" => {
                ("from", "to", from, to, width)
            }
            crate::pool::Primitive::Rect { min, max, width } if kind == "Rect" => {
                if edited != raw && (min.x >= max.x || min.y >= max.y || width <= 0) {
                    return Err(refusal(
                        "Rectangle requires ordered corners and positive stroke width",
                    ));
                }
                ("min", "max", min, max, width)
            }
            _ => return Err(refusal("Correction retains the captured graphic kind")),
        };
        for (point, value) in [(first, a), (second, b)] {
            permitted[kind][point]["x"] = json!(value.x);
            permitted[kind][point]["y"] = json!(value.y);
        }
        permitted[kind]["width"] = json!(width);
    }
    if &permitted != edited {
        return Err(refusal(
            "Graphic correction preserves unrelated raw row and Point metadata",
        ));
    }
    Ok(permitted)
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
    let mut removed = HashSet::new();
    for (kind, edits, removals, payload) in [
        ("Line", "line_edits", "line_removals", "line"),
        ("Rect", "rect_edits", "rect_removals", "rect"),
        ("Text", "text_edits", "text_removals", "text"),
    ] {
        if let Some(value) = request.fields.get(edits) {
            for edit in value
                .as_array()
                .ok_or_else(|| refusal("Graphic edits must be an ordered array"))?
            {
                let mut fields = edit.as_object().cloned().ok_or_else(|| {
                    refusal("Graphic edit requires an original row and replacement")
                })?;
                let edited = fields
                    .remove(payload)
                    .ok_or_else(|| refusal(format!("Graphic edit requires {payload}")))?;
                let row: Row = serde_json::from_value(Value::Object(fields))?;
                let raw = original(rows, &row, kind)?;
                if !selected.insert(row.index) {
                    return Err(refusal(
                        "Conflicting edits address the same original graphic",
                    ));
                }
                let permitted = corrected(raw, &edited, kind)?;
                if next[row.index] != *raw && next[row.index] != permitted {
                    return Err(refusal(
                        "Legacy body and selected graphic edits conflict on the same original row",
                    ));
                }
                next[row.index] = permitted;
            }
        }
        if let Some(value) = request.fields.get(removals) {
            for row in serde_json::from_value::<Vec<Row>>(value.clone())? {
                let raw = original(rows, &row, kind)?;
                if !selected.insert(row.index) || next[row.index] != *raw {
                    return Err(refusal(
                        "Graphic removal conflicts with another edit of the same original row",
                    ));
                }
                removed.insert(row.index);
            }
        }
    }
    next = next
        .into_iter()
        .enumerate()
        .filter_map(|(i, raw)| (!removed.contains(&i)).then_some(raw))
        .collect();
    if request.fields.contains_key("line_additions")
        && request.fields.contains_key("graphic_additions")
    {
        return Err(refusal("Use one ordered collection of new graphics"));
    }
    let additions = request
        .fields
        .get("graphic_additions")
        .or_else(|| request.fields.get("line_additions"));
    if let Some(value) = additions {
        for raw in value
            .as_array()
            .ok_or_else(|| refusal("New graphics must be an ordered array"))?
        {
            let mut primitive: crate::pool::Primitive = serde_json::from_value(raw.clone())?;
            match &primitive {
                crate::pool::Primitive::Line { from, to, width } => {
                    if from == to || *width <= 0 {
                        return Err(refusal(
                            "New Line requires distinct endpoints and positive stroke width",
                        ));
                    }
                }
                crate::pool::Primitive::Rect { min, max, width }
                    if !request.fields.contains_key("line_additions") =>
                {
                    if min.x >= max.x || min.y >= max.y || *width <= 0 {
                        return Err(refusal(
                            "New Rectangle requires ordered corners and positive stroke width",
                        ));
                    }
                }
                crate::pool::Primitive::Text { text, .. }
                    if !request.fields.contains_key("line_additions") =>
                {
                    if text.trim().is_empty() {
                        return Err(refusal("New Symbol Text must not be empty"));
                    }
                }
                _ => {
                    return Err(refusal(
                        "Add graphic accepts only supported native Line/Rect/Text graphics",
                    ));
                }
            }
            if let crate::pool::Primitive::Text { text, .. } = &mut primitive {
                *text = text.trim().to_owned();
            }
            next.push(serde_json::to_value(primitive)?);
        }
    }
    if previous.get("drawings").is_some() || !next.is_empty() {
        object["drawings"] = Value::Array(next);
    }
    Ok(())
}
