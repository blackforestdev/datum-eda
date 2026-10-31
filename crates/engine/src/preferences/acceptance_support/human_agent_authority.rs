//! GP-CM05E Phase-3 B service-refusal family: human-agent-authority.
//!
//! Authority under test: portable agents cannot acquire direct mutation authority
//! (GP product-surface contract). Each variant drives the engine's OWN product-envelope
//! service and acceptance broker -- GlobalPreferencesProductService +
//! PreferenceAcceptanceBroker, the exact surface product_tests.rs exercises, NOT the MCP
//! adapter -- into a typed authority/acceptance refusal, and proves the refused operation
//! wrote nothing.
//!
//! READ-ONLY on the coverage work root: every scenario runs in its OWN throwaway temp
//! store (ScenarioRoot, cleaned on drop), so the producer's work root is never written
//! and after==before=empty is honest; the driver's observation booleans carry the proof
//! (the managed-offline shape). The no-unauthorized-write predicate is the store
//! fingerprint being identical across the refused operation (nothing committed), and
//! project-unchanged is the absence of any project file (preferences are
//! project-independent).
//!
//! NOT COVERED (seam, stop-and-returned, E-deferred):
//! - no-foreground-tty -- there is no TTY/foreground concept in the engine product service
//!   or PreferenceActorV1; it is a CLI-adapter/environment property, so it cannot be driven
//!   from the engine surface.
//! - daemon-restart -- a daemon process-lifecycle (adapter) property, not engine-reachable.
//!   The acceptance broker's state is purely in-memory (see product_acceptance.rs:111-123),
//!   so no portable authority can persist across a restart: cross-restart authority survival
//!   is a by-construction non-event, with no persistence seam to defeat. A close_session ->
//!   Expired proof (a genuine session-close refusal) exists but belongs under an accurately
//!   named session-lifecycle row, not "daemon-restart" (tracked for intake); it cannot stand
//!   here without overclaiming the matrix row.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::json;
use uuid::Uuid;

use super::backup_restore::store_fingerprint;
use crate::preferences::{
    AuthorizeMcpPreferenceApplyV1, FixedPreferenceLocationProvider,
    GlobalPreferencesProductService, HeadExpectationV1, PreferenceAcceptanceBroker,
    PreferenceAcceptanceRefusal, PreferenceActorKindV1, PreferenceActorV1, PreferenceErrorCodeV1,
    PreferenceLocations, PreferenceMutationRequestV1, PreferenceProductPayloadV1,
    PreferenceProductRequestV1, PreferenceProposalActionV1, PreferenceSchemaRefV1,
};

/// A stable active key used for every scenario's proposed/attempted mutation.
const PILOT_KEY: &str = "datum.accessibility.reduced_motion";
/// A fixed repository identity for broker registration (the store is "missing" -- these
/// refusals occur before any repository is published, exactly as the broker unit tests).
const REPO_IDENTITY: &str = "missing:gp-acceptance-authority";
/// Broker time anchors (unix ms): the default acceptance TTL is 300_000 ms, so a request
/// well past `AUTHORIZE_AT + TTL` is expired.
const AUTHORIZE_AT_MS: u64 = 1_000;
const WITHIN_TTL_MS: u64 = 2_000;
const PAST_TTL_MS: u64 = 400_000;

/// A throwaway scenario store root in the temp dir, cleaned on drop, so no scenario ever
/// writes the coverage work root.
struct ScenarioRoot(PathBuf);

impl ScenarioRoot {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("gp-cm05e-authority-{}", Uuid::new_v4())))
    }

    fn provider(&self) -> FixedPreferenceLocationProvider {
        FixedPreferenceLocationProvider(PreferenceLocations {
            configuration_base: self.0.clone(),
            repository_root: self.0.join("datum/preferences"),
            legacy_console_path: self.0.join("datum/gui-preferences.json"),
        })
    }

    fn open(&self) -> Result<GlobalPreferencesProductService, String> {
        GlobalPreferencesProductService::open(&self.provider(), "gp-acceptance-driver")
            .map_err(|error| format!("product service open: {error:?}"))
    }

    /// Fingerprint of the on-disk preference store (empty when no store exists).
    fn store_fingerprint(&self) -> BTreeMap<String, String> {
        store_fingerprint(&self.0.join("datum/preferences"))
    }

    /// Preferences are project-independent: a preference operation must never create a
    /// project file.
    fn project_written(&self) -> bool {
        self.0.join("project.json").exists()
    }
}

impl Drop for ScenarioRoot {
    fn drop(&mut self) {
        if self.0.starts_with(std::env::temp_dir()) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

fn actor(kind: PreferenceActorKindV1) -> PreferenceActorV1 {
    PreferenceActorV1 {
        kind,
        session_id: "gp-acceptance-session".to_owned(),
        local_actor_id: "gp-acceptance-user".to_owned(),
        invocation_id: Uuid::new_v4(),
    }
}

fn set_user(value: bool, request_id: Uuid) -> PreferenceMutationRequestV1 {
    PreferenceMutationRequestV1::SetUser {
        key: PILOT_KEY.to_owned(),
        value: json!(value),
        expected: HeadExpectationV1::Missing,
        request_id,
        reason: "gp-acceptance authority scenario".to_owned(),
    }
}

fn accept_apply(proposal: crate::preferences::PreferenceProposalV1) -> PreferenceProductRequestV1 {
    PreferenceProductRequestV1 {
        schema: PreferenceSchemaRefV1 {
            name: "datum.preferences.proposal.accept_apply".to_owned(),
            version: 1,
        },
        payload: PreferenceProductPayloadV1::Proposal(Box::new(
            PreferenceProposalActionV1::AcceptAndApply { proposal },
        )),
    }
}

/// The human-agent-authority variants the engine driver can run (no-foreground-tty is
/// intentionally absent -- it is a CLI/environment seam, E-deferred).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityVariant {
    SpoofedActor,
    DirectMcpSetReset,
    StaleProposal,
    AcceptanceMismatch,
    AcceptanceExpired,
    AcceptanceConsumed,
    RequestIdConflict,
}

impl AuthorityVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SpoofedActor => "spoofed-actor",
            Self::DirectMcpSetReset => "direct-mcp-set-reset",
            Self::StaleProposal => "stale-proposal",
            Self::AcceptanceMismatch => "acceptance-mismatch",
            Self::AcceptanceExpired => "acceptance-expired",
            Self::AcceptanceConsumed => "acceptance-consumed",
            Self::RequestIdConflict => "request-id-conflict",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "spoofed-actor" => Some(Self::SpoofedActor),
            "direct-mcp-set-reset" => Some(Self::DirectMcpSetReset),
            "stale-proposal" => Some(Self::StaleProposal),
            "acceptance-mismatch" => Some(Self::AcceptanceMismatch),
            "acceptance-expired" => Some(Self::AcceptanceExpired),
            "acceptance-consumed" => Some(Self::AcceptanceConsumed),
            "request-id-conflict" => Some(Self::RequestIdConflict),
            _ => None,
        }
    }
}

/// OBSERVE phase. Each scenario is self-contained in a throwaway store, so there is no
/// setup phase (the family is read-only on the coverage work root).
pub fn authority_observe(variant: AuthorityVariant) -> Result<Vec<(&'static str, bool)>, String> {
    match variant {
        AuthorityVariant::SpoofedActor => observe_spoofed_actor(),
        AuthorityVariant::DirectMcpSetReset => observe_direct_mcp_set_reset(),
        AuthorityVariant::StaleProposal => observe_stale_proposal(),
        AuthorityVariant::AcceptanceMismatch => observe_acceptance_mismatch(),
        AuthorityVariant::AcceptanceExpired => observe_acceptance_expired(),
        AuthorityVariant::AcceptanceConsumed => observe_acceptance_consumed(),
        AuthorityVariant::RequestIdConflict => observe_request_id_conflict(),
    }
}

/// Standard three observations for an authority refusal.
fn refusal_observations(
    refused_with_expected_code: bool,
    root: &ScenarioRoot,
    before: &BTreeMap<String, String>,
) -> Vec<(&'static str, bool)> {
    let after = root.store_fingerprint();
    vec![
        // no-unauthorized-write: the refused operation committed nothing (store identical).
        ("no-unauthorized-write", before == &after),
        // safe-error-details: the refusal is the specific typed code, not a raw leak.
        ("safe-error-details", refused_with_expected_code),
        // project-unchanged: no project file was created by the preference operation.
        ("project-unchanged", !root.project_written()),
    ]
}

/// spoofed-actor: a portable agent (McpAgent) tries to accept-and-apply its OWN proposal,
/// spoofing the human acceptance authority -> MissingAcceptance, nothing written.
fn observe_spoofed_actor() -> Result<Vec<(&'static str, bool)>, String> {
    let root = ScenarioRoot::new();
    let mut service = root.open()?;
    let agent = actor(PreferenceActorKindV1::McpAgent);
    let prepared = service
        .prepare_proposal(
            set_user(true, Uuid::new_v4()),
            "agent proposes".to_owned(),
            &agent,
        )
        .map_err(|error| format!("prepare_proposal: {error:?}"))?;
    let before = root.store_fingerprint();
    let response = service.execute(accept_apply(prepared.proposal), &agent);
    let refused = !response.ok
        && response
            .error
            .map(|error| error.code == PreferenceErrorCodeV1::MissingAcceptance)
            .unwrap_or(false);
    Ok(refusal_observations(refused, &root, &before))
}

/// direct-mcp-set-reset: a portable agent attempts a DIRECT mutation (no proposal) ->
/// ProposalRequired, no store created.
fn observe_direct_mcp_set_reset() -> Result<Vec<(&'static str, bool)>, String> {
    let root = ScenarioRoot::new();
    let mut service = root.open()?;
    let before = root.store_fingerprint();
    let refusal = service
        .mutate(
            set_user(true, Uuid::new_v4()),
            &actor(PreferenceActorKindV1::McpAgent),
        )
        .err();
    let refused = refusal
        .map(|error| error.code == PreferenceErrorCodeV1::ProposalRequired)
        .unwrap_or(false);
    Ok(refusal_observations(refused, &root, &before))
}

/// stale-proposal: a proposal prepared against the empty head is left stale by a trusted
/// human commit; validating the stale proposal -> ProposalStale, and validation writes
/// nothing (the store reflects only the trusted commit, not the stale proposal).
fn observe_stale_proposal() -> Result<Vec<(&'static str, bool)>, String> {
    let root = ScenarioRoot::new();
    let mut service = root.open()?;
    let prepared = service
        .prepare_proposal(
            set_user(true, Uuid::new_v4()),
            "agent proposes".to_owned(),
            &actor(PreferenceActorKindV1::ScriptAgent),
        )
        .map_err(|error| format!("prepare_proposal: {error:?}"))?;
    // A trusted human commits a different value, advancing the head past the proposal.
    service
        .mutate(
            set_user(false, Uuid::new_v4()),
            &actor(PreferenceActorKindV1::HumanGui),
        )
        .map_err(|error| format!("trusted commit: {error:?}"))?;
    let before = root.store_fingerprint();
    let refusal = service.validate_proposal(prepared.proposal).err();
    let refused = refusal
        .map(|error| error.code == PreferenceErrorCodeV1::ProposalStale)
        .unwrap_or(false);
    Ok(refusal_observations(refused, &root, &before))
}

/// Shared broker scenario: an McpAgent prepares a proposal, a human authorizes it, and the
/// returned closure exercises one acceptance-lifecycle refusal. No store is ever written
/// (the broker refuses before any apply).
fn broker_refusal(
    exercise: impl FnOnce(
        &mut PreferenceAcceptanceBroker,
        &crate::preferences::PreferenceProposalV1,
        &PreferenceActorV1,
    ) -> bool,
) -> Result<Vec<(&'static str, bool)>, String> {
    let root = ScenarioRoot::new();
    let service = root.open()?;
    let agent = actor(PreferenceActorKindV1::McpAgent);
    let prepared = service
        .prepare_proposal(
            set_user(true, Uuid::new_v4()),
            "request local human review".to_owned(),
            &agent,
        )
        .map_err(|error| format!("prepare_proposal: {error:?}"))?;
    let mut broker = PreferenceAcceptanceBroker::new(Uuid::new_v4());
    broker
        .register_prepared(&prepared.proposal, REPO_IDENTITY)
        .map_err(|error| format!("register_prepared: {error:?}"))?;
    broker
        .authorize_mcp_apply(
            &AuthorizeMcpPreferenceApplyV1 {
                proposal_id: prepared.proposal.proposal_id,
                proposal_digest: prepared.proposal.proposal_digest.clone(),
                originating_mcp_session: agent.session_id.clone(),
            },
            &actor(PreferenceActorKindV1::HumanCli),
            REPO_IDENTITY,
            AUTHORIZE_AT_MS,
        )
        .map_err(|error| format!("authorize_mcp_apply: {error:?}"))?;
    let before = root.store_fingerprint();
    let refused = exercise(&mut broker, &prepared.proposal, &agent);
    Ok(refusal_observations(refused, &root, &before))
}

/// acceptance-mismatch: once an apply invocation binds, a DIFFERENT invocation for the
/// same acceptance -> InvocationMismatch.
fn observe_acceptance_mismatch() -> Result<Vec<(&'static str, bool)>, String> {
    broker_refusal(|broker, proposal, agent| {
        // First invocation binds the acceptance.
        if broker
            .authorization_for_apply(
                proposal,
                &agent.session_id,
                REPO_IDENTITY,
                Uuid::new_v4(),
                WITHIN_TTL_MS,
            )
            .is_err()
        {
            return false;
        }
        // A different invocation is a mismatch.
        matches!(
            broker.authorization_for_apply(
                proposal,
                &agent.session_id,
                REPO_IDENTITY,
                Uuid::new_v4(),
                WITHIN_TTL_MS
            ),
            Err(PreferenceAcceptanceRefusal::InvocationMismatch)
        )
    })
}

/// acceptance-expired: an apply requested past the acceptance TTL -> Expired.
fn observe_acceptance_expired() -> Result<Vec<(&'static str, bool)>, String> {
    broker_refusal(|broker, proposal, agent| {
        matches!(
            broker.authorization_for_apply(
                proposal,
                &agent.session_id,
                REPO_IDENTITY,
                Uuid::new_v4(),
                PAST_TTL_MS
            ),
            Err(PreferenceAcceptanceRefusal::Expired)
        )
    })
}

/// acceptance-consumed: a one-shot acceptance handle consumed twice -> Consumed.
fn observe_acceptance_consumed() -> Result<Vec<(&'static str, bool)>, String> {
    broker_refusal(|broker, proposal, agent| {
        let handle = match broker.authorization_for_apply(
            proposal,
            &agent.session_id,
            REPO_IDENTITY,
            Uuid::new_v4(),
            WITHIN_TTL_MS,
        ) {
            Ok(handle) => handle,
            Err(_) => return false,
        };
        if broker.consume(&handle).is_err() {
            return false;
        }
        matches!(
            broker.consume(&handle),
            Err(PreferenceAcceptanceRefusal::Consumed)
        )
    })
}

/// request-id-conflict: a trusted human commits with a request id, then a second commit
/// reuses that request id with a DIFFERENT value -> IdempotencyConflict; the committed
/// value is unchanged.
fn observe_request_id_conflict() -> Result<Vec<(&'static str, bool)>, String> {
    let root = ScenarioRoot::new();
    let mut service = root.open()?;
    let request_id = Uuid::new_v4();
    let human = actor(PreferenceActorKindV1::HumanGui);
    service
        .mutate(set_user(true, request_id), &human)
        .map_err(|error| format!("first commit: {error:?}"))?;
    let before = root.store_fingerprint();
    // Reuse the request id with a different value: a conflict, not an idempotent replay.
    let refusal = service.mutate(set_user(false, request_id), &human).err();
    let refused = refusal
        .map(|error| error.code == PreferenceErrorCodeV1::IdempotencyConflict)
        .unwrap_or(false);
    Ok(refusal_observations(refused, &root, &before))
}

/// Family negative control (PM059, genuinely falsifying): a trusted HumanGui direct
/// mutation legitimately COMMITS (changed, a store is written), so the refusal /
/// no-unauthorized-write predicates are not constant -- the authority gate distinguishes a
/// trusted human write from a refused portable one. Returns true when the trusted write
/// applied.
pub fn authority_negative_control() -> Result<bool, String> {
    let root = ScenarioRoot::new();
    let mut service = root.open()?;
    let result = service
        .mutate(
            set_user(true, Uuid::new_v4()),
            &actor(PreferenceActorKindV1::HumanGui),
        )
        .map_err(|error| format!("trusted commit: {error:?}"))?;
    // The trusted write applied and a real store now exists (the refusal predicates are
    // therefore not constant-true).
    Ok(result.changed && !root.store_fingerprint().is_empty())
}
