use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{AssetDigest, AssetRevisionRef, RecordDomain, encode_metadata, invalid};
use crate::error::EngineError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Evidence {
    pub(crate) title: String,
    pub(crate) issuer: Option<String>,
    pub(crate) document_revision: Option<String>,
    pub(crate) locator: Option<String>,
    pub(crate) retained_blob: Option<AssetDigest>,
    pub(crate) access_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Actor {
    pub(crate) recorded_id: Option<String>,
    pub(crate) display_name: Option<String>,
    pub(crate) verification_evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rights {
    pub(crate) license_expression: Option<String>,
    pub(crate) attribution: Vec<String>,
    pub(crate) restrictions: Vec<String>,
    pub(crate) evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OriginKind {
    Native,
    Import,
    Derived,
    Migration,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AssetProvenance {
    pub(crate) schema_version: u32,
    pub(crate) authors: Vec<Actor>,
    pub(crate) created_at: Option<String>,
    pub(crate) change_actor: Option<Actor>,
    pub(crate) changed_at: Option<String>,
    pub(crate) change_summary: Option<String>,
    pub(crate) origin_kind: OriginKind,
    pub(crate) source_assets: Vec<AssetRevisionRef>,
    pub(crate) sources: Vec<Evidence>,
    pub(crate) rights: Rights,
    pub(crate) evidence: Vec<Evidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) extensions: Option<BTreeMap<String, Value>>,
}

impl AssetProvenance {
    pub(crate) fn fingerprint(&self) -> Result<AssetDigest, EngineError> {
        if self.schema_version != 1 {
            return Err(invalid("unsupported provenance schema version"));
        }
        super::validation::extensions(self.extensions.as_ref())?;
        super::validation::timestamp(self.created_at.as_deref())?;
        super::validation::timestamp(self.changed_at.as_deref())?;
        // Unknown dates/authors remain null/empty; no clock or identity inference.
        Ok(AssetDigest::record(
            RecordDomain::Provenance,
            &encode_metadata(self)?,
        ))
    }
}
