//! Daemon-lifetime ownership for the Global Preferences service and MCP broker.

use anyhow::{Result, anyhow};
use eda_engine::preferences::{
    AuthorizeMcpPreferenceApplyV1, GlobalPreferencesProductService,
    InstalledPreferenceLocationProvider, McpPreferenceAuthorizationResultV1,
    PreferenceAcceptanceBroker, PreferenceActorKindV1, PreferenceActorV1,
    PreferenceLocationProvider, PreferenceProductPayloadV1, PreferenceProductRequestV1,
    PreferenceProductResponseV1, PreferenceProductResultV1, PreferenceProposalActionV1,
    PreferenceProposalResultV1, ProjectGenesisRequestV1, ProjectGenesisResponseV1,
    ProjectUnitsSourceV1,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct McpPreferenceProductParams {
    pub request: PreferenceProductRequestV1,
    pub actor: PreferenceActorV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct McpProjectGenesisParams {
    pub request: ProjectGenesisRequestV1,
    pub actor: PreferenceActorV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HostPreferenceAuthorizationParams {
    pub request: AuthorizeMcpPreferenceApplyV1,
    pub actor: PreferenceActorV1,
}

#[derive(Debug)]
pub(super) struct PreferencesDaemonState {
    service: GlobalPreferencesProductService,
    acceptance: PreferenceAcceptanceBroker,
}

impl PreferencesDaemonState {
    pub fn open_installed() -> Result<Self> {
        Self::open(&InstalledPreferenceLocationProvider)
    }

    pub(super) fn open(provider: &dyn PreferenceLocationProvider) -> Result<Self> {
        let daemon_instance_id = Uuid::new_v4();
        let service =
            GlobalPreferencesProductService::open(provider, format!("daemon:{daemon_instance_id}"))
                .map_err(|error| anyhow!("Global Preferences startup refused: {error:?}"))?;
        Ok(Self {
            service,
            acceptance: PreferenceAcceptanceBroker::new(daemon_instance_id),
        })
    }

    pub fn execute_mcp(
        &mut self,
        request: PreferenceProductRequestV1,
        actor: PreferenceActorV1,
    ) -> Result<PreferenceProductResponseV1, String> {
        if actor.kind != PreferenceActorKindV1::McpAgent {
            return Err("preferences MCP actor kind must be mcp_agent".to_owned());
        }
        let proposal_action = match &request.payload {
            PreferenceProductPayloadV1::Proposal(action) => Some(action.clone()),
            _ => None,
        };
        if let Some(PreferenceProposalActionV1::AcceptAndApply { proposal }) = proposal_action {
            if let Some(accepted) = self
                .service
                .replay_accepted_proposal(&proposal)
                .map_err(|error| format!("durable acceptance replay refused: {error:?}"))?
            {
                return Ok(product_response(
                    request,
                    self.service.context(),
                    Ok(PreferenceProductResultV1::Proposal(
                        PreferenceProposalResultV1::Accepted(accepted),
                    )),
                ));
            }
            let handle = match self.acceptance.authorization_for_apply(
                &proposal,
                &actor.session_id,
                &self.service.repository_identity(),
                actor.invocation_id,
                now_unix_ms(),
            ) {
                Ok(handle) => handle,
                Err(refusal) => {
                    let error = self.service.acceptance_error(refusal, &proposal);
                    return Ok(product_response(
                        request,
                        self.service.context(),
                        Err(error),
                    ));
                }
            };
            let result =
                self.service
                    .accept_proposal_with_handle(proposal, &handle, actor.invocation_id);
            if result.is_ok() {
                let _ = self.acceptance.consume(&handle);
            }
            return Ok(product_response(
                request,
                self.service.context(),
                result.map(|accepted| {
                    PreferenceProductResultV1::Proposal(PreferenceProposalResultV1::Accepted(
                        accepted,
                    ))
                }),
            ));
        }
        let response = self.service.execute(request, &actor);
        if let Some(PreferenceProductResultV1::Proposal(PreferenceProposalResultV1::Prepared(
            prepared,
        ))) = &response.result
        {
            self.acceptance
                .register_prepared(&prepared.proposal, &self.service.repository_identity())
                .map_err(|refusal| format!("register MCP proposal: {refusal:?}"))?;
        }
        Ok(response)
    }

    pub fn authorize_mcp_apply(
        &mut self,
        request: AuthorizeMcpPreferenceApplyV1,
        actor: PreferenceActorV1,
    ) -> Result<McpPreferenceAuthorizationResultV1, String> {
        self.acceptance
            .authorize_mcp_apply(
                &request,
                &actor,
                &self.service.repository_identity(),
                now_unix_ms(),
            )
            .map_err(|refusal| format!("authorize MCP preference apply: {refusal:?}"))
    }

    pub fn execute_project_genesis(
        &self,
        request: ProjectGenesisRequestV1,
        actor: PreferenceActorV1,
    ) -> Result<ProjectGenesisResponseV1, String> {
        if actor.kind != PreferenceActorKindV1::McpAgent {
            return Err("Project-genesis MCP actor kind must be mcp_agent".to_owned());
        }
        match &request.units_source {
            ProjectUnitsSourceV1::Global { .. } => {
                Ok(self.service.execute_project_genesis(request, &actor))
            }
            ProjectUnitsSourceV1::Factory { .. } => {
                let service = GlobalPreferencesProductService::factory_only(format!(
                    "daemon-factory:{}",
                    actor.invocation_id
                ))
                .map_err(|error| format!("factory genesis startup refused: {error:?}"))?;
                Ok(service.execute_project_genesis(request, &actor))
            }
        }
    }
}

fn product_response(
    request: PreferenceProductRequestV1,
    context: eda_engine::preferences::PreferenceContextV1,
    result: Result<PreferenceProductResultV1, eda_engine::preferences::PreferenceErrorV1>,
) -> PreferenceProductResponseV1 {
    let (result, error) = match result {
        Ok(result) => (Some(result), None),
        Err(error) => (None, Some(error)),
    };
    PreferenceProductResponseV1 {
        ok: error.is_none(),
        schema: request.schema,
        context,
        result,
        error,
    }
}

fn now_unix_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use eda_engine::preferences::{
        AuthorizeMcpPreferenceApplyV1, FixedPreferenceLocationProvider, HeadExpectationV1,
        PreferenceErrorCodeV1, PreferenceLocations, PreferenceMutationRequestV1,
        PreferenceProductPayloadV1, PreferenceProposalActionV1, PreferenceProposalResultV1,
        PreferenceQueryV1, PreferenceSchemaRefV1,
    };
    use serde_json::json;

    use super::*;

    fn state(label: &str) -> (PathBuf, PreferencesDaemonState) {
        let base = std::env::temp_dir().join(format!(
            "datum-preferences-daemon-{label}-{}-{}",
            std::process::id(),
            Uuid::new_v4()
        ));
        let provider = FixedPreferenceLocationProvider(PreferenceLocations {
            configuration_base: base.clone(),
            repository_root: base.join("preferences"),
            legacy_console_path: base.join("gui-preferences.json"),
        });
        (base, PreferencesDaemonState::open(&provider).unwrap())
    }

    fn actor(kind: PreferenceActorKindV1) -> PreferenceActorV1 {
        PreferenceActorV1 {
            kind,
            session_id: "mcp-session:test".to_owned(),
            local_actor_id: "test-agent".to_owned(),
            invocation_id: Uuid::new_v4(),
        }
    }

    #[test]
    fn daemon_lifetime_service_answers_queries_without_creating_repository() {
        let (base, mut state) = state("query");
        let response = state
            .execute_mcp(
                PreferenceProductRequestV1 {
                    schema: PreferenceSchemaRefV1 {
                        name: "datum.preferences.describe".to_owned(),
                        version: 1,
                    },
                    payload: PreferenceProductPayloadV1::Query(PreferenceQueryV1::Describe),
                },
                actor(PreferenceActorKindV1::McpAgent),
            )
            .unwrap();
        assert!(response.ok);
        assert!(!base.exists());
    }

    #[test]
    fn mcp_endpoint_cannot_claim_human_authority_or_mutate_directly() {
        let (base, mut state) = state("authority");
        let request = PreferenceProductRequestV1 {
            schema: PreferenceSchemaRefV1 {
                name: "datum.preferences.set".to_owned(),
                version: 1,
            },
            payload: PreferenceProductPayloadV1::Mutation(PreferenceMutationRequestV1::SetUser {
                key: "datum.accessibility.reduced_motion".to_owned(),
                value: json!(true),
                expected: HeadExpectationV1::Missing,
                request_id: Uuid::new_v4(),
                reason: "test direct mutation refusal".to_owned(),
            }),
        };
        assert!(
            state
                .execute_mcp(request.clone(), actor(PreferenceActorKindV1::HumanCli))
                .is_err()
        );
        let response = state
            .execute_mcp(request, actor(PreferenceActorKindV1::McpAgent))
            .unwrap();
        assert_eq!(
            response.error.expect("typed refusal").code,
            PreferenceErrorCodeV1::ProposalRequired
        );
        assert!(!base.exists());
    }

    #[test]
    fn json_rpc_preferences_route_returns_the_engine_product_envelope() {
        let (base, mut preferences) = state("json-rpc");
        let mut engine = eda_engine::api::Engine::new().unwrap();
        let response = crate::dispatch::dispatch_request_with_preferences(
            &mut engine,
            &mut preferences,
            crate::JsonRpcRequest {
                jsonrpc: "2.0".to_owned(),
                id: json!(31),
                method: "preferences.mcp_product".to_owned(),
                params: serde_json::to_value(McpPreferenceProductParams {
                    request: PreferenceProductRequestV1 {
                        schema: PreferenceSchemaRefV1 {
                            name: "datum.preferences.describe".to_owned(),
                            version: 1,
                        },
                        payload: PreferenceProductPayloadV1::Query(PreferenceQueryV1::Describe),
                    },
                    actor: actor(PreferenceActorKindV1::McpAgent),
                })
                .unwrap(),
            },
        );
        assert!(response.error.is_none());
        let result = response.result.expect("product envelope");
        assert_eq!(result["ok"], true);
        assert_eq!(result["schema"]["name"], "datum.preferences.describe");
        assert_eq!(result["context"]["active_descriptor_count"], 11);
        assert!(!base.exists());
    }

    #[test]
    fn json_rpc_project_genesis_uses_the_same_typed_factory_envelope() {
        let (base, mut preferences) = state("project-genesis");
        std::fs::create_dir(&base).unwrap();
        let destination = base.join("MCP Project");
        let request_id = Uuid::new_v4();
        let mut engine = eda_engine::api::Engine::new().unwrap();
        let response = crate::dispatch::dispatch_request_with_preferences(
            &mut engine,
            &mut preferences,
            crate::JsonRpcRequest {
                jsonrpc: "2.0".to_owned(),
                id: json!(41),
                method: "project.genesis".to_owned(),
                params: serde_json::to_value(McpProjectGenesisParams {
                    request: ProjectGenesisRequestV1 {
                        request_id,
                        destination: destination.clone(),
                        project_name: "MCP Project".to_owned(),
                        project_id: Some(Uuid::new_v4()),
                        units_source: ProjectUnitsSourceV1::Factory {
                            profile_id: "datum.units.factory.v1".to_owned(),
                        },
                    },
                    actor: actor(PreferenceActorKindV1::McpAgent),
                })
                .unwrap(),
            },
        );
        assert!(response.error.is_none());
        let envelope = response.result.unwrap();
        assert_eq!(envelope["ok"], true);
        assert_eq!(envelope["schema"]["name"], "datum.project.new");
        assert_eq!(envelope["result"]["request_id"], request_id.to_string());
        assert_eq!(
            envelope["result"]["units_receipt"]["items"]
                .as_array()
                .unwrap()
                .len(),
            8
        );
        assert!(!base.join("preferences").exists());
        eda_engine::substrate::ProjectResolver::new(destination)
            .resolve()
            .unwrap();
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn daemon_broker_requires_human_authorization_and_replays_after_restart() {
        let (base, mut state) = state("acceptance-route");
        let request_id = Uuid::new_v4();
        let mcp = actor(PreferenceActorKindV1::McpAgent);
        let prepare = PreferenceProductRequestV1 {
            schema: PreferenceSchemaRefV1 {
                name: "datum.preferences.proposal.prepare".to_owned(),
                version: 1,
            },
            payload: PreferenceProductPayloadV1::Proposal(PreferenceProposalActionV1::Prepare {
                mutation: PreferenceMutationRequestV1::SetUser {
                    key: "datum.accessibility.reduced_motion".to_owned(),
                    value: json!(true),
                    expected: HeadExpectationV1::Missing,
                    request_id,
                    reason: "reduce animation".to_owned(),
                },
                rationale: "request local review".to_owned(),
            }),
        };
        let prepared = state.execute_mcp(prepare, mcp.clone()).unwrap();
        let proposal = match prepared.result.unwrap() {
            PreferenceProductResultV1::Proposal(PreferenceProposalResultV1::Prepared(value)) => {
                value.proposal
            }
            other => panic!("unexpected prepare result: {other:?}"),
        };
        let apply_request = || PreferenceProductRequestV1 {
            schema: PreferenceSchemaRefV1 {
                name: "datum.preferences.proposal.accept_apply".to_owned(),
                version: 1,
            },
            payload: PreferenceProductPayloadV1::Proposal(
                PreferenceProposalActionV1::AcceptAndApply {
                    proposal: proposal.clone(),
                },
            ),
        };
        let refused = state.execute_mcp(apply_request(), mcp.clone()).unwrap();
        assert_eq!(
            refused.error.unwrap().code,
            PreferenceErrorCodeV1::MissingAcceptance
        );
        state
            .authorize_mcp_apply(
                AuthorizeMcpPreferenceApplyV1 {
                    proposal_id: proposal.proposal_id,
                    proposal_digest: proposal.proposal_digest.clone(),
                    originating_mcp_session: mcp.session_id.clone(),
                },
                actor(PreferenceActorKindV1::HumanCli),
            )
            .unwrap();
        let accepted = state.execute_mcp(apply_request(), mcp.clone()).unwrap();
        assert!(accepted.ok);
        let receipt = match accepted.result.unwrap() {
            PreferenceProductResultV1::Proposal(PreferenceProposalResultV1::Accepted(value)) => {
                value.mutation_result.receipt.unwrap()
            }
            other => panic!("unexpected accepted result: {other:?}"),
        };
        assert_eq!(receipt.proposal_id, Some(proposal.proposal_id));
        assert!(receipt.acceptance_id.is_some());

        drop(state);
        let provider = FixedPreferenceLocationProvider(PreferenceLocations {
            configuration_base: base.clone(),
            repository_root: base.join("preferences"),
            legacy_console_path: base.join("gui-preferences.json"),
        });
        let mut reopened = PreferencesDaemonState::open(&provider).unwrap();
        let replayed = reopened.execute_mcp(apply_request(), mcp).unwrap();
        assert!(
            replayed.ok,
            "durable receipt must precede live broker state"
        );
        let _ = std::fs::remove_dir_all(base);
    }
}
