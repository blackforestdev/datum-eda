use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    AuthorityDiagnostic, AuthorityRecord, AuthorityRef, AuthoritySnapshot, EffectiveInterval,
    EffectivityId, LegacyRevisionFactMappingId, RoleScope,
};

pub const LEGACY_REVISION_FACT_KINDS: &[LegacyRevisionFactKind] = &[
    LegacyRevisionFactKind::ProposalAcceptance,
    LegacyRevisionFactKind::ReviewBooleanOrFreeFormActor,
    LegacyRevisionFactKind::CheckWaiver,
    LegacyRevisionFactKind::AcceptedCheckDeviation,
];

pub const LEGACY_MAPPING_DISPOSITIONS: &[LegacyMappingDisposition] = &[
    LegacyMappingDisposition::Mapped,
    LegacyMappingDisposition::RetainedAsLegacyEvidence,
    LegacyMappingDisposition::RejectedAsInsufficient,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyRevisionFactKind {
    ProposalAcceptance,
    ReviewBooleanOrFreeFormActor,
    CheckWaiver,
    AcceptedCheckDeviation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyMappingDisposition {
    Mapped,
    RetainedAsLegacyEvidence,
    RejectedAsInsufficient,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepartureAuthorityFields {
    pub source_fact_id: Uuid,
    pub source_fact_digest: super::AlgorithmQualifiedDigest,
    pub governing_requirement_or_finding: String,
    pub exact_scope: RoleScope,
    pub authorizing_authority: AuthorityRef,
    pub validity: EffectiveInterval,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effectivity: Option<EffectivityId>,
    pub rationale: String,
    pub disposition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaiverDepartureData {
    pub source_fact_id: Uuid,
    pub source_fact_digest: super::AlgorithmQualifiedDigest,
    pub governing_requirement_or_finding: String,
    pub exact_scope: RoleScope,
    pub authorizing_authority: AuthorityRef,
    pub validity: EffectiveInterval,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effectivity: Option<EffectivityId>,
    pub rationale: String,
    pub disposition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviationDepartureData {
    pub source_fact_id: Uuid,
    pub source_fact_digest: super::AlgorithmQualifiedDigest,
    pub governing_requirement_or_finding: String,
    pub exact_scope: RoleScope,
    pub authorizing_authority: AuthorityRef,
    pub validity: EffectiveInterval,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effectivity: Option<EffectivityId>,
    pub rationale: String,
    pub disposition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyRevisionFactMappingData {
    pub source_fact_id: Uuid,
    pub source_fact_digest: super::AlgorithmQualifiedDigest,
    pub source_kind: LegacyRevisionFactKind,
    pub disposition: LegacyMappingDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mapped_departure: Option<AuthorityRef>,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyRevisionFactAssessment {
    pub disposition: LegacyMappingDisposition,
    pub may_create_waiver: bool,
    pub may_create_deviation: bool,
}

pub fn assess_legacy_revision_fact(
    kind: LegacyRevisionFactKind,
    complete_departure_fields: bool,
) -> LegacyRevisionFactAssessment {
    match (kind, complete_departure_fields) {
        (LegacyRevisionFactKind::CheckWaiver, true) => LegacyRevisionFactAssessment {
            disposition: LegacyMappingDisposition::Mapped,
            may_create_waiver: true,
            may_create_deviation: false,
        },
        (LegacyRevisionFactKind::AcceptedCheckDeviation, true) => LegacyRevisionFactAssessment {
            disposition: LegacyMappingDisposition::Mapped,
            may_create_waiver: false,
            may_create_deviation: true,
        },
        (
            LegacyRevisionFactKind::ProposalAcceptance
            | LegacyRevisionFactKind::ReviewBooleanOrFreeFormActor,
            _,
        ) => LegacyRevisionFactAssessment {
            disposition: LegacyMappingDisposition::RetainedAsLegacyEvidence,
            may_create_waiver: false,
            may_create_deviation: false,
        },
        _ => LegacyRevisionFactAssessment {
            disposition: LegacyMappingDisposition::RejectedAsInsufficient,
            may_create_waiver: false,
            may_create_deviation: false,
        },
    }
}

pub fn query_legacy_revision_fact_mapping(
    snapshot: &AuthoritySnapshot,
    source_fact_id: Uuid,
) -> Option<(LegacyRevisionFactMappingId, LegacyRevisionFactMappingData)> {
    snapshot.records.iter().find_map(|record| match record {
        AuthorityRecord::LegacyRevisionFactMapping(body)
            if body.semantics.source_fact_id == source_fact_id =>
        {
            Some((body.id, body.semantics.clone()))
        }
        _ => None,
    })
}

pub(crate) fn validate_waiver_departure(data: &WaiverDepartureData) -> Vec<AuthorityDiagnostic> {
    validate_departure_fields(
        &data.governing_requirement_or_finding,
        &data.validity,
        &data.rationale,
        &data.disposition,
    )
}

pub(crate) fn validate_deviation_departure(
    data: &DeviationDepartureData,
) -> Vec<AuthorityDiagnostic> {
    validate_departure_fields(
        &data.governing_requirement_or_finding,
        &data.validity,
        &data.rationale,
        &data.disposition,
    )
}

fn validate_departure_fields(
    governing_requirement_or_finding: &str,
    validity: &EffectiveInterval,
    rationale: &str,
    disposition: &str,
) -> Vec<AuthorityDiagnostic> {
    if governing_requirement_or_finding.trim().is_empty()
        || rationale.trim().is_empty()
        || disposition.trim().is_empty()
        || validity
            .until_exclusive
            .is_some_and(|until| until <= validity.from_inclusive)
    {
        vec![diag(
            "departure_invalid_or_expired",
            "departure authority fields are incomplete or its validity interval is empty",
        )]
    } else {
        Vec::new()
    }
}

pub(crate) fn validate_legacy_mapping(
    snapshot: &AuthoritySnapshot,
    data: &LegacyRevisionFactMappingData,
) -> Vec<AuthorityDiagnostic> {
    let mapping_count = snapshot
        .records
        .iter()
        .filter(|record| {
            matches!(record, AuthorityRecord::LegacyRevisionFactMapping(body) if body.semantics.source_fact_id == data.source_fact_id)
        })
        .count();
    if mapping_count != 1 || data.rationale.trim().is_empty() {
        return vec![diag(
            "legacy_revision_fact_mapping_not_unique",
            "each assessed legacy fact requires exactly one append-only disposition",
        )];
    }
    let mapped_kind_ok = match (data.disposition, data.mapped_departure) {
        (LegacyMappingDisposition::Mapped, Some(AuthorityRef::WaiverDeparture(_))) => {
            data.source_kind == LegacyRevisionFactKind::CheckWaiver
        }
        (LegacyMappingDisposition::Mapped, Some(AuthorityRef::DeviationDeparture(_))) => {
            data.source_kind == LegacyRevisionFactKind::AcceptedCheckDeviation
        }
        (LegacyMappingDisposition::Mapped, _) => false,
        (_, None) => true,
        (_, Some(_)) => false,
    };
    if mapped_kind_ok {
        Vec::new()
    } else {
        vec![diag(
            "legacy_revision_fact_mapping_invalid",
            "legacy fact disposition cannot promote a non-equivalent source",
        )]
    }
}

fn diag(code: &str, message: &str) -> AuthorityDiagnostic {
    AuthorityDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}
