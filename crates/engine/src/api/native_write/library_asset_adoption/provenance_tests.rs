//! Declaration-only intent preserves bytes which cannot survive a Value edit.
use super::*;
use crate::substrate::{CommitSource, ProjectResolver};
use serde_json::json;

fn metadata() -> AssetProvenance {
    serde_json::from_value(json!({
        "schema_version":1,"authors":[{"recorded_id":"original-author","display_name":null,"verification_evidence":[]}],
        "created_at":null,"change_actor":null,"changed_at":null,"change_summary":null,
        "origin_kind":"import","source_assets":[],"sources":[],
        "rights":{"license_expression":null,"attribution":["Original credit"],"restrictions":["Recorded restriction"],"evidence":[]},
        "evidence":[]
    })).unwrap()
}

fn asset(id: Uuid, kind: AssetKind, raw: &[u8], dependencies: Vec<Dependency>) -> VerifiedAsset {
    let provenance = metadata();
    let revision = AssetRevision {
        schema_version: 1,
        asset_id: id,
        kind,
        payload_schema_version: 1,
        payload_ref: AssetDigest::blob(raw),
        parents: vec![],
        dependencies,
        provenance_ref: provenance.fingerprint().unwrap(),
        extensions: None,
    };
    VerifiedAsset::read(
        &revision.reference().unwrap(),
        &encode_metadata(&revision).unwrap(),
        &encode_metadata(&provenance).unwrap(),
        raw,
        65536,
        counter_policy(kind, 1).unwrap(),
    )
    .unwrap()
}

fn limits() -> LibraryAssetLimits {
    LibraryAssetLimits {
        metadata_bytes: 65536,
        blob_bytes: 65536,
        files: 128,
        depth: 16,
        total_bytes: 1048576,
    }
}

#[test]
fn metadata_revision_uses_verified_bytes_and_keeps_forward_provenance() {
    let root = super::super::test_support::temp_project_root("exact_provenance");
    super::super::test_support::write_minimal_project(&root, Uuid::new_v4(), Uuid::new_v4());
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    let unit_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let unit_raw = format!(
        r#"{{"schema_version":1,"uuid":"{unit_id}","name":"Pins","manufacturer":"","pins":{{}},"tags":[]}}"#
    );
    let unit = asset(unit_id, AssetKind::Unit, unit_raw.as_bytes(), vec![]);
    let dependency = Dependency {
        role: "unit".into(),
        payload_path: vec!["unit".into()],
        target: unit.reference().clone(),
    };
    // Unknown decimal precision deliberately cannot round-trip through serde Value.
    let raw = format!(
        r#"{{ "unit":"{unit_id}", "uuid":"{symbol_id}", "schema_version":1, "name":"R", "source_precision":0.12345678901234567890123456789, "large":184467440737095516160 }}"#
    );
    let symbol = asset(
        symbol_id,
        AssetKind::Symbol,
        raw.as_bytes(),
        vec![dependency.clone()],
    );
    let base = symbol.reference().clone();
    assert!(
        symbol
            .payload()
            .verify_materialization(
                symbol.payload().value(),
                counter_policy(AssetKind::Symbol, 1).unwrap()
            )
            .is_err()
    );
    let mut claims = symbol.provenance().clone();
    claims.rights.license_expression = Some("LicenseRef-Recorded-Claim".into());
    let assets = BTreeMap::from([(unit.reference().clone(), unit), (base.clone(), symbol)]);
    let provenance = WriteProvenance::new(
        "datum-eda-gui",
        CommitSource::Test,
        "Record supplied declaration",
    );
    let request = |change| CandidateRequest {
        expected_catalog: AssetDigest::blob(b"test"),
        change,
        changed_at: None,
        provenance_override: Some(claims.clone()),
    };
    let draft = capture::capture_inventory(
        &context,
        std::slice::from_ref(&base),
        &assets,
        &provenance,
        request(CandidateChange::ReviseProvenance { base: base.clone() }),
        limits(),
    )
    .unwrap();
    let records: BTreeMap<_, _> = draft
        .content
        .iter()
        .map(|c| (c.fingerprint().unwrap(), c.bytes()))
        .collect();
    let revision: AssetRevision =
        crate::library_assets::decode_metadata(records[&draft.selected.revision_id], 65536)
            .unwrap();
    assert_eq!(revision.payload_ref, assets[&base].revision().payload_ref);
    assert_eq!(records[&revision.payload_ref], raw.as_bytes());
    assert_eq!(revision.dependencies, vec![dependency.clone()]);
    assert_eq!(revision.parents, vec![base.clone()]);
    let recorded: AssetProvenance =
        crate::library_assets::decode_metadata(records[&revision.provenance_ref], 65536).unwrap();
    assert_eq!(recorded.authors, claims.authors);
    assert_eq!(recorded.origin_kind, claims.origin_kind);
    assert_eq!(recorded.rights, claims.rights);
    assert_eq!(
        recorded.change_actor.unwrap().recorded_id.as_deref(),
        Some("datum-eda-gui")
    );
    assert!(draft.references.contains(&dependency.target));
    assert!(!draft.changed.iter().any(|r| r.asset_id == unit_id));
    assert_eq!(assets[&dependency.target].provenance(), &metadata());
    // A value-edit of this same base retains its existing precision refusal.
    assert!(
        capture::capture_inventory(
            &context,
            std::slice::from_ref(&base),
            &assets,
            &provenance,
            request(CandidateChange::Revise {
                base: base.clone(),
                payload: assets[&base].payload().value().clone()
            }),
            limits()
        )
        .is_err()
    );
    let mut missing = request(CandidateChange::ReviseProvenance { base: base.clone() });
    missing.provenance_override = None;
    assert!(
        capture::capture_inventory(
            &context,
            std::slice::from_ref(&base),
            &assets,
            &provenance,
            missing,
            limits()
        )
        .is_err()
    );
    let mut stale = base.clone();
    stale.revision_id = AssetDigest::blob(b"foreign");
    assert!(
        capture::capture_inventory(
            &context,
            &[base],
            &assets,
            &provenance,
            request(CandidateChange::ReviseProvenance { base: stale }),
            limits()
        )
        .is_err()
    );
    // No backing library blobs exist: success above must use the verified snapshot.
    assert!(!root.join("library").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn wire_revision_requires_explicit_claims_when_payload_is_omitted() {
    let base = AssetRevisionRef {
        asset_id: Uuid::new_v4(),
        revision_id: AssetDigest::blob(b"revision"),
    };
    for payload in [None, Some(String::new())] {
        assert!(CandidateChange::revision(base.clone(), payload, false, 128).is_err());
    }
    assert!(matches!(
        CandidateChange::revision(base, None, true, 128).unwrap(),
        CandidateChange::ReviseProvenance { .. }
    ));
}
