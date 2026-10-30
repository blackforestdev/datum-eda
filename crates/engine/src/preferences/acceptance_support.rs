//! GP-CM05E production-acceptance TEST-PRECONDITION constructions.
//!
//! This module constructs TEST PRECONDITIONS only. Every function here assembles
//! the resolver facts, descriptor-registry selection and authority releases that
//! put the engine into the exact resolved state a production-acceptance case must
//! observe — and NOTHING here bypasses the one engine-owned mutation/journal path
//! (PRODUCT_MECHANICS "one mutation path"). The managed-offline family is a pure,
//! side-effect-free [`resolve_preference`] evaluation: there is no committed-store
//! write at all, so the single-writer law is not even engaged. A later write
//! family served by this module must still establish its precondition through the
//! engine's own `commit()` path (or the corrupt-store legitimate-damage pattern),
//! never through a private writer.
//!
//! It exists because the acceptance contract makes managed provider states
//! resolver-level only (GLOBAL_PREFERENCES_PRODUCT_SURFACE_CONTRACT §, "managed
//! fixtures … cannot activate a managed provider"): no CLI/MCP/GUI adapter can
//! inject a `ProviderGenerationState`, so the engine-surface acceptance driver
//! (`src/bin/gp_acceptance_driver.rs`) exercises these states directly. The
//! observation evaluator below is a READ-ONLY analysis of an already-resolved
//! [`PreferenceExplanation`]; it writes nothing.

use super::{
    AuthorityRelease, AuthorityReleaseLevel, AuthorityReleaseState, AvailableAction, Contribution,
    ContributionDisposition, DescriptorRegistry, FactProvenance, OrganizationDirective,
    OrganizationFact, PreferenceExplanation, PreferenceKey, ProviderGenerationState,
    ResolutionOutcome, ResolutionRequest, ResolutionSource, ValueDisclosure, ValueFact,
    active_v1_registry, resolve_preference,
};
use serde_json::{Value, json};

/// The pilot key: an ACTIVE production descriptor (`datum.units.system`, enum
/// {metric,imperial}) whose policy admits an Organization `Pin` directive and
/// whose source declaration includes the Organization source — the minimal honest
/// setting in which a managed Pin can be present yet refused for being inactive.
pub const PILOT_KEY: &str = "datum.units.system";

/// The resolver's truthful disclosure reason for a managed generation that is
/// authenticated but inactive. Pinned here so a wording change in the resolver is
/// caught by this module's test rather than silently drifting the driver.
pub const MANAGED_INACTIVE_REASON: &str =
    "managed generation is inactive under its authenticated provider state";

/// A fully-constructed managed-offline precondition: the descriptor registry, the
/// key under resolution, the contributions and authority releases to resolve, plus
/// the facts the driver observes against. Owning the registry keeps the borrow in
/// [`resolve`](ManagedOfflineScenario::resolve) sound for the caller.
pub struct ManagedOfflineScenario {
    pub registry: DescriptorRegistry,
    pub key: PreferenceKey,
    pub contributions: Vec<Contribution>,
    pub authority_releases: Vec<AuthorityRelease>,
    /// The id of the managed (Organization) contribution under test.
    pub managed_fact_id: String,
    /// The value the ordinary user contribution carries — the value that must
    /// remain effective because the managed Pin never participates.
    pub expected_effective_value: Value,
    /// The inactive provider state the managed generation carries.
    pub expected_provider_state: ProviderGenerationState,
}

impl ManagedOfflineScenario {
    /// Resolve this precondition through the ordinary engine resolver. Pure: no I/O.
    pub fn resolve(&self) -> PreferenceExplanation {
        resolve_preference(ResolutionRequest {
            registry: &self.registry,
            key: self.key.clone(),
            contributions: &self.contributions,
            authority_releases: &self.authority_releases,
            machine_scope: "machine",
        })
    }
}

fn pilot_key() -> PreferenceKey {
    PreferenceKey::parse(PILOT_KEY).expect("pilot key is a stable Datum identity")
}

/// Construct the managed-offline **inactive** precondition: an ordinary User value
/// (`imperial`) plus an authenticated Organization `Pin(metric)` whose generation
/// is `Revoked`, under a FullManagement authority release. The release is active
/// and sufficient, so the ONLY reason the Pin does not win is its inactive
/// generation state — exactly the truth the case must observe. Constructs TEST
/// PRECONDITIONS only; it performs no store mutation (pure resolver inputs).
pub fn managed_offline_inactive_scenario() -> ManagedOfflineScenario {
    let key = pilot_key();
    let user_value = json!("imperial");
    let user = Contribution::Value(ValueFact {
        id: "user".to_owned(),
        key: key.clone(),
        source: ResolutionSource::User,
        value: user_value.clone(),
        provenance: FactProvenance {
            origin: "ordinary user contribution".to_owned(),
            provider: None,
            package: None,
            generation: None,
            actor: "owner".to_owned(),
            role: Some("user".to_owned()),
            observed_at: "2026-10-30T00:00:00Z".to_owned(),
            effective_from: None,
            effective_until: None,
            offline_valid_until: None,
            last_successful_contact: None,
            reason: "user-selected measurement system".to_owned(),
        },
        disclosure: ValueDisclosure::Disclosed,
        provider_state: None,
        context: None,
    });
    let managed = Contribution::Organization(OrganizationFact {
        id: "managed".to_owned(),
        key: key.clone(),
        organization: "acme".to_owned(),
        provenance: FactProvenance {
            origin: "managed-policy".to_owned(),
            provider: Some("acme-provider".to_owned()),
            package: Some("policy-1".to_owned()),
            generation: Some("7".to_owned()),
            actor: "admin".to_owned(),
            role: Some("organization administrator".to_owned()),
            observed_at: "2026-10-30T00:00:00Z".to_owned(),
            effective_from: Some("2026-10-01T00:00:00Z".to_owned()),
            effective_until: None,
            // An offline-validity horizon is carried on the cached fact: the
            // truthful inactive disclosure is computed from it with no live
            // provider contact (resolve_preference performs no I/O).
            offline_valid_until: Some("2026-10-08T00:00:00Z".to_owned()),
            last_successful_contact: Some("2026-10-01T00:00:00Z".to_owned()),
            reason: "shop policy".to_owned(),
        },
        authenticated: true,
        disclosure: ValueDisclosure::Disclosed,
        generation_state: ProviderGenerationState::Revoked,
        remaining_freedom: "user may appeal or revoke released authority".to_owned(),
        appeal_path: Some("manage AuthorityRelease".to_owned()),
        directive: OrganizationDirective::Pin(json!("metric")),
    });
    let release = AuthorityRelease {
        id: "release-1".to_owned(),
        releasing_user: "owner".to_owned(),
        organization: "acme".to_owned(),
        machine_scope: "machine".to_owned(),
        level: AuthorityReleaseLevel::FullManagement,
        effective_at: "2026-10-01".to_owned(),
        effective_until: None,
        attribution: "owner approval".to_owned(),
        revocable: true,
        state: AuthorityReleaseState::Active,
    };
    ManagedOfflineScenario {
        registry: active_v1_registry(),
        key,
        contributions: vec![user, managed],
        authority_releases: vec![release],
        managed_fact_id: "managed".to_owned(),
        expected_effective_value: user_value,
        expected_provider_state: ProviderGenerationState::Revoked,
    }
}

/// The ordered managed-offline observation ids, matching the acceptance matrix
/// variant inventory exactly.
pub const MANAGED_OFFLINE_OBSERVATIONS: [&str; 4] = [
    "state-disclosed",
    "inactive-source-never-wins",
    "no-management-ui",
    "no-network-required",
];

/// Read-only evaluation of the four managed-offline observations over an
/// already-resolved explanation. Writes nothing; reads only the explanation and
/// the scenario's expectations. Returned in [`MANAGED_OFFLINE_OBSERVATIONS`] order.
pub fn observe_managed_offline(
    explanation: &PreferenceExplanation,
    scenario: &ManagedOfflineScenario,
) -> Vec<(&'static str, bool)> {
    let managed = explanation
        .considered
        .iter()
        .find(|fact| fact.id == scenario.managed_fact_id);

    // state-disclosed: the managed contribution is present and truthfully
    // discloses its inactive provider state, reason and disclosed directive.
    let state_disclosed = managed.is_some_and(|fact| {
        matches!(
            fact.disposition,
            ContributionDisposition::Unavailable
                | ContributionDisposition::Expired
                | ContributionDisposition::Revoked
                | ContributionDisposition::Superseded
        ) && fact.reason == MANAGED_INACTIVE_REASON
            && fact.provider_state == Some(scenario.expected_provider_state)
            && fact.disclosure == ValueDisclosure::Disclosed
    });

    // inactive-source-never-wins: the managed contribution is not effective and is
    // absent from the winning ids; the ordinary user value stays effective.
    let inactive_never_wins = managed
        .is_some_and(|fact| fact.disposition != ContributionDisposition::Effective)
        && matches!(
            &explanation.outcome,
            ResolutionOutcome::Effective { value, winning_fact_ids }
                if *value == scenario.expected_effective_value
                    && !winning_fact_ids.contains(&scenario.managed_fact_id)
        );

    // no-management-ui: the resolution offers no in-app managed-provider
    // management surface. An inactive provider yields ordinary user-scope
    // primitives only; none of the managed-authority actions appear (consistent
    // with the excluded 'Manage Preferences' product boundary).
    let no_management_ui = ![
        AvailableAction::ManageAuthorityRelease,
        AvailableAction::AppealOrRevokeManagedPin,
        AvailableAction::ReviewAuthorityRequest,
    ]
    .iter()
    .any(|action| explanation.available_actions.contains(action));

    // no-network-required: the truthful inactive disclosure is derived from the
    // cached fact's offline-validity horizon; resolve_preference is a pure
    // function and contacts no provider.
    let no_network_required = managed.is_some_and(|fact| {
        fact.provenance.offline_valid_until.is_some()
            && fact.disposition != ContributionDisposition::Effective
    });

    vec![
        ("state-disclosed", state_disclosed),
        ("inactive-source-never-wins", inactive_never_wins),
        ("no-management-ui", no_management_ui),
        ("no-network-required", no_network_required),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_offline_inactive_pilot_resolves_as_the_case_requires() {
        let scenario = managed_offline_inactive_scenario();
        let explanation = scenario.resolve();

        // The pinned reason string still matches the resolver's wording.
        let managed = explanation
            .considered
            .iter()
            .find(|fact| fact.id == "managed")
            .expect("the managed contribution is considered");
        assert_eq!(managed.reason, MANAGED_INACTIVE_REASON);
        assert_eq!(managed.disposition, ContributionDisposition::Revoked);
        assert_eq!(
            managed.provider_state,
            Some(ProviderGenerationState::Revoked)
        );

        // The ordinary user value stays effective; the managed Pin never wins.
        assert_eq!(
            explanation.outcome,
            ResolutionOutcome::Effective {
                value: json!("imperial"),
                winning_fact_ids: vec!["user".to_owned()],
            }
        );

        // All four observations hold.
        let observed = observe_managed_offline(&explanation, &scenario);
        assert_eq!(
            observed.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            MANAGED_OFFLINE_OBSERVATIONS.to_vec()
        );
        for (id, value) in observed {
            assert!(value, "managed-offline/inactive observation {id} must hold");
        }
    }
}
