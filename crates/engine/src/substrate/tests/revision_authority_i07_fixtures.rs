use uuid::Uuid;

use crate::revision::{
    AuthorityRecordKind, ReproductionAttemptData, ReproductionManifestData,
    reproduction_attempt_digest, reproduction_manifest_digest,
};

pub(super) fn rev_i07_fixture_semantics(kind: AuthorityRecordKind) -> Option<serde_json::Value> {
    let digest = |digit: char| format!("sha256:{}", digit.to_string().repeat(64));
    let authority_ref = |kind: AuthorityRecordKind| serde_json::json!({"kind": kind.wire_tag(), "id": family_id(kind)});
    Some(match kind {
        AuthorityRecordKind::ReproductionManifest => {
            let mut value = serde_json::json!({
                "release": family_id(AuthorityRecordKind::Release),
                "source_subjects": [{
                    "authority_ref": authority_ref(AuthorityRecordKind::EngineeringRevision),
                    "exact_technical_revision": "A", "digest": digest('1')
                }],
                "dependency_snapshot": family_id(AuthorityRecordKind::DependencySnapshot),
                "producer": {
                    "authority_ref": authority_ref(AuthorityRecordKind::BuildIdentity),
                    "exact_revision": "fixture-generator-1", "executable_digest": digest('2'),
                    "class": "datum_owned"
                },
                "invocation": {
                    "effective_settings": {"units": "mm"},
                    "ordered_instructions": ["render", "serialize"],
                    "invocation_digest": digest('3')
                },
                "environment": {
                    "influential": {"clock": "epoch:0", "random": "seed:1"},
                    "explicit_exclusions": ["locale", "timezone", "working_directory"],
                    "environment_digest": digest('4')
                },
                "specified_outputs": [{
                    "logical_name": "release", "kind": "zip", "byte_count": 3,
                    "digest": digest('5')
                }],
                "variance_policy": {
                    "explicitly_irrelevant_environment": ["locale", "timezone", "working_directory"],
                    "byte_identity_required": true, "network_access_allowed": false,
                    "undeclared_local_files_allowed": false, "clock_is_controlled": true,
                    "randomness_is_controlled": true
                },
                "canonicalization_policy_refs": [], "created_at": 1,
                "manifest_digest": digest('0')
            });
            let data: ReproductionManifestData = serde_json::from_value(value.clone()).unwrap();
            value["manifest_digest"] =
                serde_json::to_value(reproduction_manifest_digest(&data).unwrap()).unwrap();
            value
        }
        AuthorityRecordKind::ReproductionAttempt => {
            let manifest = rev_i07_fixture_semantics(AuthorityRecordKind::ReproductionManifest)
                .expect("manifest fixture");
            let mut value = serde_json::json!({
                "manifest": family_id(AuthorityRecordKind::ReproductionManifest),
                "manifest_digest": manifest["manifest_digest"].clone(),
                "executor_identity": {
                    "authority_ref": authority_ref(AuthorityRecordKind::BuildIdentity),
                    "exact_revision": "fixture-generator-1", "executable_digest": digest('2'),
                    "class": "datum_owned"
                },
                "attempted_at": 2,
                "independently_resolved_environment": {
                    "influential": {"clock": "epoch:0", "random": "seed:1"},
                    "explicit_exclusions": ["locale", "timezone", "working_directory"],
                    "environment_digest": digest('4')
                },
                "produced_outputs": [{
                    "logical_name": "release", "kind": "zip", "byte_count": 3,
                    "digest": digest('5')
                }],
                "result": {"result": "byte_identical"},
                "comparison_evidence": ["algorithm-qualified byte digest comparison"],
                "authenticity_evidence": [], "attempt_digest": digest('0')
            });
            let data: ReproductionAttemptData = serde_json::from_value(value.clone()).unwrap();
            value["attempt_digest"] =
                serde_json::to_value(reproduction_attempt_digest(&data).unwrap()).unwrap();
            value
        }
        _ => return None,
    })
}

fn family_id(kind: AuthorityRecordKind) -> Uuid {
    let index = AuthorityRecordKind::ALL
        .iter()
        .position(|candidate| *candidate == kind)
        .expect("family is registered");
    Uuid::from_u128(index as u128 + 1)
}
