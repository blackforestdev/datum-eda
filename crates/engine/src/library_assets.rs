//! Portable engineering records over the existing native library graph (PM066).
//!
//! This module validates immutable interchange data. It never writes a project,
//! grants approval or replaces the canonical resolver and journal.

pub(crate) mod change_review;
mod closure;
mod codec;
mod digest;
mod exact_json;
pub(crate) mod lock;
pub(crate) mod manifest;
mod projection;
mod provenance;
mod records;
pub(crate) mod review;
mod slots;
mod validation;

pub(crate) use closure::{AssetClosure, ClosureLimits, VerifiedAsset, inspect_closure};
pub(crate) use codec::{decode_metadata, encode_metadata};
pub(crate) use digest::{AssetDigest, RecordDomain};
pub(crate) use projection::{LocalCounters, VerifiedPayload, verify_native_snapshot};
pub(crate) use provenance::{Actor, AssetProvenance, Evidence, OriginKind, Rights};
pub(crate) use records::{AssetKind, AssetRevision, AssetRevisionRef, Dependency, PROFILE};
pub(crate) use slots::slots as payload_reference_slots;

use crate::error::EngineError;

fn invalid(message: impl Into<String>) -> EngineError {
    EngineError::Validation(format!("library asset: {}", message.into()))
}

#[cfg(test)]
mod closure_tests;
#[cfg(test)]
mod tests;
