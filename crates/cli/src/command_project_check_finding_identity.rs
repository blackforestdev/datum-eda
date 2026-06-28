use anyhow::Result;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::substrate::{
    PROCESS_APERTURE_STANDARDS_BASIS_ID, StandardsBasis, standards_basis_for_id,
    standards_basis_id_for_check_code,
};
use sha2::{Digest, Sha256};

pub(crate) fn is_standards_profile_finding(code: &str) -> bool {
    matches!(
        code,
        "pad_process_aperture_inherited_from_copper"
            | "pad_process_aperture_inconsistent_with_peer_footprint"
            | "pad_mask_expansion_missing"
            | "pad_mask_expansion_below_rule"
            | "pad_paste_reduction_missing"
            | "pad_paste_reduction_below_rule"
            | "track_width_below_min"
            | "via_hole_out_of_range"
            | "via_annular_below_min"
            | "zone_fill_unfilled"
            | "zone_fill_stale"
            | "zone_fill_unsupported"
    )
}

pub(crate) fn check_finding_standards_basis(code: &str) -> Option<&'static str> {
    standards_basis_id_for_check_code(code)
}

pub(crate) fn check_finding_standards_basis_detail(code: &str) -> Option<StandardsBasis> {
    let basis_id = check_finding_standards_basis(code)?;
    Some(standards_basis_detail(basis_id))
}

pub(crate) fn standards_basis_detail(basis_id: &str) -> StandardsBasis {
    standards_basis_for_id(basis_id).unwrap_or_else(|| {
        panic!("standards basis id {basis_id} must be registered before CheckRun emission")
    })
}

pub(crate) fn check_finding_rule_revision(code: &str) -> Option<&'static str> {
    check_finding_standards_basis(code).map(|_| "v1")
}

pub(crate) fn check_finding_import_key(payload: &serde_json::Value) -> Option<String> {
    payload
        .get("import_key")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
}

pub(crate) fn check_finding_evidence(
    code: &str,
    payload: &serde_json::Value,
    primary_target: &serde_json::Value,
) -> Vec<serde_json::Value> {
    let mut evidence = vec![payload.clone()];
    if let Some(standards_basis) = process_aperture_standards_basis(code, primary_target) {
        evidence.push(standards_basis);
    }
    evidence
}

fn process_aperture_standards_basis(
    code: &str,
    primary_target: &serde_json::Value,
) -> Option<serde_json::Value> {
    matches!(
        code,
        "pad_process_aperture_inherited_from_copper"
            | "pad_process_aperture_inconsistent_with_peer_footprint"
            | "pad_mask_expansion_missing"
            | "pad_mask_expansion_below_rule"
            | "pad_paste_reduction_missing"
            | "pad_paste_reduction_below_rule"
    )
    .then(|| {
        serde_json::json!({
            "evidence_kind": "standards_basis",
            "basis_id": PROCESS_APERTURE_STANDARDS_BASIS_ID,
            "rule_family": "process_aperture_policy",
            "rule_revision": "v1",
            "source": "drc",
            "code": code,
            "primary_target": primary_target,
            "policy": {
                "mask_aperture_must_be_explicit": true,
                "paste_aperture_must_be_explicit": true,
                "pad_process_aperture_must_not_inherit_from_copper": true
            }
        })
    })
}

pub(crate) fn check_finding_fingerprint(
    domain: &str,
    rule_id: &str,
    standards_basis: Option<&str>,
    rule_revision: Option<&str>,
    import_key: Option<&str>,
    primary_target: &serde_json::Value,
    evidence: &serde_json::Value,
) -> Result<String> {
    let material = serde_json::json!({
        "contract": "datum-eda:check-finding-fingerprint:v1",
        "domain": domain,
        "rule_id": rule_id,
        "standards_basis": standards_basis,
        "rule_revision": rule_revision,
        "import_key": import_key,
        "primary_target": primary_target,
        "evidence": evidence,
    });
    let mut hasher = Sha256::new();
    hasher.update(to_json_deterministic(&material)?.as_bytes());
    Ok(format!("sha256:{:x}", hasher.finalize()))
}
