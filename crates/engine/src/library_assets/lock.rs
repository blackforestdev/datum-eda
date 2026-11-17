//! Authored lock data; creation here does not adopt a profile or write a project.
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{
    AssetClosure, AssetDigest, AssetKind, AssetRevisionRef, RecordDomain, encode_metadata, invalid,
    records::PROFILE, validation,
};
use crate::error::EngineError;

pub(crate) const LOCK_RELATIVE_PATH: &str = "library/lock.json";
pub(crate) const ADOPTED_PROJECT_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ComponentBinding {
    pub(crate) component_instance_id: Uuid,
    pub(crate) source_path: String,
    pub(crate) instance_path: Vec<Uuid>,
    pub(crate) component: AssetRevisionRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LockedAsset {
    pub(crate) asset: AssetRevisionRef,
    pub(crate) local_object_id: Uuid,
    pub(crate) source_path: String,
    pub(crate) payload_ref: AssetDigest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectLibraryLock {
    pub(crate) schema_version: u32,
    pub(crate) profile: String,
    pub(crate) bindings: Vec<ComponentBinding>,
    pub(crate) assets: Vec<LockedAsset>,
    pub(crate) source_manifests: Vec<AssetDigest>,
    pub(crate) closure_digest: AssetDigest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) extensions: Option<BTreeMap<String, Value>>,
}

impl ProjectLibraryLock {
    /// Zero managed assets, not a claim that existing native content was migrated.
    pub(crate) fn empty() -> Result<Self, EngineError> {
        let empty: Vec<AssetRevisionRef> = vec![];
        Ok(Self {
            schema_version: 1,
            profile: PROFILE.into(),
            bindings: vec![],
            assets: vec![],
            source_manifests: vec![],
            closure_digest: AssetDigest::record(RecordDomain::Closure, &encode_metadata(&empty)?),
            extensions: None,
        })
    }

    pub(crate) fn validate(&self) -> Result<(), EngineError> {
        if self.schema_version != 1 || self.profile != PROFILE {
            return Err(invalid("unsupported project library lock profile/schema"));
        }
        validation::extensions(self.extensions.as_ref())?;
        if self
            .bindings
            .windows(2)
            .any(|pair| binding_key(&pair[0]) >= binding_key(&pair[1]))
            || self
                .assets
                .windows(2)
                .any(|pair| pair[0].asset >= pair[1].asset)
            || self
                .source_manifests
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(invalid("lock entries must be unique and sorted"));
        }
        let mut ids = BTreeSet::new();
        let mut materializations = BTreeSet::new();
        for asset in &self.assets {
            source_path(&asset.source_path)?;
            if !materializations.insert(asset.source_path.as_str()) {
                return Err(invalid(
                    "multiple assets share one materialization source path",
                ));
            }
            if asset.local_object_id != asset.asset.asset_id || !ids.insert(asset.asset.asset_id) {
                return Err(invalid(
                    "lock contains an alias or conflicting active asset revisions",
                ));
            }
        }
        let refs: BTreeSet<_> = self.assets.iter().map(|a| &a.asset).collect();
        for binding in &self.bindings {
            source_path(&binding.source_path)?;
            if !refs.contains(&binding.component) {
                return Err(invalid(
                    "component binding is absent from the locked asset set",
                ));
            }
        }
        let refs: Vec<_> = self.assets.iter().map(|a| &a.asset).collect();
        if self.closure_digest
            != AssetDigest::record(RecordDomain::Closure, &encode_metadata(&refs)?)
        {
            return Err(invalid(
                "lock closure fingerprint differs from its exact references",
            ));
        }
        Ok(())
    }

    pub(crate) fn verify_closure(&self, closure: &AssetClosure<'_>) -> Result<(), EngineError> {
        self.validate()?;
        if &self.closure_digest != closure.digest() || self.assets.len() != closure.assets().len() {
            return Err(invalid("lock and verified dependency closure differ"));
        }
        for asset in &self.assets {
            let verified = closure
                .assets()
                .get(&asset.asset)
                .ok_or_else(|| invalid("locked asset revision unavailable"))?;
            if asset.payload_ref != verified.revision().payload_ref {
                return Err(invalid(
                    "locked payload differs from the verified asset revision",
                ));
            }
        }
        for binding in &self.bindings {
            let component = closure
                .assets()
                .get(&binding.component)
                .ok_or_else(|| invalid("locked component revision unavailable"))?;
            if component.revision().kind != AssetKind::Part {
                return Err(invalid("managed component binding must reference a Part"));
            }
        }
        Ok(())
    }
}

fn binding_key(binding: &ComponentBinding) -> (&str, &[Uuid], Uuid) {
    (
        &binding.source_path,
        &binding.instance_path,
        binding.component_instance_id,
    )
}

fn source_path(path: &str) -> Result<(), EngineError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.chars().any(char::is_control)
        || path.as_bytes().get(1) == Some(&b':')
        || path
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(invalid(
            "lock source path must be normalized and project-relative",
        ));
    }
    Ok(())
}
