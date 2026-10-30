//! Focused integration proof for the GP-CM05E acceptance-support preconditions.
//!
//! These tests live in their own `--test preferences_acceptance_support` target (not
//! as a `#[cfg(test)]` module inside the engine library) so a routine acceptance
//! proof can run exactly them without compiling the whole engine unit-test harness.
//! They exercise only the PUBLIC `acceptance_support` API, which is the same surface
//! the `gp_acceptance_driver` binary consumes.

use eda_engine::preferences::acceptance_support::{
    MANAGED_INACTIVE_REASON, MANAGED_OFFLINE_OBSERVATIONS, ManagedOfflineVariant,
    managed_active_negative_control, managed_offline_scenario, observe_managed_offline,
};
use eda_engine::preferences::{ContributionDisposition, ResolutionOutcome};
use serde_json::json;

const VARIANTS: [ManagedOfflineVariant; 4] = [
    ManagedOfflineVariant::Inactive,
    ManagedOfflineVariant::Expired,
    ManagedOfflineVariant::Unavailable,
    ManagedOfflineVariant::Offline,
];

#[test]
fn every_managed_offline_variant_resolves_as_the_case_requires() {
    for variant in VARIANTS {
        let scenario = managed_offline_scenario(variant);
        let explanation = scenario.resolve();

        let managed = explanation
            .considered
            .iter()
            .find(|fact| fact.id == "managed")
            .expect("the managed contribution is considered");
        // The pinned reason string still matches the resolver's wording, and the
        // managed contribution is disclosed with its inactive provider state.
        assert_eq!(managed.reason, MANAGED_INACTIVE_REASON, "{variant:?}");
        assert_eq!(
            managed.provider_state,
            Some(scenario.expected_provider_state),
            "{variant:?}"
        );
        assert_ne!(
            managed.disposition,
            ContributionDisposition::Effective,
            "{variant:?}"
        );

        // The ordinary user value stays effective; the managed Pin never wins.
        assert_eq!(
            explanation.outcome,
            ResolutionOutcome::Effective {
                value: json!("imperial"),
                winning_fact_ids: vec!["user".to_owned()],
            },
            "{variant:?}"
        );

        // All four observations hold, in the matrix's order.
        let observed = observe_managed_offline(&explanation, &scenario);
        assert_eq!(
            observed.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            MANAGED_OFFLINE_OBSERVATIONS.to_vec()
        );
        for (id, value) in observed {
            assert!(value, "{variant:?} observation {id} must hold");
        }
    }
}

#[test]
fn active_managed_generation_is_the_falsifying_negative_control() {
    // PM059 negative control: an ACTIVE managed generation legitimately wins, so
    // every managed-offline observation must flip to false. This proves the
    // predicates are real, not tautological.
    let scenario = managed_active_negative_control();
    let explanation = scenario.resolve();

    // The Pin is now effective -- the managed value wins, not the user's.
    assert_eq!(
        explanation.outcome,
        ResolutionOutcome::Effective {
            value: json!("metric"),
            winning_fact_ids: vec!["managed".to_owned()],
        }
    );

    let observed = observe_managed_offline(&explanation, &scenario);
    for (id, value) in observed {
        assert!(!value, "negative-control observation {id} must be false");
    }
}
