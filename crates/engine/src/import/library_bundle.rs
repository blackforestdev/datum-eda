//! Shipped native symbol records. Runtime never parses a foreign source library.
use std::{borrow::Cow, path::Path, sync::OnceLock};

use super::{
    ids_sidecar::compute_source_hash_bytes,
    kicad_library::{
        ConvertedSymbol,
        bundle_format::{
            BundleEntry, BundleEntryOutcome, BundleLibrary, BundleManifest, NativeSymbolRecord,
        },
    },
};
use crate::error::EngineError;

const INDEX: &[u8] = include_bytes!("../../assets/library/kicad/catalog.json");
const SYMBOLS: &[&[u8]] = &[
    include_bytes!("../../assets/library/kicad/symbols-00.jsonl.part"),
    include_bytes!("../../assets/library/kicad/symbols-01.jsonl.part"),
    include_bytes!("../../assets/library/kicad/symbols-02.jsonl.part"),
    include_bytes!("../../assets/library/kicad/symbols-03.jsonl.part"),
];
pub const COPYRIGHT: &str = include_str!("../../assets/library/kicad/COPYRIGHT.txt");
static CATALOG: OnceLock<Result<BundleManifest, String>> = OnceLock::new();

fn refusal(reason: impl std::fmt::Display) -> EngineError {
    EngineError::Validation(format!("Bundled symbol library: {reason}"))
}

/// Only the lightweight inventory is decoded eagerly; native objects are read
/// from their indexed byte range when a designer selects a symbol.
pub fn catalog() -> Result<&'static BundleManifest, EngineError> {
    CATALOG
        .get_or_init(|| {
            let catalog: BundleManifest =
                serde_json::from_slice(INDEX).map_err(|e| e.to_string())?;
            if catalog.schema_version != 1
                || catalog.source_format != "kicad_symbol"
                || catalog.report_only
                || catalog.payload_total_bytes
                    != SYMBOLS.iter().map(|part| part.len() as u64).sum::<u64>()
            {
                return Err("Unsupported or incomplete native library package".into());
            }
            Ok(catalog)
        })
        .as_ref()
        .map_err(refusal)
}

/// Reads an embedded record by its logical upstream library/name, preserving
/// stable identities across installation paths and bundle updates.
pub fn preview(source: &Path, name: &str) -> Result<ConvertedSymbol, EngineError> {
    let catalog = catalog()?;
    let library = catalog
        .libraries
        .iter()
        .find(|library| library.source == source)
        .ok_or_else(|| refusal("Unknown bundled source library"))?;
    if let Some(reason) = &library.failure {
        return Err(refusal(reason));
    }
    let entry = library
        .symbols
        .iter()
        .find(|entry| entry.name == name)
        .ok_or_else(|| refusal("Unknown bundled symbol"))?;
    decode_record(catalog, library, entry, SYMBOLS)
}

fn decode_record(
    catalog: &BundleManifest,
    library: &BundleLibrary,
    entry: &BundleEntry,
    parts: &[&[u8]],
) -> Result<ConvertedSymbol, EngineError> {
    let (unit_id, symbol_id, offset, length, digest) = match &entry.outcome {
        BundleEntryOutcome::Converted {
            unit_id,
            symbol_id,
            offset,
            payload_bytes,
            payload_hash,
            ..
        } => (*unit_id, *symbol_id, *offset, *payload_bytes, payload_hash),
        BundleEntryOutcome::Refused { reason } => return Err(refusal(reason)),
    };
    let start = usize::try_from(offset).map_err(refusal)?;
    let end = start
        .checked_add(usize::try_from(length).map_err(refusal)?)
        .ok_or_else(|| refusal("Native symbol byte range overflows"))?;
    let payload = record_bytes(parts, start, end)?;
    if compute_source_hash_bytes(&payload) != *digest {
        return Err(refusal(
            "Native symbol payload does not match the catalog digest",
        ));
    }
    let record: NativeSymbolRecord = serde_json::from_slice(&payload)?;
    if record.schema_version != 1
        || record.unit.uuid != unit_id
        || record.symbol.uuid != symbol_id
        || record.symbol.unit != record.unit.uuid
        || record.symbol.name != entry.name
        || record.source.library != library.source
        || record.source.symbol != entry.name
        || library.source_hash.as_deref() != Some(record.source.hash.as_str())
        || record.source.version != catalog.source_version
    {
        return Err(refusal(
            "Native symbol identity or provenance differs from its catalog entry",
        ));
    }
    Ok(ConvertedSymbol {
        unit: record.unit,
        symbol: record.symbol,
        warnings: record.notices,
        source_hash: record.source.hash,
        source_path: record.source.library,
        source_symbol: record.source.symbol,
    })
}

/// Preserve the global index offsets without assembling the whole package.
/// Almost every selection borrows one chunk; only a crossing record is copied.
fn record_bytes<'a>(
    parts: &[&'a [u8]],
    start: usize,
    end: usize,
) -> Result<Cow<'a, [u8]>, EngineError> {
    let total = parts
        .iter()
        .try_fold(0usize, |sum, part| sum.checked_add(part.len()))
        .ok_or_else(|| refusal("Native package byte length overflows"))?;
    if start > end || end > total {
        return Err(refusal("Native symbol byte range is unavailable"));
    }
    let mut base = 0;
    for part in parts {
        let limit = base + part.len();
        if start >= base && end <= limit {
            return Ok(Cow::Borrowed(&part[start - base..end - base]));
        }
        base = limit;
    }
    let mut payload = Vec::with_capacity(end - start);
    base = 0;
    for part in parts {
        let limit = base + part.len();
        let left = start.max(base);
        let right = end.min(limit);
        if left < right {
            payload.extend_from_slice(&part[left - base..right - base]);
        }
        base = limit;
    }
    Ok(Cow::Owned(payload))
}

#[cfg(test)]
#[path = "library_bundle_tests.rs"]
mod tests;
