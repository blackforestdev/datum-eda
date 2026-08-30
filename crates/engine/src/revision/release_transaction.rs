use std::collections::BTreeSet;

use serde::Serialize;
use uuid::Uuid;

use crate::error::EngineError;

use super::{
    AUTHORITY_SCHEMA_VERSION, AlgorithmQualifiedDigest, ApprovalIntent, AttestationDisposition,
    AttestationStanding, AuthorityDiagnostic, AuthorityRecord, AuthorityRecordBody, AuthorityRef,
    AuthorityResolution, AuthoritySnapshot, ConfigurationBaselineId, DocumentIssueData,
    EngineeringRevisionData, EngineeringRevisionId, ReleaseCandidateData, ReleaseCandidateId,
    ReleaseData, ReleaseId, ReleasePackageData, RevisionAuthorityStore,
    canonical::{canonical_bytes, digest_bytes, validate_digest},
    query_attestation_standing,
    release_projection::{
        document_issue_references, package_references, release_package_manifest_digest,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseCommitFaultPoint {
    BeforeBaseline,
    AfterBaseline,
    AfterEngineeringRevisions,
    AfterRelease,
    AfterDocumentIssues,
    AfterReleasePackages,
}

impl ReleaseCommitFaultPoint {
    pub const ALL: &'static [Self] = &[
        Self::BeforeBaseline,
        Self::AfterBaseline,
        Self::AfterEngineeringRevisions,
        Self::AfterRelease,
        Self::AfterDocumentIssues,
        Self::AfterReleasePackages,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseConfigurationRequest {
    pub candidate_id: ReleaseCandidateId,
    pub expected_candidate_revision: u64,
    pub expected_evaluation_digest: AlgorithmQualifiedDigest,
    pub current_context: super::CandidateEvaluationContext,
    pub baseline_id: ConfigurationBaselineId,
    pub release_id: ReleaseId,
    pub release_number: Option<String>,
    pub released_at: i64,
    pub release_authority_attestation: super::ApprovalAttestationId,
    pub policy_evaluation_digest: AlgorithmQualifiedDigest,
    pub engine_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseConfigurationResult {
    pub snapshot: AuthoritySnapshot,
    pub baseline_id: ConfigurationBaselineId,
    pub release_id: ReleaseId,
    pub issued_revisions: Vec<EngineeringRevisionId>,
    pub reused_revisions: Vec<EngineeringRevisionId>,
    pub document_issues: Vec<super::DocumentIssueId>,
    pub release_packages: Vec<super::ReleasePackageId>,
}

#[derive(Serialize)]
struct CandidateEvaluationMaterial<'a> {
    candidate_key: Uuid,
    candidate_revision: u64,
    proposed_baseline: &'a super::ConfigurationBaselineData,
    proposed_revision_allocations: &'a [super::ProposedRevisionAllocation],
    governing_changes: &'a [super::EngineeringChangeId],
    departures: &'a [AuthorityRef],
    effectivity: &'a [super::EffectivityId],
    evidence_refs: &'a [AuthorityRef],
    proposed_document_issues: &'a [super::PreparedDocumentIssue],
    proposed_packages: &'a [super::PreparedReleasePackage],
    approval_policy: super::ProjectRevisionPolicyId,
    standards_evaluations: &'a [AuthorityRef],
    readiness_findings: &'a [super::ReleaseReadinessFinding],
    last_evaluated_context: &'a super::CandidateEvaluationContext,
}

pub fn candidate_evaluation_digest(
    data: &ReleaseCandidateData,
) -> Result<AlgorithmQualifiedDigest, EngineError> {
    Ok(digest_bytes(&canonical_bytes(
        &CandidateEvaluationMaterial {
            candidate_key: data.candidate_key,
            candidate_revision: data.candidate_revision,
            proposed_baseline: &data.proposed_baseline,
            proposed_revision_allocations: &data.proposed_revision_allocations,
            governing_changes: &data.governing_changes,
            departures: &data.departures,
            effectivity: &data.effectivity,
            evidence_refs: &data.evidence_refs,
            proposed_document_issues: &data.proposed_document_issues,
            proposed_packages: &data.proposed_packages,
            approval_policy: data.approval_policy,
            standards_evaluations: &data.standards_evaluations,
            readiness_findings: &data.readiness_findings,
            last_evaluated_context: &data.last_evaluated_context,
        },
    )?))
}

pub fn resolve_release_candidate(
    snapshot: &AuthoritySnapshot,
    candidate_id: ReleaseCandidateId,
) -> Option<(ReleaseCandidateId, ReleaseCandidateData)> {
    let first = snapshot.records.iter().find_map(|record| match record {
        AuthorityRecord::ReleaseCandidate(body) if body.id == candidate_id => Some(body),
        _ => None,
    })?;
    let key = first.semantics.candidate_key;
    let superseded: BTreeSet<_> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::ReleaseCandidate(body) if body.semantics.candidate_key == key => {
                body.semantics.supersedes
            }
            _ => None,
        })
        .collect();
    snapshot.records.iter().find_map(|record| match record {
        AuthorityRecord::ReleaseCandidate(body)
            if body.semantics.candidate_key == key && !superseded.contains(&body.id) =>
        {
            Some((body.id, body.semantics.clone()))
        }
        _ => None,
    })
}

pub fn prepare_release_configuration(
    snapshot: &AuthoritySnapshot,
    request: &ReleaseConfigurationRequest,
    fault: Option<ReleaseCommitFaultPoint>,
) -> Result<ReleaseConfigurationResult, EngineError> {
    inject(fault, ReleaseCommitFaultPoint::BeforeBaseline)?;
    let (resolved_id, candidate) = resolve_release_candidate(snapshot, request.candidate_id)
        .ok_or_else(|| {
            refusal(
                "stale_release_candidate",
                "release candidate does not resolve",
            )
        })?;
    require(
        resolved_id == request.candidate_id
            && candidate.candidate_revision == request.expected_candidate_revision
            && candidate.evaluation_digest == request.expected_evaluation_digest,
        "stale_release_candidate: exact candidate revision or digest changed",
    )?;
    require(
        candidate_evaluation_digest(&candidate)? == candidate.evaluation_digest,
        "stale_release_candidate: covered candidate inputs no longer match its digest",
    )?;
    require(
        candidate.last_evaluated_context == request.current_context,
        "stale_release_candidate: source, evidence, policy, model, or journal context changed",
    )?;
    require(
        candidate
            .readiness_findings
            .iter()
            .all(|finding| !finding.blocking || finding.cleared),
        "release_readiness_blocked: a typed readiness finding remains uncleared",
    )?;
    require_release_attestations(snapshot, request.candidate_id, &candidate)?;
    require_exact_candidate_inputs(&candidate)?;
    require_unique_revision_labels(snapshot, &candidate)?;

    let project_id = snapshot.project_id;
    let mut prepared = snapshot.clone();
    let baseline = AuthorityRecord::ConfigurationBaseline(AuthorityRecordBody {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        id: request.baseline_id,
        project_id,
        display_name: candidate.proposed_baseline.human_number.clone(),
        physical_locator: None,
        references: baseline_references(&candidate),
        semantics: candidate.proposed_baseline.clone(),
    });
    super::transaction::append(&mut prepared, baseline)?;
    inject(fault, ReleaseCommitFaultPoint::AfterBaseline)?;

    let mut issued = Vec::new();
    let mut reused = Vec::new();
    for allocation in &candidate.proposed_revision_allocations {
        if allocation.changed {
            let record = AuthorityRecord::EngineeringRevision(AuthorityRecordBody {
                schema_version: AUTHORITY_SCHEMA_VERSION,
                id: allocation.engineering_revision_id,
                project_id,
                display_name: Some(allocation.revision_label.clone()),
                physical_locator: None,
                references: revision_references(allocation, request),
                semantics: EngineeringRevisionData {
                    configuration_item: allocation.configuration_item,
                    revision_namespace: allocation.revision_namespace.clone(),
                    revision_scheme: allocation.revision_scheme,
                    scheme_version: allocation.scheme_version,
                    revision_label: allocation.revision_label.clone(),
                    suitability_or_status: allocation.suitability_or_status.clone(),
                    baseline: request.baseline_id,
                    predecessor_revision: allocation.predecessor_revision,
                    governing_changes: allocation.governing_changes.clone(),
                    allocated_by_policy: candidate.approval_policy,
                    issued_by_release: request.release_id,
                },
            });
            super::transaction::append(&mut prepared, record)?;
            issued.push(allocation.engineering_revision_id);
        } else {
            reused.push(allocation.reuse_existing_revision.ok_or_else(|| {
                refusal(
                    "floating_release_input",
                    "unchanged ConfigurationItem has no exact issued revision to reuse",
                )
            })?);
        }
    }
    inject(fault, ReleaseCommitFaultPoint::AfterEngineeringRevisions)?;

    let mut all_revisions = issued.clone();
    all_revisions.extend(reused.iter().copied());
    all_revisions.sort();
    let mut issue_ids = candidate
        .proposed_document_issues
        .iter()
        .map(|issue| issue.document_issue_id)
        .collect::<Vec<_>>();
    issue_ids.sort();
    let mut package_ids = candidate
        .proposed_packages
        .iter()
        .map(|package| package.release_package_id)
        .collect::<Vec<_>>();
    package_ids.sort();
    let release = AuthorityRecord::Release(AuthorityRecordBody {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        id: request.release_id,
        project_id,
        display_name: request.release_number.clone(),
        physical_locator: None,
        references: release_references(
            &candidate,
            request,
            &all_revisions,
            &issue_ids,
            &package_ids,
        ),
        semantics: ReleaseData {
            release_number: request.release_number.clone(),
            released_at: request.released_at,
            release_authority_attestation: request.release_authority_attestation,
            source_candidate: request.candidate_id,
            baseline: request.baseline_id,
            engineering_revisions: all_revisions,
            document_issues: issue_ids.clone(),
            release_packages: package_ids.clone(),
            governing_changes: candidate.governing_changes.clone(),
            departures: candidate.departures.clone(),
            effectivity: candidate.effectivity.clone(),
            qualifying_evidence: candidate.evidence_refs.clone(),
            audit_evaluations: candidate.standards_evaluations.clone(),
            policy_evaluation_digest: request.policy_evaluation_digest.clone(),
            engine_version: request.engine_version.clone(),
        },
    });
    super::transaction::append(&mut prepared, release)?;
    inject(fault, ReleaseCommitFaultPoint::AfterRelease)?;

    for issue in &candidate.proposed_document_issues {
        let record = AuthorityRecord::DocumentIssue(AuthorityRecordBody {
            schema_version: AUTHORITY_SCHEMA_VERSION,
            id: issue.document_issue_id,
            project_id,
            display_name: None,
            physical_locator: None,
            references: document_issue_references(issue, request),
            semantics: DocumentIssueData {
                controlled_document: issue.controlled_document,
                engineering_revision: issue.engineering_revision,
                baseline: request.baseline_id,
                source_revisions: issue.source_revisions.clone(),
                render_context_digest: issue.render_context_digest.clone(),
                outputs: issue.outputs.clone(),
                approvals: issue.approvals.clone(),
                issued_by_release: request.release_id,
            },
        });
        super::transaction::append(&mut prepared, record)?;
    }
    inject(fault, ReleaseCommitFaultPoint::AfterDocumentIssues)?;

    for package in &candidate.proposed_packages {
        let manifest_digest = release_package_manifest_digest(
            request.release_id,
            &package.ordered_member_refs,
            &package.exact_artifacts,
            &package.package_metadata_digest,
        )?;
        let record = AuthorityRecord::ReleasePackage(AuthorityRecordBody {
            schema_version: AUTHORITY_SCHEMA_VERSION,
            id: package.release_package_id,
            project_id,
            display_name: Some(package.name.clone()),
            physical_locator: None,
            references: package_references(package, request),
            semantics: ReleasePackageData {
                package_number: package.package_number.clone(),
                name: package.name.clone(),
                purpose: package.purpose.clone(),
                release: request.release_id,
                ordered_member_refs: package.ordered_member_refs.clone(),
                exact_artifacts: package.exact_artifacts.clone(),
                package_metadata_digest: package.package_metadata_digest.clone(),
                manifest_digest,
            },
        });
        super::transaction::append(&mut prepared, record)?;
    }
    inject(fault, ReleaseCommitFaultPoint::AfterReleasePackages)?;
    let diagnostics = prepared.validate();
    require(
        diagnostics.is_empty(),
        &format!(
            "atomic_release_validation_failed:{}",
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<Vec<_>>()
                .join(",")
        ),
    )?;
    Ok(ReleaseConfigurationResult {
        snapshot: prepared,
        baseline_id: request.baseline_id,
        release_id: request.release_id,
        issued_revisions: issued,
        reused_revisions: reused,
        document_issues: issue_ids,
        release_packages: package_ids,
    })
}

pub fn apply_release_configuration(
    store: &RevisionAuthorityStore,
    project_id: Uuid,
    request: &ReleaseConfigurationRequest,
) -> Result<super::IntegrityHead, EngineError> {
    let head = store.read_head().transpose()?.ok_or_else(|| {
        refusal(
            "release_without_project_transaction",
            "atomic release requires an accepted Project transaction",
        )
    })?;
    let snapshot = resolved_snapshot(store, project_id)?;
    let prepared = prepare_release_configuration(&snapshot, request, None)?;
    store.commit_authority_snapshot(project_id, &head.integrity_root, &prepared.snapshot)
}

pub(crate) fn validate_rev_i06_record(record: &AuthorityRecord) -> Vec<AuthorityDiagnostic> {
    let valid = match record {
        AuthorityRecord::EngineeringRevision(body) => {
            body.semantics.scheme_version > 0
                && !body.semantics.revision_namespace.trim().is_empty()
                && !body.semantics.revision_label.trim().is_empty()
                && is_sorted_unique(&body.semantics.governing_changes)
        }
        AuthorityRecord::ReleaseCandidate(body) => validate_candidate(&body.semantics),
        AuthorityRecord::Release(body) => {
            !body.semantics.engine_version.trim().is_empty()
                && is_sorted_unique(&body.semantics.engineering_revisions)
                && is_sorted_unique(&body.semantics.document_issues)
                && is_sorted_unique(&body.semantics.release_packages)
        }
        AuthorityRecord::SupersessionEstablished(body) => {
            body.semantics.prior_revision != body.semantics.successor_revision
                && !body.semantics.rationale.trim().is_empty()
        }
        AuthorityRecord::AuthorizationWithdrawn(body) => {
            !body.semantics.rationale.trim().is_empty()
        }
        AuthorityRecord::ObsolescenceDeclared(body) => !body.semantics.rationale.trim().is_empty(),
        AuthorityRecord::ControlledDocument(body) => {
            !body.semantics.document_number.trim().is_empty()
                && !body.semantics.name.trim().is_empty()
                && !body.semantics.document_kind.trim().is_empty()
        }
        AuthorityRecord::DocumentIssue(body) => {
            !body.semantics.source_revisions.is_empty()
                && !body.semantics.outputs.is_empty()
                && is_sorted_unique(&body.semantics.source_revisions)
                && is_sorted_unique(&body.semantics.outputs)
        }
        AuthorityRecord::ReleasePackage(body) => {
            !body.semantics.name.trim().is_empty()
                && !body.semantics.purpose.trim().is_empty()
                && !body.semantics.ordered_member_refs.is_empty()
                && !body.semantics.exact_artifacts.is_empty()
        }
        AuthorityRecord::Transmittal(body) => {
            !body.semantics.sender.trim().is_empty()
                && !body.semantics.recipients.is_empty()
                && is_sorted_unique(&body.semantics.recipients)
        }
        _ => return Vec::new(),
    };
    if valid {
        Vec::new()
    } else {
        vec![diagnostic(
            "revision_release_record_invalid",
            "REV-I06 record violates its exact immutable authority contract",
        )]
    }
}

fn validate_candidate(data: &ReleaseCandidateData) -> bool {
    data.candidate_revision > 0
        && data.evaluation_digest
            == candidate_evaluation_digest(data)
                .unwrap_or_else(|_| AlgorithmQualifiedDigest(String::new()))
        && !data.proposed_revision_allocations.is_empty()
        && is_strictly_sorted_by(&data.proposed_revision_allocations, |item| {
            item.configuration_item
        })
        && data.proposed_revision_allocations.iter().all(|allocation| {
            allocation.changed == allocation.reuse_existing_revision.is_none()
                && allocation.scheme_version > 0
                && !allocation.revision_namespace.trim().is_empty()
                && !allocation.revision_label.trim().is_empty()
        })
        && is_sorted_unique(&data.governing_changes)
        && is_sorted_unique(&data.departures)
        && is_sorted_unique(&data.effectivity)
        && is_sorted_unique(&data.evidence_refs)
        && is_sorted_unique(&data.attestations)
        && is_sorted_unique(&data.standards_evaluations)
}

fn require_release_attestations(
    snapshot: &AuthoritySnapshot,
    candidate_id: ReleaseCandidateId,
    candidate: &ReleaseCandidateData,
) -> Result<(), EngineError> {
    require(
        !candidate.attestations.is_empty(),
        "unsatisfied_approval: candidate has no approval attestations",
    )?;
    for id in &candidate.attestations {
        let body = snapshot
            .records
            .iter()
            .find_map(|record| match record {
                AuthorityRecord::ApprovalAttestation(body) if body.id == *id => Some(body),
                _ => None,
            })
            .ok_or_else(|| {
                refusal(
                    "unsatisfied_approval",
                    "candidate attestation does not resolve",
                )
            })?;
        require(
            body.semantics.target == AuthorityRef::ReleaseCandidate(candidate_id)
                && body.semantics.target_digest == candidate.evaluation_digest
                && body.semantics.disposition == AttestationDisposition::Active
                && matches!(
                    body.semantics.intent,
                    ApprovalIntent::Approve | ApprovalIntent::Release
                )
                && matches!(
                    query_attestation_standing(snapshot, *id),
                    AttestationStanding::Active
                ),
            "changed_approval_target: approval does not bind the current candidate digest",
        )?;
    }
    Ok(())
}

fn require_exact_candidate_inputs(candidate: &ReleaseCandidateData) -> Result<(), EngineError> {
    for source in &candidate.last_evaluated_context.source_revisions {
        require(
            !source.exact_technical_revision.trim().is_empty(),
            "floating_release_input: source technical revision is absent",
        )?;
        validate_digest(&source.digest)?;
    }
    for digest in &candidate.last_evaluated_context.evidence_digests {
        validate_digest(digest)?;
    }
    for issue in &candidate.proposed_document_issues {
        require(
            !issue.source_revisions.is_empty() && !issue.outputs.is_empty(),
            "floating_release_input: DocumentIssue lacks exact source or output bytes",
        )?;
        validate_digest(&issue.render_context_digest)?;
        for source in &issue.source_revisions {
            validate_digest(&source.digest)?;
        }
        for output in &issue.outputs {
            validate_digest(&output.byte_digest)?;
        }
    }
    Ok(())
}

fn require_unique_revision_labels(
    snapshot: &AuthoritySnapshot,
    candidate: &ReleaseCandidateData,
) -> Result<(), EngineError> {
    let mut labels = BTreeSet::new();
    for record in &snapshot.records {
        if let AuthorityRecord::EngineeringRevision(body) = record {
            labels.insert((
                body.semantics.configuration_item,
                body.semantics.revision_namespace.clone(),
                body.semantics.revision_scheme,
                body.semantics.scheme_version,
                body.semantics.revision_label.clone(),
                body.semantics.suitability_or_status.clone(),
            ));
        }
    }
    for allocation in candidate
        .proposed_revision_allocations
        .iter()
        .filter(|item| item.changed)
    {
        require(
            labels.insert((
                allocation.configuration_item,
                allocation.revision_namespace.clone(),
                allocation.revision_scheme,
                allocation.scheme_version,
                allocation.revision_label.clone(),
                allocation.suitability_or_status.clone(),
            )),
            "duplicate_revision_label: label already exists in the exact CI namespace and scheme version",
        )?;
    }
    Ok(())
}

fn resolved_snapshot(
    store: &RevisionAuthorityStore,
    project_id: Uuid,
) -> Result<AuthoritySnapshot, EngineError> {
    match store.resolve_authority(project_id) {
        AuthorityResolution::Resolved { snapshot } => Ok(snapshot),
        AuthorityResolution::Unconfigured => Err(refusal(
            "release_authority_unconfigured",
            "ReleaseConfiguration requires explicit Project revision authority",
        )),
        AuthorityResolution::ReadOnlyDiagnostic { .. } => Err(refusal(
            "release_authority_read_only",
            "preserved unknown authority data cannot be rewritten",
        )),
    }
}

fn baseline_references(candidate: &ReleaseCandidateData) -> Vec<AuthorityRef> {
    let mut refs = candidate.proposed_baseline.scope.clone();
    refs.extend(
        candidate
            .proposed_baseline
            .members
            .iter()
            .flat_map(|member| {
                [member.stable_authority_ref]
                    .into_iter()
                    .chain(member.target_ref)
            }),
    );
    refs.extend(
        candidate
            .governing_changes
            .iter()
            .copied()
            .map(AuthorityRef::EngineeringChange),
    );
    refs.extend(candidate.departures.iter().copied());
    refs
}

fn revision_references(
    allocation: &super::ProposedRevisionAllocation,
    request: &ReleaseConfigurationRequest,
) -> Vec<AuthorityRef> {
    let mut refs = vec![
        AuthorityRef::ConfigurationItem(allocation.configuration_item),
        AuthorityRef::RevisionScheme(allocation.revision_scheme),
        AuthorityRef::ConfigurationBaseline(request.baseline_id),
        AuthorityRef::Release(request.release_id),
    ];
    refs.extend(
        allocation
            .predecessor_revision
            .map(AuthorityRef::EngineeringRevision),
    );
    refs.extend(
        allocation
            .governing_changes
            .iter()
            .copied()
            .map(AuthorityRef::EngineeringChange),
    );
    refs
}

fn release_references(
    candidate: &ReleaseCandidateData,
    request: &ReleaseConfigurationRequest,
    revisions: &[EngineeringRevisionId],
    issues: &[super::DocumentIssueId],
    packages: &[super::ReleasePackageId],
) -> Vec<AuthorityRef> {
    let mut refs = vec![
        AuthorityRef::ReleaseCandidate(request.candidate_id),
        AuthorityRef::ConfigurationBaseline(request.baseline_id),
        AuthorityRef::ApprovalAttestation(request.release_authority_attestation),
    ];
    refs.extend(
        revisions
            .iter()
            .copied()
            .map(AuthorityRef::EngineeringRevision),
    );
    refs.extend(issues.iter().copied().map(AuthorityRef::DocumentIssue));
    refs.extend(packages.iter().copied().map(AuthorityRef::ReleasePackage));
    refs.extend(
        candidate
            .governing_changes
            .iter()
            .copied()
            .map(AuthorityRef::EngineeringChange),
    );
    refs.extend(candidate.departures.iter().copied());
    refs
}

fn inject(
    configured: Option<ReleaseCommitFaultPoint>,
    point: ReleaseCommitFaultPoint,
) -> Result<(), EngineError> {
    if configured == Some(point) {
        Err(EngineError::Operation(format!(
            "injected atomic release interruption at {point:?}"
        )))
    } else {
        Ok(())
    }
}

fn is_sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn is_strictly_sorted_by<T, K: Ord>(values: &[T], key: impl Fn(&T) -> K) -> bool {
    values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

fn diagnostic(code: &str, message: &str) -> AuthorityDiagnostic {
    AuthorityDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}

fn refusal(code: &str, message: &str) -> EngineError {
    EngineError::Validation(format!("{code}: {message}"))
}

fn require(condition: bool, message: &str) -> Result<(), EngineError> {
    if condition {
        Ok(())
    } else {
        Err(EngineError::Validation(message.to_string()))
    }
}
