//! GP-CM05E Phase-3 WRITE family: backup-restore/restore (commit()+journal).
//!
//! Extracted from `acceptance_support` as its own owned module (PM022). See the
//! banner below for the boundary rationale.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::json;
use sha2::{Digest, Sha256};

use super::pilot_key;
use crate::preferences::repository::{
    ExactBackupRef, GenerationRef, MutationMetadata, PreferenceRepository, RepositoryError,
    RepositoryStatus,
};
use crate::preferences::{GlobalPreferencesService, active_v1_registry};

// ===========================================================================
// GP-CM05E Phase-3 WRITE family: backup-restore/restore (commit()+journal).
//
// Unlike the pure-resolver managed-offline family, this precondition and its
// observed operation go through the engine's OWN committed-store path -- the
// product writer (GlobalPreferencesService::set_user) for the value mutations,
// and the repository's first-class recovery API (create_exact_backup /
// plan_restore / commit_restore) for the backup and restore. Nothing here is a
// private writer: every mutation takes the writer lease, verifies the expected
// generation, and records a MutationReceipt in the journal, exactly as the
// engine's own tests do. The restore commits a new CommitExactRestore generation
// (after != before), which the producer captures as the real post-restore store.
// ===========================================================================

/// The value captured in the backup and restored to.
pub(crate) const RESTORE_BACKED_VALUE: &str = "imperial";
/// The diverged value the restore replaces.
pub(crate) const RESTORE_DIVERGED_VALUE: &str = "metric";
/// Sidecar carrying the exact-backup ref from the setup phase to the observe phase.
pub(crate) const BACKUP_SIDECAR: &str = ".gp_backup_ref.json";

/// The on-disk Global preference store under an acceptance work root (the same
/// layout the CLI families snapshot: `<root>/config/datum/preferences`).
pub fn preferences_root(work_root: &Path) -> PathBuf {
    work_root.join("config").join("datum").join("preferences")
}

pub(crate) fn write_metadata(reason: &str) -> MutationMetadata {
    // A first-party acceptance-harness writer; the recovery operations carry no
    // product actor/proposal sidecar (audit: None), only the journal basics.
    MutationMetadata {
        actor: "gp-acceptance-driver".to_owned(),
        reason: reason.to_owned(),
        writer_instance: "gp-acceptance-driver".to_owned(),
        audit: None,
    }
}

/// A deterministic content fingerprint of every regular file under `dir`, used to
/// prove a step wrote nothing (preview-pure) at the disk level.
pub(crate) fn store_fingerprint(dir: &Path) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    fingerprint_into(dir, dir, &mut map);
    map
}

fn fingerprint_into(base: &Path, dir: &Path, map: &mut BTreeMap<String, String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = entry.metadata() else { continue };
        if meta.is_dir() {
            fingerprint_into(base, &path, map);
        } else if meta.is_file() {
            if let (Ok(bytes), Ok(rel)) = (std::fs::read(&path), path.strip_prefix(base)) {
                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                map.insert(
                    rel.to_string_lossy().into_owned(),
                    format!("{:x}", hasher.finalize()),
                );
            }
        }
    }
}

/// The current head generation number of the store, or `None` if it is not Ready.
pub fn store_generation(work_root: &Path) -> Option<u64> {
    let repository = PreferenceRepository::new(preferences_root(work_root), active_v1_registry());
    match repository.inspect() {
        RepositoryStatus::Ready(snapshot) => Some(snapshot.generation.generation),
        _ => None,
    }
}

pub(crate) fn current_generation(
    repository: &PreferenceRepository,
) -> Result<GenerationRef, String> {
    match repository.inspect() {
        RepositoryStatus::Ready(snapshot) => Ok(snapshot.generation),
        other => Err(format!("store is not Ready: {other:?}")),
    }
}

/// The backup-restore family variants the engine driver can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupRestoreVariant {
    ExactBackup,
    Restore,
    ReverseRestore,
    StalePreview,
    IncompleteBackup,
}

impl BackupRestoreVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExactBackup => "exact-backup",
            Self::Restore => "restore",
            Self::ReverseRestore => "reverse-restore",
            Self::StalePreview => "stale-preview",
            Self::IncompleteBackup => "incomplete-backup",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "exact-backup" => Some(Self::ExactBackup),
            "restore" => Some(Self::Restore),
            "reverse-restore" => Some(Self::ReverseRestore),
            "stale-preview" => Some(Self::StalePreview),
            "incomplete-backup" => Some(Self::IncompleteBackup),
            _ => None,
        }
    }
}

pub(crate) fn open_service(work_root: &Path) -> Result<GlobalPreferencesService, String> {
    let root = preferences_root(work_root);
    let legacy = work_root.join("absent-legacy-gui-preferences.json");
    GlobalPreferencesService::open(&root, &legacy, "gp-acceptance-driver", "machine")
        .map_err(|error| format!("open store: {error:?}"))
}

/// Seed generation 0 with the backed value through the product writer; return its ref.
pub(crate) fn seed_backed(work_root: &Path) -> Result<GenerationRef, String> {
    let mut service = open_service(work_root)?;
    service
        .set_user(pilot_key(), json!(RESTORE_BACKED_VALUE), None)
        .map_err(|refusal| format!("seed generation 0: {refusal:?}"))?;
    service
        .status()
        .generation()
        .cloned()
        .ok_or_else(|| "no generation after seeding".to_owned())
}

/// Set the pilot key to `value` through the product writer (expected == `from`);
/// return the new generation ref.
pub(crate) fn set_value(
    work_root: &Path,
    from: &GenerationRef,
    value: &str,
) -> Result<GenerationRef, String> {
    let mut service = open_service(work_root)?;
    service
        .set_user(pilot_key(), json!(value), Some(from))
        .map_err(|refusal| format!("set {value}: {refusal:?}"))?;
    service
        .status()
        .generation()
        .cloned()
        .ok_or_else(|| "no generation after set".to_owned())
}

pub(crate) fn write_sidecar(work_root: &Path, backup: &ExactBackupRef) -> Result<(), String> {
    std::fs::write(
        work_root.join(BACKUP_SIDECAR),
        serde_json::to_vec(backup).map_err(|error| format!("serialize backup ref: {error}"))?,
    )
    .map_err(|error| format!("write backup sidecar: {error}"))
}

pub(crate) fn read_sidecar(work_root: &Path) -> Result<ExactBackupRef, String> {
    serde_json::from_slice(
        &std::fs::read(work_root.join(BACKUP_SIDECAR))
            .map_err(|error| format!("read backup sidecar: {error}"))?,
    )
    .map_err(|error| format!("parse backup sidecar: {error}"))
}

pub(crate) fn repo(work_root: &Path) -> PreferenceRepository {
    PreferenceRepository::new(preferences_root(work_root), active_v1_registry())
}

/// Whether any `project.json` was written anywhere under the work root.
pub(crate) fn project_written(work_root: &Path) -> bool {
    fn walk(dir: &Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = entry.metadata() else { continue };
            if meta.is_dir() {
                if walk(&path) {
                    return true;
                }
            } else if path.file_name().is_some_and(|name| name == "project.json") {
                return true;
            }
        }
        false
    }
    walk(work_root)
}

/// SETUP phase for a backup-restore variant: establish its on-disk precondition
/// through the engine's own commit path (product writer for values; the repository
/// recovery API for backups). Leaves the store for the producer to snapshot as
/// `before`. No private writer.
pub fn backup_restore_setup(variant: BackupRestoreVariant, work_root: &Path) -> Result<(), String> {
    match variant {
        // exact-backup observes the backup operation itself, so setup only seeds gen 0.
        BackupRestoreVariant::ExactBackup => {
            seed_backed(work_root)?;
            Ok(())
        }
        // restore / reverse-restore / stale-preview: seed gen 0, exact-backup it, then
        // diverge to gen 1; the observe phase restores the backup over the diverged head.
        BackupRestoreVariant::Restore
        | BackupRestoreVariant::ReverseRestore
        | BackupRestoreVariant::StalePreview => {
            let backed = seed_backed(work_root)?;
            let backup = repo(work_root)
                .create_exact_backup(&backed, &write_metadata("GP-CM05E backup-restore backup"))
                .map_err(|error| format!("create_exact_backup: {error:?}"))?;
            set_value(work_root, &backed, RESTORE_DIVERGED_VALUE)?;
            write_sidecar(work_root, &backup)
        }
        // incomplete-backup: seed, exact-backup, then damage the backup manifest so its
        // digest no longer matches the recorded source generation -> load_backup refuses.
        BackupRestoreVariant::IncompleteBackup => {
            let backed = seed_backed(work_root)?;
            let backup = repo(work_root)
                .create_exact_backup(
                    &backed,
                    &write_metadata("GP-CM05E incomplete-backup backup"),
                )
                .map_err(|error| format!("create_exact_backup: {error:?}"))?;
            let manifest = preferences_root(work_root)
                .join("backups")
                .join(&backup.backup_id)
                .join("manifest.json");
            let mut bytes = std::fs::read(&manifest)
                .map_err(|error| format!("read backup manifest: {error}"))?;
            bytes.extend_from_slice(b" ");
            std::fs::write(&manifest, &bytes)
                .map_err(|error| format!("damage backup manifest: {error}"))?;
            write_sidecar(work_root, &backup)
        }
    }
}

/// OBSERVE phase for a backup-restore variant: run its recovery operation(s) and
/// return the matrix observations in order.
pub fn backup_restore_observe(
    variant: BackupRestoreVariant,
    work_root: &Path,
) -> Result<Vec<(&'static str, bool)>, String> {
    match variant {
        BackupRestoreVariant::ExactBackup => observe_exact_backup(work_root),
        BackupRestoreVariant::Restore => observe_restore(work_root),
        BackupRestoreVariant::ReverseRestore => observe_reverse_restore(work_root),
        BackupRestoreVariant::StalePreview => observe_stale_preview(work_root),
        BackupRestoreVariant::IncompleteBackup => observe_incomplete_backup(work_root),
    }
}

fn observe_exact_backup(work_root: &Path) -> Result<Vec<(&'static str, bool)>, String> {
    let repository = repo(work_root);
    let seeded = current_generation(&repository)?;
    let backup = repository
        .create_exact_backup(&seeded, &write_metadata("GP-CM05E exact-backup"))
        .map_err(|error| format!("create_exact_backup: {error:?}"))?;
    // all-bytes-preserved: restoring the fresh backup over the unchanged head is a
    // no-op (every key unchanged), so the backup captured the exact committed bytes.
    let plan = repository
        .plan_restore(&seeded, &backup)
        .map_err(|error| format!("plan_restore: {error:?}"))?;
    let all_bytes_preserved = plan.changed_keys.is_empty() && !plan.unchanged_keys.is_empty();
    let no_project_write = !project_written(work_root);
    Ok(vec![
        ("all-bytes-preserved", all_bytes_preserved),
        ("no-project-write", no_project_write),
    ])
}

fn observe_restore(work_root: &Path) -> Result<Vec<(&'static str, bool)>, String> {
    let root = preferences_root(work_root);
    let repository = repo(work_root);
    let backup = read_sidecar(work_root)?;
    let diverged = current_generation(&repository)?;

    // preview-pure: plan_restore takes no writer lease and writes nothing; the store
    // is byte-identical across it, and the plan still reports a real pending change.
    let before_plan = store_fingerprint(&root);
    let plan = repository
        .plan_restore(&diverged, &backup)
        .map_err(|error| format!("plan_restore: {error:?}"))?;
    let after_plan = store_fingerprint(&root);
    let preview_pure = before_plan == after_plan && !plan.changed_keys.is_empty();

    // backup-before-restore: commit_restore displaces the pre-restore head into a
    // backup and commits a real new generation (after != before).
    let outcome = repository
        .commit_restore(&plan, &write_metadata("GP-CM05E restore commit"))
        .map_err(|error| format!("commit_restore: {error:?}"))?;
    let restored = current_generation(&repository)?;
    let backup_before_restore = outcome.displaced_backup.source_generation.generation
        == diverged.generation
        && outcome.generation.generation == diverged.generation + 1
        && restored.generation == outcome.generation.generation;

    // exact-generation-authority / PM059 negative control: re-committing the now
    // stale plan is refused and leaves every byte untouched.
    let before_stale = store_fingerprint(&root);
    let stale = repository.commit_restore(&plan, &write_metadata("GP-CM05E stale restore"));
    let after_stale = store_fingerprint(&root);
    let exact_generation_authority = stale.is_err() && before_stale == after_stale;

    Ok(vec![
        ("preview-pure", preview_pure),
        ("backup-before-restore", backup_before_restore),
        ("exact-generation-authority", exact_generation_authority),
    ])
}

fn observe_reverse_restore(work_root: &Path) -> Result<Vec<(&'static str, bool)>, String> {
    let repository = repo(work_root);
    let backup = read_sidecar(work_root)?; // B0 = generation 0 (backed value)
    let diverged = current_generation(&repository)?; // generation 1 (diverged value)

    // Forward restore: generation 2 back to the backed value; its displaced backup
    // captures the diverged generation 1 exactly.
    let plan_forward = repository
        .plan_restore(&diverged, &backup)
        .map_err(|error| format!("plan_restore (forward): {error:?}"))?;
    let forward = repository
        .commit_restore(
            &plan_forward,
            &write_metadata("GP-CM05E reverse-restore forward"),
        )
        .map_err(|error| format!("commit_restore (forward): {error:?}"))?;
    let restored = current_generation(&repository)?;

    // Reverse restore: generation 3 back to the original diverged bytes, via the
    // displaced backup the forward restore captured.
    let plan_reverse = repository
        .plan_restore(&restored, &forward.displaced_backup)
        .map_err(|error| format!("plan_restore (reverse): {error:?}"))?;
    let reverse = repository
        .commit_restore(
            &plan_reverse,
            &write_metadata("GP-CM05E reverse-restore reverse"),
        )
        .map_err(|error| format!("commit_restore (reverse): {error:?}"))?;
    let reversed = current_generation(&repository)?;

    // original-bytes-restored: the reversed head byte-matches the displaced backup
    // (the original diverged state), so restoring it again is a no-op; and a real new
    // generation was committed each step.
    let verify = repository
        .plan_restore(&reversed, &forward.displaced_backup)
        .map_err(|error| format!("plan_restore (verify): {error:?}"))?;
    let original_bytes_restored = verify.changed_keys.is_empty()
        && forward.generation.generation == diverged.generation + 1
        && reverse.generation.generation == forward.generation.generation + 1;

    Ok(vec![("original-bytes-restored", original_bytes_restored)])
}

fn observe_stale_preview(work_root: &Path) -> Result<Vec<(&'static str, bool)>, String> {
    let root = preferences_root(work_root);
    let repository = repo(work_root);
    let backup = read_sidecar(work_root)?;
    let diverged = current_generation(&repository)?; // generation 1

    // A plan is built against the current head, then a legitimate concurrent change
    // moves the head on, making the plan's expected head stale.
    let plan = repository
        .plan_restore(&diverged, &backup)
        .map_err(|error| format!("plan_restore: {error:?}"))?;
    set_value(work_root, &diverged, RESTORE_BACKED_VALUE)?; // generation 2

    // refusal + no-write: committing the now-stale plan is refused and the refused
    // operation leaves every byte untouched (disk fingerprint identical across it).
    let before = store_fingerprint(&root);
    let stale = repository.commit_restore(&plan, &write_metadata("GP-CM05E stale-preview"));
    let after = store_fingerprint(&root);
    let refusal = matches!(stale, Err(RepositoryError::RestorePreviewStale));
    let no_write = before == after;

    Ok(vec![("refusal", refusal), ("no-write", no_write)])
}

fn observe_incomplete_backup(work_root: &Path) -> Result<Vec<(&'static str, bool)>, String> {
    let root = preferences_root(work_root);
    let repository = repo(work_root);
    let backup = read_sidecar(work_root)?; // damaged backup at generation 0
    let seeded = current_generation(&repository)?;

    // refusal + no-write: planning a restore from the damaged backup is refused as
    // incomplete (manifest digest mismatch) and writes nothing.
    let before = store_fingerprint(&root);
    let result = repository.plan_restore(&seeded, &backup);
    let after = store_fingerprint(&root);
    let refusal = matches!(result, Err(RepositoryError::BackupIncomplete(_)));
    let no_write = before == after;

    Ok(vec![("refusal", refusal), ("no-write", no_write)])
}

/// Family negative control (PM059, genuinely falsifying): a WELL-FORMED backup and
/// restore succeed end to end, so the refusal predicates are not constant-true.
/// Returns `(plan_ok, commit_ok)` — an intact backup plans without BackupIncomplete
/// (the incomplete-backup refusal can read false), and a fresh plan commits without
/// RestorePreviewStale (the stale-preview / restore refusal can read false).
pub fn backup_restore_negative_control(work_root: &Path) -> Result<(bool, bool), String> {
    let backed = seed_backed(work_root)?;
    let repository = repo(work_root);
    let backup = repository
        .create_exact_backup(&backed, &write_metadata("GP-CM05E neg-control backup"))
        .map_err(|error| format!("create_exact_backup: {error:?}"))?;
    let diverged = set_value(work_root, &backed, RESTORE_DIVERGED_VALUE)?;
    let plan = repository.plan_restore(&diverged, &backup);
    let plan_ok = plan.is_ok();
    let commit_ok = match plan {
        Ok(plan) => repository
            .commit_restore(&plan, &write_metadata("GP-CM05E neg-control restore"))
            .is_ok(),
        Err(_) => false,
    };
    Ok((plan_ok, commit_ok))
}
