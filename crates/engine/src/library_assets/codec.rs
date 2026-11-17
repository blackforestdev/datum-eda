use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use super::{exact_json, invalid};
use crate::{error::EngineError, ir::serialization::to_json_deterministic};

/// Produce the V1 metadata profile, sharing the engine's deterministic serializer.
/// Payload blobs use their original bytes instead; this is not a payload rewriter.
pub(crate) fn encode_metadata<T: Serialize>(record: &T) -> Result<Vec<u8>, EngineError> {
    let value = serde_json::to_value(record)?;
    validate_integers(&value)?;
    let mut bytes = to_json_deterministic(&value)?.into_bytes();
    bytes.push(b'\n');
    Ok(bytes)
}

/// Refuse ambiguous JSON and noncanonical encodings before typed admission.
/// The caller provides the transport's declared byte limit; no hidden load budget.
pub(crate) fn decode_metadata<T: DeserializeOwned + Serialize>(
    bytes: &[u8],
    byte_limit: usize,
) -> Result<T, EngineError> {
    if bytes.len() > byte_limit {
        return Err(invalid(
            "metadata exceeds the declared transport byte limit",
        ));
    }
    let parsed = exact_json::parse(bytes)?;
    if parsed.has_fractional_token() {
        return Err(invalid("V1 metadata permits integer tokens only"));
    }
    let record: T = serde_json::from_slice(bytes)?;
    if encode_metadata(&record)? != bytes {
        return Err(invalid(
            "metadata is not the canonical V1 byte representation",
        ));
    }
    Ok(record)
}

fn validate_integers(value: &Value) -> Result<(), EngineError> {
    match value {
        Value::Number(number) if number.is_f64() => {
            Err(invalid("V1 metadata cannot contain fractional numbers"))
        }
        Value::Array(values) => values.iter().try_for_each(validate_integers),
        Value::Object(fields) => fields.values().try_for_each(validate_integers),
        _ => Ok(()),
    }
}
