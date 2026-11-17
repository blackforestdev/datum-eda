use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{AssetDigest, RecordDomain, encode_metadata, invalid};
use crate::error::EngineError;

pub(crate) const PROFILE: &str = "datum-library-json-v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AssetRevisionRef {
    pub(crate) asset_id: Uuid,
    pub(crate) revision_id: AssetDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AssetKind {
    Unit,
    Symbol,
    Entity,
    Package,
    Padstack,
    Footprint,
    Part,
    PinPadMap,
    /// Existing owned detail content, not a ninth Pool kind.
    ModelAttachment,
}

impl AssetKind {
    pub(crate) fn pool_kind(self) -> Option<&'static str> {
        match self {
            Self::Unit => Some("units"),
            Self::Symbol => Some("symbols"),
            Self::Entity => Some("entities"),
            Self::Package => Some("packages"),
            Self::Padstack => Some("padstacks"),
            Self::Footprint => Some("footprints"),
            Self::Part => Some("parts"),
            Self::PinPadMap => Some("pin_pad_maps"),
            Self::ModelAttachment => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Dependency {
    pub(crate) role: String,
    pub(crate) payload_path: Vec<String>,
    pub(crate) target: AssetRevisionRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AssetRevision {
    pub(crate) schema_version: u32,
    pub(crate) asset_id: Uuid,
    pub(crate) kind: AssetKind,
    pub(crate) payload_schema_version: u32,
    pub(crate) payload_ref: AssetDigest,
    pub(crate) parents: Vec<AssetRevisionRef>,
    pub(crate) dependencies: Vec<Dependency>,
    pub(crate) provenance_ref: AssetDigest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) extensions: Option<BTreeMap<String, Value>>,
}

impl AssetRevision {
    pub(crate) fn reference(&self) -> Result<AssetRevisionRef, EngineError> {
        self.validate()?;
        Ok(AssetRevisionRef {
            asset_id: self.asset_id,
            revision_id: AssetDigest::record(RecordDomain::AssetRevision, &encode_metadata(self)?),
        })
    }

    pub(crate) fn validate(&self) -> Result<(), EngineError> {
        if self.schema_version != 1 || self.payload_schema_version != 1 {
            return Err(invalid("unsupported asset or payload schema version"));
        }
        super::validation::extensions(self.extensions.as_ref())?;
        let mut seen = std::collections::BTreeSet::new();
        for parent in &self.parents {
            if parent.asset_id != self.asset_id || !seen.insert(parent) {
                return Err(invalid("parent must be a distinct revision of this asset"));
            }
        }
        if self.dependencies.windows(2).any(|pair| {
            (&pair[0].payload_path, &pair[0].role) >= (&pair[1].payload_path, &pair[1].role)
                || pair[0].payload_path == pair[1].payload_path
        }) || self
            .dependencies
            .iter()
            .any(|d| d.role.is_empty() || d.payload_path.is_empty())
        {
            return Err(invalid(
                "dependency slots must be nonempty, unique and sorted",
            ));
        }
        Ok(())
    }
}
