use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EngineError;

use super::{
    AuthorityRecordBody, AuthorityRef, BuildPresentationMode, NamespaceTransitionMode,
    ProjectRevisionPolicyData, ProjectRevisionPolicyId, ProjectSeedReceiptId, RevisionSchemeId,
    canonical::canonical_bytes, canonical::digest_bytes,
};

pub const REVISION_SEED_KEYS: &[&str] = &[
    "datum.revision.profile_seed",
    "datum.revision.build_presentation_seed",
    "datum.revision.prototype_transition_seed",
];

pub(crate) type ProjectRevisionSeedRecords = (
    AuthorityRecordBody<ProjectRevisionPolicyId, ProjectRevisionPolicyData>,
    AuthorityRecordBody<ProjectSeedReceiptId, ProjectSeedReceiptData>,
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenRevisionSeedSnapshot {
    pub source_profile: String,
    pub source_generation: String,
    pub values: BTreeMap<String, String>,
}

impl FrozenRevisionSeedSnapshot {
    pub fn validate(&self) -> Result<(), EngineError> {
        let actual: BTreeSet<_> = self.values.keys().map(String::as_str).collect();
        let expected: BTreeSet<_> = REVISION_SEED_KEYS.iter().copied().collect();
        if actual != expected {
            return Err(EngineError::Validation(
                "revision seed snapshot must contain exactly the three registered keys".to_string(),
            ));
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<super::AlgorithmQualifiedDigest, EngineError> {
        Ok(digest_bytes(&canonical_bytes(self)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSeedReceiptItem {
    pub key: String,
    pub copied_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSeedReceiptData {
    pub source_profile: String,
    pub source_generation: String,
    pub source_digest: super::AlgorithmQualifiedDigest,
    pub copied_policy_id: ProjectRevisionPolicyId,
    pub items: Vec<ProjectSeedReceiptItem>,
}

pub(crate) fn build_seed_records(
    project_id: Uuid,
    policy_id: ProjectRevisionPolicyId,
    receipt_id: ProjectSeedReceiptId,
    scheme_id: RevisionSchemeId,
    mut policy: ProjectRevisionPolicyData,
    snapshot: &FrozenRevisionSeedSnapshot,
) -> Result<ProjectRevisionSeedRecords, EngineError> {
    snapshot.validate()?;
    policy.scheme_profile_selection.profile_name =
        snapshot.values["datum.revision.profile_seed"].clone();
    policy.build_presentation =
        match snapshot.values["datum.revision.build_presentation_seed"].as_str() {
            "quiet" => BuildPresentationMode::Quiet,
            "phase_build" => BuildPresentationMode::PhaseBuild,
            value => {
                return Err(EngineError::Validation(format!(
                    "unsupported revision build presentation seed: {value}"
                )));
            }
        };
    policy.namespace_transition =
        match snapshot.values["datum.revision.prototype_transition_seed"].as_str() {
            "continuous" => NamespaceTransitionMode::Continuous,
            "governed_production_identity" => NamespaceTransitionMode::GovernedProductionIdentity,
            value => {
                return Err(EngineError::Validation(format!(
                    "unsupported revision prototype transition seed: {value}"
                )));
            }
        };
    policy.scheme_profile_selection.scheme_id = scheme_id;
    let items = REVISION_SEED_KEYS
        .iter()
        .map(|key| ProjectSeedReceiptItem {
            key: (*key).to_string(),
            copied_value: snapshot.values[*key].clone(),
        })
        .collect();
    let policy_body = AuthorityRecordBody {
        schema_version: super::AUTHORITY_SCHEMA_VERSION,
        id: policy_id,
        project_id,
        display_name: Some("copied Project revision policy".to_string()),
        physical_locator: None,
        references: vec![AuthorityRef::RevisionScheme(scheme_id)],
        semantics: policy,
    };
    let receipt_body = AuthorityRecordBody {
        schema_version: super::AUTHORITY_SCHEMA_VERSION,
        id: receipt_id,
        project_id,
        display_name: Some("Project revision seed receipt".to_string()),
        physical_locator: None,
        references: vec![AuthorityRef::ProjectRevisionPolicy(policy_id)],
        semantics: ProjectSeedReceiptData {
            source_profile: snapshot.source_profile.clone(),
            source_generation: snapshot.source_generation.clone(),
            source_digest: snapshot.digest()?,
            copied_policy_id: policy_id,
            items,
        },
    };
    Ok((policy_body, receipt_body))
}

pub fn query_project_seed_receipt(
    snapshot: &super::AuthoritySnapshot,
) -> Option<ProjectSeedReceiptData> {
    snapshot.records.iter().find_map(|record| match record {
        super::AuthorityRecord::ProjectSeedReceipt(body) => Some(body.semantics.clone()),
        _ => None,
    })
}
