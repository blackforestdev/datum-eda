//! GP-CM05E existing-project-stability/fixed-identities (engine surface).
//!
//! Authority under test: a factory genesis driven by a NORMALIZED request -- the same
//! request identity (request_id + project_id), actor and units source, differing only in
//! destination -- produces a BYTE-IDENTICAL Project. The project tree is a deterministic
//! function of the normalized request, not of wall-clock, path or run order; and that
//! identity TRACKS the request (a different normalized request yields a different Project),
//! so the byte-identity is request-determined, never constant.
//!
//! READ-ONLY on the coverage work root: every scenario runs in its OWN throwaway temp dir
//! (ScenarioRoot, cleaned on drop), so the producer's work root is never written and
//! after==before=empty is honest; the driver's observation booleans carry the proof.

use std::collections::BTreeMap;
use std::path::PathBuf;

use uuid::Uuid;

use super::backup_restore::store_fingerprint;
use crate::preferences::{
    FixedPreferenceLocationProvider, GlobalPreferencesProductService, PreferenceActorKindV1,
    PreferenceActorV1, PreferenceLocations, ProjectGenesisRequestV1, ProjectUnitsSourceV1,
};

/// A throwaway scenario root in the temp dir, cleaned on drop, so no scenario ever writes
/// the coverage work root.
struct ScenarioRoot(PathBuf);

impl ScenarioRoot {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("gp-cm05e-fixed-identity-{}", Uuid::new_v4())))
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

fn actor() -> PreferenceActorV1 {
    PreferenceActorV1 {
        kind: PreferenceActorKindV1::HumanCli,
        session_id: "gp-acceptance-session".to_owned(),
        local_actor_id: "gp-acceptance-user".to_owned(),
        invocation_id: Uuid::new_v4(),
    }
}

/// A factory genesis request with an explicit normalized identity (request_id + project_id).
/// Factory source is self-contained: it reads no Global store, so the only identity inputs
/// are the request fields and the actor.
fn normalized_request(
    request_id: Uuid,
    project_id: Uuid,
    destination: PathBuf,
) -> ProjectGenesisRequestV1 {
    ProjectGenesisRequestV1 {
        request_id,
        destination,
        project_name: "Fixed Identity".to_owned(),
        project_id: Some(project_id),
        units_source: ProjectUnitsSourceV1::Factory {
            profile_id: "datum.units.factory.v1".to_owned(),
        },
    }
}

/// Create a factory Project at `destination` and fingerprint its tree (relative path -> digest).
fn create_tree(
    service: &GlobalPreferencesProductService,
    actor: &PreferenceActorV1,
    request_id: Uuid,
    project_id: Uuid,
    destination: PathBuf,
) -> Result<BTreeMap<String, String>, String> {
    service
        .create_project(
            normalized_request(request_id, project_id, destination.clone()),
            actor,
        )
        .map_err(|error| format!("create_project: {error:?}"))?;
    Ok(store_fingerprint(&destination))
}

/// OBSERVE phase. Self-contained in a throwaway store, so there is no setup phase (the family
/// is read-only on the coverage work root).
pub fn fixed_identities_observe() -> Result<Vec<(&'static str, bool)>, String> {
    let root = ScenarioRoot::new();
    let service = root.open()?;
    let actor = actor();
    let request_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();

    // byte-identical-projects: the same normalized request at three destinations yields three
    // byte-identical project trees.
    let mut trees = Vec::new();
    for name in ["identity-a", "identity-b", "identity-c"] {
        trees.push(create_tree(
            &service,
            &actor,
            request_id,
            project_id,
            root.0.join(name),
        )?);
    }
    let byte_identical_projects =
        !trees[0].is_empty() && trees[0] == trees[1] && trees[1] == trees[2];

    // same-normalized-request: the byte-identity is a consequence of the shared normalized
    // request, not a constant -- a DIFFERENT normalized request (fresh request_id + project_id)
    // yields a DIFFERENT project tree.
    let other = create_tree(
        &service,
        &actor,
        Uuid::new_v4(),
        Uuid::new_v4(),
        root.0.join("identity-other"),
    )?;
    let same_normalized_request = other != trees[0];

    Ok(vec![
        ("byte-identical-projects", byte_identical_projects),
        ("same-normalized-request", same_normalized_request),
    ])
}

/// Family negative control (PM059, genuinely falsifying): two DIFFERENT normalized requests
/// produce DIFFERENT project trees, so byte-identity is not constant-true -- the identity is
/// genuinely request-determined. Returns true when the two trees differ (and are non-empty).
pub fn fixed_identities_negative_control() -> Result<bool, String> {
    let root = ScenarioRoot::new();
    let service = root.open()?;
    let actor = actor();
    let one = create_tree(
        &service,
        &actor,
        Uuid::new_v4(),
        Uuid::new_v4(),
        root.0.join("neg-one"),
    )?;
    let two = create_tree(
        &service,
        &actor,
        Uuid::new_v4(),
        Uuid::new_v4(),
        root.0.join("neg-two"),
    )?;
    Ok(!one.is_empty() && !two.is_empty() && one != two)
}
