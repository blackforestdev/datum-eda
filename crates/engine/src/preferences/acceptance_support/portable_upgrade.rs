//! GP-CM05E Phase-3 engine families: upgrade (alias-migration) + portable-collision
//! (alias-collision, capability-injection).
//!
//! These are the retired-alias and ambiguous-portable-identity families. They share
//! backup_restore's store backbone (repo, metadata, fingerprint) and add the
//! migration/collision and capability-smuggle preconditions.
//!
//! PROVENANCE (the honest story each hand-built state tells, per the PM ruling):
//!   * alias-migration / alias-collision = a store a DIFFERENT (older) engine version
//!     wrote, holding the REAL production-active retired alias `console_duration` for its
//!     live key `datum.console.feedback_duration` (declared in the active catalog,
//!     catalog_part_one.rs). The retired-alias state is seeded through the engine's OWN
//!     write_generation (the authorized acceptance widening) exactly as the repository's
//!     own migration unit tests do, so nothing escapes the journal. This family anchors
//!     on the real active alias deliberately: a custom dotted registry would have MASKED
//!     the bare-alias migration defect (dat-preferences-bare-alias-migration-mv1k) that
//!     building it surfaced -- migrate_partition now resolves a declared alias before
//!     rejecting its spelling, so the bare active alias migrates per GP-C04 §6. (The
//!     dotted-alias shape stays covered by the repository anchor unit tests,
//!     repository/tests.rs::migration_plan_is_pure.../alias_collision...)
//!   * capability-injection = a portable/foreign source smuggled a capability-claiming
//!     envelope through the LEGITIMATE unknown-data door (PutUnknown, committed through
//!     the engine's lease+journal -- the product Set rejects PutUnknown). This version
//!     must preserve the opaque bytes and must NEVER promote the capability-looking data
//!     into capability authority. Reserved Capability descriptors have no ordinary
//!     activation path, so the defense-in-depth property is that unknown data CLAIMING a
//!     capability identity still cannot become authoritative -- it is authority-refused.
//! A state with no such story would be fabrication and is not seeded.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::json;

use super::backup_restore::{preferences_root, repo, store_fingerprint, write_metadata};
use crate::preferences::repository::acceptance_constructions::{
    RepositoryState, next_generation_number, promote_head, write_generation,
};
use crate::preferences::repository::{
    GenerationRef, MigrationChange, MigrationRegistry, PreferenceMutation, PreferencePartition,
    PreferenceRepository, RepositoryError, RepositoryStatus, StoredPreferenceValue,
    UnknownEnvelope,
};

/// The live key whose active descriptor carries the real retired alias below.
const LIVE_ALIAS_KEY: &str = "datum.console.feedback_duration";
/// The real production-active retired alias the live key supersedes (bare spelling, as
/// the active catalog declares it; it migrates via the migrate_partition alias fix).
const RETIRED_ALIAS: &str = "console_duration";
/// A valid `feedback_duration` enum value distinct from the "6s" default, so the moved
/// value is observable.
const ALIAS_VALUE: &str = "10s";

/// A capability-claiming identity a portable/foreign source smuggled through the
/// unknown-data door. It is a foreign namespace (never a registered Datum key), so it is
/// unresolvable by construction and cannot gain a descriptor's authority.
const CAPABILITY_IDENTITY: &str = "vendor.capability.signing_authority";
/// A registered, ordinarily-activatable known key used by the portable-collision family
/// negative control -- it IS active, which proves `authority-refused` is not constant.
const KNOWN_ACTIVE_KEY: &str = "datum.accessibility.reduced_motion";

fn capability_bytes() -> Vec<u8> {
    vec![0x7f, b'c', b'a', b'p', 0, 255, 42]
}

fn capability_envelope() -> UnknownEnvelope {
    UnknownEnvelope {
        identity: CAPABILITY_IDENTITY.to_owned(),
        provider: Some("vendor.portable_bundle".to_owned()),
        scope: "user".to_owned(),
        source_version: "1".to_owned(),
        required_extension: Some("vendor.capability".to_owned()),
        ordering: None,
        exact_bytes: capability_bytes(),
    }
}

/// The current head generation, tolerating a `MigrationRequired` store: a retired-alias
/// store is `MigrationRequired` (not `Ready`), yet still has an inspectable head
/// generation that `plan_migration` needs as its expected head.
fn head_generation(repository: &PreferenceRepository) -> Result<GenerationRef, String> {
    match repository.inspect() {
        RepositoryStatus::Ready(snapshot) => Ok(snapshot.generation),
        RepositoryStatus::MigrationRequired { snapshot, .. } => Ok(snapshot.generation),
        other => Err(format!("store has no inspectable generation: {other:?}")),
    }
}

/// Seed, through the engine's own write_generation, a generation an older engine wrote
/// holding the given user keys (a receipt-less valid generation, exactly as the
/// repository's migration unit tests construct). No product Set can place a retired
/// alias, which is why this goes through the authorized acceptance widening.
fn seed_user_generation(work_root: &Path, keys: &[(&str, &str)]) -> Result<(), String> {
    let root = preferences_root(work_root);
    let repository = repo(work_root);
    let initial = repository
        .initialize(&write_metadata("GP-CM05E portable/upgrade init"))
        .map_err(|error| format!("initialize: {error:?}"))?;
    let next = next_generation_number(&root);
    let mut state = RepositoryState::empty();
    for (key, value) in keys {
        state.user.insert(
            (*key).to_owned(),
            StoredPreferenceValue {
                schema_version: 1,
                value: json!(*value),
            },
        );
    }
    let generation = write_generation(
        &root,
        &initial.repository_id,
        next,
        Some(&initial),
        "old-datum",
        &state,
        &BTreeMap::new(),
    )
    .map_err(|error| format!("write_generation: {error:?}"))?;
    promote_head(&root, &generation).map_err(|error| format!("promote_head: {error:?}"))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// upgrade / alias-migration
// ---------------------------------------------------------------------------

/// The upgrade variants the engine driver can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeVariant {
    AliasMigration,
}

impl UpgradeVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AliasMigration => "alias-migration",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "alias-migration" => Some(Self::AliasMigration),
            _ => None,
        }
    }
}

/// SETUP phase for an upgrade variant: seed the retired-alias store.
pub fn upgrade_setup(variant: UpgradeVariant, work_root: &Path) -> Result<(), String> {
    match variant {
        UpgradeVariant::AliasMigration => {
            seed_user_generation(work_root, &[(RETIRED_ALIAS, ALIAS_VALUE)])
        }
    }
}

/// OBSERVE phase for an upgrade variant.
pub fn upgrade_observe(
    variant: UpgradeVariant,
    work_root: &Path,
) -> Result<Vec<(&'static str, bool)>, String> {
    match variant {
        UpgradeVariant::AliasMigration => observe_alias_migration(work_root),
    }
}

fn observe_alias_migration(work_root: &Path) -> Result<Vec<(&'static str, bool)>, String> {
    let root = preferences_root(work_root);
    let repository = repo(work_root);
    let generation = head_generation(&repository)?;

    // pure-plan: planning writes nothing and flags that a pre-migration backup is owed.
    let before = store_fingerprint(&root);
    let plan = repository
        .plan_migration(&generation, &MigrationRegistry::default())
        .map_err(|error| format!("plan_migration: {error:?}"))?;
    let after = store_fingerprint(&root);
    let pure_plan = before == after && plan.pre_migration_backup_required;

    // one-live-alias: exactly one alias move, from the retired alias to the live key.
    let alias_moves: Vec<_> = plan
        .changes
        .iter()
        .filter(|change| matches!(change, MigrationChange::AliasMoved { .. }))
        .collect();
    let one_live_alias = alias_moves.len() == 1
        && matches!(
            alias_moves[0],
            MigrationChange::AliasMoved { retired_key, live_key }
                if retired_key == RETIRED_ALIAS && live_key == LIVE_ALIAS_KEY
        );

    // Commit the migration: it creates the pre-migration backup and promotes the head.
    let (migrated, backup) = repository
        .commit_migration(&plan, &write_metadata("GP-CM05E alias-migration commit"))
        .map_err(|error| format!("commit_migration: {error:?}"))?;
    let snapshot = match repository.inspect() {
        RepositoryStatus::Ready(snapshot) => snapshot,
        other => return Err(format!("store is not Ready after migration: {other:?}")),
    };

    // atomic-promotion: the head is the single new migrated generation; the retired alias
    // is gone and the live key carries the moved value.
    let atomic_promotion = snapshot.generation == migrated
        && !snapshot.user.contains_key(RETIRED_ALIAS)
        && snapshot
            .user
            .get(LIVE_ALIAS_KEY)
            .is_some_and(|stored| stored.value == json!(ALIAS_VALUE));

    // exact-pre-migration-backup: restoring the returned backup over the migrated store
    // reverts exactly the two alias keys -- the backup still holds the retired alias, so
    // it captured the pre-migration state exactly (a post-migration backup would hold the
    // live key instead).
    let restore_plan = repository
        .plan_restore(&migrated, &backup)
        .map_err(|error| format!("plan_restore (backup): {error:?}"))?;
    let changed = &restore_plan.changed_keys;
    let exact_pre_migration_backup = changed.len() == 2
        && changed.iter().any(|key| key.contains(RETIRED_ALIAS))
        && changed.iter().any(|key| key.contains(LIVE_ALIAS_KEY));

    Ok(vec![
        ("pure-plan", pure_plan),
        ("exact-pre-migration-backup", exact_pre_migration_backup),
        ("one-live-alias", one_live_alias),
        ("atomic-promotion", atomic_promotion),
    ])
}

/// Upgrade family negative control (PM059, genuinely falsifying): a store with NO retired
/// alias (only the live key) plans with ZERO alias moves, so `one-live-alias` is not
/// constant -- the alias detection distinguishes a store that needs migrating from one
/// that does not. Returns true when the alias-free store correctly produces no move.
pub fn upgrade_negative_control(work_root: &Path) -> Result<bool, String> {
    seed_user_generation(work_root, &[(LIVE_ALIAS_KEY, ALIAS_VALUE)])?;
    let repository = repo(work_root);
    let generation = head_generation(&repository)?;
    let plan = repository
        .plan_migration(&generation, &MigrationRegistry::default())
        .map_err(|error| format!("plan_migration: {error:?}"))?;
    Ok(!plan
        .changes
        .iter()
        .any(|change| matches!(change, MigrationChange::AliasMoved { .. })))
}

// ---------------------------------------------------------------------------
// portable-collision / alias-collision + capability-injection
// ---------------------------------------------------------------------------

/// The portable-collision variants the engine driver can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortableCollisionVariant {
    AliasCollision,
    CapabilityInjection,
}

impl PortableCollisionVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AliasCollision => "alias-collision",
            Self::CapabilityInjection => "capability-injection",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "alias-collision" => Some(Self::AliasCollision),
            "capability-injection" => Some(Self::CapabilityInjection),
            _ => None,
        }
    }
}

/// SETUP phase for a portable-collision variant.
pub fn portable_collision_setup(
    variant: PortableCollisionVariant,
    work_root: &Path,
) -> Result<(), String> {
    match variant {
        // Seed BOTH the retired alias and its live key: an ambiguous portable identity.
        PortableCollisionVariant::AliasCollision => seed_user_generation(
            work_root,
            &[(RETIRED_ALIAS, ALIAS_VALUE), (LIVE_ALIAS_KEY, ALIAS_VALUE)],
        ),
        // Smuggle a capability-claiming envelope through the unknown-data door, committed
        // through the engine's own lease+journal (the product Set rejects PutUnknown).
        PortableCollisionVariant::CapabilityInjection => {
            let repository = repo(work_root);
            let initial = repository
                .initialize(&write_metadata("GP-CM05E capability-injection init"))
                .map_err(|error| format!("initialize: {error:?}"))?;
            repository
                .commit_mutations(
                    &initial,
                    &[PreferenceMutation::PutUnknown(capability_envelope())],
                    &write_metadata("GP-CM05E capability-injection smuggle"),
                )
                .map_err(|error| format!("commit capability envelope: {error:?}"))?;
            Ok(())
        }
    }
}

/// OBSERVE phase for a portable-collision variant.
pub fn portable_collision_observe(
    variant: PortableCollisionVariant,
    work_root: &Path,
) -> Result<Vec<(&'static str, bool)>, String> {
    match variant {
        PortableCollisionVariant::AliasCollision => observe_alias_collision(work_root),
        PortableCollisionVariant::CapabilityInjection => observe_capability_injection(work_root),
    }
}

fn observe_alias_collision(work_root: &Path) -> Result<Vec<(&'static str, bool)>, String> {
    let root = preferences_root(work_root);
    let repository = repo(work_root);
    let generation = head_generation(&repository)?;

    // whole-operation-refused + no-side-effects: planning the migration refuses the whole
    // operation with AliasCollision and writes nothing.
    let before = store_fingerprint(&root);
    let outcome = repository.plan_migration(&generation, &MigrationRegistry::default());
    let after = store_fingerprint(&root);
    let whole_operation_refused = matches!(
        &outcome,
        Err(RepositoryError::AliasCollision(alias)) if alias == RETIRED_ALIAS
    );
    let no_side_effects = before == after;

    // both-values-retained: both the retired alias and the live key survive in the store.
    let snapshot = match repository.inspect() {
        RepositoryStatus::MigrationRequired { snapshot, .. } => snapshot,
        RepositoryStatus::Ready(snapshot) => snapshot,
        other => return Err(format!("store is not inspectable: {other:?}")),
    };
    let both_values_retained =
        snapshot.user.contains_key(RETIRED_ALIAS) && snapshot.user.contains_key(LIVE_ALIAS_KEY);

    Ok(vec![
        ("both-values-retained", both_values_retained),
        ("whole-operation-refused", whole_operation_refused),
        ("no-side-effects", no_side_effects),
    ])
}

fn observe_capability_injection(work_root: &Path) -> Result<Vec<(&'static str, bool)>, String> {
    let repository = repo(work_root);
    let snapshot = match repository.inspect() {
        RepositoryStatus::Ready(snapshot) => snapshot,
        other => return Err(format!("store is not Ready: {other:?}")),
    };

    // authority-refused: the capability-claiming identity is preserved only as opaque
    // unknown data -- it is absent from EVERY active partition (user, installation), so it
    // is never promoted into an active/authoritative value. The identity is a foreign
    // namespace with no descriptor, so it is also unresolvable by construction.
    let authority_refused = snapshot.unknown_envelopes.contains_key(CAPABILITY_IDENTITY)
        && !snapshot.user.contains_key(CAPABILITY_IDENTITY)
        && !snapshot.installation.contains_key(CAPABILITY_IDENTITY);

    // unknown-data-preserved: the smuggled bytes are byte-identical.
    let unknown_data_preserved = repository
        .read_unknown_exact(&snapshot, CAPABILITY_IDENTITY)
        .map(|bytes| bytes == capability_bytes())
        .unwrap_or(false);

    Ok(vec![
        ("authority-refused", authority_refused),
        ("unknown-data-preserved", unknown_data_preserved),
    ])
}

/// Portable-collision family negative control (PM059, genuinely falsifying): a registered
/// known value IS active (present in the user partition), so the `authority-refused`
/// "absent from every active partition" predicate is not constant-true -- it distinguishes
/// an opaque capability-claiming envelope from a legitimately active registered value.
/// (Reserved Capability descriptors have no ordinary activation path, so the control uses
/// an ordinarily-activatable known key.)
pub fn portable_collision_negative_control(work_root: &Path) -> Result<bool, String> {
    let repository = repo(work_root);
    let initial = repository
        .initialize(&write_metadata("GP-CM05E portable-collision neg init"))
        .map_err(|error| format!("initialize: {error:?}"))?;
    repository
        .commit_mutations(
            &initial,
            &[PreferenceMutation::Set {
                partition: PreferencePartition::User,
                key: crate::preferences::PreferenceKey::parse(KNOWN_ACTIVE_KEY)
                    .expect("stable active key"),
                value: json!(true),
            }],
            &write_metadata("GP-CM05E portable-collision neg known value"),
        )
        .map_err(|error| format!("commit known: {error:?}"))?;
    let snapshot = match repository.inspect() {
        RepositoryStatus::Ready(snapshot) => snapshot,
        other => return Err(format!("store is not Ready: {other:?}")),
    };
    Ok(snapshot.user.contains_key(KNOWN_ACTIVE_KEY))
}
