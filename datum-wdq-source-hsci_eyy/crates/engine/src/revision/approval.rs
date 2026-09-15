use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{
    ActorIdentityId, AlgorithmQualifiedDigest, ApprovalAttestationId, AuthorityDiagnostic,
    AuthorityRecord, AuthorityRef, AuthoritySnapshot, CapabilityEvaluation,
    ProjectRevisionPolicyData, ProjectRevisionPolicyId, RoleAssignmentId, RoleScope,
    evaluate_capability, query_effective_role_assignments,
};

pub const APPROVAL_INTENTS: &[ApprovalIntent] = &[
    ApprovalIntent::Approve,
    ApprovalIntent::Reject,
    ApprovalIntent::Acknowledge,
    ApprovalIntent::Verify,
    ApprovalIntent::Authorize,
    ApprovalIntent::Release,
];

pub const ATTESTATION_DISPOSITIONS: &[AttestationDisposition] = &[
    AttestationDisposition::Active,
    AttestationDisposition::Superseded,
    AttestationDisposition::Revoked,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalIntent {
    Approve,
    Reject,
    Acknowledge,
    Verify,
    Authorize,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationDisposition {
    Active,
    Superseded,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalAttestationData {
    pub intent: ApprovalIntent,
    pub disposition: AttestationDisposition,
    pub target: AuthorityRef,
    pub target_digest: AlgorithmQualifiedDigest,
    pub actor_id: ActorIdentityId,
    pub role_assignment_id: RoleAssignmentId,
    pub policy_id: ProjectRevisionPolicyId,
    pub policy_version: u64,
    pub method_class: String,
    pub asserted_at: i64,
    pub signature: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposes: Option<ApprovalAttestationId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttestationStanding {
    Active,
    Superseded { by: ApprovalAttestationId },
    Revoked { by: ApprovalAttestationId },
    Refused(AuthorityDiagnostic),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalEvaluationInput {
    pub target: AuthorityRef,
    pub target_digest: AlgorithmQualifiedDigest,
    pub policy_id: ProjectRevisionPolicyId,
    pub instant: i64,
    pub cryptographically_valid: BTreeSet<ApprovalAttestationId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalPolicyEvaluation {
    Satisfied {
        attestations: Vec<ApprovalAttestationId>,
    },
    Refused(AuthorityDiagnostic),
}

pub fn query_attestation_standing(
    snapshot: &AuthoritySnapshot,
    attestation_id: ApprovalAttestationId,
) -> AttestationStanding {
    let mut original_exists = false;
    let mut disposition = None;
    for record in &snapshot.records {
        let AuthorityRecord::ApprovalAttestation(body) = record else {
            continue;
        };
        if body.id == attestation_id {
            original_exists = true;
        }
        if body.semantics.disposes == Some(attestation_id) {
            disposition = Some((body.id, body.semantics.disposition));
        }
    }
    if !original_exists {
        return AttestationStanding::Refused(diag(
            "revision_attestation_missing",
            "attestation does not exist",
        ));
    }
    match disposition {
        Some((by, AttestationDisposition::Superseded)) => AttestationStanding::Superseded { by },
        Some((by, AttestationDisposition::Revoked)) => AttestationStanding::Revoked { by },
        Some((_, AttestationDisposition::Active)) => AttestationStanding::Refused(diag(
            "revision_attestation_invalid_disposition",
            "an active attestation cannot dispose another attestation",
        )),
        None => AttestationStanding::Active,
    }
}

pub fn evaluate_approval_policy(
    snapshot: &AuthoritySnapshot,
    policy: &ProjectRevisionPolicyData,
    input: &ApprovalEvaluationInput,
) -> ApprovalPolicyEvaluation {
    let attestations: Vec<_> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::ApprovalAttestation(body)
                if body.semantics.disposition == AttestationDisposition::Active
                    && body.semantics.target == input.target
                    && body.semantics.target_digest == input.target_digest
                    && body.semantics.policy_id == input.policy_id
                    && body.semantics.policy_version == policy.policy_version
                    && matches!(
                        query_attestation_standing(snapshot, body.id),
                        AttestationStanding::Active
                    ) =>
            {
                Some(body)
            }
            _ => None,
        })
        .collect();

    let mut accepted = Vec::new();
    let mut actors_by_intent: BTreeMap<ApprovalIntent, BTreeSet<ActorIdentityId>> = BTreeMap::new();
    let mut requirements = policy.approval_policy.requirements.clone();
    requirements.sort_by_key(|requirement| requirement.order);
    for requirement in requirements {
        let allowed_methods = policy
            .required_method_classes
            .by_intent
            .get(&requirement.intent);
        let mut matching = Vec::new();
        for body in attestations
            .iter()
            .copied()
            .filter(|body| body.semantics.intent == requirement.intent)
        {
            if !input.cryptographically_valid.contains(&body.id) {
                continue;
            }
            if allowed_methods
                .is_some_and(|methods| !methods.contains(&body.semantics.method_class))
            {
                continue;
            }
            let scope = RoleScope::Authority(input.target);
            if !matches!(
                evaluate_capability(
                    snapshot,
                    body.semantics.actor_id,
                    requirement.required_capability,
                    scope,
                    input.instant,
                ),
                CapabilityEvaluation::Authorized(_)
            ) {
                continue;
            }
            if !matching_assignment(
                snapshot,
                body.semantics.role_assignment_id,
                body.semantics.actor_id,
                requirement.required_capability,
                scope,
                input.instant,
            ) {
                continue;
            }
            let actor_sets_overlap = requirement.independent_from_intents.iter().any(|intent| {
                actors_by_intent
                    .get(intent)
                    .is_some_and(|actors| actors.contains(&body.semantics.actor_id))
            });
            if actor_sets_overlap {
                continue;
            }
            matching.push(body);
        }
        let distinct: BTreeSet<_> = matching
            .iter()
            .map(|body| body.semantics.actor_id)
            .collect();
        if distinct.len() < requirement.quorum as usize {
            return ApprovalPolicyEvaluation::Refused(diag(
                "revision_approval_quorum_unsatisfied",
                "effective authorized attestations do not satisfy ordered quorum",
            ));
        }
        if policy.approval_policy.separation_of_duty
            && distinct
                .iter()
                .any(|actor| actors_by_intent.values().any(|prior| prior.contains(actor)))
        {
            return ApprovalPolicyEvaluation::Refused(diag(
                "revision_approval_separation_unsatisfied",
                "separation of duty requires distinct actors",
            ));
        }
        actors_by_intent.insert(requirement.intent, distinct);
        accepted.extend(matching.into_iter().map(|body| body.id));
    }
    accepted.sort();
    accepted.dedup();
    ApprovalPolicyEvaluation::Satisfied {
        attestations: accepted,
    }
}

fn matching_assignment(
    snapshot: &AuthoritySnapshot,
    role_assignment_id: RoleAssignmentId,
    actor_id: ActorIdentityId,
    capability: super::Capability,
    scope: RoleScope,
    instant: i64,
) -> bool {
    query_effective_role_assignments(snapshot, actor_id, instant)
        .into_iter()
        .any(|assignment| {
            assignment.source == AuthorityRef::RoleAssignment(role_assignment_id)
                && assignment.capability == capability
                && assignment.scope.contains(scope)
        })
}

pub(crate) fn validate_attestations(snapshot: &AuthoritySnapshot) -> Vec<AuthorityDiagnostic> {
    let ids: BTreeSet<_> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::ApprovalAttestation(body) => Some(body.id),
            _ => None,
        })
        .collect();
    let mut diagnostics = Vec::new();
    for record in &snapshot.records {
        let AuthorityRecord::ApprovalAttestation(body) = record else {
            continue;
        };
        if body.semantics.disposition == AttestationDisposition::Active
            && body.semantics.disposes.is_some()
        {
            diagnostics.push(diag(
                "revision_attestation_active_disposes",
                "active attestation cannot dispose another attestation",
            ));
        }
        if body.semantics.disposition != AttestationDisposition::Active
            && body
                .semantics
                .disposes
                .is_none_or(|disposed| !ids.contains(&disposed))
        {
            diagnostics.push(diag(
                "revision_attestation_disposition_target_missing",
                "supersession or revocation must name an existing attestation",
            ));
        }
    }
    diagnostics
}

fn diag(code: &str, message: &str) -> AuthorityDiagnostic {
    AuthorityDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
pub(crate) trait TestSignatureProvider {
    fn verify(&self, attestation: &ApprovalAttestationData) -> bool;
}

#[cfg(test)]
pub(crate) fn verified_attestations_for_test(
    snapshot: &AuthoritySnapshot,
    provider: &dyn TestSignatureProvider,
) -> BTreeSet<ApprovalAttestationId> {
    snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::ApprovalAttestation(body) if provider.verify(&body.semantics) => {
                Some(body.id)
            }
            _ => None,
        })
        .collect()
}
