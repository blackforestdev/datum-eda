//! Captured-row edits for saved Symbol definition fields and reference prefix.
use super::*;
use serde::Deserialize;

pub(super) const KEYS: &[&str] = &[
    "symbol_field_edits",
    "symbol_field_removals",
    "symbol_field_additions",
    "default_refdes_prefix",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapturedRow {
    index: usize,
    previous: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldEdit {
    index: usize,
    previous: Value,
    field: Value,
}

fn require_preimage(rows: &[Value], index: usize, previous: &Value) -> Result<(), EngineError> {
    if rows.get(index) != Some(previous) {
        return Err(refusal("Saved Symbol field row has changed"));
    }
    Ok(())
}

fn check_metadata(previous: &Value, edited: &Value) -> Result<(), EngineError> {
    const LEAVES: &[&str] = &["key", "value", "position", "visible"];
    if without_fields(previous, LEAVES) != without_fields(edited, LEAVES) {
        return Err(refusal("Symbol field edits must preserve raw metadata"));
    }
    if edited["position"].is_object()
        && without_fields(&previous["position"], &["x", "y"])
            != without_fields(&edited["position"], &["x", "y"])
    {
        return Err(refusal(
            "Symbol field coordinates must preserve position metadata",
        ));
    }
    Ok(())
}

pub(super) fn apply(
    request: &LibraryDefinitionRequest,
    previous: &Value,
    object: &mut Value,
) -> Result<(), EngineError> {
    if let Some(prefix) = request.fields.get("default_refdes_prefix") {
        match prefix {
            Value::String(_) => object["default_refdes_prefix"] = prefix.clone(),
            Value::Null => {
                object
                    .as_object_mut()
                    .unwrap()
                    .remove("default_refdes_prefix");
            }
            _ => return Err(refusal("Reference prefix must be a string or clear intent")),
        }
    }

    if !KEYS
        .iter()
        .any(|key| *key != "default_refdes_prefix" && request.fields.contains_key(*key))
    {
        return Ok(());
    }
    let original = match previous.get("fields") {
        None => &[][..],
        Some(raw) => raw
            .as_array()
            .ok_or_else(|| refusal("Symbol fields must remain an ordered array"))?,
    };
    let mut rows = original.to_vec();
    let mut addressed = HashSet::new();
    if let Some(edits) = request.fields.get("symbol_field_edits") {
        for edit in serde_json::from_value::<Vec<FieldEdit>>(edits.clone())? {
            require_preimage(original, edit.index, &edit.previous)?;
            if !addressed.insert(edit.index) {
                return Err(refusal("A Symbol field row may be addressed only once"));
            }
            if edit.field.get("position") == Some(&Value::Null)
                && edit.previous.get("position") != Some(&Value::Null)
            {
                return Err(refusal("Clear a Symbol field position by omitting it"));
            }
            check_metadata(&edit.previous, &edit.field)?;
            rows[edit.index] = edit.field;
        }
    }
    let mut removed = HashSet::new();
    if let Some(removals) = request.fields.get("symbol_field_removals") {
        for row in serde_json::from_value::<Vec<CapturedRow>>(removals.clone())? {
            require_preimage(original, row.index, &row.previous)?;
            if !addressed.insert(row.index) {
                return Err(refusal("A Symbol field row may be addressed only once"));
            }
            removed.insert(row.index);
        }
    }
    let mut next: Vec<_> = rows
        .into_iter()
        .enumerate()
        .filter_map(|(index, row)| (!removed.contains(&index)).then_some(row))
        .collect();
    if let Some(additions) = request.fields.get("symbol_field_additions") {
        let empty = json!({});
        for field in serde_json::from_value::<Vec<Value>>(additions.clone())? {
            check_metadata(&empty, &field)?;
            next.push(field);
        }
    }
    if next != original {
        *object = library_definition::symbol_with_fields(object.clone(), Value::Array(next))?;
    }
    Ok(())
}
