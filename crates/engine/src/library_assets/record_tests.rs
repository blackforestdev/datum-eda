use super::super::closure_tests::{asset, provenance};
use super::super::lock::{ComponentBinding, LockedAsset, ProjectLibraryLock};
use super::super::review::{AssetReview, ReviewResult};
use super::super::{AssetKind, ClosureLimits, LocalCounters, VerifiedAsset, inspect_closure};
use super::*;
use serde_json::json;

fn actor() -> Actor {
    Actor {
        recorded_id: Some("declared-test-author".into()),
        display_name: None,
        verification_evidence: vec![],
    }
}

fn unit() -> (AssetRevisionRef, VerifiedAsset) {
    asset(
        AssetKind::Unit,
        json!({"schema_version":1,"uuid":Uuid::from_u128(1),
        "name":"Unit","manufacturer":"","pins":{},"tags":[]}),
        vec![],
    )
}

fn release(reference: &AssetRevisionRef, unit: &VerifiedAsset) -> LibraryReleaseManifest {
    let mut objects = vec![
        ObjectEntry {
            digest: reference.revision_id.clone(),
            kind: ObjectKind::AssetRevision,
        },
        ObjectEntry {
            digest: unit.revision().provenance_ref.clone(),
            kind: ObjectKind::Provenance,
        },
    ];
    objects.sort_by(|a, b| a.digest.cmp(&b.digest));
    let payload = serde_json::to_vec(unit.payload().value()).unwrap();
    LibraryReleaseManifest {
        schema_version: 1,
        collection_id: Uuid::from_u128(10),
        release_id: Uuid::from_u128(11),
        publisher: actor(),
        profile: PROFILE.into(),
        roots: vec![reference.clone()],
        closure_digest: AssetDigest::record(
            RecordDomain::Closure,
            &encode_metadata(&vec![reference]).unwrap(),
        ),
        objects,
        blobs: vec![BlobEntry {
            digest: unit.revision().payload_ref.clone(),
            byte_length: payload.len().try_into().unwrap(),
            media_type: "application/json".into(),
        }],
        publication_record_refs: vec![],
        extensions: None,
    }
}

#[test]
fn review_currency_cannot_survive_changed_content_or_dependency_basis() {
    let (reference, _) = unit();
    let basis = AssetDigest::blob(b"test dependency basis");
    let review = AssetReview {
        schema_version: 1,
        review_id: Uuid::from_u128(20),
        subject: reference.clone(),
        closure_digest: basis.clone(),
        reviewer: actor(),
        evidence: vec![],
        rule_profile_refs: vec![],
        result: ReviewResult::Failed,
        recorded_at: None,
        supersedes: vec![],
        disposition_refs: vec![],
        extensions: None,
    };
    // A current FAILED review remains failed; currency grants no approval.
    assert!(review.verify_basis(&reference, &basis).is_ok());
    assert_eq!(review.result, ReviewResult::Failed);
    assert!(
        review
            .verify_basis(&reference, &AssetDigest::blob(b"changed basis"))
            .is_err()
    );
    let mut changed = reference;
    changed.revision_id = AssetDigest::blob(b"changed contents");
    assert!(review.verify_basis(&changed, &basis).is_err());
    assert!(review.fingerprint().is_ok());
}

#[test]
fn release_inventory_refuses_missing_changed_and_wrongly_typed_content() {
    let (reference, unit) = unit();
    let manifest = release(&reference, &unit);
    let mut objects = BTreeMap::from([
        (
            reference.revision_id.clone(),
            encode_metadata(unit.revision()).unwrap(),
        ),
        (
            unit.revision().provenance_ref.clone(),
            encode_metadata(unit.provenance()).unwrap(),
        ),
    ]);
    let mut blobs = BTreeMap::from([(
        unit.revision().payload_ref.clone(),
        serde_json::to_vec(unit.payload().value()).unwrap(),
    )]);
    assert!(
        manifest
            .verify_inventory(&objects, &blobs, usize::MAX)
            .is_ok()
    );
    let mut discovery = LibraryDiscovery {
        schema_version: 1,
        collection_id: manifest.collection_id,
        profile: PROFILE.into(),
        manifest_ref: manifest.fingerprint().unwrap(),
    };
    assert!(discovery.verify_manifest(&manifest).is_ok());
    discovery.collection_id = Uuid::from_u128(999);
    assert!(discovery.verify_manifest(&manifest).is_err());
    let removed = objects.remove(&reference.revision_id).unwrap();
    assert!(
        manifest
            .verify_inventory(&objects, &blobs, usize::MAX)
            .is_err()
    );
    objects.insert(reference.revision_id.clone(), removed);
    blobs
        .get_mut(&unit.revision().payload_ref)
        .unwrap()
        .push(b' ');
    assert!(
        manifest
            .verify_inventory(&objects, &blobs, usize::MAX)
            .is_err()
    );
    blobs.get_mut(&unit.revision().payload_ref).unwrap().pop();
    assert!(
        manifest
            .verify_inventory(&objects, &blobs, usize::MAX)
            .is_ok()
    );
    let mut wrong_type = manifest.clone();
    wrong_type
        .objects
        .iter_mut()
        .find(|o| o.digest == reference.revision_id)
        .unwrap()
        .kind = ObjectKind::Review;
    assert!(
        wrong_type
            .verify_inventory(&objects, &blobs, usize::MAX)
            .is_err()
    );
    let mut duplicate = manifest.clone();
    duplicate.objects.push(duplicate.objects[0].clone());
    assert!(duplicate.validate().is_err());
}

#[test]
fn empty_lock_is_only_empty_managed_coverage_and_binding_requires_a_part() {
    let mut empty = ProjectLibraryLock::empty().unwrap();
    assert!(empty.validate().is_ok());
    assert!(empty.assets.is_empty() && empty.bindings.is_empty());
    let (reference, unit) = unit();
    let payload_ref = unit.revision().payload_ref.clone();
    let available = BTreeMap::from([(reference.clone(), unit)]);
    let closure = inspect_closure(
        &[reference.clone()],
        &available,
        ClosureLimits {
            max_nodes: 8,
            max_depth: 8,
        },
    )
    .unwrap();
    let manifest = release(&reference, available.get(&reference).unwrap());
    assert!(manifest.verify_closure(&closure).is_ok());
    empty.assets.push(LockedAsset {
        asset: reference.clone(),
        local_object_id: reference.asset_id,
        source_path: "pool/units/unit.json".into(),
        payload_ref,
    });
    empty.closure_digest = closure.digest().clone();
    assert!(empty.verify_closure(&closure).is_ok());
    empty.bindings.push(ComponentBinding {
        component_instance_id: Uuid::from_u128(30),
        source_path: "schematic/sheets/root.json".into(),
        instance_path: vec![],
        component: reference,
    });
    assert!(empty.verify_closure(&closure).is_err());
    empty.bindings.clear();
    empty.assets[0].source_path = "../outside.json".into();
    assert!(empty.validate().is_err());
}

#[test]
fn provenance_claiming_retained_evidence_must_inventory_the_actual_blob() {
    let mut record = provenance();
    record.evidence.push(Evidence {
        title: "Source".into(),
        issuer: None,
        document_revision: None,
        locator: None,
        retained_blob: Some(AssetDigest::blob(b"unavailable source")),
        access_note: None,
    });
    let (reference, unit) = unit();
    let payload = serde_json::to_vec(unit.payload().value()).unwrap();
    let mut revision = unit.revision().clone();
    revision.provenance_ref = record.fingerprint().unwrap();
    let revised_ref = revision.reference().unwrap();
    let verified = VerifiedAsset::read(
        &revised_ref,
        &encode_metadata(&revision).unwrap(),
        &encode_metadata(&record).unwrap(),
        &payload,
        usize::MAX,
        LocalCounters::default(),
    )
    .unwrap();
    let manifest = release(&revised_ref, &verified);
    let objects = BTreeMap::from([
        (revised_ref.revision_id, encode_metadata(&revision).unwrap()),
        (
            record.fingerprint().unwrap(),
            encode_metadata(&record).unwrap(),
        ),
    ]);
    let blobs = BTreeMap::from([(revision.payload_ref, payload)]);
    let error = manifest
        .verify_inventory(&objects, &blobs, usize::MAX)
        .unwrap_err();
    assert!(error.to_string().contains("retained evidence blob"));
    // Source identity actually changed; the old revision does not cover this claim.
    assert_ne!(reference.revision_id, manifest.roots[0].revision_id);
}

#[test]
fn locked_assets_need_distinct_materialization_paths() {
    let (first_ref, first) = unit();
    let (second_ref, second) = asset(
        AssetKind::Unit,
        json!({"schema_version":1,
        "uuid":Uuid::from_u128(2),"name":"Other","manufacturer":"","pins":{},"tags":[]}),
        vec![],
    );
    let first_payload = first.revision().payload_ref.clone();
    let second_payload = second.revision().payload_ref.clone();
    let available = BTreeMap::from([(first_ref.clone(), first), (second_ref.clone(), second)]);
    let closure = inspect_closure(
        &[first_ref.clone(), second_ref.clone()],
        &available,
        ClosureLimits {
            max_nodes: 8,
            max_depth: 8,
        },
    )
    .unwrap();
    let mut lock = ProjectLibraryLock::empty().unwrap();
    lock.assets = vec![
        LockedAsset {
            local_object_id: first_ref.asset_id,
            asset: first_ref,
            source_path: "pool/units/00000000-0000-0000-0000-000000000001.json".into(),
            payload_ref: first_payload,
        },
        LockedAsset {
            local_object_id: second_ref.asset_id,
            asset: second_ref,
            source_path: "pool/units/00000000-0000-0000-0000-000000000002.json".into(),
            payload_ref: second_payload,
        },
    ];
    lock.closure_digest = closure.digest().clone();
    assert!(lock.verify_closure(&closure).is_ok());
    lock.assets[1].source_path = lock.assets[0].source_path.clone();
    let error = lock.verify_closure(&closure).unwrap_err();
    assert!(error.to_string().contains("materialization source path"));
}
