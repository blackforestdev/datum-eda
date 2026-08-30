use uuid::Uuid;

use crate::revision::{AuthorityExchangeEnvelopeData, AuthorityRecordKind, envelope_digest};

pub(super) fn rev_i08_fixture_semantics(kind: AuthorityRecordKind) -> Option<serde_json::Value> {
    let d = |c: char| format!("sha256:{}", c.to_string().repeat(64));
    let r = |kind: AuthorityRecordKind| serde_json::json!({"kind": kind.wire_tag(), "id": family_id(kind)});
    Some(match kind {
        AuthorityRecordKind::ExternalMappingReceipt => {
            serde_json::json!({"direction":"observed","datum_ref":r(AuthorityRecordKind::Release),"datum_digest":d('1'),"external_objects":[{"algorithm":"sha256","encoding":"hex","object_kind":"tag","value":"ab"}],"repository_identity":"repo","adapter_version":"1","observed_at":1,"verification_result":"verified","prior_receipt":null})
        }
        AuthorityRecordKind::AdapterDivergenceObservation => {
            serde_json::json!({"adapter_identity":"git","datum_ref":r(AuthorityRecordKind::Release),"observed_objects":[],"divergence":"in_sync","observed_at":1,"prior_observation":null,"details":[]})
        }
        AuthorityRecordKind::ReleaseMirrorRequest => {
            serde_json::json!({"release":family_id(AuthorityRecordKind::Release),"release_digest":d('2'),"adapter_identity":"git","destination":"tag:A","idempotency_key":Uuid::from_u128(900),"requested_at":1})
        }
        AuthorityRecordKind::ReleaseMirrorResult => {
            serde_json::json!({"request":r(AuthorityRecordKind::ReleaseMirrorRequest),"release":family_id(AuthorityRecordKind::Release),"release_digest":d('2'),"attempt":1,"completed_at":2,"outcome":{"outcome":"failed","reason_code":"offline"}})
        }
        AuthorityRecordKind::AuthorityExchangeEnvelope => {
            let mut value = serde_json::json!({"exchange_format_version":1,"exchange_schema_version":1,"exchange_id":Uuid::from_u128(901),"source_project":Uuid::from_u128(902),"source_replica":"fixture","intended_destination":"site-b","created_order":1,"asserted_time":1,"completeness":"full","base_configuration_refs":[],"manifest":[{"logical_name":"records","digest":d('3'),"byte_count":1}],"prerequisite_digests":[],"policy_and_trust_context_refs":[],"canonical_payload_digest":d('0'),"signature_evidence":[],"challenge_nonce":null,"confidentiality_and_egress_metadata":[],"optional_transport_payloads":[]});
            let data: AuthorityExchangeEnvelopeData =
                serde_json::from_value(value.clone()).unwrap();
            value["canonical_payload_digest"] =
                serde_json::to_value(envelope_digest(&data).unwrap()).unwrap();
            value
        }
        AuthorityRecordKind::AuthorityExchangeReceipt => {
            serde_json::json!({"envelope":r(AuthorityRecordKind::AuthorityExchangeEnvelope),"exchange_id":Uuid::from_u128(901),"source_project":Uuid::from_u128(902),"destination_project":Uuid::from_u128(903),"media_or_channel":"offline","observed_at":2,"integrity_valid":true,"signature_valid":null,"signer_authorized":null,"missing_prerequisites":[],"disposition":"verified","resulting_transactions":[],"retained_payload_digest":d('3')})
        }
        AuthorityRecordKind::ExternalChangeCandidate => {
            serde_json::json!({"source_kind":"exchange","source_receipt":r(AuthorityRecordKind::AuthorityExchangeReceipt),"claimed_project_id":Uuid::from_u128(903),"claimed_base":null,"received_payload_digest":d('3'),"resolver_findings":[],"migration_findings":[],"semantic_delta":null,"textual_conflicts":[],"semantic_conflicts":[],"signature_valid":null,"signer_authorized":null,"proposed_operations":[],"affected_authority_refs":[],"disposition":"quarantined"})
        }
        AuthorityRecordKind::CredentialOrTrustEvent => {
            serde_json::json!({"event_kind":"credential_registered","actor_or_root":r(AuthorityRecordKind::ActorIdentity),"method_class":"test-only","asserted_at":1,"evidence":[]})
        }
        AuthorityRecordKind::TrustedTimestampEvidence => {
            serde_json::json!({"subject":r(AuthorityRecordKind::ApprovalAttestation),"subject_digest":d('4'),"method_class":"test-only","asserted_time":1,"observed_at":2,"evidence_digest":d('5')})
        }
        _ => return None,
    })
}

fn family_id(kind: AuthorityRecordKind) -> Uuid {
    Uuid::from_u128(
        AuthorityRecordKind::ALL
            .iter()
            .position(|candidate| *candidate == kind)
            .unwrap() as u128
            + 1,
    )
}
