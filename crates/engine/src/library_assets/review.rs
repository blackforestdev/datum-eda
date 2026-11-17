//! Revision-specific evidence. A review result never grants engineering approval.
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{
    Actor, AssetDigest, AssetRevisionRef, Evidence, RecordDomain, encode_metadata, invalid,
    validation,
};
use crate::error::EngineError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReviewResult {
    Passed,
    Failed,
    NeedsReview,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AssetReview {
    pub(crate) schema_version: u32,
    pub(crate) review_id: Uuid,
    pub(crate) subject: AssetRevisionRef,
    pub(crate) closure_digest: AssetDigest,
    pub(crate) reviewer: Actor,
    pub(crate) evidence: Vec<Evidence>,
    pub(crate) rule_profile_refs: Vec<AssetDigest>,
    pub(crate) result: ReviewResult,
    pub(crate) recorded_at: Option<String>,
    pub(crate) supersedes: Vec<AssetDigest>,
    pub(crate) disposition_refs: Vec<AssetDigest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) extensions: Option<BTreeMap<String, Value>>,
}

impl AssetReview {
    pub(crate) fn validate(&self) -> Result<(), EngineError> {
        if self.schema_version != 1 {
            return Err(invalid("unsupported review schema"));
        }
        validation::extensions(self.extensions.as_ref())?;
        validation::timestamp(self.recorded_at.as_deref())?;
        for references in [
            &self.rule_profile_refs,
            &self.supersedes,
            &self.disposition_refs,
        ] {
            if references.iter().collect::<BTreeSet<_>>().len() != references.len() {
                return Err(invalid("duplicate review evidence/disposition reference"));
            }
        }
        Ok(())
    }

    pub(crate) fn fingerprint(&self) -> Result<AssetDigest, EngineError> {
        self.validate()?;
        Ok(AssetDigest::record(
            RecordDomain::Review,
            &encode_metadata(self)?,
        ))
    }

    /// Currency alone does not make a failed, claimed or unverified review approved.
    pub(crate) fn verify_basis(
        &self,
        subject: &AssetRevisionRef,
        closure: &AssetDigest,
    ) -> Result<(), EngineError> {
        self.validate()?;
        if &self.subject != subject || &self.closure_digest != closure {
            return Err(invalid("review subject/dependency basis is stale"));
        }
        Ok(())
    }
}
