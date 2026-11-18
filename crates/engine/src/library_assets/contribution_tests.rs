//! Structural evidence controls; generated geometry is not manufacturing proof.
use super::*;
use crate::library_assets::{
    Actor, AssetProvenance, AssetRevision, Dependency, LocalCounters,
    closure_tests::{asset, provenance},
    encode_metadata,
    review::ReviewResult,
};
use uuid::Uuid;

fn limits() -> ClosureLimits {
    ClosureLimits {
        max_nodes: 32,
        max_depth: 16,
    }
}

fn dependency(path: &[&str], target: &AssetRevisionRef) -> Dependency {
    Dependency {
        role: path.last().unwrap().to_string(),
        payload_path: path.iter().map(|part| part.to_string()).collect(),
        target: target.clone(),
    }
}

fn graph(
    owner: Uuid,
    license: Option<&str>,
) -> (AssetRevisionRef, BTreeMap<AssetRevisionRef, VerifiedAsset>) {
    let (unit, u) = asset(
        AssetKind::Unit,
        json!({"schema_version":1,"uuid":Uuid::from_u128(1),
        "name":"Unit","manufacturer":"","pins":{},"tags":[]}),
        vec![],
    );
    let (symbol, s) = asset(
        AssetKind::Symbol,
        json!({"schema_version":1,"uuid":Uuid::from_u128(2),
        "name":"Symbol","unit":unit.asset_id}),
        vec![dependency(&["unit"], &unit)],
    );
    let gate = Uuid::from_u128(31).to_string();
    let (entity, e) = asset(
        AssetKind::Entity,
        json!({"schema_version":1,"uuid":Uuid::from_u128(3),
        "name":"Entity","prefix":"U","manufacturer":"","tags":[],"gates":{
            gate.clone():{"uuid":gate,"name":"G","unit":unit.asset_id,"symbol":symbol.asset_id}}}),
        vec![
            dependency(&["gates", &gate, "symbol"], &symbol),
            dependency(&["gates", &gate, "unit"], &unit),
        ],
    );
    let (package, p) = asset(
        AssetKind::Package,
        json!({"schema_version":1,"uuid":Uuid::from_u128(4),"name":"Package"}),
        vec![],
    );
    let (footprint, f) = asset(
        AssetKind::Footprint,
        json!({"schema_version":1,"uuid":Uuid::from_u128(5),
        "name":"Footprint","package":package.asset_id,"courtyard":{"vertices":[],"closed":true}}),
        vec![dependency(&["package"], &package)],
    );
    let (map, m) = asset(
        AssetKind::PinPadMap,
        json!({"schema_version":1,"uuid":Uuid::from_u128(6),
        "part":owner,"footprint":footprint.asset_id,"mappings":{}}),
        vec![dependency(&["footprint"], &footprint)],
    );
    let mut part = crate::api::native_write::library::pool_part_payload(
        Uuid::from_u128(7),
        entity.asset_id,
        package.asset_id,
        "",
        "",
        "100R",
        "Contribution structural control",
        "",
        "Unknown",
    );
    part["default_footprint"] = json!(footprint.asset_id);
    part["default_pin_pad_map"] = json!(map.asset_id);
    let (mut root, mut r) = asset(
        AssetKind::Part,
        part,
        vec![
            dependency(&["default_footprint"], &footprint),
            dependency(&["default_pin_pad_map"], &map),
            dependency(&["entity"], &entity),
            dependency(&["package"], &package),
        ],
    );
    if let Some(license) = license {
        let mut claims: AssetProvenance = provenance();
        claims.rights.license_expression = Some(license.into());
        claims.rights.attribution = vec!["Declared author, unverified".into()];
        let mut record: AssetRevision = r.revision().clone();
        record.provenance_ref = claims.fingerprint().unwrap();
        root = record.reference().unwrap();
        r = VerifiedAsset::read(
            &root,
            &encode_metadata(&record).unwrap(),
            &encode_metadata(&claims).unwrap(),
            &serde_json::to_vec(r.payload().value()).unwrap(),
            usize::MAX,
            LocalCounters::default(),
        )
        .unwrap();
    }
    (
        root.clone(),
        BTreeMap::from([
            (unit, u),
            (symbol, s),
            (entity, e),
            (package, p),
            (footprint, f),
            (map, m),
            (root, r),
        ]),
    )
}

fn review(
    subject: &AssetRevisionRef,
    basis: AssetDigest,
    result: ReviewResult,
    id: u128,
) -> (AssetDigest, AssetReview) {
    let record = AssetReview {
        schema_version: 1,
        review_id: Uuid::from_u128(id),
        subject: subject.clone(),
        closure_digest: basis,
        reviewer: Actor {
            recorded_id: Some("declared-reviewer".into()),
            display_name: None,
            verification_evidence: vec![],
        },
        evidence: vec![],
        rule_profile_refs: vec![],
        result,
        recorded_at: None,
        supersedes: vec![],
        disposition_refs: vec![],
        extensions: None,
    };
    (record.fingerprint().unwrap(), record)
}

fn member<'a>(report: &'a Value, id: Uuid) -> &'a Value {
    report["closure"]["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["revision"]["asset_id"] == json!(id))
        .unwrap()
}

#[test]
fn contribution_reports_complete_part_and_missing_unknown_claims() {
    let (subject, available) = graph(Uuid::from_u128(7), None);
    let report = inspect_part(&subject, &available, &BTreeMap::new(), limits()).unwrap();
    assert_eq!(report["subject"], json!(subject));
    assert_eq!(report["closure"]["graph_valid"], true);
    assert_eq!(report["closure"]["assets"].as_array().unwrap().len(), 7);
    assert_eq!(
        report["closure"]["digest"],
        json!(
            inspect_closure(&[subject.clone()], &available, limits())
                .unwrap()
                .digest()
        )
    );
    for entry in report["closure"]["assets"].as_array().unwrap() {
        assert_eq!(entry["review_presence"], "missing");
        assert_eq!(entry["reviews"], json!([]));
        assert_eq!(
            entry["provenance"]["rights"]["license_expression"],
            Value::Null
        );
        assert_eq!(entry["provenance"]["created_at"], Value::Null);
        let reference: AssetRevisionRef =
            serde_json::from_value(entry["revision"].clone()).unwrap();
        assert_eq!(
            entry["provenance_ref"],
            json!(available[&reference].revision().provenance_ref)
        );
        assert_eq!(
            entry["dependencies"],
            json!(available[&reference].revision().dependencies)
        );
    }
    let map = available
        .keys()
        .find(|r| r.asset_id == Uuid::from_u128(6))
        .unwrap();
    // Contextual map ownership is absent in its isolated forward graph.
    assert!(
        !inspect_closure(&[map.clone()], &available, limits())
            .unwrap()
            .graph_report()
            .valid
    );
    let basis = inspect_closure(&[map.clone()], &available, limits())
        .unwrap()
        .digest()
        .clone();
    let evidence = BTreeMap::from([review(map, basis, ReviewResult::Failed, 91)]);
    let report = inspect_part(&subject, &available, &evidence, limits()).unwrap();
    let rows = &member(&report, map.asset_id)["reviews"];
    assert_eq!(rows[0]["basis_status"], "current");
    assert_eq!(rows[0]["record"]["result"], "failed");
    assert_eq!(report["closure"]["graph_valid"], true);
}

#[test]
fn contribution_separates_review_currency_result_and_declared_rights_from_authority() {
    let (subject, available) = graph(Uuid::from_u128(7), Some("CC0-1.0"));
    let basis = inspect_closure(&[subject.clone()], &available, limits())
        .unwrap()
        .digest()
        .clone();
    let mut reviews = BTreeMap::new();
    for (id, result) in [
        (11, ReviewResult::Passed),
        (12, ReviewResult::Failed),
        (13, ReviewResult::NeedsReview),
    ] {
        reviews.extend([review(&subject, basis.clone(), result, id)]);
    }
    reviews.extend([review(
        &subject,
        AssetDigest::blob(b"different dependency basis"),
        ReviewResult::Passed,
        14,
    )]);
    let mut old = subject.clone();
    old.revision_id = AssetDigest::blob(b"prior exact subject");
    reviews.extend([review(&old, basis, ReviewResult::Passed, 15)]);
    let report = inspect_part(&subject, &available, &reviews, limits()).unwrap();
    let root = member(&report, subject.asset_id);
    assert_eq!(root["review_presence"], "recorded");
    assert_eq!(
        root["provenance"]["rights"]["license_expression"],
        "CC0-1.0"
    );
    for (digest, record) in &reviews {
        let row = root["reviews"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["record_ref"] == json!(digest))
            .unwrap();
        assert_eq!(row["record"], json!(record));
        assert_eq!(
            row["basis_status"],
            if record.review_id.as_u128() <= 13 {
                "current"
            } else {
                "stale"
            }
        );
    }
    assert_eq!(report["redistribution_authority"], "not_verified");
    assert_eq!(report["engineering_approval"], "not_evaluated");
    assert_eq!(report["publication_policy"], "unavailable");
}

#[test]
fn contribution_retains_graph_findings_and_refuses_incomplete_or_forged_evidence() {
    let (subject, mut available) = graph(Uuid::from_u128(99), None);
    let report = inspect_part(&subject, &available, &BTreeMap::new(), limits()).unwrap();
    assert_eq!(report["closure"]["graph_valid"], false);
    assert!(
        !report["closure"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let (digest, mut record) = review(
        &subject,
        AssetDigest::blob(b"basis"),
        ReviewResult::Passed,
        20,
    );
    record.result = ReviewResult::Failed;
    assert!(
        inspect_part(
            &subject,
            &available,
            &BTreeMap::from([(digest, record)]),
            limits()
        )
        .is_err()
    );
    let unit = available
        .keys()
        .find(|r| r.asset_id == Uuid::from_u128(1))
        .unwrap()
        .clone();
    assert!(inspect_part(&unit, &available, &BTreeMap::new(), limits()).is_err());
    assert!(
        inspect_part(
            &subject,
            &available,
            &BTreeMap::new(),
            ClosureLimits {
                max_nodes: 6,
                max_depth: 16
            }
        )
        .is_err()
    );
    available.retain(|reference, _| reference.asset_id != Uuid::from_u128(4));
    assert!(inspect_part(&subject, &available, &BTreeMap::new(), limits()).is_err());
}

#[test]
fn contribution_unavailable_historical_slots_do_not_hide_valid_current_inventory() {
    use crate::library_assets::{
        PROFILE,
        manifest::{BlobEntry, LibraryReleaseManifest, ObjectEntry, ObjectKind},
    };

    let (original, mut available) = graph(Uuid::from_u128(7), None);
    let current = available.remove(&original).unwrap();
    let mut payload = current.payload().value().clone();
    payload["description"] = json!("Retained historical envelope with omitted dependency slots");
    // Every actual payload target exists. The older envelope's DECLARED list
    // is empty, which inventory validation permits; only closure checks detect
    // that those declarations do not cover the payload's schema-derived slots.
    let (historical, old) = asset(AssetKind::Part, payload, vec![]);
    let mut revision = current.revision().clone();
    revision.parents = vec![historical.clone()];
    let subject = revision.reference().unwrap();
    let current = VerifiedAsset::read(
        &subject,
        &encode_metadata(&revision).unwrap(),
        &encode_metadata(current.provenance()).unwrap(),
        current.payload().raw_bytes(),
        usize::MAX,
        LocalCounters::default(),
    )
    .unwrap();
    available.insert(subject.clone(), current);
    available.insert(historical.clone(), old);
    let closure = inspect_closure(std::slice::from_ref(&subject), &available, limits()).unwrap();
    assert!(closure.graph_report().valid);
    assert_eq!(closure.assets().len(), 7);
    assert_eq!(available.len(), 8);

    let mut objects = BTreeMap::new();
    let mut kinds = BTreeMap::new();
    let mut blobs = BTreeMap::new();
    for (reference, asset) in &available {
        objects.insert(
            reference.revision_id.clone(),
            encode_metadata(asset.revision()).unwrap(),
        );
        kinds.insert(reference.revision_id.clone(), ObjectKind::AssetRevision);
        objects.insert(
            asset.revision().provenance_ref.clone(),
            encode_metadata(asset.provenance()).unwrap(),
        );
        kinds.insert(
            asset.revision().provenance_ref.clone(),
            ObjectKind::Provenance,
        );
        blobs.insert(
            asset.revision().payload_ref.clone(),
            asset.payload().raw_bytes().to_vec(),
        );
    }
    let manifest = LibraryReleaseManifest {
        schema_version: 1,
        collection_id: Uuid::from_u128(50),
        release_id: Uuid::from_u128(51),
        publisher: Actor {
            recorded_id: None,
            display_name: None,
            verification_evidence: vec![],
        },
        profile: PROFILE.into(),
        roots: vec![subject.clone()],
        closure_digest: closure.digest().clone(),
        objects: kinds
            .into_iter()
            .map(|(digest, kind)| ObjectEntry { digest, kind })
            .collect(),
        blobs: blobs
            .iter()
            .map(|(digest, bytes)| BlobEntry {
                digest: digest.clone(),
                byte_length: bytes.len().try_into().unwrap(),
                media_type: "application/json".into(),
            })
            .collect(),
        publication_record_refs: vec![],
        extensions: None,
    };
    // Same existing admission invariants: complete declared inventory, exact
    // supported head closure, and empty active closure do not inspect nonroots.
    manifest
        .verify_inventory(&objects, &blobs, usize::MAX)
        .unwrap();
    manifest.verify_closure(&closure).unwrap();
    let active = inspect_closure(&[], &available, limits()).unwrap();
    crate::library_assets::lock::ProjectLibraryLock::empty()
        .unwrap()
        .verify_closure(&active)
        .unwrap();
    let error = inspect_part(&historical, &available, &BTreeMap::new(), limits()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("declared dependencies do not cover")
    );
    let reports: BTreeMap<_, _> = [subject.clone(), historical.clone()]
        .into_iter()
        .map(|reference| {
            let report = inspect_part_report(&reference, &available, &BTreeMap::new(), limits());
            (reference, report)
        })
        .collect();
    let current = &reports[&subject];
    assert_eq!(current["subject"], json!(subject));
    assert_eq!(current["inspection_status"], "available");
    assert_eq!(current["closure"]["graph_valid"], true);
    let historical_report = &reports[&historical];
    assert_eq!(historical_report["subject"], json!(historical));
    assert_eq!(historical_report["inspection_status"], "unavailable");
    assert_eq!(historical_report["reason"], error.to_string());
    assert!(historical_report.get("closure").is_none());
    assert_eq!(
        historical_report["redistribution_authority"],
        "not_verified"
    );
    assert_eq!(historical_report["engineering_approval"], "not_evaluated");
    assert_eq!(historical_report["publication_policy"], "unavailable");
    // Reporting unavailability cannot change upstream inventory integrity.
    let key = blobs.keys().next().unwrap().clone();
    blobs.get_mut(&key).unwrap().push(b' ');
    assert!(
        manifest
            .verify_inventory(&objects, &blobs, usize::MAX)
            .is_err()
    );
}
