//! Portable Global Preferences proposal preparation and validation.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::repository::GenerationRef;
use super::{
    AcceptedProposalResultV1, GlobalPreferencesProductService, HeadExpectationV1,
    PreferenceAcceptanceHandleV1, PreferenceAcceptanceRefusal, PreferenceActorV1,
    PreferenceErrorCodeV1, PreferenceErrorV1, PreferenceMutationRequestV1, PreferenceProposalV1,
    PreferenceSchemaRefV1, PreparedProposalResultV1, ProposalValidationResultV1,
    RejectedProposalResultV1,
};

const PROPOSAL_SCHEMA: &str = "datum.preferences.proposal";
const PROPOSAL_VERSION: u32 = 1;

#[derive(Serialize)]
struct ProposalDigestMaterial<'a> {
    schema_name: &'static str,
    schema_version: u32,
    proposal_id: Uuid,
    prepared_against: &'a HeadExpectationV1,
    active_catalog_digest: &'a str,
    mutation: &'a PreferenceMutationRequestV1,
    requesting_actor: &'a PreferenceActorV1,
    rationale: &'a str,
    creation_session: &'a str,
}

impl GlobalPreferencesProductService {
    pub fn prepare_proposal(
        &self,
        mutation: PreferenceMutationRequestV1,
        rationale: String,
        requesting_actor: &PreferenceActorV1,
    ) -> Result<PreparedProposalResultV1, PreferenceErrorV1> {
        super::product_actor::validate_actor(requesting_actor, &self.context())?;
        if rationale.trim().is_empty() {
            return Err(self.error(
                PreferenceErrorCodeV1::InvalidRequest,
                "Proposal rationale must not be empty",
                super::product_service::field_details("rationale", "empty"),
                mutation_draft(&mutation),
            ));
        }
        let (key, prepared_against) = self.validate_proposed_mutation(&mutation)?;
        let row = self.row_for_key(&key)?;
        let current_value = self.value_view(&row)?;
        let explanation = self.explanation(&row)?;
        let proposal_id = Uuid::new_v4();
        let creation_session = requesting_actor.session_id.clone();
        let mut proposal = PreferenceProposalV1 {
            schema: proposal_schema(),
            proposal_id,
            proposal_digest: String::new(),
            prepared_against,
            active_catalog_digest: self.context().active_catalog_digest,
            mutation,
            requesting_actor: requesting_actor.clone(),
            rationale,
            creation_session,
        };
        proposal.proposal_digest = proposal_digest(&proposal).map_err(|message| {
            self.error(
                PreferenceErrorCodeV1::RepositoryIo,
                &message,
                BTreeMap::new(),
                None,
            )
        })?;
        Ok(PreparedProposalResultV1 {
            proposal,
            current_value,
            explanation,
        })
    }

    pub fn validate_proposal(
        &self,
        proposal: PreferenceProposalV1,
    ) -> Result<ProposalValidationResultV1, PreferenceErrorV1> {
        self.validate_proposal_identity(&proposal)?;
        let current = current_expectation(self.context().generation);
        if proposal.prepared_against != current
            || proposal.active_catalog_digest != self.context().active_catalog_digest
        {
            return Err(self.proposal_error(
                PreferenceErrorCodeV1::ProposalStale,
                "Proposal was prepared against different preference authority",
                &proposal,
            ));
        }
        let (key, prepared_against) = self.validate_proposed_mutation(&proposal.mutation)?;
        if prepared_against != proposal.prepared_against {
            return Err(self.proposal_error(
                PreferenceErrorCodeV1::ProposalInvalid,
                "Proposal mutation expectation differs from its prepared authority",
                &proposal,
            ));
        }
        let row = self.row_for_key(&key)?;
        Ok(ProposalValidationResultV1 {
            valid: true,
            proposal,
            current_generation: self.context().generation,
            explanation: self.explanation(&row)?,
        })
    }

    pub fn reject_proposal(
        &self,
        proposal: PreferenceProposalV1,
    ) -> Result<RejectedProposalResultV1, PreferenceErrorV1> {
        self.validate_proposal_identity(&proposal)?;
        Ok(RejectedProposalResultV1 {
            proposal_id: proposal.proposal_id,
            rejected: true,
        })
    }

    pub fn accept_proposal_with_handle(
        &mut self,
        proposal: PreferenceProposalV1,
        handle: &PreferenceAcceptanceHandleV1,
        apply_invocation_id: Uuid,
    ) -> Result<AcceptedProposalResultV1, PreferenceErrorV1> {
        self.validate_proposal(proposal.clone())?;
        let audit = handle
            .accepted_audit(&proposal, &self.repository_identity(), apply_invocation_id)
            .map_err(|refusal| self.acceptance_error(refusal, &proposal))?;
        let mutation_result = self.commit_mutation(
            proposal.mutation.clone(),
            handle.accepting_actor(),
            Some(&audit),
        )?;
        Ok(AcceptedProposalResultV1 {
            proposal_id: proposal.proposal_id,
            mutation_result,
            acceptance_id: handle.acceptance_id(),
        })
    }

    pub fn accept_proposal_human(
        &mut self,
        proposal: PreferenceProposalV1,
        actor: &PreferenceActorV1,
    ) -> Result<AcceptedProposalResultV1, PreferenceErrorV1> {
        super::product_actor::validate_actor(actor, &self.context())?;
        if !matches!(
            actor.kind,
            super::PreferenceActorKindV1::HumanGui | super::PreferenceActorKindV1::HumanCli
        ) {
            return Err(self.proposal_error(
                PreferenceErrorCodeV1::HumanPresenceRequired,
                "Proposal acceptance requires trusted human presence",
                &proposal,
            ));
        }
        self.validate_proposal(proposal.clone())?;
        let mutation_result = self.commit_mutation(proposal.mutation.clone(), actor, None)?;
        Ok(AcceptedProposalResultV1 {
            proposal_id: proposal.proposal_id,
            mutation_result,
            acceptance_id: Uuid::new_v4(),
        })
    }

    fn validate_proposed_mutation(
        &self,
        mutation: &PreferenceMutationRequestV1,
    ) -> Result<(String, HeadExpectationV1), PreferenceErrorV1> {
        let (key, value, expected, reason) = match mutation {
            PreferenceMutationRequestV1::SetUser {
                key,
                value,
                expected,
                reason,
                ..
            } => (key, Some(value), expected, reason),
            PreferenceMutationRequestV1::ResetUser {
                key,
                expected,
                reason,
                ..
            } => (key, None, expected, reason),
        };
        if reason.trim().is_empty() {
            return Err(self.error(
                PreferenceErrorCodeV1::InvalidRequest,
                "Mutation reason must not be empty",
                super::product_service::field_details("reason", "empty"),
                value.cloned(),
            ));
        }
        let parsed = self.active_key(key, value.cloned())?;
        if let Some(value) = value {
            let descriptor = self.service.registry().get(&parsed).expect("active key");
            if !descriptor.validates(value) {
                return Err(self.error(
                    PreferenceErrorCodeV1::InvalidPreferenceValue,
                    "Preference value is invalid",
                    BTreeMap::from([
                        ("key".to_owned(), json!(key)),
                        ("reason".to_owned(), json!("descriptor_validation")),
                    ]),
                    Some(value.clone()),
                ));
            }
        }
        self.validate_expectation(expected, value.cloned())?;
        Ok((parsed.as_str().to_owned(), expected.clone()))
    }

    fn validate_proposal_identity(
        &self,
        proposal: &PreferenceProposalV1,
    ) -> Result<(), PreferenceErrorV1> {
        if proposal.schema != proposal_schema() {
            return Err(self.proposal_error(
                PreferenceErrorCodeV1::UnsupportedSchemaVersion,
                "Proposal schema is unsupported",
                proposal,
            ));
        }
        let submitted = proposal_digest(proposal).map_err(|message| {
            self.error(
                PreferenceErrorCodeV1::ProposalInvalid,
                &message,
                BTreeMap::new(),
                None,
            )
        })?;
        if submitted != proposal.proposal_digest {
            return Err(self.proposal_error(
                PreferenceErrorCodeV1::ProposalInvalid,
                "Proposal digest does not match its canonical content",
                proposal,
            ));
        }
        Ok(())
    }

    fn proposal_error(
        &self,
        code: PreferenceErrorCodeV1,
        message: &str,
        proposal: &PreferenceProposalV1,
    ) -> PreferenceErrorV1 {
        let mut error = self.error(
            code,
            message,
            BTreeMap::from([
                ("proposal_id".to_owned(), json!(proposal.proposal_id)),
                (
                    "proposal_digest".to_owned(),
                    json!(proposal.proposal_digest),
                ),
                (
                    "prepared_generation".to_owned(),
                    serde_json::to_value(&proposal.prepared_against).unwrap_or(Value::Null),
                ),
                (
                    "current_generation".to_owned(),
                    serde_json::to_value(self.context().generation).unwrap_or(Value::Null),
                ),
            ]),
            mutation_draft(&proposal.mutation),
        );
        error.preserved_proposal = serde_json::to_value(proposal).ok();
        error
    }

    fn acceptance_error(
        &self,
        refusal: PreferenceAcceptanceRefusal,
        proposal: &PreferenceProposalV1,
    ) -> PreferenceErrorV1 {
        let (code, state) = match refusal {
            PreferenceAcceptanceRefusal::UnauthorizedActor => (
                PreferenceErrorCodeV1::UnauthorizedActor,
                "unauthorized_actor",
            ),
            PreferenceAcceptanceRefusal::ProposalNotPrepared => {
                (PreferenceErrorCodeV1::MissingAcceptance, "not_prepared")
            }
            PreferenceAcceptanceRefusal::ProposalMismatch
            | PreferenceAcceptanceRefusal::RepositoryMismatch
            | PreferenceAcceptanceRefusal::InvocationMismatch
            | PreferenceAcceptanceRefusal::SessionMismatch => (
                PreferenceErrorCodeV1::AcceptanceMismatch,
                "binding_mismatch",
            ),
            PreferenceAcceptanceRefusal::Expired => {
                (PreferenceErrorCodeV1::AcceptanceExpired, "expired")
            }
            PreferenceAcceptanceRefusal::Consumed => {
                (PreferenceErrorCodeV1::AcceptanceConsumed, "consumed")
            }
        };
        let mut error = self.error(
            code,
            "Preference proposal acceptance was refused",
            BTreeMap::from([
                ("proposal_id".to_owned(), json!(proposal.proposal_id)),
                ("state".to_owned(), json!(state)),
            ]),
            mutation_draft(&proposal.mutation),
        );
        error.preserved_proposal = serde_json::to_value(proposal).ok();
        error
    }
}

fn proposal_schema() -> PreferenceSchemaRefV1 {
    PreferenceSchemaRefV1 {
        name: PROPOSAL_SCHEMA.to_owned(),
        version: PROPOSAL_VERSION,
    }
}

fn proposal_digest(proposal: &PreferenceProposalV1) -> Result<String, String> {
    let material = ProposalDigestMaterial {
        schema_name: PROPOSAL_SCHEMA,
        schema_version: PROPOSAL_VERSION,
        proposal_id: proposal.proposal_id,
        prepared_against: &proposal.prepared_against,
        active_catalog_digest: &proposal.active_catalog_digest,
        mutation: &proposal.mutation,
        requesting_actor: &proposal.requesting_actor,
        rationale: &proposal.rationale,
        creation_session: &proposal.creation_session,
    };
    let bytes =
        crate::ir::serialization::to_json_bytes(&material).map_err(|error| error.to_string())?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn mutation_draft(mutation: &PreferenceMutationRequestV1) -> Option<Value> {
    match mutation {
        PreferenceMutationRequestV1::SetUser { value, .. } => Some(value.clone()),
        PreferenceMutationRequestV1::ResetUser { .. } => None,
    }
}

fn current_expectation(generation: Option<GenerationRef>) -> HeadExpectationV1 {
    generation.map_or(HeadExpectationV1::Missing, HeadExpectationV1::Generation)
}
