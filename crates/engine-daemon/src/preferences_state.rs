//! Daemon-lifetime ownership for the Global Preferences service and MCP broker.

use anyhow::{Result, anyhow};
use eda_engine::preferences::{
    GlobalPreferencesProductService, InstalledPreferenceLocationProvider,
    PreferenceAcceptanceBroker, PreferenceActorKindV1, PreferenceActorV1,
    PreferenceLocationProvider, PreferenceProductRequestV1, PreferenceProductResponseV1,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct McpPreferenceProductParams {
    pub request: PreferenceProductRequestV1,
    pub actor: PreferenceActorV1,
}

#[derive(Debug)]
pub(super) struct PreferencesDaemonState {
    service: GlobalPreferencesProductService,
    #[allow(dead_code)]
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
        Ok(self.service.execute(request, &actor))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use eda_engine::preferences::{
        FixedPreferenceLocationProvider, HeadExpectationV1, PreferenceErrorCodeV1,
        PreferenceLocations, PreferenceMutationRequestV1, PreferenceProductPayloadV1,
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
}
