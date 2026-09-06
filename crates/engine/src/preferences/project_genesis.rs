//! Atomic native-Project genesis from one pinned eight-key Units snapshot.

use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::{
    GlobalPreferencesProductService, PreferenceActorV1, PreferenceErrorCodeV1, PreferenceErrorV1,
    PreferenceSchemaRefV1, ProjectGenesisRequestV1, ProjectGenesisResponseV1,
    ProjectGenesisResultV1, ProjectUnitsReceiptSourceV2, ProjectUnitsSeedItemV2,
    ProjectUnitsSeedReceiptV2, ProjectUnitsSourceV1,
};
use crate::api::native_write::genesis::{
    GenesisSpec, bootstrap_native_project_with_units_receipt_value,
};
use crate::ir::serialization::to_json_deterministic;
use crate::ir::units::{ACTIVE_UNITS_KEYS, ProjectUnitsSeedReceipt, project_profile_from_value};
use crate::substrate::ProjectResolver;

const RECEIPT_SCHEMA: &str = "datum.project.units_seed_receipt";
const RECEIPT_VERSION: u32 = 2;
const FACTORY_PROFILE: &str = "datum.units.factory.v1";

#[derive(Serialize)]
struct SeedDigestMaterial<'a> {
    schema_name: &'static str,
    schema_version: u32,
    project_id: uuid::Uuid,
    source: &'a ProjectUnitsReceiptSourceV2,
    items: &'a [ProjectUnitsSeedItemV2],
    units_seed_catalog_digest: &'a str,
}

#[derive(Serialize)]
struct GenesisDigestMaterial<'a> {
    schema_name: &'static str,
    schema_version: u32,
    project_name: &'a str,
    project_id: uuid::Uuid,
    units_source: &'a ProjectUnitsSourceV1,
    seed_application_digest: &'a str,
}

#[derive(Serialize, serde::Deserialize)]
struct StagingMarker {
    schema: String,
    request_id: uuid::Uuid,
    invocation_id: uuid::Uuid,
    genesis_request_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GenesisCheckpoint {
    StagingPrepared,
    ProjectBuilt,
    ProjectValidated,
    StagingSynced,
    ProjectPublished,
}

impl GlobalPreferencesProductService {
    pub fn execute_project_genesis(
        &self,
        request: ProjectGenesisRequestV1,
        actor: &PreferenceActorV1,
    ) -> ProjectGenesisResponseV1 {
        match self.create_project(request, actor) {
            Ok(result) => ProjectGenesisResponseV1 {
                ok: true,
                schema: PreferenceSchemaRefV1 {
                    name: "datum.project.new".to_owned(),
                    version: 1,
                },
                context: self.context(),
                result: Some(result),
                error: None,
            },
            Err(error) => ProjectGenesisResponseV1 {
                ok: false,
                schema: PreferenceSchemaRefV1 {
                    name: "datum.project.new".to_owned(),
                    version: 1,
                },
                context: error.current_context.clone(),
                result: None,
                error: Some(error),
            },
        }
    }

    pub fn create_project(
        &self,
        request: ProjectGenesisRequestV1,
        actor: &PreferenceActorV1,
    ) -> Result<ProjectGenesisResultV1, PreferenceErrorV1> {
        self.create_project_with_checkpoint(request, actor, |_| Ok(()))
    }

    pub(super) fn create_project_with_checkpoint(
        &self,
        request: ProjectGenesisRequestV1,
        actor: &PreferenceActorV1,
        mut checkpoint: impl FnMut(GenesisCheckpoint) -> Result<(), String>,
    ) -> Result<ProjectGenesisResultV1, PreferenceErrorV1> {
        super::product_actor::validate_actor(actor, &self.context())?;
        if request.project_name.trim().is_empty() {
            return Err(self.genesis_error(
                PreferenceErrorCodeV1::InvalidRequest,
                "Project name must not be empty",
                BTreeMap::from([("field".to_owned(), json!("project_name"))]),
            ));
        }
        let destination = normalize_absent_destination(&request.destination)
            .map_err(|message| self.genesis_io_error(message))?;
        if destination.exists() {
            return self.replay_or_refuse_existing(&destination, &request);
        }

        let preview = self.query(super::PreferenceQueryV1::PreviewProjectUnitsSeed {
            source: request.units_source.clone(),
        })?;
        let super::PreferenceQueryResultV1::PreviewProjectUnitsSeed(preview) = preview else {
            unreachable!("seed query result matches request")
        };
        let project_id = request.project_id.unwrap_or_else(uuid::Uuid::new_v4);
        let profile = project_profile_from_value(&preview.profile).map_err(|error| {
            self.genesis_error(
                PreferenceErrorCodeV1::SeedIncomplete,
                "Resolved Units seed is not a complete Project profile",
                BTreeMap::from([("invalid_key".to_owned(), json!(error.key))]),
            )
        })?;
        let source = receipt_source(&request.units_source, preview.pinned_generation.clone());
        let items = self.seed_items(&request.units_source, &preview.receipt_preview)?;
        let profile_digest =
            digest(&preview.profile).map_err(|message| self.genesis_io_error(message))?;
        let seed_application_digest = digest(&SeedDigestMaterial {
            schema_name: RECEIPT_SCHEMA,
            schema_version: RECEIPT_VERSION,
            project_id,
            source: &source,
            items: &items,
            units_seed_catalog_digest: &preview.units_seed_catalog_digest,
        })
        .map_err(|message| self.genesis_io_error(message))?;
        let genesis_request_digest = digest(&GenesisDigestMaterial {
            schema_name: "datum.project.genesis",
            schema_version: 1,
            project_name: request.project_name.trim(),
            project_id,
            units_source: &request.units_source,
            seed_application_digest: &seed_application_digest,
        })
        .map_err(|message| self.genesis_io_error(message))?;
        let receipt = ProjectUnitsSeedReceiptV2 {
            schema_name: RECEIPT_SCHEMA.to_owned(),
            schema_version: RECEIPT_VERSION,
            project_id,
            genesis_request_id: request.request_id,
            source,
            units_seed_catalog_digest: preview.units_seed_catalog_digest,
            items,
            profile_digest,
            genesis_request_digest: genesis_request_digest.clone(),
            seed_application_digest,
            creation_actor: actor.clone(),
        };
        let receipt_value = serde_json::to_value(&receipt)
            .map_err(|error| self.genesis_io_error(error.to_string()))?;
        let staging = staging_path(&destination, request.request_id, actor.invocation_id)
            .map_err(|message| self.genesis_io_error(message))?;
        prepare_staging(
            &staging,
            &StagingMarker {
                schema: "datum.project.genesis.staging.v1".to_owned(),
                request_id: request.request_id,
                invocation_id: actor.invocation_id,
                genesis_request_digest: genesis_request_digest.clone(),
            },
        )
        .map_err(|message| self.genesis_io_error(message))?;
        let payload = staging.join("project");

        let publication = (|| -> Result<(), String> {
            checkpoint(GenesisCheckpoint::StagingPrepared)?;
            bootstrap_native_project_with_units_receipt_value(
                &payload,
                GenesisSpec {
                    project_name: request.project_name.trim().to_owned(),
                    existing_ids: Some(crate::api::native_write::genesis::GenesisRootIds {
                        project: project_id,
                        schematic: genesis_child_id(project_id, "schematic"),
                        board: genesis_child_id(project_id, "board"),
                        rules: Some(genesis_child_id(project_id, "rules")),
                    }),
                },
                profile,
                receipt_value,
            )
            .map_err(|error| error.to_string())?;
            checkpoint(GenesisCheckpoint::ProjectBuilt)?;
            ProjectResolver::new(&payload)
                .resolve()
                .map_err(|error| error.to_string())?;
            checkpoint(GenesisCheckpoint::ProjectValidated)?;
            sync_tree(&payload)?;
            checkpoint(GenesisCheckpoint::StagingSynced)?;
            std::fs::rename(&payload, &destination).map_err(|error| error.to_string())?;
            sync_directory(
                destination
                    .parent()
                    .expect("normalized destination has parent"),
            )?;
            checkpoint(GenesisCheckpoint::ProjectPublished)?;
            Ok(())
        })();
        if let Err(message) = publication {
            if destination.exists() {
                cleanup_owned_staging(&staging, request.request_id, actor.invocation_id);
                return self.replay_or_refuse_existing(&destination, &request);
            }
            cleanup_owned_staging(&staging, request.request_id, actor.invocation_id);
            return Err(self.genesis_error(
                PreferenceErrorCodeV1::GenesisPublishFailed,
                "Project publication failed before the atomic commit point",
                BTreeMap::from([("reason".to_owned(), json!(message))]),
            ));
        }
        cleanup_owned_staging(&staging, request.request_id, actor.invocation_id);
        build_result(
            &destination,
            request.request_id,
            genesis_request_digest,
            receipt,
        )
        .map_err(|message| self.genesis_io_error(message))
    }

    fn seed_items(
        &self,
        source: &ProjectUnitsSourceV1,
        receipt_preview: &Value,
    ) -> Result<Vec<ProjectUnitsSeedItemV2>, PreferenceErrorV1> {
        let preview: ProjectUnitsSeedReceipt = serde_json::from_value(receipt_preview.clone())
            .map_err(|error| {
                self.genesis_error(
                    PreferenceErrorCodeV1::SeedIncomplete,
                    "Units preview receipt is malformed",
                    BTreeMap::from([("reason".to_owned(), json!(error.to_string()))]),
                )
            })?;
        let rows = self.rows();
        ACTIVE_UNITS_KEYS
            .iter()
            .map(|key| {
                let row = rows
                    .iter()
                    .find(|row| row.key.as_str() == *key)
                    .ok_or_else(|| {
                        self.genesis_error(
                            PreferenceErrorCodeV1::SeedIncomplete,
                            "Active Units seed is incomplete",
                            BTreeMap::from([("key".to_owned(), json!(key))]),
                        )
                    })?;
                let descriptor = self.registry().get(&row.key).expect("active Units key");
                let copied_value = preview.copied_values.get(*key).cloned().ok_or_else(|| {
                    self.genesis_error(
                        PreferenceErrorCodeV1::SeedIncomplete,
                        "Units preview receipt is missing an active value",
                        BTreeMap::from([("key".to_owned(), json!(key))]),
                    )
                })?;
                let (effective_source, provenance) = match source {
                    ProjectUnitsSourceV1::Global { .. } => {
                        let value_view = self.value_view(row)?;
                        let source = if row.user_value.is_some() {
                            "user"
                        } else {
                            "factory_default"
                        };
                        (source.to_owned(), value_view.provenance_summary)
                    }
                    ProjectUnitsSourceV1::Factory { profile_id } => (
                        "factory_profile".to_owned(),
                        format!("{profile_id} · version 1 · built_in"),
                    ),
                };
                Ok(ProjectUnitsSeedItemV2 {
                    key: (*key).to_owned(),
                    descriptor_schema_version: descriptor.schema_version,
                    copied_value,
                    effective_source,
                    provenance,
                })
            })
            .collect()
    }

    fn replay_or_refuse_existing(
        &self,
        destination: &Path,
        request: &ProjectGenesisRequestV1,
    ) -> Result<ProjectGenesisResultV1, PreferenceErrorV1> {
        let value: Value = serde_json::from_slice(
            &std::fs::read(destination.join("project.json"))
                .map_err(|error| self.genesis_io_error(error.to_string()))?,
        )
        .map_err(|error| self.genesis_io_error(error.to_string()))?;
        let receipt: ProjectUnitsSeedReceiptV2 = serde_json::from_value(
            value
                .get("project_units_seed_receipt")
                .cloned()
                .unwrap_or(Value::Null),
        )
        .map_err(|_| {
            self.genesis_error(
                PreferenceErrorCodeV1::ProjectTargetExists,
                "Destination already contains a Project with different genesis evidence",
                BTreeMap::new(),
            )
        })?;
        if receipt.genesis_request_id != request.request_id {
            return Err(self.genesis_error(
                PreferenceErrorCodeV1::ProjectTargetExists,
                "Destination already belongs to another Project genesis request",
                BTreeMap::new(),
            ));
        }
        let same_name =
            value.get("name").and_then(Value::as_str) == Some(request.project_name.trim());
        let same_id = request.project_id.is_none_or(|id| id == receipt.project_id);
        let expected_digest = digest(&GenesisDigestMaterial {
            schema_name: "datum.project.genesis",
            schema_version: 1,
            project_name: request.project_name.trim(),
            project_id: receipt.project_id,
            units_source: &request.units_source,
            seed_application_digest: &receipt.seed_application_digest,
        })
        .map_err(|message| self.genesis_io_error(message))?;
        if !same_name
            || !same_id
            || !source_matches(&receipt.source, &request.units_source)
            || expected_digest != receipt.genesis_request_digest
        {
            return Err(self.genesis_error(
                PreferenceErrorCodeV1::IdempotencyConflict,
                "Genesis request id was reused with different canonical content",
                BTreeMap::from([("request_id".to_owned(), json!(request.request_id))]),
            ));
        }
        build_result(
            destination,
            request.request_id,
            receipt.genesis_request_digest.clone(),
            receipt,
        )
        .map_err(|message| self.genesis_io_error(message))
    }

    fn genesis_io_error(&self, message: String) -> PreferenceErrorV1 {
        self.genesis_error(
            PreferenceErrorCodeV1::RepositoryIo,
            "Project genesis I/O failed",
            BTreeMap::from([("reason".to_owned(), json!(message))]),
        )
    }

    fn genesis_error(
        &self,
        code: PreferenceErrorCodeV1,
        message: &str,
        details: BTreeMap<String, Value>,
    ) -> PreferenceErrorV1 {
        self.error(code, message, details, None)
    }
}

fn receipt_source(
    requested: &ProjectUnitsSourceV1,
    generation: Option<super::repository::GenerationRef>,
) -> ProjectUnitsReceiptSourceV2 {
    match requested {
        ProjectUnitsSourceV1::Global { .. } => {
            let defaults_profile_id = generation.is_none().then(|| FACTORY_PROFILE.to_owned());
            ProjectUnitsReceiptSourceV2::Global {
                generation,
                defaults_profile_id,
            }
        }
        ProjectUnitsSourceV1::Factory { profile_id } => ProjectUnitsReceiptSourceV2::Factory {
            profile_id: profile_id.clone(),
            profile_version: 1,
        },
    }
}

fn source_matches(receipt: &ProjectUnitsReceiptSourceV2, request: &ProjectUnitsSourceV1) -> bool {
    match (receipt, request) {
        (
            ProjectUnitsReceiptSourceV2::Global { generation, .. },
            ProjectUnitsSourceV1::Global {
                expected_generation,
            },
        ) => expected_generation
            .as_ref()
            .is_none_or(|expected| generation.as_ref() == Some(expected)),
        (
            ProjectUnitsReceiptSourceV2::Factory { profile_id, .. },
            ProjectUnitsSourceV1::Factory {
                profile_id: requested,
            },
        ) => profile_id == requested,
        _ => false,
    }
}

fn normalize_absent_destination(destination: &Path) -> Result<PathBuf, String> {
    let absolute = if destination.is_absolute() {
        destination.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| error.to_string())?
            .join(destination)
    };
    let name = absolute
        .file_name()
        .ok_or_else(|| "Project destination requires a terminal name".to_owned())?;
    let parent = absolute
        .parent()
        .ok_or_else(|| "Project destination requires a parent".to_owned())?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    Ok(parent.join(name))
}

pub(super) fn staging_path(
    destination: &Path,
    request: uuid::Uuid,
    invocation: uuid::Uuid,
) -> Result<PathBuf, String> {
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Project destination name is not UTF-8".to_owned())?;
    Ok(destination.with_file_name(format!(".{name}.datum-genesis-{request}-{invocation}")))
}

fn prepare_staging(path: &Path, marker: &StagingMarker) -> Result<(), String> {
    if path.exists() {
        cleanup_owned_staging(path, marker.request_id, marker.invocation_id);
    }
    std::fs::create_dir(path).map_err(|error| error.to_string())?;
    let marker_path = path.join(".datum-genesis-incomplete");
    let bytes = format!(
        "{}\n",
        to_json_deterministic(marker).map_err(|error| error.to_string())?
    );
    std::fs::write(&marker_path, bytes).map_err(|error| error.to_string())?;
    File::open(&marker_path)
        .and_then(|file| file.sync_all())
        .map_err(|error| error.to_string())?;
    sync_directory(path)
}

fn cleanup_owned_staging(path: &Path, request: uuid::Uuid, invocation: uuid::Uuid) {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(_) => return,
    };
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return;
    }
    let marker = std::fs::read(path.join(".datum-genesis-incomplete"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<StagingMarker>(&bytes).ok());
    if marker
        .as_ref()
        .is_some_and(|value| value.request_id == request && value.invocation_id == invocation)
    {
        let _ = std::fs::remove_dir_all(path);
    }
}

fn sync_tree(root: &Path) -> Result<(), String> {
    let mut dirs = vec![root.to_path_buf()];
    let mut index = 0;
    while index < dirs.len() {
        let dir = dirs[index].clone();
        for entry in std::fs::read_dir(&dir).map_err(|error| error.to_string())? {
            let path = entry.map_err(|error| error.to_string())?.path();
            let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
            if metadata.file_type().is_symlink() {
                return Err("staging tree contains a symlink".to_owned());
            }
            if metadata.is_dir() {
                dirs.push(path);
            } else {
                File::open(path)
                    .and_then(|file| file.sync_all())
                    .map_err(|error| error.to_string())?;
            }
        }
        index += 1;
    }
    for dir in dirs.into_iter().rev() {
        sync_directory(&dir)?;
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| error.to_string())
}

fn build_result(
    destination: &Path,
    request_id: uuid::Uuid,
    genesis_request_digest: String,
    receipt: ProjectUnitsSeedReceiptV2,
) -> Result<ProjectGenesisResultV1, String> {
    ProjectResolver::new(destination)
        .resolve()
        .map_err(|error| error.to_string())?;
    let published_manifest_digests = [
        "project.json",
        "schematic/schematic.json",
        "board/board.json",
        "rules/rules.json",
    ]
    .into_iter()
    .map(|relative| {
        let bytes = std::fs::read(destination.join(relative)).map_err(|error| error.to_string())?;
        Ok((
            relative.to_owned(),
            format!("sha256:{:x}", Sha256::digest(bytes)),
        ))
    })
    .collect::<Result<BTreeMap<_, _>, String>>()?;
    Ok(ProjectGenesisResultV1 {
        project_id: receipt.project_id,
        request_id,
        genesis_request_digest,
        project_root_identity: destination.to_string_lossy().into_owned(),
        units_receipt: receipt,
        published_manifest_digests,
    })
}

fn digest<T: Serialize>(value: &T) -> Result<String, String> {
    let canonical = to_json_deterministic(value).map_err(|error| error.to_string())?;
    Ok(format!("sha256:{:x}", Sha256::digest(canonical.as_bytes())))
}

fn genesis_child_id(project_id: uuid::Uuid, role: &str) -> uuid::Uuid {
    uuid::Uuid::new_v5(
        &project_id,
        format!("datum.project.genesis.v1/{role}").as_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::native_write::project::{project_display_units, project_units_seed_evidence};
    use crate::ir::units::profile_to_descriptor_values;
    use crate::preferences::{
        FixedPreferenceLocationProvider, PreferenceActorKindV1, PreferenceLocations,
    };

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
                    profile_id: FACTORY_PROFILE.to_owned(),
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
    fn factory_genesis_publishes_one_complete_eight_item_project_without_preferences() {
        let fixture = Fixture::new("factory");
        let actor = fixture.actor();
        let request = fixture.request("Factory Project");
        let result = fixture
            .service
            .create_project(request.clone(), &actor)
            .unwrap();

        assert_eq!(result.project_id, request.project_id.unwrap());
        assert_eq!(result.units_receipt.items.len(), 8);
        assert_eq!(
            result
                .units_receipt
                .items
                .iter()
                .map(|item| item.key.as_str())
                .collect::<Vec<_>>(),
            ACTIVE_UNITS_KEYS
        );
        assert!(result.units_receipt.items.iter().all(|item| {
            item.effective_source == "factory_profile" && item.provenance.contains(FACTORY_PROFILE)
        }));
        assert!(!fixture.root.join("config/datum/preferences").exists());
        assert!(
            !request
                .destination
                .join(".datum-genesis-incomplete")
                .exists()
        );

        let model = ProjectResolver::new(&request.destination)
            .resolve()
            .unwrap();
        let copied: BTreeMap<_, _> = result
            .units_receipt
            .items
            .iter()
            .map(|item| (item.key.clone(), item.copied_value.clone()))
            .collect();
        assert_eq!(
            copied,
            profile_to_descriptor_values(project_display_units(&model).unwrap())
        );
        let evidence = project_units_seed_evidence(&model).unwrap();
        assert_eq!(evidence.copied_values, copied);
        assert!(evidence.source_summary.contains("Factory"));
    }

    #[test]
    fn identical_retry_replays_and_conflicting_retry_never_changes_the_project() {
        let fixture = Fixture::new("retry");
        let actor = fixture.actor();
        let request = fixture.request("Retry Project");
        let first = fixture
            .service
            .create_project(request.clone(), &actor)
            .unwrap();
        let before = std::fs::read(request.destination.join("project.json")).unwrap();
        let replay = fixture
            .service
            .create_project(request.clone(), &actor)
            .unwrap();
        assert_eq!(replay, first);
        assert_eq!(
            std::fs::read(request.destination.join("project.json")).unwrap(),
            before
        );

        let mut conflict = request.clone();
        conflict.project_name = "Different Name".to_owned();
        let refusal = fixture
            .service
            .create_project(conflict, &actor)
            .unwrap_err();
        assert_eq!(refusal.code, PreferenceErrorCodeV1::IdempotencyConflict);

        let mut other = request;
        other.request_id = uuid::Uuid::new_v4();
        let refusal = fixture.service.create_project(other, &actor).unwrap_err();
        assert_eq!(refusal.code, PreferenceErrorCodeV1::ProjectTargetExists);
    }
}
