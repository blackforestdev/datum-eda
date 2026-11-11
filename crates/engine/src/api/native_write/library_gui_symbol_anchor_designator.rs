//! Optional schematic designators on captured saved anchor rows.
use super::*;
use serde::Deserialize;

pub(super) const KEY: &str = "anchor_designators";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DesignatorEdit {
    index: usize,
    previous: Value,
    number: Value,
}

pub(super) fn apply(
    request: &LibraryDefinitionRequest,
    original: &Value,
    object: &mut Value,
    removals: &HashSet<Uuid>,
) -> Result<(), EngineError> {
    let edits: Vec<DesignatorEdit> = serde_json::from_value(request.fields[KEY].clone())?;
    let before = original["pin_anchors"]
        .as_array()
        .ok_or_else(|| refusal("Saved designators require an ordered anchor array"))?;
    let after = object["pin_anchors"]
        .as_array_mut()
        .ok_or_else(|| refusal("Saved anchor rows are unavailable"))?;
    let mut identities = HashSet::new();
    for row in before {
        let pin: Uuid = serde_json::from_value(row["pin"].clone())?;
        if pin.is_nil() || !identities.insert(pin) {
            return Err(refusal("Saved Symbol anchor identity is ambiguous"));
        }
    }
    let mut addressed = HashSet::new();
    for edit in edits {
        let raw = before
            .get(edit.index)
            .filter(|row| **row == edit.previous)
            .ok_or_else(|| refusal("Saved anchor designator basis has changed"))?;
        let pin: Uuid = serde_json::from_value(raw["pin"].clone())?;
        if removals.contains(&pin) || !addressed.insert(edit.index) {
            return Err(refusal(
                "An anchor designator edit requires one retained original row",
            ));
        }
        let row = after
            .get_mut(edit.index)
            .and_then(Value::as_object_mut)
            .filter(|row| row.get("pin") == raw.get("pin"))
            .ok_or_else(|| refusal("Saved anchor identity has changed"))?;
        match edit.number {
            Value::String(_) => {
                row.insert("number".into(), edit.number);
            }
            Value::Null => {
                row.remove("number");
            }
            _ => {
                return Err(refusal(
                    "A schematic pin designator must be a string or clear intent",
                ));
            }
        }
    }
    Ok(())
}
