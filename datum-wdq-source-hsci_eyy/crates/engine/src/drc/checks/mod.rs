// DRC checks — see specs/ENGINE_SPEC.md §1.4

use uuid::Uuid;

use crate::board::{Board, BoardText, PlacedPad, Track, Via};
use crate::ir::geometry::{LayerId, Point};
use crate::rules::ast::RuleType;

use super::{DrcLocation, DrcSeverity, DrcViolation};

pub(super) fn run_connectivity_checks(board: &Board) -> Vec<DrcViolation> {
    let mut violations = Vec::new();

    for net in board.net_info() {
        if net.pins.len() < 2 {
            continue;
        }

        if net.tracks == 0 && net.vias == 0 && net.zones == 0 {
            violations.push(DrcViolation {
                id: stable_violation_id(
                    "connectivity_no_copper",
                    RuleType::Connectivity,
                    None,
                    &[net.uuid],
                ),
                code: "connectivity_no_copper".into(),
                rule_type: RuleType::Connectivity,
                severity: DrcSeverity::Error,
                message: format!(
                    "net {} has {} pin(s) but no routed copper",
                    net.name,
                    net.pins.len()
                ),
                location: None,
                objects: vec![net.uuid],
                fingerprint: None,
                standards_basis: None,
                rule_revision: None,
                import_key: None,
                waived: false,
            });
        }
    }

    let mut unrouted_counts = std::collections::BTreeMap::<Uuid, (String, usize)>::new();
    for airwire in board.unrouted() {
        let entry = unrouted_counts
            .entry(airwire.net)
            .or_insert((airwire.net_name, 0));
        entry.1 += 1;
    }

    for (net_uuid, (net_name, count)) in unrouted_counts {
        violations.push(DrcViolation {
            id: stable_violation_id(
                "connectivity_unrouted_net",
                RuleType::Connectivity,
                None,
                &[net_uuid],
            ),
            code: "connectivity_unrouted_net".into(),
            rule_type: RuleType::Connectivity,
            severity: DrcSeverity::Error,
            message: format!("net {net_name} has {count} unrouted connection(s)"),
            location: None,
            objects: vec![net_uuid],
            fingerprint: None,
            standards_basis: None,
            rule_revision: None,
            import_key: None,
            waived: false,
        });
    }

    for net in board.net_info() {
        if net.pins.len() == 1 && net.tracks == 0 && net.vias == 0 && net.zones == 0 {
            let pin = &net.pins[0];
            violations.push(DrcViolation {
                id: stable_violation_id(
                    "connectivity_unconnected_pin",
                    RuleType::Connectivity,
                    None,
                    &[net.uuid],
                ),
                code: "connectivity_unconnected_pin".into(),
                rule_type: RuleType::Connectivity,
                severity: DrcSeverity::Error,
                message: format!(
                    "pin {}.{} on net {} is not connected to routed copper",
                    pin.component, pin.pin, net.name
                ),
                location: None,
                objects: vec![net.uuid],
                fingerprint: None,
                standards_basis: None,
                rule_revision: None,
                import_key: None,
                waived: false,
            });
        }
    }

    violations
}

pub(super) fn run_clearance_checks(board: &Board) -> Vec<DrcViolation> {
    let mut violations = Vec::new();
    let mut tracks: Vec<&Track> = board.tracks.values().collect();
    tracks.sort_by_key(|track| track.uuid);

    for i in 0..tracks.len() {
        for j in (i + 1)..tracks.len() {
            let a = tracks[i];
            let b = tracks[j];
            if a.layer != b.layer || a.net == b.net {
                continue;
            }

            let center_distance = segment_distance_nm(a.from, a.to, b.from, b.to);
            let edge_distance = center_distance - ((a.width + b.width) / 2);
            let required = required_clearance_nm(board, a.net, b.net);

            if edge_distance < required {
                let location = midpoint(a.from, a.to);
                let mut objects = vec![a.uuid, b.uuid];
                objects.sort();
                let violation_location = DrcLocation {
                    x_nm: location.x,
                    y_nm: location.y,
                    layer: Some(a.layer),
                };
                violations.push(DrcViolation {
                    id: stable_violation_id(
                        "clearance_copper",
                        RuleType::ClearanceCopper,
                        Some(&violation_location),
                        &objects,
                    ),
                    code: "clearance_copper".into(),
                    rule_type: RuleType::ClearanceCopper,
                    severity: DrcSeverity::Error,
                    message: format!(
                        "track clearance {}nm is below required {}nm on layer {}",
                        edge_distance, required, a.layer
                    ),
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

pub(super) fn run_track_width_checks(board: &Board) -> Vec<DrcViolation> {
    let mut violations = Vec::new();
    let mut tracks: Vec<&Track> = board.tracks.values().collect();
    tracks.sort_by_key(|track| track.uuid);

    for track in tracks {
        let required = required_track_width_nm(board, track.net);
        if track.width < required {
            let location = midpoint(track.from, track.to);
            let violation_location = DrcLocation {
                x_nm: location.x,
                y_nm: location.y,
                layer: Some(track.layer),
            };
            violations.push(DrcViolation {
                id: stable_violation_id(
                    "track_width_below_min",
                    RuleType::TrackWidth,
                    Some(&violation_location),
                    &[track.uuid],
                ),
                code: "track_width_below_min".into(),
                rule_type: RuleType::TrackWidth,
                severity: DrcSeverity::Error,
                message: format!(
                    "track width {}nm is below required {}nm on layer {}",
                    track.width, required, track.layer
                ),
                location: Some(violation_location),
                objects: vec![track.uuid],
                fingerprint: None,
                standards_basis: Some("datum.process_aperture_and_geometry.current".to_string()),
                rule_revision: Some("v1".to_string()),
                import_key: None,
                waived: false,
            });
        }
    }

    violations
}

pub(super) fn run_via_hole_checks(board: &Board) -> Vec<DrcViolation> {
    let mut violations = Vec::new();
    let mut vias: Vec<&Via> = board.vias.values().collect();
    vias.sort_by_key(|via| via.uuid);

    for via in vias {
        let (min_hole, max_hole) = required_via_hole_range_nm(board, via.net);
        if via.drill < min_hole || via.drill > max_hole {
            let violation_location = DrcLocation {
                x_nm: via.position.x,
                y_nm: via.position.y,
                layer: None,
            };
            violations.push(DrcViolation {
                id: stable_violation_id(
                    "via_hole_out_of_range",
                    RuleType::ViaHole,
                    Some(&violation_location),
                    &[via.uuid],
                ),
                code: "via_hole_out_of_range".into(),
                rule_type: RuleType::ViaHole,
                severity: DrcSeverity::Error,
                message: format!(
                    "via hole {}nm is outside allowed range {}nm..{}nm",
                    via.drill, min_hole, max_hole
                ),
                location: Some(violation_location),
                objects: vec![via.uuid],
                fingerprint: None,
                standards_basis: Some("datum.process_aperture_and_geometry.current".to_string()),
                rule_revision: Some("v1".to_string()),
                import_key: None,
                waived: false,
            });
        }
    }

    violations
}

pub(super) fn run_via_annular_checks(board: &Board) -> Vec<DrcViolation> {
    let mut violations = Vec::new();
    let mut vias: Vec<&Via> = board.vias.values().collect();
    vias.sort_by_key(|via| via.uuid);

    for via in vias {
        let required = required_via_annular_nm(board, via.net);
        let annular = (via.diameter - via.drill) / 2;
        if annular < required {
            let violation_location = DrcLocation {
                x_nm: via.position.x,
                y_nm: via.position.y,
                layer: None,
            };
            violations.push(DrcViolation {
                id: stable_violation_id(
                    "via_annular_below_min",
                    RuleType::ViaAnnularRing,
                    Some(&violation_location),
                    &[via.uuid],
                ),
                code: "via_annular_below_min".into(),
                rule_type: RuleType::ViaAnnularRing,
                severity: DrcSeverity::Error,
                message: format!(
                    "via annular ring {}nm is below required {}nm",
                    annular, required
                ),
                location: Some(violation_location),
                objects: vec![via.uuid],
                fingerprint: None,
                standards_basis: Some("datum.process_aperture_and_geometry.current".to_string()),
                rule_revision: Some("v1".to_string()),
                import_key: None,
                waived: false,
            });
        }
    }

    violations
}

pub(super) fn run_silk_clearance_checks(board: &Board) -> Vec<DrcViolation> {
    let mut violations = Vec::new();
    let mut silk_texts: Vec<&BoardText> = board
        .texts
        .iter()
        .filter(|text| is_silk_layer(text.layer))
        .collect();
    silk_texts.sort_by_key(|text| text.uuid);

    let mut tracks: Vec<&Track> = board.tracks.values().collect();
    tracks.sort_by_key(|track| track.uuid);
    let mut vias: Vec<&Via> = board.vias.values().collect();
    vias.sort_by_key(|via| via.uuid);

    for text in silk_texts {
        let required = required_silk_clearance_nm(board);
        let text_radius = silk_text_radius_nm(text);

        for track in &tracks {
            if !same_board_side(text.layer, track.layer) {
                continue;
            }
            let center_distance = point_to_segment_distance_nm(text.position, track.from, track.to);
            let edge_distance = center_distance - text_radius - (track.width / 2);
            if edge_distance < required {
                let mut objects = vec![text.uuid, track.uuid];
                objects.sort();
                let violation_location = DrcLocation {
                    x_nm: text.position.x,
                    y_nm: text.position.y,
                    layer: Some(text.layer),
                };
                violations.push(DrcViolation {
                    id: stable_violation_id(
                        "silk_clearance_copper",
                        RuleType::SilkClearance,
                        Some(&violation_location),
                        &objects,
                    ),
                    code: "silk_clearance_copper".into(),
                    rule_type: RuleType::SilkClearance,
                    severity: DrcSeverity::Error,
                    message: format!(
                        "silkscreen text '{}' clearance {}nm is below required {}nm",
                        text.text, edge_distance, required
                    ),
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

        for via in &vias {
            let center_distance =
                segment_distance_nm(text.position, text.position, via.position, via.position);
            let edge_distance = center_distance - text_radius - (via.diameter / 2);
            if edge_distance < required {
                let mut objects = vec![text.uuid, via.uuid];
                objects.sort();
                let violation_location = DrcLocation {
                    x_nm: text.position.x,
                    y_nm: text.position.y,
                    layer: Some(text.layer),
                };
                violations.push(DrcViolation {
                    id: stable_violation_id(
                        "silk_clearance_copper",
                        RuleType::SilkClearance,
                        Some(&violation_location),
                        &objects,
                    ),
                    code: "silk_clearance_copper".into(),
                    rule_type: RuleType::SilkClearance,
                    severity: DrcSeverity::Error,
                    message: format!(
                        "silkscreen text '{}' clearance {}nm is below required {}nm",
                        text.text, edge_distance, required
                    ),
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

pub(super) fn run_process_aperture_checks(board: &Board) -> Vec<DrcViolation> {
    let mut violations = Vec::new();
    let required_mask_expansion = board.pad_expansion_setup.pad_to_mask_clearance_nm.max(0);
    let required_paste_reduction = (-board.pad_expansion_setup.pad_to_paste_clearance_nm).max(0);

    let mut pads: Vec<&PlacedPad> = board.pads.values().collect();
    pads.sort_by_key(|pad| pad.uuid);

    let mut pads_by_package = std::collections::BTreeMap::<Uuid, Vec<&PlacedPad>>::new();
    for pad in &pads {
        pads_by_package.entry(pad.package).or_default().push(*pad);
    }

    for package_pads in pads_by_package.values() {
        violations.extend(process_aperture_peer_policy_violations(package_pads));
    }

    for pad in pads {
        let mask_inherits_copper = !pad.mask_layers.is_empty() && pad.solder_mask_margin_nm == 0;
        let paste_inherits_copper = !pad.paste_layers.is_empty()
            && pad.solder_paste_margin_nm == 0
            && pad.solder_paste_margin_ratio_ppm == 0;
        if mask_inherits_copper || paste_inherits_copper {
            let inherited = match (mask_inherits_copper, paste_inherits_copper) {
                (true, true) => "solder-mask and paste",
                (true, false) => "solder-mask",
                (false, true) => "paste",
                (false, false) => unreachable!(),
            };
            violations.push(pad_process_aperture_violation(
                pad,
                "pad_process_aperture_inherited_from_copper",
                format!(
                    "pad {} {} aperture is inherited from copper instead of an explicit process aperture",
                    pad.name, inherited
                ),
            ));
        }

        if required_mask_expansion > 0
            && !pad.mask_layers.is_empty()
            && pad.solder_mask_margin_nm < required_mask_expansion
        {
            let code = if pad.solder_mask_margin_nm == 0 {
                "pad_mask_expansion_missing"
            } else {
                "pad_mask_expansion_below_rule"
            };
            violations.push(pad_process_aperture_violation(
                pad,
                code,
                format!(
                    "pad {} solder-mask expansion {}nm is below required {}nm",
                    pad.name, pad.solder_mask_margin_nm, required_mask_expansion
                ),
            ));
        }

        if required_paste_reduction > 0
            && !pad.paste_layers.is_empty()
            && pad.solder_paste_margin_nm > -required_paste_reduction
        {
            let actual_reduction = (-pad.solder_paste_margin_nm).max(0);
            let code = if pad.solder_paste_margin_nm == 0 {
                "pad_paste_reduction_missing"
            } else {
                "pad_paste_reduction_below_rule"
            };
            violations.push(pad_process_aperture_violation(
                pad,
                code,
                format!(
                    "pad {} paste reduction {}nm is below required {}nm",
                    pad.name, actual_reduction, required_paste_reduction
                ),
            ));
        }
    }

    violations
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PadProcessAperturePolicy {
    mask_layers: Vec<LayerId>,
    paste_layers: Vec<LayerId>,
    solder_mask_margin_nm: i64,
    solder_paste_margin_nm: i64,
    solder_paste_margin_ratio_ppm: i32,
}

impl PadProcessAperturePolicy {
    fn from_pad(pad: &PlacedPad) -> Self {
        let mut mask_layers = pad.mask_layers.clone();
        mask_layers.sort();
        let mut paste_layers = pad.paste_layers.clone();
        paste_layers.sort();
        Self {
            mask_layers,
            paste_layers,
            solder_mask_margin_nm: pad.solder_mask_margin_nm,
            solder_paste_margin_nm: pad.solder_paste_margin_nm,
            solder_paste_margin_ratio_ppm: pad.solder_paste_margin_ratio_ppm,
        }
    }
}

fn process_aperture_peer_policy_violations(package_pads: &[&PlacedPad]) -> Vec<DrcViolation> {
    if package_pads.len() < 2 {
        return Vec::new();
    }

    let mut policy_counts = std::collections::BTreeMap::<PadProcessAperturePolicy, usize>::new();
    for pad in package_pads {
        *policy_counts
            .entry(PadProcessAperturePolicy::from_pad(pad))
            .or_default() += 1;
    }
    if policy_counts.len() < 2 {
        return Vec::new();
    }

    let expected_policy = policy_counts
        .into_iter()
        .max_by(|(left_policy, left_count), (right_policy, right_count)| {
            left_count
                .cmp(right_count)
                .then_with(|| right_policy.cmp(left_policy))
        })
        .map(|(policy, _)| policy)
        .expect("non-empty policy_counts");

    let mut violations = Vec::new();
    for pad in package_pads {
        if PadProcessAperturePolicy::from_pad(pad) == expected_policy {
            continue;
        }
        violations.push(pad_process_aperture_violation(
            pad,
            "pad_process_aperture_inconsistent_with_peer_footprint",
            format!(
                "pad {} mask/paste aperture policy is inconsistent with peer pads in the same footprint",
                pad.name
            ),
        ));
    }
    violations
}

fn pad_process_aperture_violation(pad: &PlacedPad, code: &str, message: String) -> DrcViolation {
    let location = DrcLocation {
        x_nm: pad.position.x,
        y_nm: pad.position.y,
        layer: Some(pad.layer),
    };
    DrcViolation {
        id: stable_violation_id(
            code,
            RuleType::ProcessAperture,
            Some(&location),
            &[pad.uuid],
        ),
        code: code.into(),
        rule_type: RuleType::ProcessAperture,
        severity: DrcSeverity::Error,
        message,
        location: Some(location),
        objects: vec![pad.uuid],
        fingerprint: None,
        standards_basis: Some("datum.process_aperture_and_geometry.current".to_string()),
        rule_revision: Some("v1".to_string()),
        import_key: None,
        waived: false,
    }
}

fn stable_violation_id(
    code: &str,
    rule_type: RuleType,
    location: Option<&DrcLocation>,
    objects: &[Uuid],
) -> Uuid {
    let mut sorted_objects = objects.to_vec();
    sorted_objects.sort();
    let location_material = location
        .map(|location| {
            format!(
                "{}:{}:{}",
                location.x_nm,
                location.y_nm,
                location
                    .layer
                    .map(|layer| layer.to_string())
                    .unwrap_or_else(|| "none".to_string())
            )
        })
        .unwrap_or_else(|| "none".to_string());
    let material = format!(
        "datum-eda:drc-violation:{:?}:{code}:{location_material}:{:?}",
        rule_type, sorted_objects
    );
    Uuid::new_v5(&Uuid::NAMESPACE_OID, material.as_bytes())
}

fn required_clearance_nm(board: &Board, net_a: Uuid, net_b: Uuid) -> i64 {
    let from_class = |net_uuid: Uuid| -> Option<i64> {
        let net = board.nets.get(&net_uuid)?;
        let class = board.net_classes.get(&net.class)?;
        Some(class.clearance)
    };
    match (from_class(net_a), from_class(net_b)) {
        (Some(a), Some(b)) => a.max(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => 100_000, // 0.1mm fallback
    }
}

fn required_track_width_nm(board: &Board, net_uuid: Uuid) -> i64 {
    let from_class = || -> Option<i64> {
        let net = board.nets.get(&net_uuid)?;
        let class = board.net_classes.get(&net.class)?;
        Some(class.track_width)
    };
    from_class().filter(|value| *value > 0).unwrap_or(100_000)
}

fn required_via_hole_range_nm(board: &Board, net_uuid: Uuid) -> (i64, i64) {
    let from_class = || -> Option<i64> {
        let net = board.nets.get(&net_uuid)?;
        let class = board.net_classes.get(&net.class)?;
        Some(class.via_drill)
    };
    let min = from_class().filter(|value| *value > 0).unwrap_or(100_000);
    (min, i64::MAX)
}

fn required_via_annular_nm(board: &Board, net_uuid: Uuid) -> i64 {
    let from_class = || -> Option<i64> {
        let net = board.nets.get(&net_uuid)?;
        let class = board.net_classes.get(&net.class)?;
        if class.via_diameter > 0 && class.via_drill > 0 {
            Some((class.via_diameter - class.via_drill) / 2)
        } else {
            None
        }
    };
    from_class().filter(|value| *value > 0).unwrap_or(100_000)
}

fn required_silk_clearance_nm(_board: &Board) -> i64 {
    100_000
}

fn silk_text_radius_nm(text: &BoardText) -> i64 {
    let chars = text.text.chars().count().max(1) as i64;
    (chars * 250_000).max(250_000)
}

fn is_silk_layer(layer: LayerId) -> bool {
    layer == 36 || layer == 37
}

fn same_board_side(a: LayerId, b: LayerId) -> bool {
    board_side(a) == board_side(b)
}

fn board_side(layer: LayerId) -> i8 {
    match layer {
        0 | 37 => 1,
        31 | 36 => -1,
        _ => 0,
    }
}

fn midpoint(a: Point, b: Point) -> Point {
    Point::new((a.x + b.x) / 2, (a.y + b.y) / 2)
}

fn segment_distance_nm(a0: Point, a1: Point, b0: Point, b1: Point) -> i64 {
    let d0 = point_to_segment_distance_nm(a0, b0, b1);
    let d1 = point_to_segment_distance_nm(a1, b0, b1);
    let d2 = point_to_segment_distance_nm(b0, a0, a1);
    let d3 = point_to_segment_distance_nm(b1, a0, a1);
    d0.min(d1).min(d2).min(d3)
}

fn point_to_segment_distance_nm(p: Point, s0: Point, s1: Point) -> i64 {
    let px = p.x as f64;
    let py = p.y as f64;
    let x0 = s0.x as f64;
    let y0 = s0.y as f64;
    let x1 = s1.x as f64;
    let y1 = s1.y as f64;
    let dx = x1 - x0;
    let dy = y1 - y0;

    if dx == 0.0 && dy == 0.0 {
        return ((px - x0).hypot(py - y0).round()) as i64;
    }

    let t = ((px - x0) * dx + (py - y0) * dy) / (dx * dx + dy * dy);
    let clamped_t = t.clamp(0.0, 1.0);
    let cx = x0 + clamped_t * dx;
    let cy = y0 + clamped_t * dy;
    ((px - cx).hypot(py - cy).round()) as i64
}
