use std::collections::BTreeMap;

use serde_json::{Value, json};
use uuid::Uuid;

use super::*;

fn unit_revision(bytes: &[u8]) -> AssetRevision {
    AssetRevision {
        schema_version: 1,
        asset_id: Uuid::from_u128(1),
        kind: AssetKind::Unit,
        payload_schema_version: 1,
        payload_ref: AssetDigest::blob(bytes),
        parents: vec![],
        dependencies: vec![],
        provenance_ref: AssetDigest::blob(b"unit-test placeholder, not review evidence"),
        extensions: None,
    }
}

fn payload(extra: &str) -> Vec<u8> {
    format!(r#"{{"schema_version":1,"uuid":"00000000-0000-0000-0000-000000000001","name":"Unit","manufacturer":"","pins":{{}},"tags":[]{extra}}}"#).into_bytes()
}

#[test]
fn metadata_requires_canonical_integer_json_without_duplicate_keys() {
    let canonical = b"{\"schema_version\":1}\n";
    assert_eq!(
        decode_metadata::<Value>(canonical, canonical.len()).unwrap(),
        json!({"schema_version":1})
    );
    for bytes in [
        b"{\"schema_version\":1,\"schema_version\":2}\n".as_slice(),
        b"{\"nested\":{\"a\":1,\"\\u0061\":2}}\n",
        b"{\"schema_version\":1.0}\n",
        b"{\"schema_version\":1e0}\n",
        b"{ \"schema_version\":1}\n",
        b"{\"schema_version\":1}",
        b"{\"schema_version\":-0}\n",
    ] {
        assert!(
            decode_metadata::<Value>(bytes, bytes.len()).is_err(),
            "{:?}",
            bytes
        );
    }
    assert!(decode_metadata::<Value>(canonical, canonical.len() - 1).is_err());
    assert!(encode_metadata(&json!({"nested":[1.25]})).is_err());
}

#[test]
fn fingerprints_match_independent_known_vector_and_separate_record_types() {
    let bytes = b"{\"schema_version\":1}\n";
    assert_eq!(
        AssetDigest::record(RecordDomain::AssetRevision, bytes).as_str(),
        "sha256:b3416cebe6fcb6ffa26729ade00def214f91326ff49b0664c3c67f5285399191"
    );
    assert_eq!(
        AssetDigest::record(RecordDomain::Provenance, bytes).as_str(),
        "sha256:cb24a8601f57da829e82e1fc8b07b261e43eaa85868846848f21931acecb879d"
    );
    assert_ne!(
        AssetDigest::record(RecordDomain::AssetRevision, bytes),
        AssetDigest::blob(bytes)
    );
    assert!(AssetDigest::parse(format!("sha256:{}", "A".repeat(64))).is_err());
    assert!(AssetDigest::parse(format!("sha1:{}", "a".repeat(64))).is_err());
}

#[test]
fn exact_payload_and_projection_do_not_hide_unknown_precision_changes() {
    let bytes = payload(",\"extension\":0.10000000000000001");
    let revision = unit_revision(&bytes);
    let verified =
        VerifiedPayload::read(&revision, &bytes, bytes.len(), LocalCounters::default()).unwrap();
    let rounded: Value = serde_json::from_slice(&bytes).unwrap();
    // serde_json's default numeric representation has rounded this unknown field.
    assert!(
        verified
            .verify_materialization(&rounded, LocalCounters::default())
            .is_err()
    );
    let different = payload(",\"extension\":0.1");
    assert!(
        VerifiedPayload::read(
            &revision,
            &different,
            different.len(),
            LocalCounters::default()
        )
        .is_err()
    );
    let duplicate = payload(",\"name\":\"Replacement\"");
    assert!(
        VerifiedPayload::read(
            &unit_revision(&duplicate),
            &duplicate,
            duplicate.len(),
            LocalCounters::default()
        )
        .is_err()
    );
}

#[test]
fn only_explicitly_classified_local_counters_can_differ() {
    let bytes = payload(",\"extension\":{\"rating\":125}");
    let policy = LocalCounters {
        object_revision: true,
        model_revision: false,
    };
    let verified =
        VerifiedPayload::read(&unit_revision(&bytes), &bytes, bytes.len(), policy).unwrap();
    let mut local = verified.value().clone();
    local["object_revision"] = json!(42);
    assert!(verified.verify_materialization(&local, policy).is_ok());
    assert!(
        verified
            .verify_materialization(&local, LocalCounters::default())
            .is_err()
    );
    local["extension"]["rating"] = json!(126);
    assert!(verified.verify_materialization(&local, policy).is_err());
    let with_counter = payload(",\"object_revision\":4");
    assert!(
        VerifiedPayload::read(
            &unit_revision(&with_counter),
            &with_counter,
            with_counter.len(),
            policy
        )
        .is_err()
    );
}

#[test]
fn required_nullable_fields_unknown_metadata_and_uuid_spelling_are_checked() {
    let bytes = payload("");
    let revision = unit_revision(&bytes);
    let canonical = encode_metadata(&revision).unwrap();
    let decoded: AssetRevision = decode_metadata(&canonical, canonical.len()).unwrap();
    assert_eq!(decoded.reference().unwrap(), revision.reference().unwrap());
    let mut value = serde_json::to_value(&revision).unwrap();
    value["unrecognized"] = json!(true);
    let invalid = encode_metadata(&value).unwrap();
    assert!(decode_metadata::<AssetRevision>(&invalid, invalid.len()).is_err());
    let mut value = serde_json::to_value(&revision).unwrap();
    value["asset_id"] = json!("ABCDEF00-0000-0000-0000-000000000001");
    let uppercase = encode_metadata(&value).unwrap();
    assert!(decode_metadata::<AssetRevision>(&uppercase, uppercase.len()).is_err());
    let provenance = AssetProvenance {
        schema_version: 1,
        authors: vec![],
        created_at: None,
        change_actor: None,
        changed_at: None,
        change_summary: None,
        origin_kind: OriginKind::Unknown,
        source_assets: vec![],
        sources: vec![],
        rights: Rights {
            license_expression: None,
            attribution: vec![],
            restrictions: vec![],
            evidence: vec![],
        },
        evidence: vec![],
        extensions: Some(BTreeMap::new()),
    };
    let mut value = serde_json::to_value(&provenance).unwrap();
    value.as_object_mut().unwrap().remove("created_at");
    let missing = encode_metadata(&value).unwrap();
    assert!(decode_metadata::<AssetProvenance>(&missing, missing.len()).is_err());
    let canonical = encode_metadata(&provenance).unwrap();
    let read: AssetProvenance = decode_metadata(&canonical, canonical.len()).unwrap();
    assert!(read.authors.is_empty() && read.created_at.is_none());
}
