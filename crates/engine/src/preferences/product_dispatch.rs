//! Versioned request-envelope dispatch for every public Global Preferences adapter.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::{
    GlobalPreferencesProductService, PreferenceActorV1, PreferenceErrorCodeV1, PreferenceErrorV1,
    PreferenceProductPayloadV1, PreferenceProductRequestV1, PreferenceProductResponseV1,
    PreferenceProductResultV1, PreferenceProposalActionV1, PreferenceProposalResultV1,
    PreferenceSchemaRefV1,
};

const PRODUCT_SCHEMA_VERSION: u32 = 1;

impl GlobalPreferencesProductService {
    /// Execute a versioned product request that does not require a private
    /// daemon-held acceptance capability. GUI, CLI, and MCP adapters all use
    /// this entry point rather than selecting repository methods themselves.
    pub fn execute(
        &mut self,
        request: PreferenceProductRequestV1,
        actor: &PreferenceActorV1,
    ) -> PreferenceProductResponseV1 {
        let expected_schema = schema_for_payload(&request.payload);
        if request.schema.name != expected_schema
            || request.schema.version != PRODUCT_SCHEMA_VERSION
        {
            let received_schema_name = request.schema.name.clone();
            let received_version = request.schema.version;
            return self.response_error(
                request.schema,
                PreferenceErrorCodeV1::UnsupportedSchemaVersion,
                "Preference product request schema is unsupported",
                BTreeMap::from([
                    ("schema_name".to_owned(), json!(received_schema_name)),
                    (
                        "supported_version".to_owned(),
                        json!(PRODUCT_SCHEMA_VERSION),
                    ),
                    ("received_version".to_owned(), json!(received_version)),
                ]),
            );
        }

        if let Err(error) = super::product_actor::validate_actor(actor, &self.context()) {
            return response(request.schema, self.context(), Err(error));
        }

        let result = match request.payload {
            PreferenceProductPayloadV1::Query(query) => {
                self.query(query).map(PreferenceProductResultV1::Query)
            }
            PreferenceProductPayloadV1::Mutation(mutation) => self
                .mutate(mutation, actor)
                .map(PreferenceProductResultV1::Mutation),
            PreferenceProductPayloadV1::Proposal(action) => match action {
                PreferenceProposalActionV1::Prepare {
                    mutation,
                    rationale,
                } => self
                    .prepare_proposal(mutation, rationale, actor)
                    .map(PreferenceProposalResultV1::Prepared)
                    .map(PreferenceProductResultV1::Proposal),
                PreferenceProposalActionV1::Validate { proposal } => self
                    .validate_proposal(proposal)
                    .map(PreferenceProposalResultV1::Validated)
                    .map(PreferenceProductResultV1::Proposal),
                PreferenceProposalActionV1::Reject { proposal } => self
                    .reject_proposal(proposal)
                    .map(PreferenceProposalResultV1::Rejected)
                    .map(PreferenceProductResultV1::Proposal),
                PreferenceProposalActionV1::AcceptAndApply { proposal } => Err(self.error(
                    PreferenceErrorCodeV1::MissingAcceptance,
                    "Proposal application requires a daemon-held human acceptance capability",
                    BTreeMap::from([("proposal_id".to_owned(), json!(proposal.proposal_id))]),
                    mutation_draft(&proposal),
                )),
            },
        };
        response(request.schema, self.context(), result)
    }

    fn response_error(
        &self,
        schema: PreferenceSchemaRefV1,
        code: PreferenceErrorCodeV1,
        message: &str,
        details: BTreeMap<String, Value>,
    ) -> PreferenceProductResponseV1 {
        response(
            schema,
            self.context(),
            Err(self.error(code, message, details, None)),
        )
    }
}

fn response(
    schema: PreferenceSchemaRefV1,
    context: super::PreferenceContextV1,
    result: Result<PreferenceProductResultV1, PreferenceErrorV1>,
) -> PreferenceProductResponseV1 {
    match result {
        Ok(result) => PreferenceProductResponseV1 {
            ok: true,
            schema,
            context,
            result: Some(result),
            error: None,
        },
        Err(error) => PreferenceProductResponseV1 {
            ok: false,
            schema,
            context,
            result: None,
            error: Some(error),
        },
    }
}

fn schema_for_payload(payload: &PreferenceProductPayloadV1) -> &'static str {
    match payload {
        PreferenceProductPayloadV1::Query(query) => match query {
            super::PreferenceQueryV1::Describe => "datum.preferences.describe",
            super::PreferenceQueryV1::List { .. } => "datum.preferences.list",
            super::PreferenceQueryV1::Get { .. } => "datum.preferences.get",
            super::PreferenceQueryV1::Search { .. } => "datum.preferences.search",
            super::PreferenceQueryV1::Explain { .. } => "datum.preferences.explain",
            super::PreferenceQueryV1::PreviewProjectUnitsSeed { .. } => {
                "datum.preferences.preview_project_units_seed"
            }
        },
        PreferenceProductPayloadV1::Mutation(mutation) => match mutation {
            super::PreferenceMutationRequestV1::SetUser { .. } => "datum.preferences.set",
            super::PreferenceMutationRequestV1::ResetUser { .. } => "datum.preferences.reset",
        },
        PreferenceProductPayloadV1::Proposal(action) => match action {
            PreferenceProposalActionV1::Prepare { .. } => "datum.preferences.proposal.prepare",
            PreferenceProposalActionV1::Validate { .. } => "datum.preferences.proposal.validate",
            PreferenceProposalActionV1::AcceptAndApply { .. } => {
                "datum.preferences.proposal.accept_apply"
            }
            PreferenceProposalActionV1::Reject { .. } => "datum.preferences.proposal.reject",
        },
    }
}

fn mutation_draft(proposal: &super::PreferenceProposalV1) -> Option<Value> {
    match &proposal.mutation {
        super::PreferenceMutationRequestV1::SetUser { value, .. } => Some(value.clone()),
        super::PreferenceMutationRequestV1::ResetUser { .. } => None,
    }
}
