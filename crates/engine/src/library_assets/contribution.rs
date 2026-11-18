//! Read-only contribution facts over an already verified local snapshot.
//! Declared rights, evidence currency and engineering approval are independent.
use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::{
    AssetDigest, AssetKind, AssetRevisionRef, ClosureLimits, VerifiedAsset, inspect_closure,
    invalid, review::AssetReview,
};
use crate::error::EngineError;

/// Inspection is additive: a historical nonroot envelope may be unavailable
/// for full contribution inspection without hiding the current local catalog.
/// Integrity/transport faults still refuse in the resolver before this reader.
pub(crate) fn inspect_part_report(
    subject: &AssetRevisionRef,
    available: &BTreeMap<AssetRevisionRef, VerifiedAsset>,
    reviews: &BTreeMap<AssetDigest, AssetReview>,
    limits: ClosureLimits,
) -> Value {
    match inspect_part(subject, available, reviews, limits) {
        Ok(mut report) => {
            report["inspection_status"] = json!("available");
            report
        }
        Err(error) => json!({"subject":subject,"inspection_status":"unavailable",
            "reason":error.to_string(),"redistribution_authority":"not_verified",
            "engineering_approval":"not_evaluated","publication_policy":"unavailable"}),
    }
}

pub(crate) fn inspect_part(
    subject: &AssetRevisionRef,
    available: &BTreeMap<AssetRevisionRef, VerifiedAsset>,
    reviews: &BTreeMap<AssetDigest, AssetReview>,
    limits: ClosureLimits,
) -> Result<Value, EngineError> {
    if available
        .get(subject)
        .is_none_or(|asset| asset.revision().kind != AssetKind::Part)
    {
        return Err(invalid(
            "contribution inspection requires an exact available Part",
        ));
    }
    let closure = inspect_closure(std::slice::from_ref(subject), available, limits)?;
    let mut members = Vec::new();
    for (reference, asset) in closure.assets() {
        let records: Vec<_> = reviews
            .iter()
            .filter(|(_, review)| review.subject.asset_id == reference.asset_id)
            .collect();
        // A child's forward closure may need contextual Part ownership for
        // native graph validity. Here only its exact review-basis digest is used.
        let basis = if records.is_empty() {
            None
        } else {
            Some(inspect_closure(
                std::slice::from_ref(reference),
                available,
                limits,
            )?)
        };
        let evidence = records.iter().map(|(digest, review)| {
            if review.fingerprint()? != **digest {
                return Err(invalid("contribution review record fingerprint differs"));
            }
            Ok(json!({"record_ref":digest,"record":review,
                "basis_status":if review.verify_basis(reference, basis.as_ref().unwrap().digest()).is_ok() {"current"} else {"stale"}}))
        }).collect::<Result<Vec<_>, EngineError>>()?;
        members.push(json!({"revision":reference,"kind":asset.revision().kind,
            "dependencies":asset.revision().dependencies,
            "payload_ref":asset.revision().payload_ref,
            "provenance_ref":asset.revision().provenance_ref,
            "provenance":asset.provenance(),
            "review_presence":if evidence.is_empty() {"missing"} else {"recorded"},
            "reviews":evidence}));
    }
    let graph = closure.graph_report();
    let diagnostics: Vec<_> = graph
        .diagnostics
        .iter()
        .map(|finding| {
            json!({
        "severity":finding.severity,"code":finding.code,
        "subject":finding.subject,"message":finding.message})
        })
        .collect();
    Ok(
        json!({"subject":subject,"closure":{"digest":closure.digest(),
        "graph_valid":graph.valid,"diagnostics":diagnostics,"assets":members},
        "redistribution_authority":"not_verified",
        "engineering_approval":"not_evaluated","publication_policy":"unavailable"}),
    )
}

#[cfg(test)]
#[path = "contribution_tests.rs"]
mod tests;
