//! Exact nominal Track clearance. Arithmetic/capability errors remain findings.
use super::*;
use crate::board::{nominal_geometry::DistanceBoundary, track_contact::tracks_within};

pub(in crate::drc) fn run_clearance_checks(board: &Board) -> Vec<DrcViolation> {
    let mut violations = arc_peer_capability_findings(board);
    let mut tracks: Vec<&Track> = board.tracks.values().collect();
    tracks.sort_by_key(|track| track.uuid);

    for i in 0..tracks.len() {
        for j in (i + 1)..tracks.len() {
            let a = tracks[i];
            let b = tracks[j];
            if a.layer != b.layer || a.net == b.net {
                continue;
            }

            let required = required_clearance_nm(board, a.net, b.net);
            let result = tracks_within(a, b, required, DistanceBoundary::Strict);
            if matches!(result, Ok(false)) {
                continue;
            }
            {
                let location = a.midpoint.unwrap_or_else(|| {
                    crate::ir::geometry::Point::new(
                        ((i128::from(a.from.x) + i128::from(a.to.x)) / 2) as i64,
                        ((i128::from(a.from.y) + i128::from(a.to.y)) / 2) as i64,
                    )
                });
                let mut objects = vec![a.uuid, b.uuid];
                objects.sort();
                let violation_location = DrcLocation {
                    x_nm: location.x,
                    y_nm: location.y,
                    layer: Some(a.layer),
                };
                let (code, message) = match result {
                    Ok(true) => {
                        let message = if a.midpoint.is_none() && b.midpoint.is_none() {
                            // Preserve the established straight report/fingerprint. The
                            // rounded diagnostic is not the classification predicate.
                            let edge = i128::from(segment_distance_nm(a.from, a.to, b.from, b.to))
                                - (i128::from(a.width) + i128::from(b.width)) / 2;
                            format!(
                                "track clearance {edge}nm is below required {required}nm on layer {}",
                                a.layer
                            )
                        } else {
                            format!(
                                "nominal Track clearance is below required {required}nm on layer {}",
                                a.layer
                            )
                        };
                        ("clearance_copper", message)
                    }
                    Err(error) => (
                        "nominal_geometry_unavailable",
                        format!(
                            "certified Track clearance unavailable: {error:?}; source pair requires resolution"
                        ),
                    ),
                    Ok(false) => unreachable!(),
                };
                violations.push(DrcViolation {
                    id: stable_violation_id(
                        code,
                        RuleType::ClearanceCopper,
                        Some(&violation_location),
                        &objects,
                    ),
                    code: code.into(),
                    rule_type: RuleType::ClearanceCopper,
                    severity: DrcSeverity::Error,
                    message,
                    location: Some(violation_location),
                    objects,
                    fingerprint: None,
                    standards_basis: None,
                    rule_revision: None,
                    import_key: None,
                    waived: false,
                });
            }
        }
    }

    violations
}

fn arc_peer_capability_findings(board: &Board) -> Vec<DrcViolation> {
    let mut result = Vec::new();
    let mut arcs = board
        .tracks
        .values()
        .filter(|track| track.midpoint.is_some())
        .collect::<Vec<_>>();
    arcs.sort_by_key(|track| track.uuid);
    for arc in arcs {
        let mut peers = std::collections::BTreeSet::new();
        for pad in board.pads.values() {
            if pad.net != Some(arc.net)
                && (if pad.copper_layers.is_empty() {
                    pad.layer == arc.layer
                } else {
                    pad.copper_layers.contains(&arc.layer)
                })
            {
                peers.insert(pad.uuid);
            }
        }
        for via in board.vias.values() {
            let layer_index = |id| board.stackup.layers.iter().position(|layer| layer.id == id);
            let relevant = match (
                layer_index(via.from_layer),
                layer_index(via.to_layer),
                layer_index(arc.layer),
            ) {
                (Some(a), Some(b), Some(p)) => p >= a.min(b) && p <= a.max(b),
                _ => true, // Unknown span is unavailable, never proof of separation.
            };
            if via.net != arc.net && relevant {
                peers.insert(via.uuid);
            }
        }
        for zone in board.zones.values() {
            if zone.net != arc.net && zone.layer == arc.layer {
                peers.insert(zone.uuid);
            }
        }
        for peer in peers {
            let mut objects = vec![arc.uuid, peer];
            objects.sort();
            result.push(DrcViolation {id:stable_violation_id("nominal_geometry_unavailable",RuleType::ClearanceCopper,None,&objects),code:"nominal_geometry_unavailable".into(),rule_type:RuleType::ClearanceCopper,severity:DrcSeverity::Error,message:"certified arc/pad, via-span or current-fill clearance is not yet available; no chord or outline fallback".into(),location:None,objects,fingerprint:None,standards_basis:None,rule_revision:None,import_key:None,waived:false});
        }
    }
    result
}
