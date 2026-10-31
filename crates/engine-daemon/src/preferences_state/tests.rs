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

#[test]
fn host_route_commits_a_human_accept_apply_and_rejects_creation_ops() {
    let (base, mut state) = state("host-accept");
    let request_id = Uuid::new_v4();
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
                    reason: "host accept test".to_owned(),
                },
                rationale: "host accept".to_owned(),
            },
        )),
    };
    // A side-effect-free creation op is NOT carried on the host route (it stays local).
    assert!(
        state
            .execute_host(prepare.clone(), actor(PreferenceActorKindV1::HumanCli))
            .is_err(),
        "the host route must reject proposal creation ops"
    );
    // Obtain a prepared proposal (via the MCP route), then a HUMAN accept routed through the
    // host route commits it through the daemon's single writer.
    let prepared = state
        .execute_mcp(prepare, actor(PreferenceActorKindV1::McpAgent))
        .unwrap();
    let proposal = match prepared.result.unwrap() {
        PreferenceProductResultV1::Proposal(PreferenceProposalResultV1::Prepared(value)) => {
            value.proposal
        }
        other => panic!("unexpected prepare result: {other:?}"),
    };
    let accept = PreferenceProductRequestV1 {
        schema: PreferenceSchemaRefV1 {
            name: "datum.preferences.proposal.accept_apply".to_owned(),
            version: 1,
        },
        payload: PreferenceProductPayloadV1::Proposal(Box::new(
            PreferenceProposalActionV1::AcceptAndApply { proposal },
        )),
    };
    let response = state
        .execute_host(accept, actor(PreferenceActorKindV1::HumanCli))
        .unwrap();
    assert!(
        response.ok,
        "a human accept_apply routed through the host must commit: {:?}",
        response.error
    );
    match response.result.expect("accepted result") {
        PreferenceProductResultV1::Proposal(PreferenceProposalResultV1::Accepted(value)) => {
            assert!(
                value.mutation_result.receipt.is_some(),
                "the routed accept records a receipt"
            )
        }
        other => panic!("unexpected accept result: {other:?}"),
    }
    let _ = std::fs::remove_dir_all(base);
}
