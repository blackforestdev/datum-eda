use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    AuthorityDiagnostic, AuthorityRecord, AuthoritySnapshot, ConfigurationItemId,
    EngineeringChangeId, RevisionReservationId, RevisionSchemeId,
};

pub const REVISION_RESERVATION_STANDINGS: &[RevisionReservationStanding] = &[
    RevisionReservationStanding::Active,
    RevisionReservationStanding::Released,
    RevisionReservationStanding::Expired,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionReservationStanding {
    Active,
    Released,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionReservationData {
    pub reservation_key: uuid::Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<RevisionReservationId>,
    pub configuration_item: ConfigurationItemId,
    pub scheme_id: RevisionSchemeId,
    pub scheme_version: u64,
    pub proposed_label: String,
    pub governing_change: EngineeringChangeId,
    pub expires_at: i64,
    pub standing: RevisionReservationStanding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRevisionReservation {
    pub record_id: RevisionReservationId,
    pub data: RevisionReservationData,
}

pub fn resolve_revision_reservation(
    snapshot: &AuthoritySnapshot,
    id: RevisionReservationId,
) -> Option<ResolvedRevisionReservation> {
    let first = snapshot.records.iter().find_map(|record| match record {
        AuthorityRecord::RevisionReservation(body) if body.id == id => Some(body),
        _ => None,
    })?;
    let key = first.semantics.reservation_key;
    let superseded: BTreeSet<_> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::RevisionReservation(body) if body.semantics.reservation_key == key => {
                body.semantics.supersedes
            }
            _ => None,
        })
        .collect();
    snapshot.records.iter().find_map(|record| match record {
        AuthorityRecord::RevisionReservation(body)
            if body.semantics.reservation_key == key && !superseded.contains(&body.id) =>
        {
            Some(ResolvedRevisionReservation {
                record_id: body.id,
                data: body.semantics.clone(),
            })
        }
        _ => None,
    })
}

pub fn evaluate_reservation_availability(
    snapshot: &AuthoritySnapshot,
    configuration_item: ConfigurationItemId,
    scheme_id: RevisionSchemeId,
    scheme_version: u64,
    proposed_label: &str,
) -> bool {
    !snapshot.records.iter().any(|record| match record {
        AuthorityRecord::RevisionReservation(body)
            if body.semantics.configuration_item == configuration_item
                && body.semantics.scheme_id == scheme_id
                && body.semantics.scheme_version == scheme_version
                && body.semantics.proposed_label == proposed_label =>
        {
            resolve_revision_reservation(snapshot, body.id).is_some_and(|resolved| {
                resolved.data.standing == RevisionReservationStanding::Active
            })
        }
        _ => false,
    })
}

pub(crate) fn validate_revision_reservation(
    data: &RevisionReservationData,
) -> Vec<AuthorityDiagnostic> {
    if data.scheme_version == 0 || data.proposed_label.trim().is_empty() {
        vec![AuthorityDiagnostic {
            code: "revision_reservation_invalid".to_string(),
            message: "reservation requires a positive scheme version and nonblank label"
                .to_string(),
        }]
    } else {
        Vec::new()
    }
}
