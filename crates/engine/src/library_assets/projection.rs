//! Exact portable payload checks; local revision guards remain engine-owned.
use serde_json::Value;

use super::{
    AssetKind, AssetRevision,
    exact_json::{self, ExactValue},
    invalid,
};
use crate::{error::EngineError, pool};

/// Capture refuses a lossy parsed view before classifying any root counters.
pub(crate) fn verify_native_snapshot(raw: &[u8], local: &Value) -> Result<(), EngineError> {
    if exact_json::parse(raw)?.value != exact_json::parse(&serde_json::to_vec(local)?)?.value {
        return Err(invalid(
            "native source snapshot loses exact authored values",
        ));
    }
    Ok(())
}

/// Only the owning native schema may classify these names as local bookkeeping.
/// There is deliberately no arbitrary key-prefix or unknown-field exclusion.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct LocalCounters {
    pub(crate) object_revision: bool,
    pub(crate) model_revision: bool,
}

pub(crate) struct VerifiedPayload {
    value: Value,
    exact: ExactValue,
    raw: Box<[u8]>,
}

impl VerifiedPayload {
    pub(crate) fn value(&self) -> &Value {
        &self.value
    }

    /// Original acquired bytes, retained only after the existing transport
    /// bound, fingerprint, exact syntax, identity and native shape checks.
    pub(crate) fn raw_bytes(&self) -> &[u8] {
        &self.raw
    }

    /// Exact comparison for an internal owning-schema view. Exclusions are
    /// static root-field classifications supplied by that owner, never wire
    /// input. All nested and unknown values remain in the engineering view.
    pub(crate) fn matches_with_descriptive_fields(
        &self,
        native: &Value,
        counters: LocalCounters,
        descriptive: &'static [&'static str],
    ) -> Result<bool, EngineError> {
        let mut acquired = self.exact.clone();
        let mut current = exact_json::parse(&serde_json::to_vec(native)?)?.value;
        for value in [&mut acquired, &mut current] {
            let ExactValue::Object(fields) = value else {
                return Err(invalid("physical engineering projection must be an object"));
            };
            for (key, classified) in counter_fields(counters) {
                if classified {
                    fields.remove(key);
                }
            }
            for key in descriptive {
                fields.remove(*key);
            }
        }
        Ok(acquired == current)
    }

    pub(crate) fn read(
        revision: &AssetRevision,
        bytes: &[u8],
        byte_limit: usize,
        counters: LocalCounters,
    ) -> Result<Self, EngineError> {
        revision.validate()?;
        if bytes.len() > byte_limit {
            return Err(invalid("payload exceeds the declared transport byte limit"));
        }
        revision.payload_ref.verify_blob(bytes)?;
        let exact = exact_json::parse(bytes)?.value;
        let value: Value = serde_json::from_slice(bytes)?;
        if value.get("schema_version").and_then(Value::as_u64) != Some(1)
            || value
                .get("uuid")
                .and_then(Value::as_str)
                .and_then(|id| id.parse().ok())
                != Some(revision.asset_id)
        {
            return Err(invalid("payload schema/identity differs from the revision"));
        }
        for (key, classified) in counter_fields(counters) {
            if classified && value.get(key).is_some() {
                return Err(invalid(format!(
                    "portable payload contains local counter {key}"
                )));
            }
        }
        validate_shape(revision.kind, &value)?;
        Ok(Self {
            value,
            exact,
            raw: bytes.into(),
        })
    }

    /// Never hash materialized/normalized bytes as the original acquired blob.
    pub(crate) fn verify_materialization(
        &self,
        local: &Value,
        counters: LocalCounters,
    ) -> Result<(), EngineError> {
        let mut exact = exact_json::parse(&serde_json::to_vec(local)?)?.value;
        let ExactValue::Object(fields) = &mut exact else {
            return Err(invalid("materialized library object must be an object"));
        };
        for (key, classified) in counter_fields(counters) {
            if classified {
                fields.remove(key);
            }
        }
        if exact != self.exact {
            return Err(invalid(
                "materialization differs from the acquired engineering projection",
            ));
        }
        Ok(())
    }
}

fn counter_fields(policy: LocalCounters) -> [(&'static str, bool); 2] {
    [
        ("object_revision", policy.object_revision),
        ("model_revision", policy.model_revision),
    ]
}

fn validate_shape(kind: AssetKind, value: &Value) -> Result<(), EngineError> {
    macro_rules! admit {
        ($ty:ty) => {
            serde_json::from_value::<$ty>(value.clone())
                .map(|_| ())
                .map_err(EngineError::from)
        };
    }
    // Shape admission only; keep raw/exact content instead of reserializing these
    // structs, which intentionally do not own every preserved extension field.
    match kind {
        AssetKind::Unit => admit!(pool::Unit),
        AssetKind::Symbol => admit!(pool::Symbol),
        AssetKind::Entity => admit!(pool::Entity),
        AssetKind::Package => admit!(pool::Package),
        AssetKind::Padstack => admit!(pool::Padstack),
        AssetKind::Footprint => admit!(pool::Footprint),
        AssetKind::Part => admit!(pool::Part),
        AssetKind::PinPadMap => admit!(pool::PinPadMap),
        AssetKind::ModelAttachment => admit!(pool::ModelAttachment),
    }
}
