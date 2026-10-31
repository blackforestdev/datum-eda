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

/// Params for `preferences.host_product`: an ordinary host (human_cli / script_agent) Preferences
/// query or mutation executed through the owning daemon's single product service, so a reachable
/// owner is the sole writer (GP-CM03/§2.1; bead dat-preferences-cli-daemon-routing-i4d). Same shape
/// as the MCP route, but the host doorway is actor-gated differently (see `execute_host`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HostPreferenceProductParams {
    pub request: PreferenceProductRequestV1,
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

    /// Structured decode-rejection response for an adapter-layer decode failure of
    /// a `preferences.mcp_product` envelope (an unknown field or an unknown/
    /// unsupported schema), delegating classification to the product service so
    /// the daemon surfaces a symbolic code instead of a bare JSON-RPC error.
    pub(super) fn product_decode_rejection(
        &self,
        raw_params: &serde_json::Value,
        message: &str,
    ) -> PreferenceProductResponseV1 {
        self.service.decode_rejection_response(raw_params, message)
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
            PreferenceProductPayloadV1::Proposal(action) => Some(action.as_ref().clone()),
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

    /// Execute an ordinary HOST (human_cli / script_agent) Preferences query or mutation through the
    /// daemon's single long-lived product service, so a reachable owning daemon is the sole writer
    /// for the repository (GP-CM03/§2.1 owning-daemon routing; bead i4d).
    ///
    /// The host doorway is NOT the MCP doorway: it rejects `mcp_agent` (which must use the
    /// proposal-only `preferences.mcp_product` route and its broker) and carries no acceptance
    /// handle. A `human_cli` direct Set/Reset commits through `service.execute` exactly as the
    /// standalone CLI does, recording a MutationReceipt in the journal; the CLI's foreground-TTY
    /// confirmation is performed client-side before the mutation request is sent. A `script_agent`
    /// mutation receives the engine's own `proposal_required` refusal unchanged. Proposal payloads
    /// are not carried on the host route in this slice (the CLI keeps those on their existing path).
    pub fn execute_host(
        &mut self,
        request: PreferenceProductRequestV1,
        actor: PreferenceActorV1,
    ) -> Result<PreferenceProductResponseV1, String> {
        if actor.kind == PreferenceActorKindV1::McpAgent {
            return Err(
                "preferences host route rejects mcp_agent actors; use preferences.mcp_product"
                    .to_owned(),
            );
        }
        match request.payload {
            PreferenceProductPayloadV1::Query(_) | PreferenceProductPayloadV1::Mutation(_) => {
                Ok(self.service.execute(request, &actor))
            }
            PreferenceProductPayloadV1::Proposal(_) => Err(
                "preferences host route does not carry proposal operations; \
                 use preferences.mcp_product / the proposal workflow"
                    .to_owned(),
            ),
        }
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
            payload: PreferenceProductPayloadV1::Proposal(Box::new(
                PreferenceProposalActionV1::Prepare {
                    mutation: PreferenceMutationRequestV1::SetUser {
                        key: "datum.accessibility.reduced_motion".to_owned(),
                        value: json!(true),
                        expected: HeadExpectationV1::Missing,
                        request_id,
                        reason: "reduce animation".to_owned(),
                    },
                    rationale: "request local review".to_owned(),
                },
            )),
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
            payload: PreferenceProductPayloadV1::Proposal(Box::new(
                PreferenceProposalActionV1::AcceptAndApply {
                    proposal: proposal.clone(),
                },
            )),
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

    fn host_set_request() -> PreferenceProductRequestV1 {
        PreferenceProductRequestV1 {
            schema: PreferenceSchemaRefV1 {
                name: "datum.preferences.set".to_owned(),
                version: 1,
            },
            payload: PreferenceProductPayloadV1::Mutation(PreferenceMutationRequestV1::SetUser {
                key: "datum.accessibility.reduced_motion".to_owned(),
                value: json!(true),
                expected: HeadExpectationV1::Missing,
                request_id: Uuid::new_v4(),
                reason: "host route mutation".to_owned(),
            }),
        }
    }

    #[test]
    fn host_route_commits_a_human_cli_mutation_through_the_daemon_service() {
        let (base, mut state) = state("host-set");
        let response = state
            .execute_host(host_set_request(), actor(PreferenceActorKindV1::HumanCli))
            .unwrap();
        assert!(
            response.ok,
            "human_cli host set must commit: {:?}",
            response.error
        );
        // The commit is journaled through the daemon's own service: a matching receipt is returned.
        match response.result.expect("mutation result") {
            PreferenceProductResultV1::Mutation(result) => {
                assert!(
                    result.receipt.is_some(),
                    "a direct human commit records a receipt"
                )
            }
            other => panic!("unexpected host set result: {other:?}"),
        }
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn host_route_rejects_an_mcp_actor() {
        let (base, mut state) = state("host-reject-mcp");
        assert!(
            state
                .execute_host(host_set_request(), actor(PreferenceActorKindV1::McpAgent))
                .is_err(),
            "the host route must reject mcp_agent actors"
        );
        assert!(!base.exists());
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn host_route_refuses_a_script_agent_mutation_with_proposal_required() {
        let (base, mut state) = state("host-script");
        let response = state
            .execute_host(
                host_set_request(),
                actor(PreferenceActorKindV1::ScriptAgent),
            )
            .unwrap();
        assert_eq!(
            response.error.expect("typed refusal").code,
            PreferenceErrorCodeV1::ProposalRequired
        );
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn json_rpc_host_product_route_returns_the_engine_product_envelope() {
        let (base, mut preferences) = state("host-json-rpc");
        let mut engine = eda_engine::api::Engine::new().unwrap();
        let response = crate::dispatch::dispatch_request_with_preferences(
            &mut engine,
            &mut preferences,
            crate::JsonRpcRequest {
                jsonrpc: "2.0".to_owned(),
                id: json!(51),
                method: "preferences.host_product".to_owned(),
                params: serde_json::to_value(HostPreferenceProductParams {
                    request: PreferenceProductRequestV1 {
                        schema: PreferenceSchemaRefV1 {
                            name: "datum.preferences.describe".to_owned(),
                            version: 1,
                        },
                        payload: PreferenceProductPayloadV1::Query(PreferenceQueryV1::Describe),
                    },
                    actor: actor(PreferenceActorKindV1::HumanCli),
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
}
