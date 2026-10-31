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
        // The committing accept_apply routes to the daemon's single writer so a present human
        // applying a proposal is not a second local writer (bead i4d accept_apply follow-on; the
        // ratified owning-daemon routing design). The engine enforces actor authority on the apply
        // exactly as the standalone CLI path does. The side-effect-free proposal creation ops
        // (prepare/validate/reject) stay on the CLI's local path and are not carried here.
        let routes_accept_apply = matches!(
            &request.payload,
            PreferenceProductPayloadV1::Proposal(action)
                if matches!(action.as_ref(), PreferenceProposalActionV1::AcceptAndApply { .. })
        );
        match request.payload {
            PreferenceProductPayloadV1::Query(_) | PreferenceProductPayloadV1::Mutation(_) => {
                Ok(self.service.execute(request, &actor))
            }
            PreferenceProductPayloadV1::Proposal(_) if routes_accept_apply => {
                Ok(self.service.execute(request, &actor))
            }
            PreferenceProductPayloadV1::Proposal(_) => Err(
                "preferences host route carries queries, mutations and the committing accept_apply \
                 only; proposal prepare/validate/reject stay on the local side-effect-free path"
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
mod tests;
