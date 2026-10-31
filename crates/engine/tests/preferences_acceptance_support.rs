//! Focused integration proof for the GP-CM05E acceptance-support preconditions.
//!
//! These tests live in their own `--test preferences_acceptance_support` target (not
//! as a `#[cfg(test)]` module inside the engine library) so a routine acceptance
//! proof can run exactly them without compiling the whole engine unit-test harness.
//! They exercise only the PUBLIC `acceptance_support` API, which is the same surface
//! the `gp_acceptance_driver` binary consumes.

use eda_engine::preferences::acceptance_support::backup_restore::{
    BackupRestoreVariant, backup_restore_negative_control, backup_restore_observe,
    backup_restore_setup, store_generation,
};
use eda_engine::preferences::acceptance_support::downgrade_preservation::{
    DowngradeVariant, downgrade_negative_control, downgrade_observe, downgrade_setup,
};
use eda_engine::preferences::acceptance_support::human_agent_authority::{
    AuthorityVariant, authority_negative_control, authority_observe,
};
use eda_engine::preferences::acceptance_support::portable_upgrade::{
    PortableCollisionVariant, UpgradeVariant, portable_collision_negative_control,
    portable_collision_observe, portable_collision_setup, upgrade_negative_control,
    upgrade_observe, upgrade_setup,
};
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

#[test]
fn backup_restore_restore_commits_through_the_journal_and_refuses_stale() {
    // The write pattern-setter: a full seed -> exact-backup -> diverge -> restore
    // through the engine's own committed-store path, with a stale-plan refusal as
    // the PM059 negative control. Runs setup+observe in a throwaway work root.
    let work = std::env::temp_dir().join(format!("gp-cm05e-br-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&work).expect("create work root");
    let outcome = (|| {
        backup_restore_setup(BackupRestoreVariant::Restore, &work)?;
        let diverged_gen = store_generation(&work);
        let observed = backup_restore_observe(BackupRestoreVariant::Restore, &work)?;
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

fn run_backup_restore(variant: BackupRestoreVariant) -> Result<Vec<(&'static str, bool)>, String> {
    let work = std::env::temp_dir().join(format!("gp-cm05e-br-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&work).expect("create work root");
    let result = (|| {
        backup_restore_setup(variant, &work)?;
        backup_restore_observe(variant, &work)
    })();
    let _ = std::fs::remove_dir_all(&work);
    result
}

#[test]
fn every_backup_restore_variant_yields_its_matrix_observations() {
    let expected: &[(BackupRestoreVariant, &[&str])] = &[
        (
            BackupRestoreVariant::ExactBackup,
            &["all-bytes-preserved", "no-project-write"],
        ),
        (
            BackupRestoreVariant::Restore,
            &[
                "preview-pure",
                "backup-before-restore",
                "exact-generation-authority",
            ],
        ),
        (
            BackupRestoreVariant::ReverseRestore,
            &["original-bytes-restored"],
        ),
        (BackupRestoreVariant::StalePreview, &["refusal", "no-write"]),
        (
            BackupRestoreVariant::IncompleteBackup,
            &["refusal", "no-write"],
        ),
    ];
    for (variant, ids) in expected {
        let observed =
            run_backup_restore(*variant).unwrap_or_else(|error| panic!("{variant:?}: {error}"));
        assert_eq!(
            observed.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            ids.to_vec(),
            "{variant:?} observation ids"
        );
        for (id, value) in observed {
            assert!(value, "{variant:?} observation {id} must hold");
        }
    }
}

#[test]
fn well_formed_backup_restore_succeeds_as_the_family_negative_control() {
    // PM059 genuinely-falsifying control: a well-formed backup plans and a fresh plan
    // commits, so the stale-preview / incomplete-backup refusal predicates are not
    // constant-true.
    let work = std::env::temp_dir().join(format!("gp-cm05e-br-neg-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&work).expect("create work root");
    let result = backup_restore_negative_control(&work);
    let _ = std::fs::remove_dir_all(&work);

    let (plan_ok, commit_ok) = result.expect("negative control runs");
    assert!(
        plan_ok,
        "an intact backup must plan without BackupIncomplete (incomplete-backup refusal is falsifiable)"
    );
    assert!(
        commit_ok,
        "a fresh plan must commit without RestorePreviewStale (stale-preview refusal is falsifiable)"
    );
}

fn run_downgrade(variant: DowngradeVariant) -> Result<Vec<(&'static str, bool)>, String> {
    let work = std::env::temp_dir().join(format!("gp-cm05e-dg-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&work).expect("create work root");
    let result = (|| {
        downgrade_setup(variant, &work)?;
        downgrade_observe(variant, &work)
    })();
    let _ = std::fs::remove_dir_all(&work);
    result
}

#[test]
fn every_downgrade_preservation_c_variant_yields_its_matrix_observations() {
    let expected: &[(DowngradeVariant, &[&str])] = &[
        (
            DowngradeVariant::RestoreBothDirections,
            &["original-and-restored-digests", "no-silent-rewrite"],
        ),
        (
            DowngradeVariant::UnknownField,
            &["inactive", "exact-bytes-survive"],
        ),
        (
            DowngradeVariant::FutureVersion,
            &["typed-refusal", "exact-bytes-survive"],
        ),
        (
            DowngradeVariant::UnknownKey,
            &["inactive", "exact-bytes-survive"],
        ),
    ];
    for (variant, ids) in expected {
        let observed =
            run_downgrade(*variant).unwrap_or_else(|error| panic!("{variant:?}: {error}"));
        assert_eq!(
            observed.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            ids.to_vec(),
            "{variant:?} observation ids"
        );
        for (id, value) in observed {
            assert!(value, "{variant:?} observation {id} must hold");
        }
    }
}

#[test]
fn downgrade_known_value_is_active_as_the_family_negative_control() {
    // PM059 genuinely-falsifying control: a registered known value IS active, so the
    // `inactive` predicate is not constant-true.
    let work = std::env::temp_dir().join(format!("gp-cm05e-dg-neg-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&work).expect("create work root");
    let result = downgrade_negative_control(&work);
    let _ = std::fs::remove_dir_all(&work);

    assert!(
        result.expect("negative control runs"),
        "a registered known value must be active (the inactive predicate is falsifiable)"
    );
}

#[test]
fn alias_migration_yields_its_matrix_observations() {
    // The upgrade family: a retired-alias store plans a pure one-alias move, commits it
    // with a pre-migration backup, and atomically promotes the head.
    let work = std::env::temp_dir().join(format!("gp-cm05e-up-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&work).expect("create work root");
    let observed = (|| {
        upgrade_setup(UpgradeVariant::AliasMigration, &work)?;
        upgrade_observe(UpgradeVariant::AliasMigration, &work)
    })();
    let _ = std::fs::remove_dir_all(&work);

    let observed = observed.expect("alias-migration runs");
    assert_eq!(
        observed.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        vec![
            "pure-plan",
            "exact-pre-migration-backup",
            "one-live-alias",
            "atomic-promotion"
        ]
    );
    for (id, value) in observed {
        assert!(value, "alias-migration observation {id} must hold");
    }
}

#[test]
fn alias_free_store_moves_no_alias_as_the_upgrade_negative_control() {
    // PM059 genuinely-falsifying control: a store with no retired alias plans ZERO alias
    // moves, so `one-live-alias` is not constant-true.
    let work = std::env::temp_dir().join(format!("gp-cm05e-up-neg-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&work).expect("create work root");
    let result = upgrade_negative_control(&work);
    let _ = std::fs::remove_dir_all(&work);

    assert!(
        result.expect("negative control runs"),
        "a store with no retired alias must produce no alias move (one-live-alias is falsifiable)"
    );
}

fn run_portable_collision(
    variant: PortableCollisionVariant,
) -> Result<Vec<(&'static str, bool)>, String> {
    let work = std::env::temp_dir().join(format!("gp-cm05e-pc-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&work).expect("create work root");
    let result = (|| {
        portable_collision_setup(variant, &work)?;
        portable_collision_observe(variant, &work)
    })();
    let _ = std::fs::remove_dir_all(&work);
    result
}

#[test]
fn every_portable_collision_variant_yields_its_matrix_observations() {
    let expected: &[(PortableCollisionVariant, &[&str])] = &[
        (
            PortableCollisionVariant::AliasCollision,
            &[
                "both-values-retained",
                "whole-operation-refused",
                "no-side-effects",
            ],
        ),
        (
            PortableCollisionVariant::CapabilityInjection,
            &["authority-refused", "unknown-data-preserved"],
        ),
    ];
    for (variant, ids) in expected {
        let observed =
            run_portable_collision(*variant).unwrap_or_else(|error| panic!("{variant:?}: {error}"));
        assert_eq!(
            observed.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            ids.to_vec(),
            "{variant:?} observation ids"
        );
        for (id, value) in observed {
            assert!(value, "{variant:?} observation {id} must hold");
        }
    }
}

#[test]
fn registered_value_is_active_as_the_portable_collision_negative_control() {
    // PM059 genuinely-falsifying control: a registered known value IS active, so the
    // `authority-refused` "absent from every active partition" predicate is not constant.
    let work = std::env::temp_dir().join(format!("gp-cm05e-pc-neg-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&work).expect("create work root");
    let result = portable_collision_negative_control(&work);
    let _ = std::fs::remove_dir_all(&work);

    assert!(
        result.expect("negative control runs"),
        "a registered known value must be active (authority-refused is falsifiable)"
    );
}

#[test]
fn every_human_agent_authority_variant_refuses_without_an_unauthorized_write() {
    // Every B service-refusal variant drives the engine product service / acceptance
    // broker into a typed refusal and writes nothing. Each is self-contained in a
    // throwaway store (no work root needed).
    let variants = [
        AuthorityVariant::SpoofedActor,
        AuthorityVariant::DirectMcpSetReset,
        AuthorityVariant::StaleProposal,
        AuthorityVariant::AcceptanceMismatch,
        AuthorityVariant::AcceptanceExpired,
        AuthorityVariant::AcceptanceConsumed,
        AuthorityVariant::RequestIdConflict,
        AuthorityVariant::DaemonRestart,
    ];
    for variant in variants {
        let observed =
            authority_observe(variant).unwrap_or_else(|error| panic!("{variant:?}: {error}"));
        assert_eq!(
            observed.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            vec![
                "no-unauthorized-write",
                "safe-error-details",
                "project-unchanged"
            ],
            "{variant:?} observation ids"
        );
        for (id, value) in observed {
            assert!(value, "{variant:?} observation {id} must hold");
        }
    }
}

#[test]
fn trusted_human_write_commits_as_the_authority_negative_control() {
    // PM059 genuinely-falsifying control: a trusted HumanGui direct mutation legitimately
    // commits (changed + a store is written), so the refusal / no-unauthorized-write
    // predicates are not constant-true.
    assert!(
        authority_negative_control().expect("negative control runs"),
        "a trusted human write must commit (the authority refusals are falsifiable)"
    );
}
