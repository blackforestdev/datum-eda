use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::json;

use super::project_genesis::{GenesisCheckpoint, staging_path};
use super::*;
use crate::substrate::ProjectResolver;

struct Fixture {
    root: PathBuf,
    service: GlobalPreferencesProductService,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "datum-product-genesis-{label}-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir(&root).unwrap();
        let provider = FixedPreferenceLocationProvider(PreferenceLocations {
            configuration_base: root.join("config"),
            repository_root: root.join("config/datum/preferences"),
            legacy_console_path: root.join("config/datum/gui-preferences.json"),
        });
        let service = GlobalPreferencesProductService::open(&provider, "genesis-test").unwrap();
        Self { root, service }
    }

    fn actor(&self) -> PreferenceActorV1 {
        PreferenceActorV1 {
            kind: PreferenceActorKindV1::HumanCli,
            session_id: "genesis-test-session".to_owned(),
            local_actor_id: "genesis-test-user".to_owned(),
            invocation_id: uuid::Uuid::new_v4(),
        }
    }

    fn request(&self, name: &str) -> ProjectGenesisRequestV1 {
        ProjectGenesisRequestV1 {
            request_id: uuid::Uuid::new_v4(),
            destination: self.root.join(name),
            project_name: name.to_owned(),
            project_id: Some(uuid::Uuid::new_v4()),
            units_source: ProjectUnitsSourceV1::Factory {
                profile_id: "datum.units.factory.v1".to_owned(),
            },
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn every_prepublication_checkpoint_leaves_destination_absent_and_owned_stage_clean() {
    for failure in [
        GenesisCheckpoint::StagingPrepared,
        GenesisCheckpoint::ProjectBuilt,
        GenesisCheckpoint::ProjectValidated,
        GenesisCheckpoint::StagingSynced,
    ] {
        let fixture = Fixture::new(&format!("fault-{failure:?}"));
        let actor = fixture.actor();
        let request = fixture.request("Fault Project");
        let stage = staging_path(
            &request.destination,
            request.request_id,
            actor.invocation_id,
        )
        .unwrap();
        let refusal = fixture
            .service
            .create_project_with_checkpoint(request.clone(), &actor, |point| {
                if point == failure {
                    Err(format!("injected at {point:?}"))
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
        assert_eq!(refusal.code, PreferenceErrorCodeV1::GenesisPublishFailed);
        assert!(!request.destination.exists());
        assert!(!stage.exists());
    }
}

#[test]
fn postpublication_failure_recovers_the_complete_published_project() {
    let fixture = Fixture::new("post-publication");
    let actor = fixture.actor();
    let request = fixture.request("Published Project");
    let result = fixture
        .service
        .create_project_with_checkpoint(request.clone(), &actor, |point| {
            if point == GenesisCheckpoint::ProjectPublished {
                Err("simulated process loss after rename".to_owned())
            } else {
                Ok(())
            }
        })
        .unwrap();
    assert_eq!(result.request_id, request.request_id);
    ProjectResolver::new(&request.destination)
        .resolve()
        .unwrap();
    let stage = staging_path(
        &request.destination,
        request.request_id,
        actor.invocation_id,
    )
    .unwrap();
    assert!(!stage.exists());
}

#[test]
fn concurrent_identical_creators_publish_once_and_replay_one_result() {
    let fixture = Fixture::new("concurrent");
    let provider = FixedPreferenceLocationProvider(PreferenceLocations {
        configuration_base: fixture.root.join("config"),
        repository_root: fixture.root.join("config/datum/preferences"),
        legacy_console_path: fixture.root.join("config/datum/gui-preferences.json"),
    });
    let second = GlobalPreferencesProductService::open(&provider, "genesis-test-2").unwrap();
    let request = fixture.request("Concurrent Project");
    let actor_a = fixture.actor();
    let actor_b = PreferenceActorV1 {
        invocation_id: uuid::Uuid::new_v4(),
        ..actor_a.clone()
    };
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let barrier_b = barrier.clone();
    let request_b = request.clone();
    let handle = std::thread::spawn(move || {
        second.create_project_with_checkpoint(request_b, &actor_b, |point| {
            if point == GenesisCheckpoint::StagingSynced {
                barrier_b.wait();
            }
            Ok(())
        })
    });
    let result_a = fixture
        .service
        .create_project_with_checkpoint(request, &actor_a, |point| {
            if point == GenesisCheckpoint::StagingSynced {
                barrier.wait();
            }
            Ok(())
        })
        .unwrap();
    let result_b = handle.join().unwrap().unwrap();
    assert_eq!(result_a, result_b);
}

#[test]
fn later_global_mutation_cannot_change_a_published_project() {
    let mut fixture = Fixture::new("no-live-following");
    let actor = fixture.actor();
    let mut request = fixture.request("Pinned Global Project");
    request.units_source = ProjectUnitsSourceV1::Global {
        expected_generation: None,
    };
    fixture
        .service
        .create_project(request.clone(), &actor)
        .unwrap();
    let before: BTreeMap<_, _> = [
        "project.json",
        "schematic/schematic.json",
        "board/board.json",
        "rules/rules.json",
    ]
    .into_iter()
    .map(|path| (path, std::fs::read(request.destination.join(path)).unwrap()))
    .collect();

    fixture
        .service
        .mutate(
            crate::preferences::PreferenceMutationRequestV1::SetUser {
                key: "datum.units.board_length".to_owned(),
                value: json!("mil"),
                expected: crate::preferences::HeadExpectationV1::Missing,
                request_id: uuid::Uuid::new_v4(),
                reason: "prove copy-once Project ownership".to_owned(),
            },
            &actor,
        )
        .unwrap();
    for (path, bytes) in before {
        assert_eq!(
            std::fs::read(request.destination.join(path)).unwrap(),
            bytes
        );
    }
}
