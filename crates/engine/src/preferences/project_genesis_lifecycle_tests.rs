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

    fn provider(&self) -> FixedPreferenceLocationProvider {
        FixedPreferenceLocationProvider(PreferenceLocations {
            configuration_base: self.root.join("config"),
            repository_root: self.root.join("config/datum/preferences"),
            legacy_console_path: self.root.join("config/datum/gui-preferences.json"),
        })
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

fn project_tree(root: &std::path::Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(base: &std::path::Path, at: &std::path::Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(base, &path, out);
            } else {
                out.insert(
                    path.strip_prefix(base).unwrap().to_path_buf(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn factory_only_service_cannot_be_reused_as_a_global_preferences_reader() {
    let service = GlobalPreferencesProductService::factory_only("factory-only-test").unwrap();
    let refusal = service
        .query(PreferenceQueryV1::PreviewProjectUnitsSeed {
            source: ProjectUnitsSourceV1::Global {
                expected_generation: None,
            },
        })
        .unwrap_err();
    assert_eq!(refusal.code, PreferenceErrorCodeV1::SeedSourceUnavailable);
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
fn fixed_identity_factory_projects_are_byte_identical_across_destinations() {
    let fixture = Fixture::new("deterministic-roots");
    let actor = fixture.actor();
    let request = fixture.request("deterministic-a");
    let mut trees = Vec::new();
    for name in ["deterministic-a", "deterministic-b", "deterministic-c"] {
        let mut at = request.clone();
        at.destination = fixture.root.join(name);
        fixture.service.create_project(at.clone(), &actor).unwrap();
        trees.push(project_tree(&at.destination));
    }
    assert_eq!(trees[0], trees[1]);
    assert_eq!(trees[1], trees[2]);
}

#[test]
fn global_seed_is_pinned_once_and_never_follows_later_edits() {
    let fixture = Fixture::new("no-live-following");
    let provider = fixture.provider();
    let actor = fixture.actor();
    let mut writer = GlobalPreferencesProductService::open(&provider, "seed-writer").unwrap();
    let pinned = writer
        .mutate(
            PreferenceMutationRequestV1::SetUser {
                key: "datum.units.board_length".to_owned(),
                value: json!("mil"),
                expected: HeadExpectationV1::Missing,
                request_id: uuid::Uuid::new_v4(),
                reason: "establish pinned test generation".to_owned(),
            },
            &actor,
        )
        .unwrap()
        .generation
        .unwrap();
    drop(writer);

    let genesis = GlobalPreferencesProductService::open(&provider, "genesis-reader").unwrap();
    let mut request = fixture.request("Pinned Global Project");
    request.units_source = ProjectUnitsSourceV1::Global {
        expected_generation: Some(pinned.clone()),
    };
    let mut changed_during_creation = false;
    let result = genesis
        .create_project_with_checkpoint(request.clone(), &actor, |checkpoint| {
            if checkpoint == GenesisCheckpoint::StagingPrepared && !changed_during_creation {
                let mut concurrent =
                    GlobalPreferencesProductService::open(&provider, "concurrent-writer")
                        .map_err(|error| format!("{error:?}"))?;
                concurrent
                    .mutate(
                        PreferenceMutationRequestV1::SetUser {
                            key: "datum.units.board_length".to_owned(),
                            value: json!("inch"),
                            expected: HeadExpectationV1::Generation(pinned.clone()),
                            request_id: uuid::Uuid::new_v4(),
                            reason: "change Global after genesis pins its seed".to_owned(),
                        },
                        &actor,
                    )
                    .map_err(|error| format!("{error:?}"))?;
                changed_during_creation = true;
            }
            Ok(())
        })
        .unwrap();
    assert!(changed_during_creation);
    assert!(matches!(
        &result.units_receipt.source,
        ProjectUnitsReceiptSourceV2::Global {
            generation: Some(generation),
            ..
        } if generation == &pinned
    ));
    let board = result
        .units_receipt
        .items
        .iter()
        .find(|item| item.key == "datum.units.board_length")
        .unwrap();
    assert_eq!(board.copied_value, json!("mil"));

    let before = project_tree(&request.destination);
    let reopened = GlobalPreferencesProductService::open(&provider, "post-genesis-reader").unwrap();
    assert_eq!(
        reopened
            .context()
            .generation
            .as_ref()
            .map(|value| value.generation),
        Some(1)
    );
    assert_eq!(project_tree(&request.destination), before);
}

#[test]
fn long_lived_global_reader_refuses_stale_and_absent_head_until_explicit_refresh() {
    let fixture = Fixture::new("stale-selected-global");
    let actor = fixture.actor();
    let mut reader =
        GlobalPreferencesProductService::open(&fixture.provider(), "selected-reader").unwrap();
    let mut writer =
        GlobalPreferencesProductService::open(&fixture.provider(), "concurrent-writer").unwrap();
    let mut request = fixture.request("Stale Global Project");
    request.units_source = ProjectUnitsSourceV1::Global {
        expected_generation: None,
    };
    let first = writer
        .mutate(
            PreferenceMutationRequestV1::SetUser {
                key: "datum.units.system".into(),
                value: json!("imperial"),
                expected: HeadExpectationV1::Missing,
                request_id: uuid::Uuid::new_v4(),
                reason: "create Global head after defaults preview".into(),
            },
            &actor,
        )
        .unwrap()
        .generation
        .unwrap();
    let error = reader.create_project(request.clone(), &actor).unwrap_err();
    assert_eq!(error.code, PreferenceErrorCodeV1::StaleGeneration);
    assert_eq!(error.current_context.generation.as_ref(), Some(&first));
    assert!(!request.destination.exists());
    reader.refresh_global_project_units_source();
    request.units_source = ProjectUnitsSourceV1::Global {
        expected_generation: Some(first.clone()),
    };
    let second = writer
        .mutate(
            PreferenceMutationRequestV1::SetUser {
                key: "datum.units.system".into(),
                value: json!("metric"),
                expected: HeadExpectationV1::Generation(first),
                request_id: uuid::Uuid::new_v4(),
                reason: "advance the independently held Global reader".into(),
            },
            &actor,
        )
        .unwrap()
        .generation
        .unwrap();
    let error = reader.create_project(request.clone(), &actor).unwrap_err();
    assert_eq!(error.code, PreferenceErrorCodeV1::StaleGeneration);
    assert_eq!(error.current_context.generation.as_ref(), Some(&second));
    assert!(!request.destination.exists());
    reader.refresh_global_project_units_source();
    request.units_source = ProjectUnitsSourceV1::Global {
        expected_generation: Some(second.clone()),
    };
    let result = reader.create_project(request.clone(), &actor).unwrap();
    assert!(
        matches!(&result.units_receipt.source, ProjectUnitsReceiptSourceV2::Global { generation: Some(generation), .. } if generation == &second)
    );
    // Replaying the already-published identity still uses its immutable receipt.
    let replay = reader.create_project(request, &actor).unwrap();
    assert_eq!(replay, result);
}

#[test]
fn unreadable_live_global_source_refuses_without_creation_and_factory_stays_isolated() {
    let fixture = Fixture::new("unreadable-live-global");
    let actor = fixture.actor();
    let repository = fixture.provider().0.repository_root;
    std::fs::create_dir_all(&repository).unwrap();
    std::fs::write(repository.join("head.json"), b"{").unwrap();
    let before = project_tree(&repository);
    let mut request = fixture.request("Unavailable Global Project");
    request.units_source = ProjectUnitsSourceV1::Global {
        expected_generation: None,
    };
    let error = fixture
        .service
        .create_project(request.clone(), &actor)
        .unwrap_err();
    assert_eq!(error.code, PreferenceErrorCodeV1::SeedSourceUnavailable);
    assert_eq!(
        error.current_context.repository_status,
        PreferenceRepositoryStatusV1::PreservedUnreadable
    );
    assert!(!request.destination.exists());
    assert_eq!(project_tree(&repository), before);
    let factory = fixture.request("Explicit Factory Project");
    fixture.service.create_project(factory, &actor).unwrap();
    assert_eq!(project_tree(&repository), before);
}

#[test]
fn injected_missing_units_default_in_ready_snapshot_refuses_genesis_without_writes() {
    let mut fixture = Fixture::new("injected-unresolved-units");
    let actor = fixture.actor();
    let generation = fixture
        .service
        .mutate(
            PreferenceMutationRequestV1::SetUser {
                key: "datum.accessibility.reduced_motion".into(),
                value: json!(true),
                expected: HeadExpectationV1::Missing,
                request_id: uuid::Uuid::new_v4(),
                reason: "establish a real ready snapshot without overriding Units".into(),
            },
            &actor,
        )
        .unwrap()
        .generation
        .unwrap();
    let repository = fixture.provider().0.repository_root;
    let before = project_tree(&repository);
    // Shipped Units descriptors all have valid literal defaults. This injected
    // descriptor fault exercises the defensive unresolved-value guard; it is
    // not a normally reachable Global repository or native workflow fixture.
    let mut registry = DescriptorRegistry::default();
    for key in fixture.service.registry().keys() {
        let mut descriptor = fixture.service.registry().get(key).unwrap().clone();
        if key.as_str() == "datum.units.board_length" {
            descriptor.default_value = DescriptorDefault::Absent;
        }
        registry.register(descriptor).unwrap();
    }
    fixture.service.service.registry = registry;
    assert!(matches!(
        fixture.service.status(),
        PreferenceServiceStatus::Ready { .. }
    ));
    let unresolved = fixture
        .service
        .rows()
        .into_iter()
        .find(|row| row.key.as_str() == "datum.units.board_length")
        .unwrap();
    assert_eq!(unresolved.effective_value, None);
    let mut request = fixture.request("Unresolved Units Project");
    request.units_source = ProjectUnitsSourceV1::Global {
        expected_generation: Some(generation),
    };
    let error = fixture
        .service
        .create_project(request.clone(), &actor)
        .unwrap_err();
    assert_eq!(error.code, PreferenceErrorCodeV1::SeedSourceUnavailable);
    assert_eq!(
        error.current_context.repository_status,
        PreferenceRepositoryStatusV1::Ready
    );
    assert!(error.message.contains("has no effective value"));
    assert!(!request.destination.exists());
    assert_eq!(project_tree(&repository), before);
}
