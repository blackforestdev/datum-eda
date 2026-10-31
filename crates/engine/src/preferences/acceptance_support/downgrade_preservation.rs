//! GP-CM05E Phase-3 WRITE families: downgrade-preservation.
//!
//! Extracted from `backup_restore` at the PM022 ceiling crossing (real ownership).
//! These variants share backup_restore's store/recovery helpers (seed, set_value,
//! backup, fingerprint, sidecar, current-generation) -- the shared store backbone --
//! and add the write_generation-seeded and format-version-damaged preconditions.
//!
//! PROVENANCE (the honest story each hand-built state tells, per the PM ruling):
//!   * unknown-key    = a key a FUTURE engine version registers, unregistered here.
//!   * future-version = a store a NEWER engine version wrote at a newer format_version.
//!
//! A state with no such story would be fabrication and is not seeded.

use std::path::Path;

use serde_json::{Value, json};

use super::backup_restore::{
    RESTORE_DIVERGED_VALUE, current_generation, preferences_root, read_sidecar, repo, seed_backed,
    set_value, store_fingerprint, write_metadata, write_sidecar,
};
use crate::preferences::PreferenceKey;
use crate::preferences::repository::acceptance_constructions::{
    RepositoryState, next_generation_number, promote_head, write_generation,
};
use crate::preferences::repository::{
    MutationReceipt, PreferenceMutation, PreferencePartition, RepositoryStatus,
    StoredPreferenceValue, UnknownEnvelope,
};

// ---------------------------------------------------------------------------
// downgrade-preservation (C variants). These share the recovery / commit journal
// machinery above (which is why they live here rather than in a separate module):
// restore-both-directions is a bidirectional recovery round-trip, and unknown-field
// preserves an opaque envelope committed through the repository journal (the product
// Set rejects PutUnknown, so it is committed repository-direct -- still the engine's
// own lease+journal path, never a private writer).
// ---------------------------------------------------------------------------

const UNKNOWN_IDENTITY: &str = "vendor.future.setting";
const UNITS_SYSTEM_CHANGED_KEY: &str = "user:datum.units.system";

fn unknown_bytes() -> Vec<u8> {
    vec![0, 255, b'{', b'\n', 17, 0, 99]
}

fn reduced_motion_key() -> PreferenceKey {
    PreferenceKey::parse("datum.accessibility.reduced_motion").expect("stable active key")
}

fn unknown_envelope() -> UnknownEnvelope {
    UnknownEnvelope {
        identity: UNKNOWN_IDENTITY.to_owned(),
        provider: Some("vendor.example".to_owned()),
        scope: "user".to_owned(),
        source_version: "9".to_owned(),
        required_extension: Some("vendor.future".to_owned()),
        ordering: Some("after:vendor.base".to_owned()),
        exact_bytes: unknown_bytes(),
    }
}

/// The downgrade-preservation variants the engine driver can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DowngradeVariant {
    RestoreBothDirections,
    UnknownField,
    FutureVersion,
    UnknownKey,
}

impl DowngradeVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RestoreBothDirections => "restore-both-directions",
            Self::UnknownField => "unknown-field",
            Self::FutureVersion => "future-version",
            Self::UnknownKey => "unknown-key",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "restore-both-directions" => Some(Self::RestoreBothDirections),
            "unknown-field" => Some(Self::UnknownField),
            "future-version" => Some(Self::FutureVersion),
            "unknown-key" => Some(Self::UnknownKey),
            _ => None,
        }
    }
}

/// The unregistered key seeded by unknown-key: a key a FUTURE engine version would
/// register, which THIS version must preserve-but-not-activate on read.
const UNREGISTERED_KEY: &str = "datum.future.unregistered_setting";
/// An unsupported future repository format version (this engine writes v1/v2 only).
const FUTURE_FORMAT_VERSION: u32 = 9999;

/// A minimal valid journaled receipt for a harness-seeded generation. Per the PM
/// seed ruling it is honestly named (gp-acceptance-driver) and records the operation,
/// generations and affected keys; per-key before/after digests are left empty because
/// the engine's digest helpers are intentionally OUTSIDE the authorized acceptance
/// widening surface (write_generation/promote_head/next_generation_number +
/// RepositoryState only) -- the omission is honest, never a fabricated digest.
fn seed_receipt(
    operation: &str,
    reason: &str,
    affected: Vec<String>,
    resulting: u64,
) -> MutationReceipt {
    MutationReceipt {
        operation: operation.to_owned(),
        actor: "gp-acceptance-driver".to_owned(),
        reason: reason.to_owned(),
        expected_generation: resulting.checked_sub(1),
        resulting_generation: resulting,
        affected_keys: affected,
        before_digests: std::collections::BTreeMap::new(),
        after_digests: std::collections::BTreeMap::new(),
        redacted_keys: Vec::new(),
        request_id: None,
        canonical_request_digest: None,
        actor_kind: None,
        local_actor_id: None,
        actor_session_id: None,
        invocation_id: None,
        expected_generation_ref: None,
        proposal_id: None,
        proposal_digest: None,
        acceptance_id: None,
        requesting_actor: None,
        accepting_actor: None,
        originating_mcp_session: None,
    }
}

/// SETUP phase for a downgrade-preservation variant.
pub fn downgrade_setup(variant: DowngradeVariant, work_root: &Path) -> Result<(), String> {
    match variant {
        // Same precondition as the backup-restore round-trip: seed, exact-backup, diverge.
        DowngradeVariant::RestoreBothDirections => {
            let backed = seed_backed(work_root)?;
            let backup = repo(work_root)
                .create_exact_backup(&backed, &write_metadata("GP-CM05E restore-both backup"))
                .map_err(|error| format!("create_exact_backup: {error:?}"))?;
            set_value(work_root, &backed, RESTORE_DIVERGED_VALUE)?;
            write_sidecar(work_root, &backup)
        }
        // Store a known value plus an opaque unknown envelope through the repository
        // journal (the product Set rejects PutUnknown).
        DowngradeVariant::UnknownField => {
            let repository = repo(work_root);
            let initial = repository
                .initialize(&write_metadata("GP-CM05E unknown-field init"))
                .map_err(|error| format!("initialize: {error:?}"))?;
            repository
                .commit_mutations(
                    &initial,
                    &[
                        PreferenceMutation::Set {
                            partition: PreferencePartition::User,
                            key: reduced_motion_key(),
                            value: json!(true),
                        },
                        PreferenceMutation::PutUnknown(unknown_envelope()),
                    ],
                    &write_metadata("GP-CM05E unknown-field store known+unknown"),
                )
                .map_err(|error| format!("commit known+unknown: {error:?}"))?;
            Ok(())
        }
        // future-version: a real store, then damage head's recorded format_version to an
        // unsupported value -- what a NEWER engine version would have written. Pure artifact
        // damage (no write_generation); digest-breaking is confined to this refusal case.
        DowngradeVariant::FutureVersion => {
            seed_backed(work_root)?;
            let head = preferences_root(work_root).join("head.json");
            let mut record: Value = serde_json::from_slice(
                &std::fs::read(&head).map_err(|error| format!("read head: {error}"))?,
            )
            .map_err(|error| format!("parse head: {error}"))?;
            record["generation"]["format_version"] = json!(FUTURE_FORMAT_VERSION);
            std::fs::write(
                &head,
                serde_json::to_vec(&record).map_err(|error| format!("serialize head: {error}"))?,
            )
            .map_err(|error| format!("write head: {error}"))?;
            Ok(())
        }
        // unknown-key: seed, through the engine's own write_generation, a VALID generation
        // holding a key a FUTURE engine version registers (unregistered here) -- a state the
        // product Set rejects, constructed via the authorized acceptance widening with a
        // valid journaled receipt.
        DowngradeVariant::UnknownKey => {
            let root = preferences_root(work_root);
            let repository = repo(work_root);
            let initial = repository
                .initialize(&write_metadata("GP-CM05E unknown-key init"))
                .map_err(|error| format!("initialize: {error:?}"))?;
            let next = next_generation_number(&root);
            let mut state = RepositoryState::empty();
            state.user.insert(
                UNREGISTERED_KEY.to_owned(),
                StoredPreferenceValue {
                    schema_version: 1,
                    value: json!("future-value"),
                },
            );
            state.receipts.push(seed_receipt(
                "SeedUnregisteredKey",
                "a future engine version's registered key, unregistered in this version",
                vec![format!("user:{UNREGISTERED_KEY}")],
                next,
            ));
            let generation = write_generation(
                &root,
                &initial.repository_id,
                next,
                Some(&initial),
                "gp-acceptance-driver",
                &state,
                &std::collections::BTreeMap::new(),
            )
            .map_err(|error| format!("write_generation: {error:?}"))?;
            promote_head(&root, &generation).map_err(|error| format!("promote_head: {error:?}"))?;
            Ok(())
        }
    }
}

/// OBSERVE phase for a downgrade-preservation variant.
pub fn downgrade_observe(
    variant: DowngradeVariant,
    work_root: &Path,
) -> Result<Vec<(&'static str, bool)>, String> {
    match variant {
        DowngradeVariant::RestoreBothDirections => observe_restore_both_directions(work_root),
        DowngradeVariant::UnknownField => observe_unknown_field(work_root),
        DowngradeVariant::FutureVersion => observe_future_version(work_root),
        DowngradeVariant::UnknownKey => observe_unknown_key(work_root),
    }
}

fn observe_future_version(work_root: &Path) -> Result<Vec<(&'static str, bool)>, String> {
    let root = preferences_root(work_root);
    let repository = repo(work_root);
    let before = store_fingerprint(&root);
    let status = repository.inspect();
    let after = store_fingerprint(&root);
    // typed-refusal: the unsupported version is refused as unreadable with a reason that
    // names the SPECIFIC unsupported version. (inspect() collapses load errors into
    // Unreadable, and load_snapshot -- which returns the typed UnsupportedRepositoryVersion
    // -- is not on the acceptance surface, so the version-typed refusal is observed through
    // the reason text.) The reason must contain the exact FUTURE_FORMAT_VERSION constant: a
    // generic corruption that merely mentions the word "version" cannot forge the precise
    // number this seed wrote, so the predicate can read false on a non-version Unreadable.
    let typed_refusal = match &status {
        RepositoryStatus::Unreadable(evidence) => {
            evidence.reason.contains(&FUTURE_FORMAT_VERSION.to_string())
        }
        _ => false,
    };
    // exact-bytes-survive: the refused inspect is pure -- every byte identical across it.
    let exact_bytes_survive = before == after;
    Ok(vec![
        ("typed-refusal", typed_refusal),
        ("exact-bytes-survive", exact_bytes_survive),
    ])
}

fn observe_unknown_key(work_root: &Path) -> Result<Vec<(&'static str, bool)>, String> {
    let repository = repo(work_root);
    let (snapshot, issues) = match repository.inspect() {
        RepositoryStatus::MigrationRequired { snapshot, issues } => (snapshot, issues),
        other => return Err(format!("expected MigrationRequired, got {other:?}")),
    };
    // inactive: the unregistered key is flagged to remain inactive (not applied as a value).
    let inactive = issues.iter().any(|issue| {
        issue.key == UNREGISTERED_KEY && issue.reason.contains("must remain inactive")
    });
    // exact-bytes-survive: the seeded value is preserved byte-for-byte in the loaded snapshot.
    let exact_bytes_survive = snapshot
        .user
        .get(UNREGISTERED_KEY)
        .is_some_and(|stored| stored.value == json!("future-value"));
    Ok(vec![
        ("inactive", inactive),
        ("exact-bytes-survive", exact_bytes_survive),
    ])
}

fn observe_restore_both_directions(work_root: &Path) -> Result<Vec<(&'static str, bool)>, String> {
    let repository = repo(work_root);
    let backed = read_sidecar(work_root)?; // B0 = generation 0 (backed value)
    let diverged = current_generation(&repository)?; // generation 1 (diverged value)
    let diverged_backup = repository
        .create_exact_backup(
            &diverged,
            &write_metadata("GP-CM05E restore-both diverged backup"),
        )
        .map_err(|error| format!("create_exact_backup (diverged): {error:?}"))?;

    // Forward restore to the backed state; it must change exactly the units key.
    let plan_forward = repository
        .plan_restore(&diverged, &backed)
        .map_err(|error| format!("plan_restore (forward): {error:?}"))?;
    let forward_changed = plan_forward.changed_keys.clone();
    let forward = repository
        .commit_restore(
            &plan_forward,
            &write_metadata("GP-CM05E restore-both forward"),
        )
        .map_err(|error| format!("commit_restore (forward): {error:?}"))?;
    let forward_matches = repository
        .plan_restore(&forward.generation, &backed)
        .map_err(|error| format!("verify (forward): {error:?}"))?
        .changed_keys
        .is_empty();

    // Reverse restore back to the diverged state; again exactly the units key.
    let plan_reverse = repository
        .plan_restore(&forward.generation, &diverged_backup)
        .map_err(|error| format!("plan_restore (reverse): {error:?}"))?;
    let reverse_changed = plan_reverse.changed_keys.clone();
    let reverse = repository
        .commit_restore(
            &plan_reverse,
            &write_metadata("GP-CM05E restore-both reverse"),
        )
        .map_err(|error| format!("commit_restore (reverse): {error:?}"))?;
    let reverse_matches = repository
        .plan_restore(&reverse.generation, &diverged_backup)
        .map_err(|error| format!("verify (reverse): {error:?}"))?
        .changed_keys
        .is_empty();

    // original-and-restored-digests: both directions land on digest-matching states.
    let original_and_restored_digests = forward_matches && reverse_matches;
    // no-silent-rewrite: each restore changed exactly the one expected key, nothing else.
    let expected = vec![UNITS_SYSTEM_CHANGED_KEY.to_owned()];
    let no_silent_rewrite = forward_changed == expected && reverse_changed == expected;

    Ok(vec![
        (
            "original-and-restored-digests",
            original_and_restored_digests,
        ),
        ("no-silent-rewrite", no_silent_rewrite),
    ])
}

fn observe_unknown_field(work_root: &Path) -> Result<Vec<(&'static str, bool)>, String> {
    let repository = repo(work_root);
    let before = current_generation(&repository)?;
    // An intervening known mutation (the "downgrade" write) must not disturb the
    // opaque envelope.
    repository
        .commit_mutations(
            &before,
            &[PreferenceMutation::Set {
                partition: PreferencePartition::User,
                key: reduced_motion_key(),
                value: json!(false),
            }],
            &write_metadata("GP-CM05E unknown-field intervening mutation"),
        )
        .map_err(|error| format!("intervening commit: {error:?}"))?;
    let snapshot = match repository.inspect() {
        RepositoryStatus::Ready(snapshot) => snapshot,
        other => return Err(format!("store is not Ready: {other:?}")),
    };

    // inactive: the unknown envelope is preserved opaquely but is NOT promoted to a
    // known active value in either partition.
    let inactive = snapshot.unknown_envelopes.contains_key(UNKNOWN_IDENTITY)
        && !snapshot.user.contains_key(UNKNOWN_IDENTITY)
        && !snapshot.installation.contains_key(UNKNOWN_IDENTITY);
    // exact-bytes-survive: the opaque bytes are byte-identical after the intervening
    // mutation.
    let exact_bytes_survive = repository
        .read_unknown_exact(&snapshot, UNKNOWN_IDENTITY)
        .map(|bytes| bytes == unknown_bytes())
        .unwrap_or(false);

    Ok(vec![
        ("inactive", inactive),
        ("exact-bytes-survive", exact_bytes_survive),
    ])
}

/// Family negative control (PM059, genuinely falsifying): a registered known value
/// IS active (present in the user partition), so the `inactive` predicate is not
/// constant-true -- it distinguishes an opaque unknown envelope from an active value.
pub fn downgrade_negative_control(work_root: &Path) -> Result<bool, String> {
    let repository = repo(work_root);
    let initial = repository
        .initialize(&write_metadata("GP-CM05E downgrade neg init"))
        .map_err(|error| format!("initialize: {error:?}"))?;
    repository
        .commit_mutations(
            &initial,
            &[PreferenceMutation::Set {
                partition: PreferencePartition::User,
                key: reduced_motion_key(),
                value: json!(true),
            }],
            &write_metadata("GP-CM05E downgrade neg known value"),
        )
        .map_err(|error| format!("commit known: {error:?}"))?;
    let snapshot = match repository.inspect() {
        RepositoryStatus::Ready(snapshot) => snapshot,
        other => return Err(format!("store is not Ready: {other:?}")),
    };
    Ok(snapshot
        .user
        .contains_key("datum.accessibility.reduced_motion"))
}
