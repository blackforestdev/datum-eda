use uuid::Uuid;

use crate::error::EngineError;

use super::{
    ActorIdentityData, ActorIdentityId, ApprovalAttestationData, ApprovalAttestationId,
    ApprovalEvaluationInput, ApprovalPolicyEvaluation, AttestationDisposition, AttestationStanding,
    AuthorityEvent, AuthorityRecord, AuthorityRecordBody, AuthorityResolution, AuthoritySnapshot,
    Capability, CapabilityEvaluation, EffectivityData, EffectivityId, EffectivityResolution,
    EffectivityResolutionContext, FrozenRevisionSeedSnapshot, ProjectRevisionPolicyData,
    ProjectRevisionPolicyId, ProjectRevisionPolicyResolution, ProjectSeedReceiptData,
    ProjectSeedReceiptId, RevisionAuthorityStore, RevisionSchemeData, RevisionSchemeId,
    RevisionSchemeResolution, RoleAssignmentData, RoleAssignmentId, RoleDelegationData,
    RoleDelegationId, RoleScope, build_seed_records, evaluate_approval_policy, evaluate_capability,
    query_attestation_standing, query_effective_role_assignments, query_project_seed_receipt,
    resolve_effectivity, resolve_project_revision_policy, resolve_revision_scheme,
};

pub const REV_I03_MUTATIONS: &[&str] = &[
    "adopt_project_revision_policy",
    "supersede_project_revision_policy",
    "register_revision_scheme",
    "supersede_revision_scheme",
    "register_actor_identity",
    "assign_scoped_role",
    "revoke_scoped_role",
    "delegate_scoped_role",
    "revoke_role_delegation",
    "record_approval_attestation",
    "supersede_approval_attestation",
    "revoke_approval_attestation",
    "define_effectivity",
    "supersede_effectivity",
    "consume_project_revision_seed_snapshot",
];

pub const REV_I03_QUERIES: &[&str] = &[
    "resolve_project_revision_policy",
    "resolve_revision_scheme",
    "query_effective_role_assignments",
    "evaluate_capability",
    "evaluate_approval_policy",
    "query_attestation_standing",
    "resolve_effectivity",
    "query_project_seed_receipt",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RevisionMutationKind {
    AdoptProjectRevisionPolicy,
    SupersedeProjectRevisionPolicy,
    RegisterRevisionScheme,
    SupersedeRevisionScheme,
    RegisterActorIdentity,
    AssignScopedRole,
    RevokeScopedRole,
    DelegateScopedRole,
    RevokeRoleDelegation,
    RecordApprovalAttestation,
    SupersedeApprovalAttestation,
    RevokeApprovalAttestation,
    DefineEffectivity,
    SupersedeEffectivity,
    ConsumeProjectRevisionSeedSnapshot,
}

impl RevisionMutationKind {
    pub(crate) const ALL: &'static [Self] = &[
        Self::AdoptProjectRevisionPolicy,
        Self::SupersedeProjectRevisionPolicy,
        Self::RegisterRevisionScheme,
        Self::SupersedeRevisionScheme,
        Self::RegisterActorIdentity,
        Self::AssignScopedRole,
        Self::RevokeScopedRole,
        Self::DelegateScopedRole,
        Self::RevokeRoleDelegation,
        Self::RecordApprovalAttestation,
        Self::SupersedeApprovalAttestation,
        Self::RevokeApprovalAttestation,
        Self::DefineEffectivity,
        Self::SupersedeEffectivity,
        Self::ConsumeProjectRevisionSeedSnapshot,
    ];

    pub(crate) const fn wire_tag(self) -> &'static str {
        match self {
            Self::AdoptProjectRevisionPolicy => "adopt_project_revision_policy",
            Self::SupersedeProjectRevisionPolicy => "supersede_project_revision_policy",
            Self::RegisterRevisionScheme => "register_revision_scheme",
            Self::SupersedeRevisionScheme => "supersede_revision_scheme",
            Self::RegisterActorIdentity => "register_actor_identity",
            Self::AssignScopedRole => "assign_scoped_role",
            Self::RevokeScopedRole => "revoke_scoped_role",
            Self::DelegateScopedRole => "delegate_scoped_role",
            Self::RevokeRoleDelegation => "revoke_role_delegation",
            Self::RecordApprovalAttestation => "record_approval_attestation",
            Self::SupersedeApprovalAttestation => "supersede_approval_attestation",
            Self::RevokeApprovalAttestation => "revoke_approval_attestation",
            Self::DefineEffectivity => "define_effectivity",
            Self::SupersedeEffectivity => "supersede_effectivity",
            Self::ConsumeProjectRevisionSeedSnapshot => "consume_project_revision_seed_snapshot",
        }
    }
}

pub(crate) enum RevisionMutation {
    AdoptProjectRevisionPolicy(
        AuthorityRecordBody<ProjectRevisionPolicyId, ProjectRevisionPolicyData>,
    ),
    SupersedeProjectRevisionPolicy(
        AuthorityRecordBody<ProjectRevisionPolicyId, ProjectRevisionPolicyData>,
    ),
    RegisterRevisionScheme(AuthorityRecordBody<RevisionSchemeId, RevisionSchemeData>),
    SupersedeRevisionScheme(AuthorityRecordBody<RevisionSchemeId, RevisionSchemeData>),
    RegisterActorIdentity(AuthorityRecordBody<ActorIdentityId, ActorIdentityData>),
    AssignScopedRole(AuthorityRecordBody<RoleAssignmentId, RoleAssignmentData>),
    RevokeScopedRole(AuthorityRecordBody<RoleAssignmentId, RoleAssignmentData>),
    DelegateScopedRole(AuthorityRecordBody<RoleDelegationId, RoleDelegationData>),
    RevokeRoleDelegation(AuthorityRecordBody<RoleDelegationId, RoleDelegationData>),
    RecordApprovalAttestation(AuthorityRecordBody<ApprovalAttestationId, ApprovalAttestationData>),
    SupersedeApprovalAttestation(
        AuthorityRecordBody<ApprovalAttestationId, ApprovalAttestationData>,
    ),
    RevokeApprovalAttestation(AuthorityRecordBody<ApprovalAttestationId, ApprovalAttestationData>),
    DefineEffectivity(AuthorityRecordBody<EffectivityId, EffectivityData>),
    SupersedeEffectivity(AuthorityRecordBody<EffectivityId, EffectivityData>),
    ConsumeProjectRevisionSeedSnapshot {
        policy: Box<AuthorityRecordBody<ProjectRevisionPolicyId, ProjectRevisionPolicyData>>,
        receipt: AuthorityRecordBody<ProjectSeedReceiptId, ProjectSeedReceiptData>,
        frozen_snapshot: FrozenRevisionSeedSnapshot,
    },
}

impl RevisionMutation {
    pub(crate) const fn kind(&self) -> RevisionMutationKind {
        match self {
            Self::AdoptProjectRevisionPolicy(_) => RevisionMutationKind::AdoptProjectRevisionPolicy,
            Self::SupersedeProjectRevisionPolicy(_) => {
                RevisionMutationKind::SupersedeProjectRevisionPolicy
            }
            Self::RegisterRevisionScheme(_) => RevisionMutationKind::RegisterRevisionScheme,
            Self::SupersedeRevisionScheme(_) => RevisionMutationKind::SupersedeRevisionScheme,
            Self::RegisterActorIdentity(_) => RevisionMutationKind::RegisterActorIdentity,
            Self::AssignScopedRole(_) => RevisionMutationKind::AssignScopedRole,
            Self::RevokeScopedRole(_) => RevisionMutationKind::RevokeScopedRole,
            Self::DelegateScopedRole(_) => RevisionMutationKind::DelegateScopedRole,
            Self::RevokeRoleDelegation(_) => RevisionMutationKind::RevokeRoleDelegation,
            Self::RecordApprovalAttestation(_) => RevisionMutationKind::RecordApprovalAttestation,
            Self::SupersedeApprovalAttestation(_) => {
                RevisionMutationKind::SupersedeApprovalAttestation
            }
            Self::RevokeApprovalAttestation(_) => RevisionMutationKind::RevokeApprovalAttestation,
            Self::DefineEffectivity(_) => RevisionMutationKind::DefineEffectivity,
            Self::SupersedeEffectivity(_) => RevisionMutationKind::SupersedeEffectivity,
            Self::ConsumeProjectRevisionSeedSnapshot { .. } => {
                RevisionMutationKind::ConsumeProjectRevisionSeedSnapshot
            }
        }
    }

    pub(crate) const fn wire_tag(&self) -> &'static str {
        self.kind().wire_tag()
    }
}

pub(crate) fn apply_revision_mutations(
    store: &RevisionAuthorityStore,
    project_id: Uuid,
    mutations: Vec<RevisionMutation>,
) -> Result<super::IntegrityHead, EngineError> {
    if mutations.is_empty() {
        return Err(EngineError::Validation(
            "revision authority mutation batch cannot be empty".to_string(),
        ));
    }
    let head = store.read_head().transpose()?.ok_or_else(|| {
        EngineError::Validation(
            "revision authority mutation requires an accepted Project transaction".to_string(),
        )
    })?;
    let mut snapshot = match store.resolve_authority(project_id) {
        AuthorityResolution::Unconfigured => AuthoritySnapshot {
            schema_version: super::AUTHORITY_SCHEMA_VERSION,
            project_id,
            records: Vec::new(),
            events: Vec::new(),
            opaque_records: Vec::new(),
            opaque_events: Vec::new(),
        },
        AuthorityResolution::Resolved { snapshot } => snapshot,
        AuthorityResolution::ReadOnlyDiagnostic { .. } => {
            return Err(EngineError::Validation(
                "revision authority is read-only; preserved unknown data cannot be rewritten"
                    .to_string(),
            ));
        }
    };
    for mutation in mutations {
        apply_one(&mut snapshot, mutation)?;
    }
    let diagnostics = snapshot.validate();
    if !diagnostics.is_empty() {
        return Err(EngineError::Validation(format!(
            "revision authority mutation refused: {}",
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<Vec<_>>()
                .join(",")
        )));
    }
    store.commit_authority_snapshot(project_id, &head.integrity_root, &snapshot)
}

fn apply_one(
    snapshot: &mut AuthoritySnapshot,
    mutation: RevisionMutation,
) -> Result<(), EngineError> {
    match mutation {
        RevisionMutation::AdoptProjectRevisionPolicy(body) => {
            require(
                body.semantics.supersedes.is_none(),
                "adopted policy cannot supersede",
            )?;
            require(
                matches!(
                    resolve_project_revision_policy(Some(snapshot)),
                    ProjectRevisionPolicyResolution::Unmanaged
                ),
                "Project already has an adopted revision policy",
            )?;
            append(snapshot, AuthorityRecord::ProjectRevisionPolicy(body))
        }
        RevisionMutation::SupersedeProjectRevisionPolicy(body) => {
            require_existing_policy(snapshot, body.semantics.supersedes)?;
            append(snapshot, AuthorityRecord::ProjectRevisionPolicy(body))
        }
        RevisionMutation::RegisterRevisionScheme(body) => {
            require(
                body.semantics.supersedes.is_none(),
                "registered scheme cannot supersede",
            )?;
            append(snapshot, AuthorityRecord::RevisionScheme(body))
        }
        RevisionMutation::SupersedeRevisionScheme(body) => {
            require_existing_scheme(snapshot, body.semantics.supersedes)?;
            append(snapshot, AuthorityRecord::RevisionScheme(body))
        }
        RevisionMutation::RegisterActorIdentity(body) => {
            append(snapshot, AuthorityRecord::ActorIdentity(body))
        }
        RevisionMutation::AssignScopedRole(body) => {
            require(body.semantics.revokes.is_none(), "assignment cannot revoke")?;
            append(snapshot, AuthorityRecord::RoleAssignment(body))
        }
        RevisionMutation::RevokeScopedRole(body) => {
            require(
                body.semantics.revokes.is_some_and(|id| {
                    snapshot.records.iter().any(|record| {
                        matches!(record, AuthorityRecord::RoleAssignment(existing) if existing.id == id)
                    })
                }) && body.semantics.capabilities.is_empty(),
                "role revocation must name a prior assignment and grant no capability",
            )?;
            append(snapshot, AuthorityRecord::RoleAssignment(body))
        }
        RevisionMutation::DelegateScopedRole(body) => {
            require(body.semantics.revokes.is_none(), "delegation cannot revoke")?;
            append(snapshot, AuthorityRecord::RoleDelegation(body))
        }
        RevisionMutation::RevokeRoleDelegation(body) => {
            require(
                body.semantics.revokes.is_some_and(|id| {
                    snapshot.records.iter().any(|record| {
                        matches!(record, AuthorityRecord::RoleDelegation(existing) if existing.id == id)
                    })
                }) && body.semantics.capabilities.is_empty(),
                "delegation revocation must name a prior delegation and grant no capability",
            )?;
            append(snapshot, AuthorityRecord::RoleDelegation(body))
        }
        RevisionMutation::RecordApprovalAttestation(body) => {
            require(
                body.semantics.disposition == AttestationDisposition::Active
                    && body.semantics.disposes.is_none(),
                "recorded attestation must be active and cannot dispose another",
            )?;
            append(snapshot, AuthorityRecord::ApprovalAttestation(body))
        }
        RevisionMutation::SupersedeApprovalAttestation(body) => {
            require_attestation_disposition(snapshot, &body, AttestationDisposition::Superseded)?;
            append(snapshot, AuthorityRecord::ApprovalAttestation(body))
        }
        RevisionMutation::RevokeApprovalAttestation(body) => {
            require_attestation_disposition(snapshot, &body, AttestationDisposition::Revoked)?;
            append(snapshot, AuthorityRecord::ApprovalAttestation(body))
        }
        RevisionMutation::DefineEffectivity(body) => {
            require(
                body.semantics.supersedes.is_none(),
                "defined effectivity cannot supersede",
            )?;
            append(snapshot, AuthorityRecord::Effectivity(body))
        }
        RevisionMutation::SupersedeEffectivity(body) => {
            let prior = body.semantics.supersedes;
            require(
                prior.is_some_and(|id| {
                    snapshot.records.iter().any(|record| {
                        matches!(record, AuthorityRecord::Effectivity(existing) if existing.id == id)
                    })
                }),
                "effectivity supersession must name an existing effectivity",
            )?;
            append(snapshot, AuthorityRecord::Effectivity(body))
        }
        RevisionMutation::ConsumeProjectRevisionSeedSnapshot {
            policy,
            receipt,
            frozen_snapshot,
        } => {
            frozen_snapshot.validate()?;
            require(
                matches!(
                    resolve_project_revision_policy(Some(snapshot)),
                    ProjectRevisionPolicyResolution::Unmanaged
                ) && query_project_seed_receipt(snapshot).is_none(),
                "revision seed may be consumed only once for an unmanaged Project",
            )?;
            require(
                receipt.semantics.source_digest == frozen_snapshot.digest()?
                    && receipt.semantics.copied_policy_id == policy.id,
                "revision seed receipt does not match the frozen snapshot and copied policy",
            )?;
            append(snapshot, AuthorityRecord::ProjectRevisionPolicy(*policy))?;
            append(snapshot, AuthorityRecord::ProjectSeedReceipt(receipt))
        }
    }
}

pub(crate) fn append(
    snapshot: &mut AuthoritySnapshot,
    mut record: AuthorityRecord,
) -> Result<(), EngineError> {
    record.references_mut().sort();
    record.references_mut().dedup();
    require(
        record.project_id() == snapshot.project_id,
        "revision authority record belongs to another Project",
    )?;
    require(
        snapshot.record(record.record_ref()).is_none(),
        "revision authority record identity already exists",
    )?;
    let previous = snapshot
        .events
        .last()
        .map(|event| event.event_digest.clone());
    let event = AuthorityEvent::structural_append(
        snapshot.project_id,
        snapshot.events.len() as u64,
        &record,
        previous,
    );
    snapshot.records.push(record);
    snapshot.events.push(event);
    Ok(())
}

fn require_existing_policy(
    snapshot: &AuthoritySnapshot,
    prior: Option<ProjectRevisionPolicyId>,
) -> Result<(), EngineError> {
    require(
        prior.is_some_and(|id| {
            snapshot.records.iter().any(|record| {
                matches!(record, AuthorityRecord::ProjectRevisionPolicy(body) if body.id == id)
            })
        }),
        "policy supersession must name an existing policy",
    )
}

fn require_existing_scheme(
    snapshot: &AuthoritySnapshot,
    prior: Option<RevisionSchemeId>,
) -> Result<(), EngineError> {
    require(
        prior.is_some_and(|id| {
            snapshot.records.iter().any(
                |record| matches!(record, AuthorityRecord::RevisionScheme(body) if body.id == id),
            )
        }),
        "scheme supersession must name an existing scheme",
    )
}

fn require_attestation_disposition(
    snapshot: &AuthoritySnapshot,
    body: &AuthorityRecordBody<ApprovalAttestationId, ApprovalAttestationData>,
    disposition: AttestationDisposition,
) -> Result<(), EngineError> {
    require(
        body.semantics.disposition == disposition
            && body.semantics.disposes.is_some_and(|id| {
                snapshot.records.iter().any(|record| {
                    matches!(record, AuthorityRecord::ApprovalAttestation(existing) if existing.id == id)
                })
            }),
        "attestation disposition must name an existing attestation",
    )
}

fn require(condition: bool, message: &str) -> Result<(), EngineError> {
    if condition {
        Ok(())
    } else {
        Err(EngineError::Validation(message.to_string()))
    }
}

#[allow(dead_code)]
pub(crate) enum RevisionQuery<'a> {
    ResolveProjectRevisionPolicy,
    ResolveRevisionScheme {
        scheme_id: RevisionSchemeId,
        configuration_item_id: super::ConfigurationItemId,
    },
    QueryEffectiveRoleAssignments {
        actor_id: ActorIdentityId,
        instant: i64,
    },
    EvaluateCapability {
        actor_id: ActorIdentityId,
        capability: Capability,
        scope: RoleScope,
        instant: i64,
    },
    EvaluateApprovalPolicy {
        policy: &'a ProjectRevisionPolicyData,
        input: &'a ApprovalEvaluationInput,
    },
    QueryAttestationStanding {
        attestation_id: ApprovalAttestationId,
    },
    ResolveEffectivity {
        effectivity: Option<&'a EffectivityData>,
        context: &'a EffectivityResolutionContext,
    },
    QueryProjectSeedReceipt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RevisionQueryKind {
    ResolveProjectRevisionPolicy,
    ResolveRevisionScheme,
    QueryEffectiveRoleAssignments,
    EvaluateCapability,
    EvaluateApprovalPolicy,
    QueryAttestationStanding,
    ResolveEffectivity,
    QueryProjectSeedReceipt,
}

impl RevisionQueryKind {
    pub(crate) const ALL: &'static [Self] = &[
        Self::ResolveProjectRevisionPolicy,
        Self::ResolveRevisionScheme,
        Self::QueryEffectiveRoleAssignments,
        Self::EvaluateCapability,
        Self::EvaluateApprovalPolicy,
        Self::QueryAttestationStanding,
        Self::ResolveEffectivity,
        Self::QueryProjectSeedReceipt,
    ];

    pub(crate) const fn wire_tag(self) -> &'static str {
        match self {
            Self::ResolveProjectRevisionPolicy => "resolve_project_revision_policy",
            Self::ResolveRevisionScheme => "resolve_revision_scheme",
            Self::QueryEffectiveRoleAssignments => "query_effective_role_assignments",
            Self::EvaluateCapability => "evaluate_capability",
            Self::EvaluateApprovalPolicy => "evaluate_approval_policy",
            Self::QueryAttestationStanding => "query_attestation_standing",
            Self::ResolveEffectivity => "resolve_effectivity",
            Self::QueryProjectSeedReceipt => "query_project_seed_receipt",
        }
    }
}

impl RevisionQuery<'_> {
    pub(crate) const fn kind(&self) -> RevisionQueryKind {
        match self {
            Self::ResolveProjectRevisionPolicy => RevisionQueryKind::ResolveProjectRevisionPolicy,
            Self::ResolveRevisionScheme { .. } => RevisionQueryKind::ResolveRevisionScheme,
            Self::QueryEffectiveRoleAssignments { .. } => {
                RevisionQueryKind::QueryEffectiveRoleAssignments
            }
            Self::EvaluateCapability { .. } => RevisionQueryKind::EvaluateCapability,
            Self::EvaluateApprovalPolicy { .. } => RevisionQueryKind::EvaluateApprovalPolicy,
            Self::QueryAttestationStanding { .. } => RevisionQueryKind::QueryAttestationStanding,
            Self::ResolveEffectivity { .. } => RevisionQueryKind::ResolveEffectivity,
            Self::QueryProjectSeedReceipt => RevisionQueryKind::QueryProjectSeedReceipt,
        }
    }
}

#[allow(dead_code)]
pub(crate) enum RevisionQueryResult {
    ProjectRevisionPolicy(ProjectRevisionPolicyResolution),
    RevisionScheme(RevisionSchemeResolution),
    EffectiveRoleAssignments(Vec<super::EffectiveRoleAssignment>),
    Capability(CapabilityEvaluation),
    ApprovalPolicy(ApprovalPolicyEvaluation),
    AttestationStanding(AttestationStanding),
    Effectivity(EffectivityResolution),
    ProjectSeedReceipt(Option<ProjectSeedReceiptData>),
}

#[allow(dead_code)]
pub(crate) fn execute_revision_query(
    snapshot: Option<&AuthoritySnapshot>,
    query: RevisionQuery<'_>,
) -> Result<RevisionQueryResult, EngineError> {
    match query {
        RevisionQuery::ResolveProjectRevisionPolicy => Ok(
            RevisionQueryResult::ProjectRevisionPolicy(resolve_project_revision_policy(snapshot)),
        ),
        RevisionQuery::ResolveRevisionScheme {
            scheme_id,
            configuration_item_id,
        } => Ok(RevisionQueryResult::RevisionScheme(
            resolve_revision_scheme(
                snapshot
                    .ok_or_else(|| EngineError::Validation("authority is unconfigured".into()))?,
                scheme_id,
                configuration_item_id,
            ),
        )),
        RevisionQuery::QueryEffectiveRoleAssignments { actor_id, instant } => Ok(
            RevisionQueryResult::EffectiveRoleAssignments(query_effective_role_assignments(
                snapshot
                    .ok_or_else(|| EngineError::Validation("authority is unconfigured".into()))?,
                actor_id,
                instant,
            )),
        ),
        RevisionQuery::EvaluateCapability {
            actor_id,
            capability,
            scope,
            instant,
        } => Ok(RevisionQueryResult::Capability(evaluate_capability(
            snapshot.ok_or_else(|| EngineError::Validation("authority is unconfigured".into()))?,
            actor_id,
            capability,
            scope,
            instant,
        ))),
        RevisionQuery::EvaluateApprovalPolicy { policy, input } => Ok(
            RevisionQueryResult::ApprovalPolicy(evaluate_approval_policy(
                snapshot
                    .ok_or_else(|| EngineError::Validation("authority is unconfigured".into()))?,
                policy,
                input,
            )),
        ),
        RevisionQuery::QueryAttestationStanding { attestation_id } => Ok(
            RevisionQueryResult::AttestationStanding(query_attestation_standing(
                snapshot
                    .ok_or_else(|| EngineError::Validation("authority is unconfigured".into()))?,
                attestation_id,
            )),
        ),
        RevisionQuery::ResolveEffectivity {
            effectivity,
            context,
        } => Ok(RevisionQueryResult::Effectivity(resolve_effectivity(
            effectivity,
            context,
        ))),
        RevisionQuery::QueryProjectSeedReceipt => Ok(RevisionQueryResult::ProjectSeedReceipt(
            snapshot.and_then(query_project_seed_receipt),
        )),
    }
}

#[allow(dead_code)]
pub(crate) fn consume_seed_mutation(
    project_id: Uuid,
    policy_id: ProjectRevisionPolicyId,
    receipt_id: ProjectSeedReceiptId,
    scheme_id: RevisionSchemeId,
    policy: ProjectRevisionPolicyData,
    frozen_snapshot: FrozenRevisionSeedSnapshot,
) -> Result<RevisionMutation, EngineError> {
    let (policy, receipt) = build_seed_records(
        project_id,
        policy_id,
        receipt_id,
        scheme_id,
        policy,
        &frozen_snapshot,
    )?;
    Ok(RevisionMutation::ConsumeProjectRevisionSeedSnapshot {
        policy: Box::new(policy),
        receipt,
        frozen_snapshot,
    })
}
