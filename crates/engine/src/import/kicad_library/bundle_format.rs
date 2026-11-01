//! Native, versioned interchange between the maintenance exporter and shipped reader.
use crate::pool::{Symbol, Unit};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleManifest {
    pub schema_version: u32,
    pub source_format: String,
    pub source_version: String,
    pub converter_version: String,
    pub converter_profile: String,
    pub complete_inventory: bool,
    pub complete_conversion: bool,
    pub converted: u64,
    pub refused: u64,
    pub unreadable_libraries: u64,
    pub payload_total_bytes: u64,
    pub report_only: bool,
    pub libraries: Vec<BundleLibrary>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleLibrary {
    pub source: PathBuf,
    pub source_hash: Option<String>,
    pub format_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
    pub symbols: Vec<BundleEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleEntry {
    pub name: String,
    pub description: Option<String>,
    #[serde(flatten)]
    pub outcome: BundleEntryOutcome,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum BundleEntryOutcome {
    Converted {
        unit_id: Uuid,
        symbol_id: Uuid,
        offset: u64,
        payload_bytes: u64,
        payload_hash: String,
        warnings: Vec<String>,
    },
    Refused {
        reason: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleSource {
    pub library: PathBuf,
    pub symbol: String,
    pub hash: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSymbolRecord {
    pub schema_version: u32,
    pub unit: Unit,
    pub symbol: Symbol,
    pub source: BundleSource,
    pub notices: Vec<String>,
}
