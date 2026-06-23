use sha2::{Digest, Sha256};

use crate::ir::serialization::to_json_deterministic;

use super::DrcViolation;

pub(crate) fn attach_drc_violation_fingerprints(violations: &mut [DrcViolation]) {
    for index in 0..violations.len() {
        let fingerprint = drc_violation_fingerprint(&violations[index]);
        violations[index].fingerprint = Some(fingerprint);
    }
}

pub(crate) fn drc_violation_fingerprint(violation: &DrcViolation) -> String {
    let material = serde_json::json!({
        "contract": "datum-eda:drc-violation-fingerprint:v1",
        "domain": "drc",
        "rule_id": violation.code,
        "primary_target": {
            "objects": violation.objects,
            "location": violation.location,
        },
        "evidence": {
            "id": violation.id,
            "rule_type": violation.rule_type,
            "severity": violation.severity,
            "message": violation.message,
            "objects": violation.objects,
            "location": violation.location,
        },
    });
    let canonical = to_json_deterministic(&material)
        .expect("DRC fingerprint material must be deterministic JSON");
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}
