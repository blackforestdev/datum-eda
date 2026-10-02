//! Exact nominal Track clearance. Arithmetic/capability errors remain findings.
use super::*;
use crate::board::{nominal_geometry::DistanceBoundary, track_contact::tracks_within};

pub(in crate::drc) fn run_clearance_checks(board: &Board) -> Vec<DrcViolation> {
    run_clearance_checks_with_zone_copper(board, &std::collections::BTreeMap::new())
}
pub(in crate::drc) fn run_clearance_checks_with_zone_copper(
    board: &Board,
    copper: &std::collections::BTreeMap<
        Uuid,
        Result<Vec<crate::ir::geometry::Polygon>, crate::board::nominal_geometry::GeometryError>,
    >,
) -> Vec<DrcViolation> {
    let mut violations = arc_peer_findings(board, copper);
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

fn arc_peer_findings(
    board: &Board,
    copper: &std::collections::BTreeMap<
        Uuid,
        Result<Vec<crate::ir::geometry::Polygon>, crate::board::nominal_geometry::GeometryError>,
    >,
) -> Vec<DrcViolation> {
    use crate::board::nominal_geometry::GeometryError;
    use crate::board::occupied_copper::{
        pad_layers, track_layers, track_pad_within, track_via_within, via_layers,
    };
    let mut result = Vec::new();
    let mut arcs = board
        .tracks
        .values()
        .filter(|t| t.midpoint.is_some())
        .collect::<Vec<_>>();
    arcs.sort_by_key(|t| t.uuid);
    for arc in arcs {
        let mut peers = std::collections::BTreeMap::new();
        for pad in board.pads.values() {
            if pad.net == Some(arc.net) {
                continue;
            }
            let classification = (|| {
                let layers = pad_layers(&board.stackup, pad)?;
                track_layers(&board.stackup, arc)?;
                if !layers.contains(&arc.layer) {
                    return Ok(false);
                }
                let Some(net) = pad.net else {
                    return Err(GeometryError::UnknownNetAssignment);
                };
                if !board.nets.contains_key(&net) || !board.nets.contains_key(&arc.net) {
                    return Err(GeometryError::UnknownNetAssignment);
                }
                track_pad_within(
                    arc,
                    pad,
                    required_clearance_nm(board, arc.net, net),
                    DistanceBoundary::Strict,
                )
            })();
            peers.insert(pad.uuid, classification);
        }
        for via in board.vias.values() {
            if via.net == arc.net {
                continue;
            }
            let classification = (|| {
                let layers = via_layers(&board.stackup, via)?;
                track_layers(&board.stackup, arc)?;
                if !layers.contains(&arc.layer) {
                    return Ok(false);
                }
                if !board.nets.contains_key(&via.net) || !board.nets.contains_key(&arc.net) {
                    return Err(GeometryError::UnknownNetAssignment);
                }
                track_via_within(
                    arc,
                    via,
                    required_clearance_nm(board, arc.net, via.net),
                    DistanceBoundary::Strict,
                )
            })();
            peers.insert(via.uuid, classification);
        }
        for zone in board.zones.values() {
            if zone.net != arc.net && zone.layer == arc.layer {
                let classification = (|| {
                    let polygons = copper
                        .get(&zone.uuid)
                        .ok_or(GeometryError::UnverifiedFillBasis)?
                        .as_ref()
                        .map_err(|e| *e)?;
                    if polygons.is_empty() {
                        return Ok(false);
                    }
                    crate::board::occupied_copper::conductive_layer(&board.stackup, zone.layer)?;
                    track_layers(&board.stackup, arc)?;
                    if !board.nets.contains_key(&zone.net) || !board.nets.contains_key(&arc.net) {
                        return Err(GeometryError::UnknownNetAssignment);
                    }
                    let mut violates = false;
                    for p in polygons {
                        violates |= crate::board::occupied_copper::track_polygon_within(
                            arc,
                            p,
                            required_clearance_nm(board, arc.net, zone.net),
                            DistanceBoundary::Strict,
                        )?;
                    }
                    Ok(violates)
                })();
                peers.insert(zone.uuid, classification);
            }
        }
        for (peer, classification) in peers {
            let (code, message) = match classification {
                Ok(false) => continue,
                Ok(true) => (
                    "clearance_copper",
                    format!(
                        "certified nominal arc clearance violates applicable copper clearance on layer {}",
                        arc.layer
                    ),
                ),
                Err(error) => (
                    "nominal_geometry_unavailable",
                    format!(
                        "certified nominal arc clearance unavailable: {error:?}; source pair requires resolution"
                    ),
                ),
            };
            let mut objects = vec![arc.uuid, peer];
            objects.sort();
            result.push(DrcViolation {
                id: stable_violation_id(code, RuleType::ClearanceCopper, None, &objects),
                code: code.into(),
                rule_type: RuleType::ClearanceCopper,
                severity: DrcSeverity::Error,
                message,
                location: None,
                objects,
                fingerprint: None,
                standards_basis: None,
                rule_revision: None,
                import_key: None,
                waived: false,
            });
        }
    }
    result
}
