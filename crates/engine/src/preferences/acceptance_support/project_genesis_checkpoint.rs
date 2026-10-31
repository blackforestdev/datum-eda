//! GP-CM05E project-genesis-crash NON-fault checkpoint family (five variants).
//!
//! Authority under test: a project-creation interrupted at any lifecycle checkpoint leaves
//! honest truth -- the destination is either absent or a complete valid Project (never a
//! partial or falsely-acknowledged one), only the genesis's OWN staging is cleaned, and a
//! restart recovers a complete Project. Each variant drives the engine's REAL genesis path
//! (`create_project_with_checkpoint`, the function `create_project` itself uses) and
//! interrupts it by returning `Err` from the checkpoint callback at the named point -- the
//! engine's own lifecycle interruption, leaving real on-disk state. This is construction,
//! not fault fabrication: no OS/IO fault is injected (those six fault variants are
//! Class-C scope-exits deferred to the GP-CM05A gate).
//!
//! READ-ONLY on the coverage work root: every scenario runs in its OWN throwaway temp dir
//! (ScenarioRoot, cleaned on drop), so the producer's work root is never written and
//! after==before=empty is honest; the driver's observation booleans carry the proof.

use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::preferences::project_genesis::{GenesisCheckpoint, staging_path};
use crate::preferences::{
    FixedPreferenceLocationProvider, GlobalPreferencesProductService, PreferenceActorKindV1,
    PreferenceActorV1, PreferenceErrorCodeV1, PreferenceLocations, ProjectGenesisRequestV1,
    ProjectUnitsSourceV1,
};
use crate::substrate::ProjectResolver;

/// A throwaway scenario root in the temp dir, cleaned on drop, so no scenario ever writes
/// the coverage work root.
struct ScenarioRoot(PathBuf);

impl ScenarioRoot {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("gp-cm05e-genesis-checkpoint-{}", Uuid::new_v4())))
    }

    fn provider(&self) -> FixedPreferenceLocationProvider {
        FixedPreferenceLocationProvider(PreferenceLocations {
            configuration_base: self.0.join("config"),
            repository_root: self.0.join("config/datum/preferences"),
            legacy_console_path: self.0.join("config/datum/gui-preferences.json"),
        })
    }

    fn open(&self) -> Result<GlobalPreferencesProductService, String> {
        std::fs::create_dir_all(&self.0).map_err(|error| format!("scenario root: {error}"))?;
        GlobalPreferencesProductService::open(&self.provider(), "gp-acceptance-driver")
            .map_err(|error| format!("product service open: {error:?}"))
    }
}

impl Drop for ScenarioRoot {
    fn drop(&mut self) {
        if self.0.starts_with(std::env::temp_dir()) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

/// The five NON-fault genesis checkpoints (pre-publication plus the post-publication
/// recovery point). The six fault-injection variants are intentionally absent -- they need
/// an OS/IO fault seam and are E-deferred (Class-C) to the GP-CM05A gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointVariant {
    StagingPrepared,
    ProjectBuilt,
    ProjectValidated,
    StagingSynced,
    ProjectPublished,
}

impl CheckpointVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StagingPrepared => "staging-prepared",
            Self::ProjectBuilt => "project-built",
            Self::ProjectValidated => "project-validated",
            Self::StagingSynced => "staging-synced",
            Self::ProjectPublished => "project-published",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "staging-prepared" => Some(Self::StagingPrepared),
            "project-built" => Some(Self::ProjectBuilt),
            "project-validated" => Some(Self::ProjectValidated),
            "staging-synced" => Some(Self::StagingSynced),
            "project-published" => Some(Self::ProjectPublished),
            _ => None,
        }
    }

    fn checkpoint(self) -> GenesisCheckpoint {
        match self {
            Self::StagingPrepared => GenesisCheckpoint::StagingPrepared,
            Self::ProjectBuilt => GenesisCheckpoint::ProjectBuilt,
            Self::ProjectValidated => GenesisCheckpoint::ProjectValidated,
            Self::StagingSynced => GenesisCheckpoint::StagingSynced,
            Self::ProjectPublished => GenesisCheckpoint::ProjectPublished,
        }
    }

    /// The post-publication checkpoint fires AFTER the durable rename, so the Project is
    /// already published: the engine returns Ok and recovery yields the complete Project.
    /// The four pre-publication checkpoints abort before the rename: the engine returns a
    /// typed GenesisPublishFailed and the destination stays absent.
    fn is_post_publication(self) -> bool {
        matches!(self, Self::ProjectPublished)
    }
}

fn actor() -> PreferenceActorV1 {
    PreferenceActorV1 {
        kind: PreferenceActorKindV1::HumanCli,
        session_id: "gp-acceptance-session".to_owned(),
        local_actor_id: "gp-acceptance-user".to_owned(),
        invocation_id: Uuid::new_v4(),
    }
}

fn factory_request(destination: PathBuf) -> ProjectGenesisRequestV1 {
    ProjectGenesisRequestV1 {
        request_id: Uuid::new_v4(),
        destination,
        project_name: "Checkpoint Project".to_owned(),
        project_id: Some(Uuid::new_v4()),
        units_source: ProjectUnitsSourceV1::Factory {
            profile_id: "datum.units.factory.v1".to_owned(),
        },
    }
}

fn resolves(destination: &Path) -> bool {
    ProjectResolver::new(destination).resolve().is_ok()
}

/// OBSERVE phase. Self-contained in a throwaway store, so there is no setup phase (the family
/// is read-only on the coverage work root).
pub fn checkpoint_observe(variant: CheckpointVariant) -> Result<Vec<(&'static str, bool)>, String> {
    let root = ScenarioRoot::new();
    let service = root.open()?;
    let interrupting_actor = actor();
    let destination = root.0.join("project");
    let request = factory_request(destination.clone());
    let stage = staging_path(
        &destination,
        request.request_id,
        interrupting_actor.invocation_id,
    )
    .map_err(|error| format!("staging_path: {error}"))?;

    // An unrelated sibling next to the destination proves owned-cleanup-only: interruption
    // cleanup must remove only the genesis's own staging, never a neighbour.
    let sentinel = root.0.join("unrelated-neighbour.txt");
    std::fs::write(&sentinel, b"keep").map_err(|error| format!("sentinel: {error}"))?;

    let target = variant.checkpoint();
    let outcome =
        service.create_project_with_checkpoint(request.clone(), &interrupting_actor, |point| {
            if point == target {
                Err(format!("injected lifecycle interruption at {point:?}"))
            } else {
                Ok(())
            }
        });

    let post = variant.is_post_publication();
    // destination-absent-or-complete: pre-publication leaves it absent; post-publication
    // leaves a complete, resolvable Project. Never a partial tree.
    let destination_absent_or_complete = if post {
        resolves(&destination)
    } else {
        !destination.exists()
    };
    // no-false-acknowledgement: a pre-publication interruption returns the typed
    // GenesisPublishFailed (not a false Ok); a post-publication interruption returns Ok only
    // because the durable Project genuinely exists.
    let no_false_acknowledgement = if post {
        outcome.is_ok() && resolves(&destination)
    } else {
        outcome
            .as_ref()
            .err()
            .map(|error| error.code == PreferenceErrorCodeV1::GenesisPublishFailed)
            .unwrap_or(false)
    };
    // owned-cleanup-only: the genesis's own staging is gone and the unrelated neighbour is
    // untouched (no sweep of sibling files).
    let owned_cleanup_only = !stage.exists()
        && std::fs::read(&sentinel)
            .map(|bytes| bytes == b"keep")
            .unwrap_or(false);

    // restart-recovery: a restart recovers a complete Project. Pre-publication (destination
    // absent) recovers through a fresh create; post-publication is already complete and a
    // same-request replay returns the identical published result.
    let restart_recovery = if post {
        match service.create_project(request.clone(), &interrupting_actor) {
            Ok(replay) => replay.request_id == request.request_id && resolves(&destination),
            Err(_) => false,
        }
    } else {
        let restart_actor = actor();
        service
            .create_project(factory_request(destination.clone()), &restart_actor)
            .is_ok()
            && resolves(&destination)
    };

    Ok(vec![
        (
            "destination-absent-or-complete",
            destination_absent_or_complete,
        ),
        ("no-false-acknowledgement", no_false_acknowledgement),
        ("owned-cleanup-only", owned_cleanup_only),
        ("restart-recovery", restart_recovery),
    ])
}

/// Family negative control (PM059, genuinely falsifying): an uninterrupted genesis publishes
/// a complete, resolvable Project, so the checkpoint predicates are not constant-true -- the
/// interruption path is distinguished from the clean path. Returns true when the clean create
/// published a complete Project.
pub fn checkpoint_negative_control() -> Result<bool, String> {
    let root = ScenarioRoot::new();
    let service = root.open()?;
    let destination = root.0.join("project");
    service
        .create_project(factory_request(destination.clone()), &actor())
        .map_err(|error| format!("clean create: {error:?}"))?;
    Ok(resolves(&destination))
}
