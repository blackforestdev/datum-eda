//! Focused integration proof for the GP-CM05E acceptance-support preconditions.
//!
//! These tests live in their own `--test preferences_acceptance_support` target (not
//! as a `#[cfg(test)]` module inside the engine library) so a routine acceptance
//! proof can run exactly them without compiling the whole engine unit-test harness.
//! They exercise only the PUBLIC `acceptance_support` API, which is the same surface
//! the `gp_acceptance_driver` binary consumes.

use eda_engine::preferences::acceptance_support::{
    MANAGED_INACTIVE_REASON, MANAGED_OFFLINE_OBSERVATIONS, ManagedOfflineVariant,
    backup_restore_observe, backup_restore_setup, managed_active_negative_control,
    managed_offline_scenario, observe_managed_offline, store_generation,
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

#[test]
fn backup_restore_restore_commits_through_the_journal_and_refuses_stale() {
    // The write pattern-setter: a full seed -> exact-backup -> diverge -> restore
    // through the engine's own committed-store path, with a stale-plan refusal as
    // the PM059 negative control. Runs setup+observe in a throwaway work root.
    let work = std::env::temp_dir().join(format!("gp-cm05e-br-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&work).expect("create work root");
    let outcome = (|| {
        backup_restore_setup(&work)?;
        let diverged_gen = store_generation(&work);
        let observed = backup_restore_observe(&work)?;
        let restored_gen = store_generation(&work);
        Ok::<_, String>((diverged_gen, observed, restored_gen))
    })();
    let _ = std::fs::remove_dir_all(&work);

    let (diverged_gen, observed, restored_gen) = outcome.expect("backup-restore/restore runs");
    assert_eq!(
        diverged_gen,
        Some(1),
        "setup leaves the diverged generation 1"
    );
    assert_eq!(
        restored_gen,
        Some(2),
        "the restore commits a real new generation 2"
    );
    assert_eq!(
        observed.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        vec![
            "preview-pure",
            "backup-before-restore",
            "exact-generation-authority"
        ]
    );
    for (id, value) in observed {
        assert!(value, "backup-restore/restore observation {id} must hold");
    }
}
