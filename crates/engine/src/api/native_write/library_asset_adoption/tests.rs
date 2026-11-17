//! Producer controls without installing any process-global service policy.
use super::*;
use crate::substrate::{CommitSource, ProjectResolver};
use serde_json::json;
use std::path::{Path, PathBuf};

fn limits() -> LibraryAssetLimits {
    // Fixture limits only; these do not configure a production server.
    LibraryAssetLimits {
        metadata_bytes: 65536,
        blob_bytes: 65536,
        files: 128,
        depth: 16,
        total_bytes: 1048576,
    }
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "migrator",
        CommitSource::Test,
        "Capture current engineering data",
    )
}
fn request(change: CandidateChange) -> CandidateRequest {
    CandidateRequest {
        expected_catalog: AssetDigest::blob(b"test catalog basis"),
        change,
        changed_at: None,
        provenance_override: None,
    }
}
fn fixture() -> (PathBuf, Uuid, Uuid, Uuid) {
    let root = super::super::test_support::temp_project_root("library_asset_capture");
    super::super::test_support::write_minimal_project(&root, Uuid::new_v4(), Uuid::new_v4());
    let mut project: Value =
        serde_json::from_slice(&std::fs::read(root.join("project.json")).unwrap()).unwrap();
    project["pools"] = json!([{"path":"pool","priority":1}]);
    std::fs::write(
        root.join("project.json"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    let (unit, first, second) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    for (kind, id, value) in [
        (
            "units",
            unit,
            json!({"schema_version":1,"uuid":unit,"object_revision":37,"name":"Reusable pins","manufacturer":"","pins":{},"tags":[],"vendor_extension":{"unclassified":23}}),
        ),
        (
            "symbols",
            first,
            json!({"schema_version":1,"uuid":first,"name":"First","unit":unit}),
        ),
        (
            "symbols",
            second,
            json!({"schema_version":1,"uuid":second,"name":"Second","unit":unit}),
        ),
    ] {
        let path = root.join(format!("pool/{kind}/{id}.json"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }
    (root, unit, first, second)
}

fn verified(draft: &capture::Draft) -> BTreeMap<AssetRevisionRef, VerifiedAsset> {
    let files: BTreeMap<_, _> = draft
        .content
        .iter()
        .map(|c| (c.fingerprint().unwrap(), c.bytes()))
        .collect();
    let mut result = BTreeMap::new();
    for item in &draft.content {
        if let LibraryContent::Record {
            record_type: LibraryRecordType::AssetRevision,
            bytes,
            ..
        } = item
        {
            let revision: AssetRevision =
                crate::library_assets::decode_metadata(bytes, limits().metadata_bytes).unwrap();
            let reference = revision.reference().unwrap();
            let asset = VerifiedAsset::read(
                &reference,
                bytes,
                files[&revision.provenance_ref],
                files[&revision.payload_ref],
                limits().blob_bytes,
                counter_policy(revision.kind, 1).unwrap(),
            )
            .unwrap();
            result.insert(reference, asset);
        }
    }
    result
}
fn retain_fixture(root: &Path, draft: &capture::Draft) {
    for content in &draft.content {
        let path = root.join(content.relative_path().unwrap());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content.bytes()).unwrap();
    }
}

#[test]
fn native_capture_preserves_exact_source_and_does_not_credit_migrator_as_author() {
    let (root, unit, _, _) = fixture();
    let path = root.join(format!("pool/units/{unit}.json"));
    let original = std::fs::read(&path).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    let draft = capture::capture_inventory(
        &context,
        &[],
        &BTreeMap::new(),
        &provenance(),
        request(CandidateChange::CaptureNative { asset_id: unit }),
        limits(),
    )
    .unwrap();
    let assets = verified(&draft);
    let captured = &assets[&draft.selected];
    assert!(captured.provenance().authors.is_empty());
    assert_eq!(captured.provenance().created_at, None);
    assert_eq!(captured.provenance().origin_kind, OriginKind::Migration);
    assert_eq!(
        captured
            .provenance()
            .change_actor
            .as_ref()
            .unwrap()
            .recorded_id
            .as_deref(),
        Some("migrator")
    );
    assert_eq!(
        captured.provenance().sources[0].retained_blob,
        Some(AssetDigest::blob(&original))
    );
    assert!(captured.payload().value().get("object_revision").is_none());
    assert_eq!(
        captured.payload().value()["vendor_extension"]["unclassified"],
        23
    );
    assert_eq!(std::fs::read(path).unwrap(), original);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_capture_refuses_unknown_numeric_rounding_without_rewriting_source() {
    let (root, unit, _, _) = fixture();
    let path = root.join(format!("pool/units/{unit}.json"));
    let mut raw = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    let end = raw.rfind('}').unwrap();
    raw.insert_str(end, ",\"precise_vendor_field\":0.10000000000000001");
    std::fs::write(&path, &raw).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    assert!(
        capture::capture_inventory(
            &context,
            &[],
            &BTreeMap::new(),
            &provenance(),
            request(CandidateChange::CaptureNative { asset_id: unit }),
            limits()
        )
        .is_err()
    );
    assert_eq!(std::fs::read_to_string(path).unwrap(), raw);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn shared_unit_edit_repins_both_candidate_symbols_and_retains_old_payloads() {
    let (root, unit, first, second) = fixture();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    let first_draft = capture::capture_inventory(
        &context,
        &[],
        &BTreeMap::new(),
        &provenance(),
        request(CandidateChange::CaptureNative { asset_id: first }),
        limits(),
    )
    .unwrap();
    retain_fixture(&root, &first_draft);
    let mut assets = verified(&first_draft);
    let second_draft = capture::capture_inventory(
        &context,
        &first_draft.roots,
        &assets,
        &provenance(),
        request(CandidateChange::CaptureNative { asset_id: second }),
        limits(),
    )
    .unwrap();
    retain_fixture(&root, &second_draft);
    assets.extend(verified(&second_draft));
    let base = assets.keys().find(|r| r.asset_id == unit).unwrap().clone();
    let mut payload = assets[&base].payload().value().clone();
    payload["name"] = "Renamed shared unit".into();
    let changed = capture::capture_inventory(
        &context,
        &second_draft.roots,
        &assets,
        &provenance(),
        request(CandidateChange::Revise {
            base: base.clone(),
            payload,
        }),
        limits(),
    )
    .unwrap();
    let updated = verified(&changed);
    let mut all_refs = changed.references.clone();
    all_refs.sort();
    all_refs.dedup_by_key(|r| r.asset_id);
    assert_eq!(all_refs.len(), 3);
    assert_eq!(changed.changed.len(), 3);
    for root_ref in changed.roots.iter().filter(|r| r.asset_id != unit) {
        assert_ne!(
            root_ref,
            &second_draft
                .roots
                .iter()
                .find(|r| r.asset_id == root_ref.asset_id)
                .unwrap()
                .clone()
        );
        assert_eq!(
            updated[root_ref].revision().dependencies[0].target,
            changed.selected
        );
    }
    assert_eq!(
        updated[&changed.selected].revision().parents,
        vec![base.clone()]
    );
    assert_eq!(assets[&base].payload().value()["name"], "Reusable pins");
    assert_eq!(
        std::fs::read(root.join(format!("pool/units/{unit}.json"))).unwrap(),
        draft_source(
            &first_draft,
            &assets[&base].provenance().sources[0]
                .retained_blob
                .clone()
                .unwrap()
        )
    );
    std::fs::remove_dir_all(root).unwrap();
}

fn draft_source(draft: &capture::Draft, digest: &AssetDigest) -> Vec<u8> {
    draft
        .content
        .iter()
        .find(|c| c.fingerprint().unwrap() == *digest)
        .unwrap()
        .bytes()
        .to_vec()
}

#[test]
fn stale_parent_and_native_recapture_refuse_instead_of_replacing_managed_history() {
    let (root, unit, _, _) = fixture();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    let draft = capture::capture_inventory(
        &context,
        &[],
        &BTreeMap::new(),
        &provenance(),
        request(CandidateChange::CaptureNative { asset_id: unit }),
        limits(),
    )
    .unwrap();
    let assets = verified(&draft);
    assert!(
        capture::capture_inventory(
            &context,
            &draft.roots,
            &assets,
            &provenance(),
            request(CandidateChange::CaptureNative { asset_id: unit }),
            limits()
        )
        .is_err()
    );
    let mut wrong = draft.selected.clone();
    wrong.revision_id = AssetDigest::blob(b"wrong parent");
    assert!(
        capture::capture_inventory(
            &context,
            &draft.roots,
            &assets,
            &provenance(),
            request(CandidateChange::Revise {
                base: wrong,
                payload: assets[&draft.selected].payload().value().clone()
            }),
            limits()
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn candidate_save_preserves_exact_lock_and_retains_a_complete_honest_inventory() {
    let (root, unit, _, _) = fixture();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    let draft = capture::capture_inventory(
        &context,
        &[],
        &BTreeMap::new(),
        &provenance(),
        request(CandidateChange::CaptureNative { asset_id: unit }),
        limits(),
    )
    .unwrap();
    let lock = crate::library_assets::lock::ProjectLibraryLock::empty().unwrap();
    let release = LibraryReleaseManifest {
        schema_version: 1,
        collection_id: model.project.project_id,
        release_id: Uuid::new_v4(),
        publisher: actor(&provenance()),
        profile: PROFILE.into(),
        roots: vec![],
        closure_digest: lock.closure_digest.clone(),
        objects: vec![],
        blobs: vec![],
        publication_record_refs: vec![],
        extensions: None,
    };
    let discovery = LibraryDiscovery {
        schema_version: 1,
        collection_id: release.collection_id,
        profile: PROFILE.into(),
        manifest_ref: release.fingerprint().unwrap(),
    };
    let previous = LibraryStateValues {
        lock: encode_metadata(&lock).unwrap(),
        catalog: encode_metadata(&discovery).unwrap(),
    };
    let (next, content) =
        catalog::candidate_state_from_manifest(&release, &previous, &provenance(), &draft).unwrap();
    assert_eq!(next.lock, previous.lock);
    assert_ne!(next.catalog, previous.catalog);
    let discovery: LibraryDiscovery =
        crate::library_assets::decode_metadata(&next.catalog, limits().metadata_bytes).unwrap();
    let raw = content
        .iter()
        .find(|c| {
            matches!(
                c,
                LibraryContent::Record {
                    record_type: LibraryRecordType::ReleaseManifest,
                    ..
                }
            )
        })
        .unwrap()
        .bytes();
    let manifest: LibraryReleaseManifest =
        crate::library_assets::decode_metadata(raw, limits().metadata_bytes).unwrap();
    discovery.verify_manifest(&manifest).unwrap();
    let objects = content
        .iter()
        .filter_map(|c| match c {
            LibraryContent::Record {
                record_type: LibraryRecordType::ReleaseManifest,
                ..
            } => None,
            LibraryContent::Record { .. } => Some((c.fingerprint().unwrap(), c.bytes().to_vec())),
            _ => None,
        })
        .collect();
    let blobs = content
        .iter()
        .filter_map(|c| {
            matches!(c, LibraryContent::Blob { .. })
                .then(|| (c.fingerprint().unwrap(), c.bytes().to_vec()))
        })
        .collect();
    manifest
        .verify_inventory(&objects, &blobs, limits().metadata_bytes)
        .unwrap();
    assert!(manifest.publisher.verification_evidence.is_empty());
    assert_eq!(manifest.roots, vec![draft.selected]);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_capture_refuses_two_sources_for_the_same_asset_identity() {
    let (root, unit, _, _) = fixture();
    let raw = std::fs::read(root.join(format!("pool/units/{unit}.json"))).unwrap();
    std::fs::write(
        root.join(format!("pool/units/{}.json", Uuid::new_v4())),
        &raw,
    )
    .unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    assert!(
        capture::capture_inventory(
            &context,
            &[],
            &BTreeMap::new(),
            &provenance(),
            request(CandidateChange::CaptureNative { asset_id: unit }),
            limits()
        )
        .is_err()
    );
    assert_eq!(
        std::fs::read(root.join(format!("pool/units/{unit}.json"))).unwrap(),
        raw
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_author_claim_applies_only_to_selected_asset_not_its_dependencies() {
    let (root, unit, symbol, _) = fixture();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    let initial = capture::capture_inventory(
        &context,
        &[],
        &BTreeMap::new(),
        &provenance(),
        request(CandidateChange::CaptureNative { asset_id: symbol }),
        limits(),
    )
    .unwrap();
    let original = verified(&initial);
    let mut metadata = original[&initial.selected].provenance().clone();
    metadata.authors = vec![Actor {
        recorded_id: Some("explicit-vendor-claim".into()),
        display_name: None,
        verification_evidence: vec![],
    }];
    let mut change = request(CandidateChange::CaptureNative { asset_id: symbol });
    change.provenance_override = Some(metadata);
    let draft = capture::capture_inventory(
        &context,
        &[],
        &BTreeMap::new(),
        &provenance(),
        change,
        limits(),
    )
    .unwrap();
    let assets = verified(&draft);
    assert_eq!(
        assets[&draft.selected].provenance().authors[0]
            .recorded_id
            .as_deref(),
        Some("explicit-vendor-claim")
    );
    assert!(
        assets[&draft.selected].provenance().authors[0]
            .verification_evidence
            .is_empty()
    );
    let unit = assets
        .values()
        .find(|a| a.reference().asset_id == unit)
        .unwrap();
    assert!(unit.provenance().authors.is_empty());
    assert_eq!(unit.provenance().origin_kind, OriginKind::Migration);
    std::fs::remove_dir_all(root).unwrap();
}

fn acquired(
    revision: &AssetRevision,
    provenance: &AssetProvenance,
    payload: &[u8],
) -> VerifiedAsset {
    VerifiedAsset::read(
        &revision.reference().unwrap(),
        &encode_metadata(revision).unwrap(),
        &encode_metadata(provenance).unwrap(),
        payload,
        limits().blob_bytes,
        counter_policy(revision.kind, revision.payload_schema_version).unwrap(),
    )
    .unwrap()
}

#[test]
fn acquired_precise_unknown_refuses_unrelated_value_edit_and_preserves_original_bytes() {
    let (root, unit, _, _) = fixture();
    let source = root.join(format!("pool/units/{unit}.json"));
    let original = std::fs::read(&source).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    let draft = capture::capture_inventory(
        &context,
        &[],
        &BTreeMap::new(),
        &provenance(),
        request(CandidateChange::CaptureNative { asset_id: unit }),
        limits(),
    )
    .unwrap();
    let native = verified(&draft);
    let raw = format!(
        "{{\"schema_version\":1,\"uuid\":\"{unit}\",\"name\":\"Acquired\",\"manufacturer\":\"\",\"pins\":{{}},\"tags\":[],\"vendor\":0.10000000000000001}}"
    );
    let mut revision = native[&draft.selected].revision().clone();
    revision.payload_ref = AssetDigest::blob(raw.as_bytes());
    let asset = acquired(
        &revision,
        native[&draft.selected].provenance(),
        raw.as_bytes(),
    );
    let base = asset.reference().clone();
    assert!(
        asset
            .payload()
            .verify_materialization(
                asset.payload().value(),
                counter_policy(AssetKind::Unit, 1).unwrap()
            )
            .is_err()
    );
    let mut payload = asset.payload().value().clone();
    payload["name"] = "Unrelated name edit".into();
    let assets = BTreeMap::from([(base.clone(), asset)]);
    let result = capture::capture_inventory(
        &context,
        &[base.clone()],
        &assets,
        &provenance(),
        request(CandidateChange::Revise {
            base: base.clone(),
            payload,
        }),
        limits(),
    );
    assert!(
        result
            .err()
            .unwrap()
            .to_string()
            .contains("lossless acquired base")
    );
    assert_eq!(
        assets[&base].revision().payload_ref,
        AssetDigest::blob(raw.as_bytes())
    );
    assert_eq!(std::fs::read(source).unwrap(), original);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn managed_revision_preserves_unedited_qualified_envelope_extensions() {
    let (root, unit, _, _) = fixture();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    let draft = capture::capture_inventory(
        &context,
        &[],
        &BTreeMap::new(),
        &provenance(),
        request(CandidateChange::CaptureNative { asset_id: unit }),
        limits(),
    )
    .unwrap();
    let native = verified(&draft);
    let mut revision = native[&draft.selected].revision().clone();
    let extensions = Some(BTreeMap::from([(
        "vendor.example".into(),
        json!({"unclassified":"retain"}),
    )]));
    revision.extensions = extensions.clone();
    let asset = acquired(
        &revision,
        native[&draft.selected].provenance(),
        &draft_source(&draft, &revision.payload_ref),
    );
    let base = asset.reference().clone();
    let mut payload = asset.payload().value().clone();
    payload["name"] = "Representable edit".into();
    let assets = BTreeMap::from([(base.clone(), asset)]);
    let changed = capture::capture_inventory(
        &context,
        &[base.clone()],
        &assets,
        &provenance(),
        request(CandidateChange::Revise {
            base: base.clone(),
            payload,
        }),
        limits(),
    )
    .unwrap();
    let updated = verified(&changed);
    assert_ne!(changed.selected, base);
    assert_eq!(
        updated[&changed.selected].revision().parents,
        vec![base.clone()]
    );
    assert_eq!(updated[&changed.selected].revision().extensions, extensions);
    assert_eq!(assets[&base].revision().extensions, extensions);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn candidate_release_does_not_inherit_prior_subject_publication_claims() {
    let lock = crate::library_assets::lock::ProjectLibraryLock::empty().unwrap();
    let marker = AssetDigest::blob(b"prior release publication evidence");
    let release = LibraryReleaseManifest {
        schema_version: 1,
        collection_id: Uuid::new_v4(),
        release_id: Uuid::new_v4(),
        publisher: actor(&provenance()),
        profile: PROFILE.into(),
        roots: vec![],
        closure_digest: lock.closure_digest.clone(),
        objects: vec![],
        blobs: vec![BlobEntry {
            digest: marker.clone(),
            byte_length: b"prior release publication evidence".len() as u64,
            media_type: "text/plain".into(),
        }],
        publication_record_refs: vec![marker.clone()],
        extensions: None,
    };
    let old_bytes = encode_metadata(&release).unwrap();
    let discovery = LibraryDiscovery {
        schema_version: 1,
        collection_id: release.collection_id,
        profile: PROFILE.into(),
        manifest_ref: release.fingerprint().unwrap(),
    };
    let previous = LibraryStateValues {
        lock: encode_metadata(&lock).unwrap(),
        catalog: encode_metadata(&discovery).unwrap(),
    };
    let draft = capture::Draft {
        selected: AssetRevisionRef {
            asset_id: Uuid::new_v4(),
            revision_id: AssetDigest::blob(b"unused"),
        },
        roots: vec![],
        references: vec![],
        content: vec![],
        changed: vec![],
    };
    let (state, content) =
        catalog::candidate_state_from_manifest(&release, &previous, &provenance(), &draft).unwrap();
    let raw = content.last().unwrap().bytes();
    let candidate: LibraryReleaseManifest =
        crate::library_assets::decode_metadata(raw, limits().metadata_bytes).unwrap();
    assert_ne!(candidate.release_id, release.release_id);
    assert!(candidate.publication_record_refs.is_empty());
    assert_eq!(candidate.blobs, release.blobs);
    assert_eq!(release.publication_record_refs, vec![marker]);
    assert_eq!(encode_metadata(&release).unwrap(), old_bytes);
    assert_eq!(state.lock, previous.lock);
}
