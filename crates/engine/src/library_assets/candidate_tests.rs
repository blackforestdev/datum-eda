//! Common-profile collection controls; not CAD or publication qualification.
use super::*;
use crate::{
    api::Engine,
    library_assets::{
        Actor, Dependency,
        closure_tests::asset,
        encode_metadata,
        manifest::{BlobEntry, LibraryDiscovery, LibraryReleaseManifest, ObjectEntry, ObjectKind},
        records::PROFILE,
        review::ReviewResult,
    },
    substrate::library_asset_resolver::{LibraryAssetLimits, configure_server_policy},
};
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn isolated(name: &str, mode: &str) -> bool {
    const KEY: &str = "DATUM_LIBRARY_CANDIDATE_TEST_CHILD";
    if let Ok(actual) = std::env::var(KEY) {
        assert_eq!(actual, mode);
        return false;
    }
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            name.strip_prefix(concat!(env!("CARGO_CRATE_NAME"), "::"))
                .unwrap(),
            "--nocapture",
        ])
        .env(KEY, mode)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("1 passed"));
    true
}

fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut result = BTreeMap::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            result.extend(files(&path));
        } else {
            result.insert(path.clone(), std::fs::read(path).unwrap());
        }
    }
    result
}

struct Fixture {
    root: PathBuf,
    symbol: AssetRevisionRef,
    unit: AssetRevisionRef,
    manifest: LibraryReleaseManifest,
    policy: LibraryAssetLimits,
}

fn fixture(unsupported: bool) -> Fixture {
    let mut unit_payload = json!({"schema_version":1,"uuid":Uuid::from_u128(1),
        "name":"Native Unit","manufacturer":"","pins":{},"tags":[]});
    if unsupported {
        unit_payload["models_3d"] = json!(["unsupported owned model"]);
    }
    let (unit, u) = asset(AssetKind::Unit, unit_payload, vec![]);
    let (symbol, s) = asset(
        AssetKind::Symbol,
        json!({"schema_version":1,"uuid":Uuid::from_u128(2),"name":"Native Symbol","unit":unit.asset_id}),
        vec![Dependency {
            role: "unit".into(),
            payload_path: vec!["unit".into()],
            target: unit.clone(),
        }],
    );
    let assets = BTreeMap::from([(unit.clone(), u), (symbol.clone(), s)]);
    let bounds = ClosureLimits {
        max_nodes: assets.len(),
        max_depth: assets.len(),
    };
    let mut objects = BTreeMap::new();
    let mut blobs = BTreeMap::new();
    for (reference, asset) in &assets {
        objects.insert(
            reference.revision_id.clone(),
            (
                ObjectKind::AssetRevision,
                encode_metadata(asset.revision()).unwrap(),
            ),
        );
        objects.insert(
            asset.revision().provenance_ref.clone(),
            (
                ObjectKind::Provenance,
                encode_metadata(asset.provenance()).unwrap(),
            ),
        );
        blobs.insert(
            asset.revision().payload_ref.clone(),
            asset.payload().raw_bytes().to_vec(),
        );
    }
    let reviewer = Actor {
        recorded_id: Some("declared reviewer".into()),
        display_name: None,
        verification_evidence: vec![],
    };
    for (result, digest) in [
        (
            ReviewResult::Failed,
            inspect_closure(std::slice::from_ref(&unit), &assets, bounds)
                .unwrap()
                .digest()
                .clone(),
        ),
        (
            ReviewResult::Passed,
            AssetDigest::blob(b"earlier dependency basis"),
        ),
    ] {
        let review = AssetReview {
            schema_version: 1,
            review_id: Uuid::new_v4(),
            subject: unit.clone(),
            closure_digest: digest,
            reviewer: reviewer.clone(),
            evidence: vec![],
            rule_profile_refs: vec![],
            result,
            recorded_at: None,
            supersedes: vec![],
            disposition_refs: vec![],
            extensions: None,
        };
        objects.insert(
            review.fingerprint().unwrap(),
            (ObjectKind::Review, encode_metadata(&review).unwrap()),
        );
    }
    let manifest = LibraryReleaseManifest {
        schema_version: 1,
        collection_id: Uuid::new_v4(),
        release_id: Uuid::new_v4(),
        publisher: reviewer,
        profile: PROFILE.into(),
        roots: vec![symbol.clone()],
        closure_digest: inspect_closure(std::slice::from_ref(&symbol), &assets, bounds)
            .unwrap()
            .digest()
            .clone(),
        objects: objects
            .iter()
            .map(|(digest, (kind, _))| ObjectEntry {
                digest: digest.clone(),
                kind: *kind,
            })
            .collect(),
        blobs: blobs
            .iter()
            .map(|(digest, bytes)| BlobEntry {
                digest: digest.clone(),
                byte_length: bytes.len() as u64,
                media_type: "application/json".into(),
            })
            .collect(),
        publication_record_refs: vec![],
        extensions: None,
    };
    let root = std::env::temp_dir().join(format!("datum_candidate_{}", Uuid::new_v4()));
    std::fs::create_dir_all(root.join("objects/sha256")).unwrap();
    std::fs::create_dir_all(root.join("blobs/sha256")).unwrap();
    for (digest, (_, bytes)) in objects {
        std::fs::write(
            root.join(format!("objects/sha256/{}.json", digest.hex())),
            bytes,
        )
        .unwrap();
    }
    for (digest, bytes) in blobs {
        std::fs::write(root.join(format!("blobs/sha256/{}", digest.hex())), bytes).unwrap();
    }
    let discovery = LibraryDiscovery {
        schema_version: 1,
        collection_id: manifest.collection_id,
        profile: PROFILE.into(),
        manifest_ref: manifest.fingerprint().unwrap(),
    };
    std::fs::write(
        root.join(format!(
            "objects/sha256/{}.json",
            discovery.manifest_ref.hex()
        )),
        encode_metadata(&manifest).unwrap(),
    )
    .unwrap();
    std::fs::write(
        root.join("library.json"),
        encode_metadata(&discovery).unwrap(),
    )
    .unwrap();
    let inventory = files(&root);
    let bytes = inventory.values().map(Vec::len).sum::<usize>();
    // Actual fixture inventory bounds cover both inventory and repeated verified
    // envelope reads, without configuring defaults or resetting shared policy.
    let policy = LibraryAssetLimits {
        metadata_bytes: bytes,
        blob_bytes: bytes,
        files: inventory.len(),
        depth: inventory
            .keys()
            .map(|p| p.strip_prefix(&root).unwrap().components().count())
            .max()
            .unwrap()
            + 1,
        total_bytes: bytes * 4,
    };
    Fixture {
        root,
        symbol,
        unit,
        manifest,
        policy,
    }
}

#[test]
fn offline_candidate_reports_exact_closure_reviews_unknown_rights_without_project_or_writes() {
    const NAME: &str = concat!(
        module_path!(),
        "::offline_candidate_reports_exact_closure_reviews_unknown_rights_without_project_or_writes"
    );
    if isolated(NAME, "valid") {
        return;
    }
    let f = fixture(false);
    assert!(
        Engine::inspect_library_candidate(&f.root).is_err(),
        "unconfigured host policy must refuse"
    );
    configure_server_policy(f.policy).unwrap();
    let before = files(&f.root);
    let report = Engine::inspect_library_candidate(&f.root).unwrap();
    assert!(!f.root.join("project.json").exists());
    assert!(!f.root.join("lock.json").exists());
    assert_eq!(
        report["discovery"]["manifest_ref"],
        json!(f.manifest.fingerprint().unwrap())
    );
    assert_eq!(report["release_manifest"]["roots"], json!([f.symbol]));
    assert_eq!(
        report["closure"]["digest"],
        json!(f.manifest.closure_digest)
    );
    let members = report["closure"]["assets"].as_array().unwrap();
    assert_eq!(members.len(), 2);
    let unit = members
        .iter()
        .find(|m| m["revision"] == json!(f.unit))
        .unwrap();
    assert!(unit["provenance"]["rights"]["license_expression"].is_null());
    assert!(
        unit["reviews"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["basis_status"] == "current" && r["record"]["result"] == "failed")
    );
    assert!(
        unit["reviews"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["basis_status"] == "stale" && r["record"]["result"] == "passed")
    );
    assert!(members.iter().any(|m| m["review_presence"] == "missing"));
    for (key, value) in [
        ("redistribution_authority", "not_verified"),
        ("engineering_approval", "not_evaluated"),
        ("promotion_authority", "unavailable"),
        ("publication_policy", "unavailable"),
    ] {
        assert_eq!(report[key], value);
    }
    assert_eq!(files(&f.root), before);
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn offline_candidate_refuses_tampered_missing_unsafe_and_unsupported_sources() {
    const NAME: &str = concat!(
        module_path!(),
        "::offline_candidate_refuses_tampered_missing_unsafe_and_unsupported_sources"
    );
    if std::env::var_os("DATUM_LIBRARY_CANDIDATE_TEST_CHILD").is_none() {
        for mode in ["tamper", "missing", "link", "unsupported"] {
            assert!(isolated(NAME, mode));
        }
        return;
    }
    let mode = std::env::var("DATUM_LIBRARY_CANDIDATE_TEST_CHILD").unwrap();
    let f = fixture(mode == "unsupported");
    configure_server_policy(f.policy).unwrap();
    let path = f.root.join(f.manifest.blobs[0].relative_path());
    let external = f.root.with_extension("outside");
    match mode.as_str() {
        "tamper" => std::fs::write(&path, b"tampered").unwrap(),
        "missing" => std::fs::remove_file(&path).unwrap(),
        "link" => {
            std::fs::write(&external, std::fs::read(&path).unwrap()).unwrap();
            std::fs::remove_file(&path).unwrap();
            std::os::unix::fs::symlink(&external, &path).unwrap();
        }
        "unsupported" => {}
        _ => panic!("unexpected source mode"),
    }
    let before = files(&f.root);
    let error = Engine::inspect_library_candidate(&f.root)
        .unwrap_err()
        .to_string();
    if mode == "unsupported" {
        assert!(
            error.contains("unsupported owned model dependencies"),
            "{error}"
        );
    }
    if mode == "link" {
        assert!(error.contains("link or unsupported file type"), "{error}");
    }
    assert_eq!(files(&f.root), before);
    std::fs::remove_dir_all(f.root).unwrap();
    if external.exists() {
        std::fs::remove_file(external).unwrap();
    }
}

#[test]
fn offline_candidate_obeys_explicit_host_metadata_blob_inventory_depth_and_aggregate_limits() {
    const NAME: &str = concat!(
        module_path!(),
        "::offline_candidate_obeys_explicit_host_metadata_blob_inventory_depth_and_aggregate_limits"
    );
    if std::env::var_os("DATUM_LIBRARY_CANDIDATE_TEST_CHILD").is_none() {
        for mode in ["metadata", "blob", "files", "depth", "total"] {
            assert!(isolated(NAME, mode));
        }
        return;
    }
    let mode = std::env::var("DATUM_LIBRARY_CANDIDATE_TEST_CHILD").unwrap();
    let f = fixture(false);
    let mut limits = f.policy;
    match mode.as_str() {
        "metadata" => limits.metadata_bytes = 1,
        "blob" => limits.blob_bytes = 1,
        "files" => limits.files = 1,
        "depth" => limits.depth = 1,
        "total" => limits.total_bytes = files(&f.root).values().map(Vec::len).sum(),
        _ => panic!("unexpected limit mode"),
    }
    configure_server_policy(limits).unwrap();
    let before = files(&f.root);
    assert!(
        Engine::inspect_library_candidate(&f.root).is_err(),
        "{mode}"
    );
    assert_eq!(files(&f.root), before);
    std::fs::remove_dir_all(f.root).unwrap();
}
