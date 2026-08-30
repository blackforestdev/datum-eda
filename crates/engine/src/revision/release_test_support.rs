use super::*;

fn id(value: u128) -> uuid::Uuid {
    uuid::Uuid::from_u128(value)
}

fn body<I, P>(
    id: I,
    project_id: uuid::Uuid,
    references: Vec<AuthorityRef>,
    semantics: P,
) -> AuthorityRecordBody<I, P> {
    AuthorityRecordBody {
        schema_version: AUTHORITY_SCHEMA_VERSION,
        id,
        project_id,
        display_name: None,
        physical_locator: None,
        references,
        semantics,
    }
}

pub(super) fn append_standing_fixtures(
    mut snapshot: AuthoritySnapshot,
    prior_revision: EngineeringRevisionId,
    baseline: ConfigurationBaselineId,
    release: ReleaseId,
) -> AuthoritySnapshot {
    let project_id = snapshot.project_id;
    let successor = EngineeringRevisionId(id(73));
    let attestation = ApprovalAttestationId(id(31));
    transaction::append(
        &mut snapshot,
        AuthorityRecord::EngineeringRevision(body(
            successor,
            project_id,
            vec![
                AuthorityRef::ConfigurationItem(ConfigurationItemId(id(10))),
                AuthorityRef::RevisionScheme(RevisionSchemeId(id(12))),
                AuthorityRef::ConfigurationBaseline(baseline),
                AuthorityRef::EngineeringRevision(prior_revision),
                AuthorityRef::Release(release),
            ],
            EngineeringRevisionData {
                configuration_item: ConfigurationItemId(id(10)),
                revision_namespace: "ci-10".into(),
                revision_scheme: RevisionSchemeId(id(12)),
                scheme_version: 1,
                revision_label: "B".into(),
                suitability_or_status: None,
                baseline,
                predecessor_revision: Some(prior_revision),
                governing_changes: Vec::new(),
                allocated_by_policy: ProjectRevisionPolicyId(id(13)),
                issued_by_release: release,
            },
        )),
    )
    .unwrap();
    let records = [
        AuthorityRecord::SupersessionEstablished(body(
            SupersessionEstablishedId(id(74)),
            project_id,
            vec![
                AuthorityRef::EngineeringRevision(prior_revision),
                AuthorityRef::EngineeringRevision(successor),
                AuthorityRef::ApprovalAttestation(attestation),
            ],
            SupersessionEstablishedData {
                prior_revision,
                successor_revision: successor,
                authority_attestation: attestation,
                rationale: "successor issued".into(),
                effectivity: Vec::new(),
                established_at: 201,
            },
        )),
        AuthorityRecord::AuthorizationWithdrawn(body(
            AuthorizationWithdrawnId(id(75)),
            project_id,
            vec![
                AuthorityRef::Release(release),
                AuthorityRef::ApprovalAttestation(attestation),
            ],
            AuthorizationWithdrawnData {
                subject: AuthorityRef::Release(release),
                authority_attestation: attestation,
                rationale: "authority withdrawn".into(),
                effectivity: Vec::new(),
                withdrawn_at: 202,
            },
        )),
        AuthorityRecord::ObsolescenceDeclared(body(
            ObsolescenceDeclaredId(id(76)),
            project_id,
            vec![
                AuthorityRef::ControlledDocument(ControlledDocumentId(id(16))),
                AuthorityRef::ApprovalAttestation(attestation),
            ],
            ObsolescenceDeclaredData {
                subject: AuthorityRef::ControlledDocument(ControlledDocumentId(id(16))),
                authority_attestation: attestation,
                rationale: "document retired".into(),
                effectivity: Vec::new(),
                declared_at: 203,
            },
        )),
    ];
    for record in records {
        transaction::append(&mut snapshot, record).unwrap();
    }
    snapshot
}
