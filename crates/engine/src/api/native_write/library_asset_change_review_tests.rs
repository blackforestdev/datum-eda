//! Prepared, unsaved contribution evidence over a real canonical project.
use super::*;
use crate::{
    api::native_write::{self as native, WriteProvenance},
    library_assets::{
        Actor, AssetDigest, encode_metadata,
        manifest::{LibraryDiscovery, ObjectEntry, ObjectKind},
        review::{AssetReview, ReviewResult},
    },
    substrate::{
        CommitSource, ComponentInstanceAuthority, DesignModel,
        library_asset_operations::{LibraryContent, LibraryRecordType, LibraryStateValues},
        library_asset_resolver::{configure_server_policy, read_state},
    },
};
use std::path::{Path, PathBuf};

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "candidate-preview-test",
        CommitSource::Test,
        "Review unsaved candidate",
    )
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

fn prepare(
    model: &DesignModel,
    root: &Path,
    request: Value,
) -> super::super::PreparedLibraryChange {
    super::super::prepare_library_change(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        request,
    )
    .unwrap()
}

fn snapshot(root: &Path, limits: LibraryAssetLimits) -> LibraryAssetSnapshot {
    resolve_state(
        root,
        &read_state(root, limits).unwrap(),
        &BTreeMap::new(),
        limits,
    )
    .unwrap()
}

fn review(
    subject: &AssetRevisionRef,
    snapshot: &LibraryAssetSnapshot,
    result: ReviewResult,
    limits: LibraryAssetLimits,
) -> AssetReview {
    AssetReview {
        schema_version: 1,
        review_id: Uuid::new_v4(),
        subject: subject.clone(),
        closure_digest: inspect_closure(
            std::slice::from_ref(subject),
            snapshot.assets(),
            ClosureLimits {
                max_nodes: limits.files,
                max_depth: limits.depth,
            },
        )
        .unwrap()
        .digest()
        .clone(),
        reviewer: Actor {
            recorded_id: Some("recorded-reviewer".into()),
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
    }
}

fn retain_reviews(
    model: &mut DesignModel,
    root: &Path,
    records: &[AssetReview],
    limits: LibraryAssetLimits,
) {
    let snapshot = snapshot(root, limits);
    let mut manifest = snapshot.manifest().clone();
    manifest.release_id = Uuid::new_v4();
    let mut content = Vec::new();
    for record in records {
        let digest = record.fingerprint().unwrap();
        manifest.objects.push(ObjectEntry {
            digest: digest.clone(),
            kind: ObjectKind::Review,
        });
        content.push(LibraryContent::Record {
            record_type: LibraryRecordType::Review,
            fingerprint: digest.as_str().into(),
            bytes: encode_metadata(record).unwrap(),
        });
    }
    manifest.objects.sort_by(|a, b| a.digest.cmp(&b.digest));
    let digest = manifest.fingerprint().unwrap();
    content.push(LibraryContent::Record {
        record_type: LibraryRecordType::ReleaseManifest,
        fingerprint: digest.as_str().into(),
        bytes: encode_metadata(&manifest).unwrap(),
    });
    let discovery = LibraryDiscovery {
        schema_version: 1,
        collection_id: manifest.collection_id,
        profile: manifest.profile.clone(),
        manifest_ref: digest,
    };
    let state = LibraryStateValues {
        lock: encode_metadata(snapshot.lock()).unwrap(),
        catalog: encode_metadata(&discovery).unwrap(),
    };
    let prepared = super::super::prepare_library_state(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        state,
        content,
        limits,
    )
    .unwrap();
    native::commit_prepared(model, root, prepared).unwrap();
}

fn row(summary: &Value, id: Uuid) -> &Value {
    summary["asset_changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["asset_id"] == json!(id))
        .unwrap()
}

fn member(report: &Value, id: Uuid) -> &Value {
    report["closure"]["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["revision"]["asset_id"] == json!(id))
        .unwrap()
}

#[test]
fn unsaved_part_preview_uses_pending_closure_reviews_and_preserves_project() {
    const CHILD: &str = "DATUM_CANDIDATE_CONTRIBUTION_TEST_CHILD";
    const NAME: &str = concat!(
        module_path!(),
        "::unsaved_part_preview_uses_pending_closure_reviews_and_preserves_project"
    );
    if std::env::var_os(CHILD).is_none() {
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                NAME.strip_prefix(concat!(env!("CARGO_CRATE_NAME"), "::"))
                    .unwrap(),
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(String::from_utf8_lossy(&result.stdout).contains("1 passed"));
        return;
    }
    let (root, mut model) =
        native::schematic_library::part_placement::tests::complete_assignment_history_fixture();
    let part = model
        .component_instances
        .values()
        .find(|ci| ci.authority == ComponentInstanceAuthority::Authored)
        .unwrap()
        .part_ref
        .unwrap();
    let baseline = files(&root);
    let bytes = baseline.values().map(Vec::len).sum::<usize>();
    let depth = baseline
        .keys()
        .map(|p| p.strip_prefix(&root).unwrap().components().count())
        .max()
        .unwrap();
    // Process-isolated fixture bounds derived from its actual inventory and
    // exercised transitions; these install no production defaults.
    let transitions = 8;
    let limits = LibraryAssetLimits {
        metadata_bytes: bytes * transitions,
        blob_bytes: bytes * transitions,
        files: baseline.len() * transitions,
        depth: depth + 1,
        total_bytes: bytes * baseline.len() * transitions,
    };
    configure_server_policy(limits).unwrap();
    let prepared = prepare(&model, &root, json!({"action":"adopt_profile"}));
    native::commit_prepared(&mut model, &root, prepared.prepared).unwrap();
    let catalog = snapshot(&root, limits).catalog().manifest_ref.clone();
    let captured = prepare(
        &model,
        &root,
        json!({"action":"save_candidate","request":{
        "expected_catalog":catalog,"change":{"action":"capture_native","asset_id":part}}}),
    );
    let original: AssetRevisionRef =
        serde_json::from_value(captured.summary["candidate"].clone()).unwrap();
    native::commit_prepared(&mut model, &root, captured.prepared).unwrap();
    let catalog = snapshot(&root, limits).catalog().manifest_ref.clone();
    let adopted = prepare(
        &model,
        &root,
        json!({"action":"adopt_selection","request":{
        "expected_catalog":catalog,"selected":original}}),
    );
    assert!(
        adopted.summary["asset_changes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r.get("candidate_contribution").is_none())
    );
    native::commit_prepared(&mut model, &root, adopted.prepared).unwrap();
    let state = snapshot(&root, limits);
    let unit = state
        .assets()
        .iter()
        .find(|(_, a)| a.revision().kind == AssetKind::Unit)
        .unwrap()
        .0
        .clone();
    retain_reviews(
        &mut model,
        &root,
        &[
            review(&original, &state, ReviewResult::Passed, limits),
            review(&unit, &state, ReviewResult::Failed, limits),
        ],
        limits,
    );
    let state = snapshot(&root, limits);
    let before = files(&root);
    let model_before = serde_json::to_value(&model.journal).unwrap();
    let mut payload = state.assets()[&original].payload().value().clone();
    payload["name"] = json!("Unsaved Part candidate");
    let request = json!({"action":"save_candidate","request":{"expected_catalog":state.catalog().manifest_ref,
        "change":{"action":"revise","base":original,"payload_json":serde_json::to_string(&payload).unwrap()}}});
    let proposed = prepare(&model, &root, request.clone());
    let wrapper = &row(&proposed.summary, part)["candidate_contribution"];
    let report = &wrapper["contribution"];
    assert_eq!(wrapper["kind"], "part");
    assert_eq!(wrapper["revision"], proposed.summary["candidate"]);
    assert_eq!(report["subject"], wrapper["revision"]);
    assert_ne!(report["subject"], json!(original));
    assert_eq!(report["inspection_status"], "available");
    assert_eq!(report["closure"]["graph_valid"], true);
    assert_eq!(
        report["closure"]["assets"].as_array().unwrap().len(),
        inspect_closure(
            std::slice::from_ref(&original),
            state.assets(),
            ClosureLimits {
                max_nodes: limits.files,
                max_depth: limits.depth,
            }
        )
        .unwrap()
        .assets()
        .len()
    );
    assert_eq!(member(report, part)["reviews"][0]["basis_status"], "stale");
    assert_eq!(
        member(report, part)["reviews"][0]["record"]["result"],
        "passed"
    );
    assert_eq!(
        member(report, unit.asset_id)["reviews"][0]["basis_status"],
        "current"
    );
    assert_eq!(
        member(report, unit.asset_id)["reviews"][0]["record"]["result"],
        "failed"
    );
    assert!(
        report["closure"]["assets"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["review_presence"] == "missing")
    );
    assert!(member(report, part)["provenance"]["rights"]["license_expression"].is_null());
    assert_eq!(report["redistribution_authority"], "not_verified");
    assert_eq!(report["engineering_approval"], "not_evaluated");
    assert_eq!(report["publication_policy"], "unavailable");
    // A shared dependency edit repins the Part in the pending candidate. None
    // of these proposed bytes have been committed or installed on disk.
    let mut payload = state.assets()[&unit].payload().value().clone();
    payload["name"] = json!("Unsaved dependency candidate");
    let dependency = prepare(
        &model,
        &root,
        json!({"action":"save_candidate","request":{
        "expected_catalog":state.catalog().manifest_ref,"change":{"action":"revise","base":unit,
        "payload_json":serde_json::to_string(&payload).unwrap()}}}),
    );
    assert!(
        row(&dependency.summary, unit.asset_id)
            .get("candidate_contribution")
            .is_none()
    );
    let report = &row(&dependency.summary, part)["candidate_contribution"]["contribution"];
    assert_eq!(
        member(report, unit.asset_id)["revision"],
        dependency.summary["candidate"]
    );
    assert_ne!(member(report, unit.asset_id)["revision"], json!(unit));
    assert_eq!(
        member(report, unit.asset_id)["reviews"][0]["basis_status"],
        "stale"
    );
    assert_ne!(
        report["closure"]["digest"],
        proposed.summary["asset_changes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["asset_id"] == json!(part))
            .unwrap()["candidate_contribution"]["contribution"]["closure"]["digest"]
    );
    let mut stale = request;
    stale["request"]["expected_catalog"] = json!(AssetDigest::blob(b"different catalog"));
    assert!(
        super::super::prepare_library_change(
            &NativeWriteContext {
                model: &model,
                project_root: &root
            },
            provenance(),
            stale
        )
        .is_err()
    );
    drop((proposed, dependency));
    assert_eq!(files(&root), before);
    assert_eq!(serde_json::to_value(&model.journal).unwrap(), model_before);
    assert_eq!(
        snapshot(&root, limits)
            .lock()
            .assets
            .iter()
            .find(|a| a.asset.asset_id == part)
            .unwrap()
            .asset,
        original
    );
    std::fs::remove_dir_all(root).unwrap();
}
