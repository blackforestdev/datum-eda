use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Map, Value};

/// Deterministic serialization utilities.
/// See docs/CANONICAL_IR.md §5 and specs/ENGINE_SPEC.md §4.
///
/// Contract: same authored data -> byte-identical JSON on every run, every platform.
/// This normalizes all JSON object keys recursively before emission so callers
/// do not accidentally inherit `HashMap` iteration order in serialized output.
fn canonicalize_json(value: Value) -> Value {
    match value {
        Value::Object(obj) => {
            let sorted: BTreeMap<String, Value> = obj
                .into_iter()
                .map(|(k, v)| (k, canonicalize_json(v)))
                .collect();

            let mut normalized = Map::new();
            for (k, v) in sorted {
                normalized.insert(k, v);
            }
            Value::Object(normalized)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(canonicalize_json).collect()),
        other => other,
    }
}

/// Serialize a value to canonical JSON with recursively sorted object keys.
pub fn to_json_deterministic<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let normalized = canonicalize_json(serde_json::to_value(value)?);
    serde_json::to_string(&normalized)
}

/// Serialize to canonical UTF-8 bytes for hashing or golden-file comparison.
pub fn to_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    Ok(to_json_deterministic(value)?.into_bytes())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::Serialize;

    use super::*;

    #[derive(Serialize)]
    struct Wrapper {
        data: HashMap<String, i32>,
    }

    #[test]
    fn sorts_object_keys_recursively() {
        let mut nested = HashMap::new();
        nested.insert("z".to_string(), 1);
        nested.insert("a".to_string(), 2);

        let json = to_json_deterministic(&Wrapper { data: nested }).unwrap();
        assert_eq!(json, r#"{"data":{"a":2,"z":1}}"#);
    }
}
