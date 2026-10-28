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
            PreferenceProductPayloadV1::Proposal(action) => match *action {
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
                PreferenceProposalActionV1::AcceptAndApply { proposal }
                    if matches!(
                        actor.kind,
                        super::PreferenceActorKindV1::HumanGui
                            | super::PreferenceActorKindV1::HumanCli
                    ) =>
                {
                    self.accept_proposal_human(proposal, actor)
                        .map(PreferenceProposalResultV1::Accepted)
                        .map(PreferenceProductResultV1::Proposal)
                }
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

    /// Build a structured product response for an adapter-layer decode failure
    /// (for example a serde `deny_unknown_fields` rejection at the daemon
    /// boundary) so public clients receive a symbolic code instead of a bare
    /// transport error. Per the supported-schema contract (sections 3 and 7): an
    /// unknown field in a supported schema is `invalid_request`; an unknown or
    /// unsupported schema name/version is `unsupported_schema_version`. The schema
    /// reference is recovered leniently from the raw params because the typed
    /// decode has already failed.
    pub fn decode_rejection_response(
        &self,
        raw_params: &Value,
        message: &str,
    ) -> PreferenceProductResponseV1 {
        let schema_value = raw_params
            .get("request")
            .and_then(|request| request.get("schema"));
        let name = schema_value
            .and_then(|schema| schema.get("name"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let version = schema_value
            .and_then(|schema| schema.get("version"))
            .and_then(Value::as_u64);
        let supported =
            is_product_schema_name(name) && version == Some(u64::from(PRODUCT_SCHEMA_VERSION));
        let code = if supported {
            PreferenceErrorCodeV1::InvalidRequest
        } else {
            PreferenceErrorCodeV1::UnsupportedSchemaVersion
        };
        self.response_error(
            PreferenceSchemaRefV1 {
                name: name.to_owned(),
                version: version
                    .and_then(|value| u32::try_from(value).ok())
                    .unwrap_or_default(),
            },
            code,
            message,
            BTreeMap::from([("schema_name".to_owned(), json!(name))]),
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
        PreferenceProductPayloadV1::Proposal(action) => match action.as_ref() {
            PreferenceProposalActionV1::Prepare { .. } => "datum.preferences.proposal.prepare",
            PreferenceProposalActionV1::Validate { .. } => "datum.preferences.proposal.validate",
            PreferenceProposalActionV1::AcceptAndApply { .. } => {
                "datum.preferences.proposal.accept_apply"
            }
            PreferenceProposalActionV1::Reject { .. } => "datum.preferences.proposal.reject",
        },
    }
}

/// The stable set of versioned V1 product schema names. Mirrors the names
/// produced by `schema_for_payload`; keep the two in lockstep.
fn is_product_schema_name(name: &str) -> bool {
    matches!(
        name,
        "datum.preferences.describe"
            | "datum.preferences.list"
            | "datum.preferences.get"
            | "datum.preferences.search"
            | "datum.preferences.explain"
            | "datum.preferences.preview_project_units_seed"
            | "datum.preferences.set"
            | "datum.preferences.reset"
            | "datum.preferences.proposal.prepare"
            | "datum.preferences.proposal.validate"
            | "datum.preferences.proposal.accept_apply"
            | "datum.preferences.proposal.reject"
    )
}

fn mutation_draft(proposal: &super::PreferenceProposalV1) -> Option<Value> {
    match &proposal.mutation {
        super::PreferenceMutationRequestV1::SetUser { value, .. } => Some(value.clone()),
        super::PreferenceMutationRequestV1::ResetUser { .. } => None,
    }
}

#[cfg(test)]
mod decode_tests {
    use crate::preferences::*;
    use serde_json::json;
    use uuid::Uuid;

    fn service() -> GlobalPreferencesProductService {
        let base = std::env::temp_dir().join(format!("datum-decode-{}", Uuid::new_v4()));
        let provider = FixedPreferenceLocationProvider(PreferenceLocations {
            configuration_base: base.clone(),
            repository_root: base.join("datum/preferences"),
            legacy_console_path: base.join("datum/gui-preferences.json"),
        });
        GlobalPreferencesProductService::open(&provider, "decode-test")
            .expect("decode-test product service opens")
    }

    #[test]
    fn maps_decode_failures_to_symbolic_codes() {
        // Adapter decode failures (serde deny_unknown_fields) get a symbolic code,
        // not a bare -32602: an unknown field in a supported schema -> invalid_request;
        // an unknown or unsupported schema name/version (or a malformed envelope) ->
        // unsupported_schema_version.
        let service = service();
        let cases = [
            (
                "datum.preferences.get",
                1,
                PreferenceErrorCodeV1::InvalidRequest,
            ),
            (
                "datum.preferences.nope",
                1,
                PreferenceErrorCodeV1::UnsupportedSchemaVersion,
            ),
            (
                "datum.preferences.get",
                2,
                PreferenceErrorCodeV1::UnsupportedSchemaVersion,
            ),
        ];
        for (name, version, expected) in cases {
            let raw = json!({"request": {"schema": {"name": name, "version": version}}});
            let got = service
                .decode_rejection_response(&raw, "invalid params: decode failure")
                .error
                .expect("typed decode rejection")
                .code;
            assert_eq!(got, expected, "{name} v{version}");
        }

        let garbled = service.decode_rejection_response(&json!({"request": {}}), "invalid params");
        assert_eq!(
            garbled.error.expect("typed").code,
            PreferenceErrorCodeV1::UnsupportedSchemaVersion
        );

        let named = service.decode_rejection_response(
            &json!({"request": {"schema": {"name": "datum.preferences.get", "version": 1}}}),
            "invalid params: unknown field `smuggled`",
        );
        assert_eq!(named.schema.name, "datum.preferences.get");
        assert_eq!(
            named.error.expect("typed").details["schema_name"],
            "datum.preferences.get"
        );
    }
}
