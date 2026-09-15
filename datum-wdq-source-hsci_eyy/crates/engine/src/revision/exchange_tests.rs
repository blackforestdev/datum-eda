use std::collections::BTreeSet;

use super::*;

fn id(value: u128) -> uuid::Uuid {
    uuid::Uuid::from_u128(value)
}
fn digest(value: u8) -> AlgorithmQualifiedDigest {
    AlgorithmQualifiedDigest(format!("sha256:{}", format!("{value:02x}").repeat(32)))
}
fn entry(name: &str, value: u8) -> ExchangeManifestEntry {
    ExchangeManifestEntry {
        logical_name: name.into(),
        digest: digest(value),
        byte_count: 1,
    }
}
fn envelope(
    completeness: ExchangeCompleteness,
    prerequisites: Vec<AlgorithmQualifiedDigest>,
) -> AuthorityExchangeEnvelopeData {
    let mut value = AuthorityExchangeEnvelopeData {
        exchange_format_version: 1,
        exchange_schema_version: 1,
        exchange_id: id(1),
        source_project: id(2),
        source_replica: "air-gap-a".into(),
        intended_destination: Some("site-b".into()),
        created_order: 1,
        asserted_time: Some(10),
        completeness,
        base_configuration_refs: Vec::new(),
        manifest: vec![entry("records", 1)],
        prerequisite_digests: prerequisites,
        policy_and_trust_context_refs: Vec::new(),
        canonical_payload_digest: AlgorithmQualifiedDigest(String::new()),
        signature_evidence: Vec::new(),
        challenge_nonce: Some("nonce-1".into()),
        confidentiality_and_egress_metadata: vec!["offline".into()],
        optional_transport_payloads: Vec::new(),
    };
    value.canonical_payload_digest = envelope_digest(&value).unwrap();
    value
}

#[test]
fn rev_i08_record_inventory_is_bidirectionally_exact() {
    assert_eq!(
        REV_I08_RECORD_FAMILIES,
        &[
            AuthorityRecordKind::ExternalMappingReceipt,
            AuthorityRecordKind::AdapterDivergenceObservation,
            AuthorityRecordKind::ReleaseMirrorRequest,
            AuthorityRecordKind::ReleaseMirrorResult,
            AuthorityRecordKind::AuthorityExchangeEnvelope,
            AuthorityRecordKind::AuthorityExchangeReceipt,
            AuthorityRecordKind::ExternalChangeCandidate,
            AuthorityRecordKind::CredentialOrTrustEvent,
            AuthorityRecordKind::TrustedTimestampEvidence,
        ]
    );
}

#[test]
fn full_and_incremental_air_gap_exchange_verify_without_git_or_network() {
    for value in [
        envelope(ExchangeCompleteness::Full, Vec::new()),
        envelope(ExchangeCompleteness::Incremental, vec![digest(8)]),
    ] {
        let known = value.prerequisite_digests.iter().cloned().collect();
        let receipt = verify_exchange(
            AuthorityRef::AuthorityExchangeEnvelope(AuthorityExchangeEnvelopeId(id(3))),
            &value,
            id(4),
            "site-b",
            &known,
            &BTreeSet::new(),
            20,
        );
        assert_eq!(receipt.disposition, ExchangeDisposition::Verified);
    }
}

#[test]
fn exchange_refuses_missing_replay_destination_corruption_and_unknown_schema() {
    let base = envelope(ExchangeCompleteness::Incremental, vec![digest(8)]);
    let receipt = |value: &AuthorityExchangeEnvelopeData,
                   known: &BTreeSet<_>,
                   seen: &BTreeSet<_>,
                   destination: &str| {
        verify_exchange(
            AuthorityRef::AuthorityExchangeEnvelope(AuthorityExchangeEnvelopeId(id(3))),
            value,
            id(4),
            destination,
            known,
            seen,
            20,
        )
        .disposition
    };
    assert_eq!(
        receipt(&base, &BTreeSet::new(), &BTreeSet::new(), "site-b"),
        ExchangeDisposition::MissingPrerequisite
    );
    assert_eq!(
        receipt(
            &base,
            &BTreeSet::from([digest(8)]),
            &BTreeSet::from([id(1)]),
            "site-b"
        ),
        ExchangeDisposition::ReplayRefused
    );
    assert_eq!(
        receipt(
            &base,
            &BTreeSet::from([digest(8)]),
            &BTreeSet::new(),
            "wrong"
        ),
        ExchangeDisposition::DestinationRefused
    );
    let mut corrupt = base.clone();
    corrupt.manifest[0].byte_count = 2;
    assert_eq!(
        receipt(
            &corrupt,
            &BTreeSet::from([digest(8)]),
            &BTreeSet::new(),
            "site-b"
        ),
        ExchangeDisposition::Corrupt
    );
    let mut unknown = base.clone();
    unknown.exchange_schema_version = 2;
    assert_eq!(
        receipt(
            &unknown,
            &BTreeSet::from([digest(8)]),
            &BTreeSet::new(),
            "site-b"
        ),
        ExchangeDisposition::UnknownSchema
    );
}

#[test]
fn ref_moves_append_divergence_and_mirror_retry_cannot_mutate_release() {
    let request = ReleaseMirrorRequestData {
        release: ReleaseId(id(5)),
        release_digest: digest(9),
        adapter_identity: "git-system-process".into(),
        destination: "refs/tags/A".into(),
        idempotency_key: id(6),
        requested_at: 1,
    };
    let failed = record_mirror_attempt(
        AuthorityRef::ReleaseMirrorRequest(ReleaseMirrorRequestId(id(7))),
        &request,
        1,
        2,
        ReleaseMirrorOutcome::Failed {
            reason_code: "adapter_crash".into(),
        },
    );
    let success = record_mirror_attempt(
        AuthorityRef::ReleaseMirrorRequest(ReleaseMirrorRequestId(id(7))),
        &request,
        2,
        3,
        ReleaseMirrorOutcome::Succeeded {
            external_objects: vec![ExternalObjectIdentity {
                algorithm: "sha256".into(),
                encoding: "hex".into(),
                object_kind: "tag".into(),
                value: "ab".repeat(32),
            }],
        },
    );
    assert_eq!(failed.release_digest, success.release_digest);
    assert!(matches!(
        failed.outcome,
        ReleaseMirrorOutcome::Failed { .. }
    ));
    assert!(matches!(
        success.outcome,
        ReleaseMirrorOutcome::Succeeded { .. }
    ));
    let prior = AdapterDivergenceObservationData {
        adapter_identity: "git-system-process".into(),
        datum_ref: Some(AuthorityRef::Release(request.release)),
        observed_objects: Vec::new(),
        divergence: AdapterDivergence::InSync,
        observed_at: 2,
        prior_observation: None,
        details: Vec::new(),
    };
    let moved = AdapterDivergenceObservationData {
        divergence: AdapterDivergence::RefMovedOrDeleted,
        observed_at: 4,
        prior_observation: Some(AuthorityRef::AdapterDivergenceObservation(
            AdapterDivergenceObservationId(id(8)),
        )),
        ..prior.clone()
    };
    assert_eq!(prior.divergence, AdapterDivergence::InSync);
    assert_eq!(moved.divergence, AdapterDivergence::RefMovedOrDeleted);
}

#[test]
fn semantic_candidate_stays_quarantined_and_trust_results_are_separate() {
    let candidate = ExternalChangeCandidateData {
        source_kind: "git_tree".into(),
        source_receipt: AuthorityRef::AuthorityExchangeReceipt(AuthorityExchangeReceiptId(id(9))),
        claimed_project_id: id(2),
        claimed_base: None,
        received_payload_digest: digest(1),
        resolver_findings: Vec::new(),
        migration_findings: Vec::new(),
        semantic_delta: None,
        textual_conflicts: Vec::new(),
        semantic_conflicts: vec!["dangling identity".into()],
        signature_valid: Some(true),
        signer_authorized: Some(false),
        proposed_operations: Vec::new(),
        affected_authority_refs: Vec::new(),
        disposition: ExternalCandidateDisposition::Quarantined,
    };
    assert!(candidate.textual_conflicts.is_empty());
    assert!(!candidate.semantic_conflicts.is_empty());
    assert_eq!(candidate.signature_valid, Some(true));
    assert_eq!(candidate.signer_authorized, Some(false));
    assert_eq!(
        candidate.disposition,
        ExternalCandidateDisposition::Quarantined
    );
}
