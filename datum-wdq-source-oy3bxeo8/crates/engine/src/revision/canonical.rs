use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{error::EngineError, ir::serialization::to_json_deterministic};

use super::model::AlgorithmQualifiedDigest;

pub(crate) fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, EngineError> {
    let mut bytes = to_json_deterministic(value)?.into_bytes();
    bytes.push(b'\n');
    Ok(bytes)
}

pub(crate) fn digest_bytes(bytes: &[u8]) -> AlgorithmQualifiedDigest {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    AlgorithmQualifiedDigest(format!("sha256:{:x}", hasher.finalize()))
}

pub(crate) fn validate_digest(value: &AlgorithmQualifiedDigest) -> Result<(), EngineError> {
    let Some(hex) = value.0.strip_prefix("sha256:") else {
        return Err(EngineError::Validation(format!(
            "revision integrity digest `{}` is not algorithm-qualified sha256",
            value.0
        )));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(EngineError::Validation(format!(
            "revision integrity digest `{}` must contain 64 lowercase hexadecimal digits",
            value.0
        )));
    }
    Ok(())
}
