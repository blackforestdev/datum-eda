use sha2::{Digest, Sha256};

use crate::ir::serialization::to_json_deterministic;

use super::DrcViolation;

pub(crate) fn attach_drc_violation_fingerprints(violations: &mut [DrcViolation]) {
    for violation in violations.iter_mut() {
        let fingerprint = drc_violation_fingerprint(violation);
        violation.fingerprint = Some(fingerprint);
    }
}

pub(crate) fn drc_violation_fingerprint(violation: &DrcViolation) -> String {
    if matches!(
        violation.code.as_str(),
        "connectivity_no_copper" | "connectivity_unrouted_net" | "connectivity_unconnected_pin"
    ) && violation.rule_type == crate::rules::ast::RuleType::Connectivity
        && let Some(fingerprint) = &violation.fingerprint
    {
        return fingerprint.clone();
    }
    let material = serde_json::json!({
        "contract": "datum-eda:drc-violation-fingerprint:v2",
        "domain": "drc",
        "rule_id": violation.code,
        "standards_basis": violation.standards_basis,
        "rule_revision": violation.rule_revision,
        "import_key": violation.import_key,
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

/// Called only by connectivity producers, with the observations they evaluated.
/// Other rule families retain their existing full-message fingerprint contract.
pub(super) fn attach_connectivity(violation: &mut DrcViolation, observation: &serde_json::Value) {
    let material = serde_json::json!({
        "contract": "datum-eda:drc-connectivity-observation:v1",
        "rule": violation.code, "type": violation.rule_type,
        "severity": violation.severity, "objects": violation.objects,
        "basis": violation.standards_basis, "revision": violation.rule_revision,
        "import_key": violation.import_key, "observation": observation,
    });
    let canonical = to_json_deterministic(&material).expect("connectivity observation JSON");
    violation.fingerprint = Some(format!("sha256:{:x}", Sha256::digest(canonical.as_bytes())));
}

pub(super) fn legacy_connectivity(
    board: &crate::board::Board,
    mut findings: Vec<DrcViolation>,
) -> Vec<DrcViolation> {
    use std::collections::BTreeMap;
    let info = board.net_info();
    let mut unrouted = BTreeMap::<uuid::Uuid, usize>::new();
    for wire in board.unrouted() {
        *unrouted.entry(wire.net).or_default() += 1;
    }
    for finding in &mut findings {
        if !matches!(
            finding.code.as_str(),
            "connectivity_no_copper" | "connectivity_unrouted_net" | "connectivity_unconnected_pin"
        ) {
            continue;
        }
        let Some(net) = finding
            .objects
            .first()
            .and_then(|id| info.iter().find(|n| n.uuid == *id))
        else {
            continue;
        };
        let pads: BTreeMap<_, _> = board
            .pads
            .iter()
            .filter(|(_, p)| p.net == Some(net.uuid))
            .collect();
        let tracks: BTreeMap<_, _> = board
            .tracks
            .iter()
            .filter(|(_, p)| p.net == net.uuid)
            .collect();
        let vias: BTreeMap<_, _> = board
            .vias
            .iter()
            .filter(|(_, p)| p.net == net.uuid)
            .collect();
        let zones: BTreeMap<_, _> = board
            .zones
            .iter()
            .filter(|(_, p)| p.net == net.uuid)
            .collect();
        attach_connectivity(
            finding,
            &serde_json::json!({
                "evaluator": "legacy-pin-anchor", "net": net.uuid,
                "pin_count": net.pins.len(), "unrouted_count": unrouted.get(&net.uuid).copied().unwrap_or(0),
                "tracks": tracks, "pads": pads, "vias": vias, "zones": zones,
            }),
        );
    }
    findings
}

pub(super) fn native_connectivity_observation(
    snapshot: &crate::substrate::ElectricalSelectionSnapshot,
    source: &crate::connectivity::BoardCopperSource,
    net: uuid::Uuid,
    components: &[std::collections::BTreeSet<crate::substrate::ElectricalOccurrence>],
) -> serde_json::Value {
    use crate::board::query_pad::QueryPadGeometry;
    use std::collections::BTreeMap;
    let mut components = components.to_vec();
    components.sort();
    let pads: BTreeMap<_, _> = source
        .pads
        .iter()
        .filter(|(_, p)| p.net == Some(net))
        .collect();
    let tracks: BTreeMap<_, _> = source.tracks.iter().filter(|(_, p)| p.net == net).collect();
    let vias: BTreeMap<_, _> = source.vias.iter().filter(|(_, p)| p.net == net).collect();
    let zones: BTreeMap<_, _> = source.zones.iter().filter(|(_, p)| p.net == net).collect();
    let fills: BTreeMap<_, _> = zones
        .keys()
        .map(|id| (id, snapshot.current_zone_fill(**id).map(|f| &f.islands)))
        .collect();
    // Query pads may carry exact source-relative geometry that is absent from
    // the integer Board projection. Include its evaluated coordinates, without
    // source revisions or presentation metadata from the binding record.
    let query_pads: BTreeMap<_, _> = source.query_pads.iter().filter(|(id, _)| pads.contains_key(id)).map(|(id, pad)| {
        let value = match pad {
            QueryPadGeometry::Integer(pad) => serde_json::json!({"integer": pad}),
            QueryPadGeometry::SourceRelative(pad) => {
                let rational = |r: crate::board::nominal_geometry::Rational| [r.numerator.to_string(), r.denominator.to_string()];
                let center: Vec<_> = pad.center.iter().map(|c| [rational(c.a), rational(c.b)]).collect();
                serde_json::json!({"local": pad.local, "origin": pad.origin, "back": pad.back, "center": center})
            }
        };
        (id, value)
    }).collect();
    serde_json::json!({"evaluator": "native-nominal", "net": net, "components": components,
        "pads": pads, "query_pads": query_pads, "tracks": tracks, "vias": vias, "zones": zones, "fills": fills})
}
