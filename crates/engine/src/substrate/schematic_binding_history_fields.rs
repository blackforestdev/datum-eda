//! Exact recorded single-Symbol property edits, preserving every unrelated raw member.
use serde_json::Value;
use std::collections::BTreeSet;
use uuid::Uuid;

pub(super) fn symbol_fields(old: &Value, next: &Value) -> bool {
    let mut scalar = old.clone();
    for key in ["reference", "value"] {
        if next.get(key).is_none_or(|value| !value.is_string()) {
            return false;
        }
        scalar[key] = next[key].clone();
    }
    if scalar == *next {
        return true;
    }
    let (Some(before), Some(after)) = (old["fields"].as_array(), next["fields"].as_array()) else {
        return false;
    };
    if !unique_fields(before) || !unique_fields(after) {
        return false;
    }
    let mut allowed = old.clone();
    allowed["fields"] = next["fields"].clone();
    if allowed != *next {
        return false;
    }
    let first = before
        .iter()
        .zip(after)
        .position(|(a, b)| a != b)
        .unwrap_or(before.len().min(after.len()));
    match before.len().cmp(&after.len()) {
        std::cmp::Ordering::Equal => {
            first < before.len()
                && before[first + 1..] == after[first + 1..]
                && field_edit(&before[first], &after[first])
        }
        std::cmp::Ordering::Less => {
            after.len() == before.len() + 1 && before[first..] == after[first + 1..]
        }
        std::cmp::Ordering::Greater => {
            before.len() == after.len() + 1 && before[first + 1..] == after[first..]
        }
    }
}
fn unique_fields(fields: &[Value]) -> bool {
    let mut ids = BTreeSet::new();
    fields.iter().all(|value| {
        serde_json::from_value::<crate::schematic::SymbolField>(value.clone())
            .is_ok_and(|field| ids.insert(field.uuid))
    })
}
fn copy_property(old: &mut Value, next: &Value, key: &str) -> Option<()> {
    if let Some(value) = next.get(key) {
        old[key] = value.clone();
    } else {
        old.as_object_mut()?.remove(key);
    }
    Some(())
}
fn field_edit(old: &Value, next: &Value) -> bool {
    let check = || -> Option<Value> {
        let old_id: Uuid = serde_json::from_value(old["uuid"].clone()).ok()?;
        let next_id: Uuid = serde_json::from_value(next["uuid"].clone()).ok()?;
        if old_id != next_id {
            return None;
        }
        let mut allowed = old.clone();
        for key in ["key", "value", "visible"] {
            copy_property(&mut allowed, next, key)?;
        }
        let a = old.get("position");
        let b = next.get("position");
        match (a.and_then(Value::as_object), b.and_then(Value::as_object)) {
            (Some(_), Some(_)) => {
                next["position"]["x"].as_i64()?;
                next["position"]["y"].as_i64()?;
                allowed["position"]["x"] = next["position"]["x"].clone();
                allowed["position"]["y"] = next["position"]["y"].clone();
            }
            (Some(point), None) | (None, Some(point)) => {
                // The writer introduces an unextended x/y point from None. Its
                // inverse may restore None; an existing extended point cannot vanish.
                if point.len() != 2 || !point.contains_key("x") || !point.contains_key("y") {
                    return None;
                }
                copy_property(&mut allowed, next, "position")?;
            }
            (None, None) if a == b => {}
            _ => return None,
        }
        Some(allowed)
    };
    check().as_ref() == Some(next)
}
