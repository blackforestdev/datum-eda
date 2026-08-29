use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ActorIdentityId, AuthorityDiagnostic, AuthorityRecord, AuthorityRef, AuthoritySnapshot,
    RoleAssignmentId, RoleDelegationId,
};

pub const ACTOR_KINDS: &[ActorKind] = &[
    ActorKind::Person,
    ActorKind::Service,
    ActorKind::Organization,
    ActorKind::ControlledAutomation,
];

pub const CAPABILITIES: &[Capability] = &[
    Capability::Author,
    Capability::ChangeCoordinator,
    Capability::Reviewer,
    Capability::Verifier,
    Capability::ConfigurationAuthority,
    Capability::ReleaseAuthority,
    Capability::Auditor,
    Capability::RecordsAuthority,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    Person,
    Service,
    Organization,
    ControlledAutomation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Author,
    ChangeCoordinator,
    Reviewer,
    Verifier,
    ConfigurationAuthority,
    ReleaseAuthority,
    Auditor,
    RecordsAuthority,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorIdentityData {
    pub kind: ActorKind,
    pub stable_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "scope", content = "id", rename_all = "snake_case")]
pub enum RoleScope {
    Project(Uuid),
    Authority(AuthorityRef),
}

impl RoleScope {
    pub(crate) fn contains(self, requested: Self) -> bool {
        self == requested
            || matches!((self, requested), (Self::Project(left), Self::Authority(_)) if left != Uuid::nil())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveInterval {
    pub from_inclusive: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until_exclusive: Option<i64>,
}

impl EffectiveInterval {
    pub fn contains(&self, instant: i64) -> bool {
        instant >= self.from_inclusive && self.until_exclusive.is_none_or(|until| instant < until)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleAssignmentData {
    pub actor_id: ActorIdentityId,
    pub capabilities: BTreeSet<Capability>,
    pub scope: RoleScope,
    pub effective: EffectiveInterval,
    pub source: String,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<RoleAssignmentId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revokes: Option<RoleAssignmentId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleDelegationData {
    pub delegator_actor_id: ActorIdentityId,
    pub delegate_actor_id: ActorIdentityId,
    pub capabilities: BTreeSet<Capability>,
    pub scope: RoleScope,
    pub effective: EffectiveInterval,
    pub basis_assignment: RoleAssignmentId,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<RoleDelegationId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revokes: Option<RoleDelegationId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveRoleAssignment {
    pub actor_id: ActorIdentityId,
    pub capability: Capability,
    pub scope: RoleScope,
    pub source: AuthorityRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityEvaluation {
    Authorized(EffectiveRoleAssignment),
    Refused(AuthorityDiagnostic),
}

pub fn query_effective_role_assignments(
    snapshot: &AuthoritySnapshot,
    actor_id: ActorIdentityId,
    instant: i64,
) -> Vec<EffectiveRoleAssignment> {
    let revoked_assignments: BTreeSet<_> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::RoleAssignment(body) => body.semantics.revokes,
            _ => None,
        })
        .collect();
    let superseded_assignments: BTreeSet<_> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::RoleAssignment(body) => body.semantics.supersedes,
            _ => None,
        })
        .collect();
    let revoked_delegations: BTreeSet<_> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::RoleDelegation(body) => body.semantics.revokes,
            _ => None,
        })
        .collect();
    let superseded_delegations: BTreeSet<_> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::RoleDelegation(body) => body.semantics.supersedes,
            _ => None,
        })
        .collect();
    let mut effective = Vec::new();
    for record in &snapshot.records {
        match record {
            AuthorityRecord::RoleAssignment(body)
                if body.semantics.actor_id == actor_id
                    && body.semantics.revokes.is_none()
                    && !revoked_assignments.contains(&body.id)
                    && !superseded_assignments.contains(&body.id)
                    && body.semantics.effective.contains(instant) =>
            {
                effective.extend(body.semantics.capabilities.iter().map(|capability| {
                    EffectiveRoleAssignment {
                        actor_id,
                        capability: *capability,
                        scope: body.semantics.scope,
                        source: AuthorityRef::RoleAssignment(body.id),
                    }
                }));
            }
            AuthorityRecord::RoleDelegation(body)
                if body.semantics.delegate_actor_id == actor_id
                    && body.semantics.revokes.is_none()
                    && !revoked_delegations.contains(&body.id)
                    && !superseded_delegations.contains(&body.id)
                    && body.semantics.effective.contains(instant) =>
            {
                effective.extend(body.semantics.capabilities.iter().map(|capability| {
                    EffectiveRoleAssignment {
                        actor_id,
                        capability: *capability,
                        scope: body.semantics.scope,
                        source: AuthorityRef::RoleDelegation(body.id),
                    }
                }));
            }
            _ => {}
        }
    }
    effective.sort_by_key(|item| (item.capability, item.scope, item.source));
    effective
}

pub fn evaluate_capability(
    snapshot: &AuthoritySnapshot,
    actor_id: ActorIdentityId,
    capability: Capability,
    scope: RoleScope,
    instant: i64,
) -> CapabilityEvaluation {
    query_effective_role_assignments(snapshot, actor_id, instant)
        .into_iter()
        .find(|assignment| assignment.capability == capability && assignment.scope.contains(scope))
        .map(CapabilityEvaluation::Authorized)
        .unwrap_or_else(|| {
            CapabilityEvaluation::Refused(capability_refusal(
                snapshot, actor_id, capability, scope, instant,
            ))
        })
}

fn capability_refusal(
    snapshot: &AuthoritySnapshot,
    actor_id: ActorIdentityId,
    capability: Capability,
    scope: RoleScope,
    instant: i64,
) -> AuthorityDiagnostic {
    let revoked_assignments: BTreeSet<_> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::RoleAssignment(body) => body.semantics.revokes,
            _ => None,
        })
        .collect();
    let superseded_assignments: BTreeSet<_> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::RoleAssignment(body) => body.semantics.supersedes,
            _ => None,
        })
        .collect();
    let candidates: Vec<_> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::RoleAssignment(body)
                if body.semantics.actor_id == actor_id
                    && body.semantics.capabilities.contains(&capability) =>
            {
                Some((body.id, body.semantics.scope, &body.semantics.effective))
            }
            _ => None,
        })
        .collect();
    if candidates.is_empty() {
        return diag(
            "revision_capability_not_assigned",
            "actor has no assignment for the requested capability",
        );
    }
    if candidates
        .iter()
        .any(|(id, _, _)| revoked_assignments.contains(id))
    {
        return diag(
            "revision_role_revoked",
            "matching role assignment was revoked",
        );
    }
    if candidates
        .iter()
        .any(|(id, _, _)| superseded_assignments.contains(id))
    {
        return diag(
            "revision_role_superseded",
            "matching role assignment was superseded",
        );
    }
    if candidates
        .iter()
        .any(|(_, _, interval)| instant < interval.from_inclusive)
    {
        return diag(
            "revision_role_not_yet_effective",
            "matching role assignment is not yet effective",
        );
    }
    if candidates.iter().any(|(_, _, interval)| {
        interval
            .until_exclusive
            .is_some_and(|until| instant >= until)
    }) {
        return diag("revision_role_expired", "matching role assignment expired");
    }
    if candidates
        .iter()
        .any(|(_, granted, _)| !granted.contains(scope))
    {
        return diag(
            "revision_role_wrong_scope",
            "matching role assignment does not cover the exact scope",
        );
    }
    diag(
        "revision_capability_not_effective",
        "actor has no effective capability for the exact scope",
    )
}

pub(crate) fn validate_role_records(snapshot: &AuthoritySnapshot) -> Vec<AuthorityDiagnostic> {
    let assignments: BTreeMap<_, _> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::RoleAssignment(body) => Some((body.id, body)),
            _ => None,
        })
        .collect();
    let mut diagnostics = Vec::new();
    for record in &snapshot.records {
        match record {
            AuthorityRecord::RoleAssignment(body) => {
                validate_interval(&body.semantics.effective, &mut diagnostics);
                if body.semantics.capabilities.is_empty() && body.semantics.revokes.is_none() {
                    diagnostics.push(diag(
                        "revision_role_empty_capability",
                        "active role assignment must grant at least one capability",
                    ));
                }
            }
            AuthorityRecord::RoleDelegation(body) => {
                validate_interval(&body.semantics.effective, &mut diagnostics);
                let Some(basis) = assignments.get(&body.semantics.basis_assignment) else {
                    diagnostics.push(diag(
                        "revision_delegation_basis_missing",
                        "delegation basis assignment does not exist",
                    ));
                    continue;
                };
                if basis.semantics.actor_id != body.semantics.delegator_actor_id
                    || !body
                        .semantics
                        .capabilities
                        .is_subset(&basis.semantics.capabilities)
                    || !basis.semantics.scope.contains(body.semantics.scope)
                    || body.semantics.effective.from_inclusive
                        < basis.semantics.effective.from_inclusive
                    || body.semantics.effective.until_exclusive
                        > basis.semantics.effective.until_exclusive
                {
                    diagnostics.push(diag(
                        "revision_delegation_expands_authority",
                        "delegation exceeds the delegator's active capability, scope, or interval",
                    ));
                }
            }
            _ => {}
        }
    }
    diagnostics
}

fn validate_interval(interval: &EffectiveInterval, diagnostics: &mut Vec<AuthorityDiagnostic>) {
    if interval
        .until_exclusive
        .is_some_and(|until| until <= interval.from_inclusive)
    {
        diagnostics.push(diag(
            "revision_role_invalid_interval",
            "effective interval must end after it begins",
        ));
    }
}

fn diag(code: &str, message: &str) -> AuthorityDiagnostic {
    AuthorityDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}
