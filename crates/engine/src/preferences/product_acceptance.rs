//! Daemon-local, single-use human acceptance capabilities for MCP proposals.

use std::collections::{BTreeMap, BTreeSet};

use uuid::Uuid;

use super::{
    AuthorizeMcpPreferenceApplyV1, HeadExpectationV1, McpPreferenceAuthorizationResultV1,
    PreferenceActorKindV1, PreferenceActorV1, PreferenceProposalV1,
};

const ACCEPTANCE_LIFETIME_MS: u64 = 5 * 60 * 1_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreferenceAcceptanceRefusal {
    UnauthorizedActor,
    ProposalNotPrepared,
    ProposalMismatch,
    SessionMismatch,
    RepositoryMismatch,
    InvocationMismatch,
    Expired,
    Consumed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PreparedBinding {
    proposal_digest: String,
    repository_identity: String,
    prepared_against: HeadExpectationV1,
    originating_mcp_session: String,
}

/// An opaque, non-serializable capability. Public callers may pass it back to
/// the product service but cannot construct, inspect, transfer, or retarget it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreferenceAcceptanceHandleV1 {
    opaque_id: Uuid,
    acceptance_id: Uuid,
    daemon_instance_id: Uuid,
    proposal_id: Uuid,
    proposal_digest: String,
    repository_identity: String,
    prepared_against: HeadExpectationV1,
    accepting_actor: PreferenceActorV1,
    originating_mcp_session: String,
    invocation_id: Uuid,
    expires_at_unix_ms: u64,
}

impl PreferenceAcceptanceHandleV1 {
    pub(crate) fn acceptance_id(&self) -> Uuid {
        self.acceptance_id
    }

    pub(crate) fn accepting_actor(&self) -> &PreferenceActorV1 {
        &self.accepting_actor
    }
}

#[derive(Debug)]
pub struct PreferenceAcceptanceBroker {
    daemon_instance_id: Uuid,
    prepared: BTreeMap<Uuid, PreparedBinding>,
    slots: BTreeMap<String, PreferenceAcceptanceHandleV1>,
    consumed: BTreeSet<Uuid>,
}

impl PreferenceAcceptanceBroker {
    pub fn new(daemon_instance_id: Uuid) -> Self {
        Self {
            daemon_instance_id,
            prepared: BTreeMap::new(),
            slots: BTreeMap::new(),
            consumed: BTreeSet::new(),
        }
    }

    pub fn register_prepared(
        &mut self,
        proposal: &PreferenceProposalV1,
        repository_identity: &str,
    ) -> Result<(), PreferenceAcceptanceRefusal> {
        if proposal.requesting_actor.kind != PreferenceActorKindV1::McpAgent
            || proposal.creation_session != proposal.requesting_actor.session_id
        {
            return Err(PreferenceAcceptanceRefusal::SessionMismatch);
        }
        self.prepared.insert(
            proposal.proposal_id,
            PreparedBinding {
                proposal_digest: proposal.proposal_digest.clone(),
                repository_identity: repository_identity.to_owned(),
                prepared_against: proposal.prepared_against.clone(),
                originating_mcp_session: proposal.requesting_actor.session_id.clone(),
            },
        );
        Ok(())
    }

    pub fn authorize_mcp_apply(
        &mut self,
        request: &AuthorizeMcpPreferenceApplyV1,
        accepting_actor: &PreferenceActorV1,
        repository_identity: &str,
        apply_invocation_id: Uuid,
        now_unix_ms: u64,
    ) -> Result<McpPreferenceAuthorizationResultV1, PreferenceAcceptanceRefusal> {
        if !matches!(
            accepting_actor.kind,
            PreferenceActorKindV1::HumanGui | PreferenceActorKindV1::HumanCli
        ) {
            return Err(PreferenceAcceptanceRefusal::UnauthorizedActor);
        }
        let prepared = self
            .prepared
            .get(&request.proposal_id)
            .ok_or(PreferenceAcceptanceRefusal::ProposalNotPrepared)?;
        if prepared.proposal_digest != request.proposal_digest {
            return Err(PreferenceAcceptanceRefusal::ProposalMismatch);
        }
        if prepared.originating_mcp_session != request.originating_mcp_session {
            return Err(PreferenceAcceptanceRefusal::SessionMismatch);
        }
        if prepared.repository_identity != repository_identity {
            return Err(PreferenceAcceptanceRefusal::RepositoryMismatch);
        }
        let expires_at_unix_ms = now_unix_ms.saturating_add(ACCEPTANCE_LIFETIME_MS);
        let handle = PreferenceAcceptanceHandleV1 {
            opaque_id: Uuid::new_v4(),
            acceptance_id: Uuid::new_v4(),
            daemon_instance_id: self.daemon_instance_id,
            proposal_id: request.proposal_id,
            proposal_digest: request.proposal_digest.clone(),
            repository_identity: repository_identity.to_owned(),
            prepared_against: prepared.prepared_against.clone(),
            accepting_actor: accepting_actor.clone(),
            originating_mcp_session: request.originating_mcp_session.clone(),
            invocation_id: apply_invocation_id,
            expires_at_unix_ms,
        };
        self.slots
            .insert(request.originating_mcp_session.clone(), handle);
        Ok(McpPreferenceAuthorizationResultV1 {
            proposal_id: request.proposal_id,
            authorized: true,
            expires_at_unix_ms,
        })
    }

    pub fn authorization_for_apply(
        &self,
        proposal: &PreferenceProposalV1,
        originating_mcp_session: &str,
        repository_identity: &str,
        apply_invocation_id: Uuid,
        now_unix_ms: u64,
    ) -> Result<PreferenceAcceptanceHandleV1, PreferenceAcceptanceRefusal> {
        let handle = self
            .slots
            .get(originating_mcp_session)
            .ok_or(PreferenceAcceptanceRefusal::Expired)?;
        if self.consumed.contains(&handle.acceptance_id) {
            return Err(PreferenceAcceptanceRefusal::Consumed);
        }
        if now_unix_ms > handle.expires_at_unix_ms {
            return Err(PreferenceAcceptanceRefusal::Expired);
        }
        if handle.daemon_instance_id != self.daemon_instance_id
            || handle.originating_mcp_session != originating_mcp_session
        {
            return Err(PreferenceAcceptanceRefusal::SessionMismatch);
        }
        if handle.repository_identity != repository_identity {
            return Err(PreferenceAcceptanceRefusal::RepositoryMismatch);
        }
        if handle.proposal_id != proposal.proposal_id
            || handle.proposal_digest != proposal.proposal_digest
            || handle.prepared_against != proposal.prepared_against
        {
            return Err(PreferenceAcceptanceRefusal::ProposalMismatch);
        }
        if handle.invocation_id != apply_invocation_id {
            return Err(PreferenceAcceptanceRefusal::InvocationMismatch);
        }
        Ok(handle.clone())
    }

    pub fn consume(
        &mut self,
        handle: &PreferenceAcceptanceHandleV1,
    ) -> Result<(), PreferenceAcceptanceRefusal> {
        if self.consumed.contains(&handle.acceptance_id) {
            return Err(PreferenceAcceptanceRefusal::Consumed);
        }
        let current = self
            .slots
            .get(&handle.originating_mcp_session)
            .ok_or(PreferenceAcceptanceRefusal::Expired)?;
        if current.opaque_id != handle.opaque_id {
            return Err(PreferenceAcceptanceRefusal::ProposalMismatch);
        }
        self.consumed.insert(handle.acceptance_id);
        self.slots.remove(&handle.originating_mcp_session);
        Ok(())
    }

    pub fn close_session(&mut self, originating_mcp_session: &str) {
        self.slots.remove(originating_mcp_session);
        self.prepared
            .retain(|_, binding| binding.originating_mcp_session != originating_mcp_session);
    }
}
