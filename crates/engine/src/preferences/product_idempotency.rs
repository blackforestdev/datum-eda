//! Durable product-mutation idempotency over immutable repository receipts.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::product_acceptance::AcceptedMutationAudit;
use super::repository::MutationReceipt;
use super::{
    GlobalPreferencesProductService, PreferenceActorV1, PreferenceErrorCodeV1, PreferenceErrorV1,
    PreferenceKey, PreferenceMutationRequestV1, PreferenceMutationResultV1,
};

#[derive(Serialize)]
struct CanonicalMutationRequest<'a> {
    schema_name: &'static str,
    schema_version: u32,
    request: &'a PreferenceMutationRequestV1,
    trusted_actor: &'a PreferenceActorV1,
    accepted_proposal: Option<&'a AcceptedMutationAudit>,
}

pub(super) fn canonical_mutation_request_digest(
    request: &PreferenceMutationRequestV1,
    actor: &PreferenceActorV1,
    accepted: Option<&AcceptedMutationAudit>,
) -> Result<String, String> {
    let bytes = crate::ir::serialization::to_json_bytes(&CanonicalMutationRequest {
        schema_name: "datum.preferences.mutation",
        schema_version: 1,
        request,
        trusted_actor: actor,
        accepted_proposal: accepted,
    })
    .map_err(|error| error.to_string())?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

pub(super) fn mutation_request_id(request: &PreferenceMutationRequestV1) -> Uuid {
    match request {
        PreferenceMutationRequestV1::SetUser { request_id, .. }
        | PreferenceMutationRequestV1::ResetUser { request_id, .. } => *request_id,
    }
}

impl GlobalPreferencesProductService {
    pub(super) fn replay_mutation(
        &self,
        key: &PreferenceKey,
        receipt: MutationReceipt,
    ) -> Result<PreferenceMutationResultV1, PreferenceErrorV1> {
        let historical_service = self
            .service
            .at_generation(receipt.resulting_generation)
            .map_err(|error| {
                self.error(
                    PreferenceErrorCodeV1::RepositoryIo,
                    "Original idempotent result is unavailable",
                    BTreeMap::from([
                        ("stage".to_owned(), json!("idempotent_replay")),
                        ("reason".to_owned(), json!(error.to_string())),
                    ]),
                    None,
                )
            })?;
        let historical = Self {
            service: historical_service,
            active_catalog_digest: self.active_catalog_digest.clone(),
        };
        let row = historical
            .service
            .rows()
            .into_iter()
            .find(|row| row.key == *key)
            .expect("committed active key remains in historical catalog");
        Ok(PreferenceMutationResultV1 {
            changed: true,
            generation: historical.service.status().generation().cloned(),
            value: historical.value_view(&row)?,
            explanation: historical.explanation(&row)?,
            receipt: Some(receipt),
        })
    }
}
