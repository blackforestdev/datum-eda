//! Build-only native entry points. Canonical commit/proposal policy owns writes.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

#[path = "library_asset_change_review.rs"]
mod change_review;
#[path = "library_asset_selection.rs"]
mod selection;

use super::{
    context::{PreparedWrite, WriteProvenance, build_batch},
    registry::NativeWriteContext,
};

/// Trusted process startup only. Projects, publishers and native requests cannot
/// enable library loading or raise these limits. No production defaults are
/// selected here; embedders and all three application binaries use this owner.
pub fn initialize_library_service_from_environment() -> Result<bool, EngineError> {
    let Some(path) = std::env::var_os("DATUM_LIBRARY_RESOURCE_POLICY_FILE") else {
        return Ok(false);
    };
    let path = std::path::PathBuf::from(path);
    if !path.is_absolute() || !std::fs::metadata(&path)?.is_file() {
        return Err(refusal(
            "library host policy requires an absolute regular-file path",
        ));
    }
    let limits = read_host_policy(std::io::BufReader::new(std::fs::File::open(path)?))?;
    crate::substrate::library_asset_resolver::configure_server_policy(limits)?;
    Ok(true)
}

fn read_host_policy(reader: impl std::io::Read) -> Result<LibraryAssetLimits, EngineError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct DeclaredLimits {
        metadata_bytes: usize,
        blob_bytes: usize,
        files: usize,
        depth: usize,
        total_bytes: usize,
    }
    let declared: DeclaredLimits = serde_json::from_reader(reader)?;
    Ok(LibraryAssetLimits {
        metadata_bytes: declared.metadata_bytes,
        blob_bytes: declared.blob_bytes,
        files: declared.files,
        depth: declared.depth,
        total_bytes: declared.total_bytes,
    })
}
use crate::{
    error::EngineError,
    library_assets::{
        Actor, AssetDigest, AssetProvenance, AssetRevisionRef, PROFILE, encode_metadata,
        lock::ProjectLibraryLock,
        manifest::{LibraryDiscovery, LibraryReleaseManifest},
    },
    substrate::{
        Operation, SourceShardKind,
        library_asset_operations::{
            LibraryAssetOperation, LibraryContent, LibraryRecordType, LibraryStateValues, refusal,
        },
        library_asset_resolver::{LibraryAssetLimits, read_state, resolve_state, server_policy},
    },
};

/// One build-only result for existing native preview/Proposal/commit consumers.
/// The summary describes this prepared change; it supplies no review, rights,
/// publication or write authority and is not a replacement for the guarded batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedLibraryChange {
    pub prepared: PreparedWrite,
    pub summary: Value,
}

/// Adjacent tagging keeps profile adoption, candidate saving and project adoption
/// distinct, without adding an operation family for each library object kind.
/// Service limits and policy configuration are deliberately absent from the wire.
#[derive(Debug, Deserialize)]
#[serde(
    tag = "action",
    content = "request",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum LibraryChangeRequest {
    AdoptProfile,
    SaveCandidate(CandidateRequest),
    Adopt {
        expected_catalog: AssetDigest,
        roots: Vec<AssetRevisionRef>,
        physical_geometry: Option<crate::substrate::component_pad_source::PhysicalGeometryIntent>,
    },
    AdoptSelection {
        expected_catalog: AssetDigest,
        selected: AssetRevisionRef,
        physical_geometry: Option<crate::substrate::component_pad_source::PhysicalGeometryIntent>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CandidateRequest {
    expected_catalog: AssetDigest,
    change: CandidateChange,
    changed_at: Option<String>,
    provenance_override: Option<AssetProvenance>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum CandidateChange {
    CaptureNative {
        asset_id: Uuid,
    },
    Revise {
        base: AssetRevisionRef,
        /// Keep authored numeric tokens and duplicate keys observable until the
        /// existing exact-JSON owner qualifies the structured editor snapshot.
        payload_json: Option<String>,
    },
}

impl CandidateRequest {
    fn into_core(
        self,
        limits: LibraryAssetLimits,
    ) -> Result<super::library_asset_adoption::CandidateRequest, EngineError> {
        use super::library_asset_adoption::CandidateChange as CoreChange;
        let change = match self.change {
            CandidateChange::CaptureNative { asset_id } => CoreChange::CaptureNative { asset_id },
            CandidateChange::Revise { base, payload_json } => CoreChange::revision(
                base,
                payload_json,
                self.provenance_override.is_some(),
                limits.blob_bytes.min(limits.total_bytes),
            )?,
        };
        Ok(super::library_asset_adoption::CandidateRequest {
            expected_catalog: self.expected_catalog,
            change,
            changed_at: self.changed_at,
            provenance_override: self.provenance_override,
        })
    }
}

pub(super) fn edited_payload(raw: &str, limit: usize) -> Result<Value, EngineError> {
    if raw.len() > limit {
        return Err(refusal("edited library payload exceeds the service limit"));
    }
    let value = serde_json::from_str(raw)?;
    crate::library_assets::verify_native_snapshot(raw.as_bytes(), &value)?;
    Ok(value)
}

/// Exact selection anchors for read-only physical correspondence discovery.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeometrySelectionRequest {
    expected_catalog: AssetDigest,
    selected: AssetRevisionRef,
}

/// Read-only correspondence discovery for the existing qualified review service.
/// The returned IDs are review inputs, not a publication or commit certificate.
pub fn inspect_geometry_selection(
    context: &NativeWriteContext<'_>,
    request: Value,
) -> Result<Value, EngineError> {
    let request: GeometrySelectionRequest = serde_json::from_value(request)?;
    let limits = server_policy()?;
    let roots = selection::update_roots(
        context,
        &request.expected_catalog,
        &request.selected,
        limits,
    )?;
    let plan = super::library_asset_adoption::prepare_adoption(
        context,
        WriteProvenance::new(
            "library-inspection",
            crate::substrate::CommitSource::Manual,
            "Inspect exact physical library update",
        ),
        super::library_asset_adoption::AdoptionRequest::discover(
            request.expected_catalog.clone(),
            roots,
        ),
        limits,
    )?;
    if plan.prepared.is_some() {
        return Err(refusal("geometry inspection unexpectedly supplied a write"));
    }
    Ok(
        json!({"project_id":context.model.project.project_id,"model_revision":context.model.model_revision,
        "transaction_tip":crate::revision::transaction_tip(&context.model.journal),
        "expected_catalog":request.expected_catalog,"selected":request.selected,
        "physical_geometry":plan.physical_geometry,"physical_geometry_updates":plan.physical_geometry_updates}),
    )
}

/// Build a deliberate library action without source writes or Proposal acceptance.
/// Normal request guards and catalog CAS remain independent commit requirements.
pub fn prepare_library_change(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: Value,
) -> Result<PreparedLibraryChange, EngineError> {
    let request: LibraryChangeRequest = serde_json::from_value(request)?;
    let limits = server_policy()?;
    let (prepared, mut summary) = match request {
        LibraryChangeRequest::AdoptProfile => (
            prepare_profile_adoption(context, provenance, limits)?,
            json!({"action":"adopt_profile","project_schema_version":2,
                "library_lock_baseline":"empty","undo_to_previous_format":false,
                "legacy_common_metadata_coverage":"not_inferred"}),
        ),
        LibraryChangeRequest::SaveCandidate(request) => {
            let plan = super::library_asset_adoption::prepare_candidate(
                context,
                provenance,
                request.into_core(limits)?,
                limits,
            )?;
            let summary = json!({"action":"save_candidate","candidate":plan.candidate,
                "changed_assets":plan.changed_assets,"active_lock_unchanged":true});
            (plan.prepared, summary)
        }
        LibraryChangeRequest::Adopt {
            expected_catalog,
            roots,
            physical_geometry,
        } => {
            let plan = super::library_asset_adoption::prepare_adoption(
                context,
                provenance,
                super::library_asset_adoption::AdoptionRequest::new(
                    expected_catalog,
                    roots,
                    physical_geometry,
                ),
                limits,
            )?;
            let summary = json!({"action":"adopt","before":plan.before,"after":plan.after,
                "changed_materializations":plan.changed_materializations,
                "affected_components":plan.affected_components,"physical_geometry_updates":plan.physical_geometry_updates});
            (
                plan.prepared
                    .ok_or_else(|| refusal("inspection is not an adoption"))?,
                summary,
            )
        }
        LibraryChangeRequest::AdoptSelection {
            expected_catalog,
            selected,
            physical_geometry,
        } => {
            let roots = selection::update_roots(context, &expected_catalog, &selected, limits)?;
            let plan = super::library_asset_adoption::prepare_adoption(
                context,
                provenance,
                super::library_asset_adoption::AdoptionRequest::new(
                    expected_catalog,
                    roots.clone(),
                    physical_geometry,
                ),
                limits,
            )?;
            let summary = json!({"action":"adopt_selection", "selected":selected,
                "selected_roots":roots,"before":plan.before,"after":plan.after,
                "changed_materializations":plan.changed_materializations,
                "affected_components":plan.affected_components,"physical_geometry_updates":plan.physical_geometry_updates});
            (
                plan.prepared
                    .ok_or_else(|| refusal("inspection is not an adoption"))?,
                summary,
            )
        }
    };
    change_review::attach(context, &prepared, &mut summary, limits)?;
    Ok(PreparedLibraryChange { prepared, summary })
}

/// Inspect without caller-selected limits or implicit format adoption. Legacy
/// native metadata remains available even when revision support is unconfigured.
pub fn inspect_local_library(context: &NativeWriteContext<'_>) -> Result<Value, EngineError> {
    let manifest = captured_metadata(context, SourceShardKind::ProjectManifest, "project.json")?;
    if manifest.get("library_lock").is_none() {
        return Ok(legacy_inspection(context));
    }
    inspect_library_assets(context, server_policy()?)
}

fn captured_metadata(
    context: &NativeWriteContext<'_>,
    kind: SourceShardKind,
    relative_path: &str,
) -> Result<Value, EngineError> {
    let mut sources = context
        .model
        .source_shards
        .iter()
        .filter(|s| s.kind == kind && s.relative_path == relative_path);
    let source = sources
        .next()
        .ok_or_else(|| refusal("library inspection source is unavailable"))?;
    if sources.next().is_some() {
        return Err(refusal("library inspection source is ambiguous"));
    }
    context.model.capture_source_shard_value(source.shard_id)
}

fn legacy_inspection(context: &NativeWriteContext<'_>) -> Value {
    json!({"managed":false,"project_id":context.model.project.project_id,
        "model_revision":&context.model.model_revision,"assets":[],"available_assets":[],
        "catalog_manifest":null,"catalog_roots":[],"active_closure_digest":null,
        "common_record_status":"not_adopted","legacy_metadata":"retained in native definitions"})
}

/// This explicit format action is separate from reversible asset changes.
pub fn prepare_profile_adoption(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    limits: LibraryAssetLimits,
) -> Result<PreparedWrite, EngineError> {
    let limits = crate::substrate::library_asset_resolver::restrict_limits(limits)?;
    let manifest = context
        .model
        .materialized_source_shard_value(SourceShardKind::ProjectManifest)?;
    if manifest
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(1)
        || manifest.get("library_lock").is_some()
    {
        return Err(refusal(
            "project is not an unadopted native version1 project",
        ));
    }
    let lock = ProjectLibraryLock::empty()?;
    let release = LibraryReleaseManifest {
        schema_version: 1,
        collection_id: context.model.project.project_id,
        release_id: Uuid::new_v4(),
        publisher: Actor {
            recorded_id: Some(provenance.actor.clone()),
            display_name: None,
            verification_evidence: vec![],
        },
        profile: PROFILE.into(),
        roots: vec![],
        closure_digest: lock.closure_digest.clone(),
        objects: vec![],
        blobs: vec![],
        publication_record_refs: vec![],
        extensions: None,
    };
    let release_bytes = encode_metadata(&release)?;
    let digest = release.fingerprint()?;
    let discovery = LibraryDiscovery {
        schema_version: 1,
        collection_id: release.collection_id,
        profile: PROFILE.into(),
        manifest_ref: digest.clone(),
    };
    let state = LibraryStateValues {
        lock: encode_metadata(&lock)?,
        catalog: encode_metadata(&discovery)?,
    };
    let preimage = crate::substrate::library_asset_resolver::manifest_preimage(
        context.project_root,
        limits.metadata_bytes,
    )?;
    if serde_json::from_str::<serde_json::Value>(&preimage)? != manifest {
        return Err(refusal("stale project manifest snapshot"));
    }
    let operation = LibraryAssetOperation::AdoptProfile {
        project_id: context.model.project.project_id,
        preimage,
        state,
        content: vec![LibraryContent::Record {
            record_type: LibraryRecordType::ReleaseManifest,
            fingerprint: digest.as_str().into(),
            bytes: release_bytes,
        }],
    };
    crate::substrate::library_asset_journal_ops::validate_preimages(
        context.project_root,
        &operation,
        limits,
    )?;
    Ok(PreparedWrite {
        batch: build_batch(
            context.model,
            provenance,
            vec![Operation::LibraryAssets { operation }],
        )?,
        primary_object_id: Some(context.model.project.project_id),
    })
}

/// The caller supplies already-reviewed state/content. This prepares the state
/// portion only; source materialization and binding operations must be composed
/// atomically by the adoption planner and revalidated by batch admission.
pub fn prepare_library_state(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    state: LibraryStateValues,
    content: Vec<LibraryContent>,
    limits: LibraryAssetLimits,
) -> Result<PreparedWrite, EngineError> {
    let limits = crate::substrate::library_asset_resolver::restrict_limits(limits)?;
    let previous = read_state(context.project_root, limits)?;
    let operation = LibraryAssetOperation::SetState {
        project_id: context.model.project.project_id,
        previous,
        state,
        content,
    };
    let files = operation.content_map(
        limits.metadata_bytes,
        limits.blob_bytes,
        limits.files,
        limits.total_bytes,
    )?;
    resolve_state(context.project_root, operation.state()?, &files, limits)?;
    Ok(PreparedWrite {
        batch: build_batch(
            context.model,
            provenance,
            vec![Operation::LibraryAssets { operation }],
        )?,
        primary_object_id: Some(context.model.project.project_id),
    })
}

/// Inspection reports the managed snapshot; it never treats the local counter
/// or an empty baseline as portable history for legacy objects.
pub fn inspect_library_assets(
    context: &NativeWriteContext<'_>,
    limits: LibraryAssetLimits,
) -> Result<serde_json::Value, EngineError> {
    let manifest = captured_metadata(context, SourceShardKind::ProjectManifest, "project.json")?;
    if manifest.get("library_lock").is_none() {
        return Ok(legacy_inspection(context));
    }
    let state = read_state(context.project_root, limits)?;
    for (kind, path, bytes) in [
        (
            SourceShardKind::LibraryLock,
            "library/lock.json",
            &state.lock,
        ),
        (
            SourceShardKind::LibraryCatalog,
            "library/library.json",
            &state.catalog,
        ),
    ] {
        if captured_metadata(context, kind, path)? != serde_json::from_slice::<Value>(bytes)? {
            return Err(refusal(
                "library changed since the inspected model snapshot",
            ));
        }
    }
    let snapshot = resolve_state(
        context.project_root,
        &state,
        &std::collections::BTreeMap::new(),
        limits,
    )?;
    let assets: Vec<_> = snapshot
        .lock()
        .assets
        .iter()
        .map(|entry| {
            let verified = &snapshot.assets()[&entry.asset];
            json!({"revision":entry.asset,"source_path":entry.source_path,
            "provenance":verified.provenance()})
        })
        .collect();
    let available_assets: Vec<_> = snapshot
        .assets()
        .iter()
        .map(|(reference, verified)| {
            let active = snapshot
                .lock()
                .assets
                .iter()
                .find(|a| a.asset == *reference);
            let local = active.and_then(|a| context.model.objects.get(&a.local_object_id));
            let contribution = if verified.revision().kind == crate::library_assets::AssetKind::Part
            {
                Some(crate::library_assets::contribution::inspect_part_report(
                    reference,
                    snapshot.assets(),
                    snapshot.reviews(),
                    crate::library_assets::ClosureLimits {
                        max_nodes: limits.files,
                        max_depth: limits.depth,
                    },
                ))
            } else {
                None
            };
            json!({"revision":reference,"kind":verified.revision().kind,
                "name":verified.payload().value().get("name").and_then(Value::as_str),
                "payload_ref":verified.revision().payload_ref,
                "parents":verified.revision().parents,
                "dependencies":verified.revision().dependencies,
                "provenance":verified.provenance(),
                "active":active.is_some(),
                "catalog_root":snapshot.manifest().roots.contains(reference),
                "active_source_path":active.map(|a| &a.source_path),
                "local_object_revision":local.map(|o| o.object_revision.0),
                "contribution":contribution})
        })
        .collect();
    Ok(
        json!({"managed":true,"project_id":context.model.project.project_id,
        "model_revision":&context.model.model_revision,
        "catalog_manifest":snapshot.catalog().manifest_ref,
        "catalog_roots":snapshot.manifest().roots,
        "active_closure_digest":snapshot.lock().closure_digest,
        "assets":assets,"available_assets":available_assets,
        "native_materialization":"not_evaluated_by_this_read",
        "release_manifest":{"collection_id":snapshot.manifest().collection_id,
            "release_id":snapshot.manifest().release_id,
            "publisher_record":snapshot.manifest().publisher,
            "publication_record_refs":snapshot.manifest().publication_record_refs},
        "publication_status":"not_inferred","legacy_coverage":"not_inferred"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::native_write::{
        genesis::{GenesisSpec, bootstrap_native_project},
        test_support::temp_project_root,
    };
    use crate::substrate::ProjectResolver;

    #[test]
    fn host_policy_requires_all_declared_integer_fields_and_no_extra_input() {
        let valid = json!({"metadata_bytes":8,"blob_bytes":16,"files":3,"depth":2,"total_bytes":4});
        let parse = |v: &Value| read_host_policy(serde_json::to_vec(v).unwrap().as_slice());
        assert_eq!(parse(&valid).unwrap().files, 3);
        for field in [
            "metadata_bytes",
            "blob_bytes",
            "files",
            "depth",
            "total_bytes",
        ] {
            let mut missing = valid.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(parse(&missing).is_err());
            for invalid in [json!(-1), json!(1.5), json!("4"), Value::Null] {
                let mut changed = valid.clone();
                changed[field] = invalid;
                assert!(parse(&changed).is_err());
            }
        }
        let mut extra = valid;
        extra["enable"] = json!(true);
        assert!(parse(&extra).is_err());
        assert!(read_host_policy(br#"{"metadata_bytes":1,"metadata_bytes":2,"blob_bytes":1,"files":1,"depth":1,"total_bytes":1}"#.as_slice()).is_err());
    }

    #[test]
    fn trusted_host_policy_is_explicit_and_process_local() {
        const CHILD: &str = "DATUM_LIBRARY_HOST_POLICY_TEST_CHILD";
        const POLICY: &str = "DATUM_LIBRARY_RESOURCE_POLICY_FILE";
        const NAME: &str = "api::native_write::library_assets::tests::trusted_host_policy_is_explicit_and_process_local";
        if let Ok(mode) = std::env::var(CHILD) {
            use crate::substrate::library_asset_resolver::server_policy;
            match mode.as_str() {
                "absent" => {
                    assert!(!initialize_library_service_from_environment().unwrap());
                    assert!(server_policy().is_err());
                }
                "zero" => {
                    assert!(initialize_library_service_from_environment().is_err());
                    assert!(server_policy().is_err());
                }
                "valid" => {
                    assert!(initialize_library_service_from_environment().unwrap());
                    assert_eq!(
                        server_policy().unwrap(),
                        LibraryAssetLimits {
                            metadata_bytes: 4,
                            blob_bytes: 4,
                            files: 3,
                            depth: 2,
                            total_bytes: 4,
                        }
                    );
                    assert!(initialize_library_service_from_environment().unwrap());
                }
                _ => panic!("unexpected child mode"),
            }
            return;
        }
        let root = temp_project_root("host_policy_control");
        std::fs::create_dir_all(&root).unwrap();
        for mode in ["absent", "valid", "zero"] {
            // Schema/initialization controls only, never production budgets.
            let path = root.join(format!("{mode}.json"));
            std::fs::write(
                &path,
                serde_json::to_vec(&json!({"metadata_bytes":8,
                "blob_bytes":16,"files":if mode=="zero" {0} else {3},
                "depth":2,"total_bytes":4}))
                .unwrap(),
            )
            .unwrap();
            let mut command = std::process::Command::new(std::env::current_exe().unwrap());
            command
                .args(["--exact", NAME, "--nocapture"])
                .env(CHILD, mode);
            if mode == "absent" {
                command.env_remove(POLICY);
            } else {
                command.env(POLICY, &path);
            }
            let output = command.output().unwrap();
            assert!(
                output.status.success()
                    && String::from_utf8_lossy(&output.stdout).contains("1 passed"),
                "{mode}: {}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn edited_payload_preserves_unknown_values_and_refuses_lossy_or_ambiguous_input() {
        let raw = r#"{"unknown":9007199254740993,"fraction":0.125,"ordered":["z","a"]}"#;
        let value = edited_payload(raw, raw.len()).unwrap();
        assert_eq!(value["unknown"].as_u64(), Some(9_007_199_254_740_993));
        assert_eq!(value["ordered"], json!(["z", "a"]));
        assert!(edited_payload(raw, raw.len() - 1).is_err());
        for raw in [
            r#"{"unknown":0.10000000000000001}"#,
            r#"{"unknown":1,"unknown":2}"#,
        ] {
            assert!(edited_payload(raw, raw.len()).is_err(), "{raw}");
        }
    }

    #[test]
    fn legacy_inspection_does_not_adopt_or_invent_history_and_refuses_a_stale_snapshot() {
        let root = temp_project_root("library_inspection_basis");
        bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Library inspection".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let model = ProjectResolver::new(&root).resolve().unwrap();
        let context = NativeWriteContext {
            model: &model,
            project_root: &root,
        };
        let manifest = root.join("project.json");
        let before = std::fs::read(&manifest).unwrap();
        let read = inspect_local_library(&context).unwrap();
        assert_eq!(read["managed"], false);
        assert_eq!(read["common_record_status"], "not_adopted");
        assert!(read["catalog_manifest"].is_null());
        assert_eq!(read["available_assets"], json!([]));
        assert_eq!(std::fs::read(&manifest).unwrap(), before);
        assert!(!root.join("library").exists());

        let mut changed: Value = serde_json::from_slice(&before).unwrap();
        changed["name"] = json!("Unjournaled different project name");
        std::fs::write(&manifest, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(inspect_local_library(&context).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
