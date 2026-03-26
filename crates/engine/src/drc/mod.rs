use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, BoardText, Track, Via};
use crate::ir::geometry::{LayerId, Point};
use crate::rules::ast::RuleType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrcSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrcLocation {
    pub x_nm: i64,
    pub y_nm: i64,
    pub layer: Option<LayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrcViolation {
    pub id: Uuid,
    pub code: String,
    pub rule_type: RuleType,
    pub severity: DrcSeverity,
    pub message: String,
    pub location: Option<DrcLocation>,
    pub objects: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrcSummary {
    pub errors: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrcReport {
    pub passed: bool,
    pub violations: Vec<DrcViolation>,
    pub summary: DrcSummary,
}

pub fn run(board: &Board, selected_rules: &[RuleType]) -> DrcReport {
    let run_all = selected_rules.is_empty();
    let mut violations = Vec::new();

    if run_all || selected_rules.contains(&RuleType::Connectivity) {
        violations.extend(run_connectivity_checks(board));
    }
    if run_all || selected_rules.contains(&RuleType::ClearanceCopper) {
        violations.extend(run_clearance_checks(board));
    }
    if run_all || selected_rules.contains(&RuleType::TrackWidth) {
        violations.extend(run_track_width_checks(board));
    }
    if run_all || selected_rules.contains(&RuleType::ViaHole) {
        violations.extend(run_via_hole_checks(board));
    }
    if run_all || selected_rules.contains(&RuleType::ViaAnnularRing) {
        violations.extend(run_via_annular_checks(board));
    }
    if run_all || selected_rules.contains(&RuleType::SilkClearance) {
        violations.extend(run_silk_clearance_checks(board));
    }

    violations.sort_by(|a, b| {
        a.code
            .cmp(&b.code)
            .then_with(|| a.message.cmp(&b.message))
            .then_with(|| a.objects.cmp(&b.objects))
            .then_with(|| a.id.cmp(&b.id))
    });

    let mut summary = DrcSummary {
        errors: 0,
        warnings: 0,
    };
    for violation in &violations {
        match violation.severity {
            DrcSeverity::Error => summary.errors += 1,
            DrcSeverity::Warning => summary.warnings += 1,
        }
    }

    DrcReport {
        passed: summary.errors == 0,
        violations,
        summary,
    }
}

fn run_connectivity_checks(board: &Board) -> Vec<DrcViolation> {
    let mut violations = Vec::new();

    for net in board.net_info() {
        if net.pins.len() < 2 {
            continue;
        }

        if net.tracks == 0 && net.vias == 0 && net.zones == 0 {
            violations.push(DrcViolation {
                id: Uuid::new_v4(),
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
            id: Uuid::new_v4(),
            code: "connectivity_unrouted_net".into(),
            rule_type: RuleType::Connectivity,
            severity: DrcSeverity::Error,
            message: format!("net {net_name} has {count} unrouted connection(s)"),
            location: None,
            objects: vec![net_uuid],
        });
    }

    for net in board.net_info() {
        if net.pins.len() == 1 && net.tracks == 0 && net.vias == 0 && net.zones == 0 {
            let pin = &net.pins[0];
            violations.push(DrcViolation {
                id: Uuid::new_v4(),
                code: "connectivity_unconnected_pin".into(),
                rule_type: RuleType::Connectivity,
                severity: DrcSeverity::Error,
                message: format!(
                    "pin {}.{} on net {} is not connected to routed copper",
                    pin.component, pin.pin, net.name
                ),
                location: None,
                objects: vec![net.uuid],
            });
        }
    }

    violations
}

fn run_clearance_checks(board: &Board) -> Vec<DrcViolation> {
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
                violations.push(DrcViolation {
                    id: Uuid::new_v4(),
                    code: "clearance_copper".into(),
                    rule_type: RuleType::ClearanceCopper,
                    severity: DrcSeverity::Error,
                    message: format!(
                        "track clearance {}nm is below required {}nm on layer {}",
                        edge_distance, required, a.layer
                    ),
                    location: Some(DrcLocation {
                        x_nm: location.x,
                        y_nm: location.y,
                        layer: Some(a.layer),
                    }),
                    objects,
                });
            }
        }
    }

    violations
}

fn run_track_width_checks(board: &Board) -> Vec<DrcViolation> {
    let mut violations = Vec::new();
    let mut tracks: Vec<&Track> = board.tracks.values().collect();
    tracks.sort_by_key(|track| track.uuid);

    for track in tracks {
        let required = required_track_width_nm(board, track.net);
        if track.width < required {
            let location = midpoint(track.from, track.to);
            violations.push(DrcViolation {
                id: Uuid::new_v4(),
                code: "track_width_below_min".into(),
                rule_type: RuleType::TrackWidth,
                severity: DrcSeverity::Error,
                message: format!(
                    "track width {}nm is below required {}nm on layer {}",
                    track.width, required, track.layer
                ),
                location: Some(DrcLocation {
                    x_nm: location.x,
                    y_nm: location.y,
                    layer: Some(track.layer),
                }),
                objects: vec![track.uuid],
            });
        }
    }

    violations
}

fn run_via_hole_checks(board: &Board) -> Vec<DrcViolation> {
    let mut violations = Vec::new();
    let mut vias: Vec<&Via> = board.vias.values().collect();
    vias.sort_by_key(|via| via.uuid);

    for via in vias {
        let (min_hole, max_hole) = required_via_hole_range_nm(board, via.net);
        if via.drill < min_hole || via.drill > max_hole {
            violations.push(DrcViolation {
                id: Uuid::new_v4(),
                code: "via_hole_out_of_range".into(),
                rule_type: RuleType::ViaHole,
                severity: DrcSeverity::Error,
                message: format!(
                    "via hole {}nm is outside allowed range {}nm..{}nm",
                    via.drill, min_hole, max_hole
                ),
                location: Some(DrcLocation {
                    x_nm: via.position.x,
                    y_nm: via.position.y,
                    layer: None,
                }),
                objects: vec![via.uuid],
            });
        }
    }

    violations
}

fn run_via_annular_checks(board: &Board) -> Vec<DrcViolation> {
    let mut violations = Vec::new();
    let mut vias: Vec<&Via> = board.vias.values().collect();
    vias.sort_by_key(|via| via.uuid);

    for via in vias {
        let required = required_via_annular_nm(board, via.net);
        let annular = (via.diameter - via.drill) / 2;
        if annular < required {
            violations.push(DrcViolation {
                id: Uuid::new_v4(),
                code: "via_annular_below_min".into(),
                rule_type: RuleType::ViaAnnularRing,
                severity: DrcSeverity::Error,
                message: format!(
                    "via annular ring {}nm is below required {}nm",
                    annular, required
                ),
                location: Some(DrcLocation {
                    x_nm: via.position.x,
                    y_nm: via.position.y,
                    layer: None,
                }),
                objects: vec![via.uuid],
            });
        }
    }

    violations
}

fn run_silk_clearance_checks(board: &Board) -> Vec<DrcViolation> {
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
                violations.push(DrcViolation {
                    id: Uuid::new_v4(),
                    code: "silk_clearance_copper".into(),
                    rule_type: RuleType::SilkClearance,
                    severity: DrcSeverity::Error,
                    message: format!(
                        "silkscreen text '{}' clearance {}nm is below required {}nm",
                        text.text, edge_distance, required
                    ),
                    location: Some(DrcLocation {
                        x_nm: text.position.x,
                        y_nm: text.position.y,
                        layer: Some(text.layer),
                    }),
                    objects,
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
                violations.push(DrcViolation {
                    id: Uuid::new_v4(),
                    code: "silk_clearance_copper".into(),
                    rule_type: RuleType::SilkClearance,
                    severity: DrcSeverity::Error,
                    message: format!(
                        "silkscreen text '{}' clearance {}nm is below required {}nm",
                        text.text, edge_distance, required
                    ),
                    location: Some(DrcLocation {
                        x_nm: text.position.x,
                        y_nm: text.position.y,
                        layer: Some(text.layer),
                    }),
                    objects,
                });
            }
        }
    }

    violations
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::board::{
        Board, Keepout, Net, NetClass, PlacedPackage, Stackup, StackupLayer, StackupLayerType, Zone,
    };
    use crate::ir::geometry::Polygon;

    fn empty_board() -> Board {
        Board {
            uuid: Uuid::new_v4(),
            name: "drc-demo".into(),
            stackup: Stackup {
                layers: vec![StackupLayer {
                    id: 1,
                    name: "F.Cu".into(),
                    layer_type: StackupLayerType::Copper,
                    thickness_nm: 35_000,
                }],
            },
            outline: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(100_000_000, 0),
                Point::new(100_000_000, 100_000_000),
                Point::new(0, 100_000_000),
            ]),
            packages: HashMap::<Uuid, PlacedPackage>::new(),
            pads: HashMap::new(),
            tracks: HashMap::new(),
            vias: HashMap::new(),
            zones: HashMap::<Uuid, Zone>::new(),
            nets: HashMap::new(),
            net_classes: HashMap::new(),
            rules: Vec::new(),
            keepouts: Vec::<Keepout>::new(),
            dimensions: Vec::new(),
            texts: Vec::new(),
        }
    }

    #[test]
    fn connectivity_check_reports_no_copper_net_with_two_pins() {
        let mut board = empty_board();
        let class_uuid = Uuid::new_v4();
        let net_uuid = Uuid::new_v4();
        board.net_classes.insert(
            class_uuid,
            NetClass {
                uuid: class_uuid,
                name: "default".into(),
                clearance: 100_000,
                track_width: 200_000,
                via_drill: 300_000,
                via_diameter: 600_000,
                diffpair_width: 0,
                diffpair_gap: 0,
            },
        );
        board.nets.insert(
            net_uuid,
            Net {
                uuid: net_uuid,
                name: "SIG".into(),
                class: class_uuid,
            },
        );
        let pkg_a = Uuid::new_v4();
        let pkg_b = Uuid::new_v4();
        board.packages.insert(
            pkg_a,
            PlacedPackage {
                uuid: pkg_a,
                part: Uuid::new_v4(),
                package: Uuid::nil(),
                reference: "R1".into(),
                value: "10k".into(),
                position: Point::new(10_000_000, 10_000_000),
                rotation: 0,
                layer: 1,
                locked: false,
            },
        );
        board.packages.insert(
            pkg_b,
            PlacedPackage {
                uuid: pkg_b,
                part: Uuid::new_v4(),
                package: Uuid::nil(),
                reference: "R2".into(),
                value: "10k".into(),
                position: Point::new(40_000_000, 10_000_000),
                rotation: 0,
                layer: 1,
                locked: false,
            },
        );
        board.pads.insert(
            Uuid::new_v4(),
            crate::board::PlacedPad {
                uuid: Uuid::new_v4(),
                package: pkg_a,
                name: "1".into(),
                net: Some(net_uuid),
                position: Point::new(10_000_000, 10_000_000),
                layer: 1,
            },
        );
        board.pads.insert(
            Uuid::new_v4(),
            crate::board::PlacedPad {
                uuid: Uuid::new_v4(),
                package: pkg_b,
                name: "1".into(),
                net: Some(net_uuid),
                position: Point::new(40_000_000, 10_000_000),
                layer: 1,
            },
        );

        let report = run(&board, &[RuleType::Connectivity]);
        assert!(!report.passed);
        assert_eq!(report.summary.errors, 2);
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "connectivity_no_copper")
        );
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "connectivity_unrouted_net")
        );
    }

    #[test]
    fn clearance_check_reports_overlapping_tracks_on_different_nets() {
        let mut board = empty_board();
        let class_uuid = Uuid::new_v4();
        board.net_classes.insert(
            class_uuid,
            NetClass {
                uuid: class_uuid,
                name: "default".into(),
                clearance: 200_000,
                track_width: 200_000,
                via_drill: 300_000,
                via_diameter: 600_000,
                diffpair_width: 0,
                diffpair_gap: 0,
            },
        );
        let net_a = Uuid::new_v4();
        let net_b = Uuid::new_v4();
        board.nets.insert(
            net_a,
            Net {
                uuid: net_a,
                name: "A".into(),
                class: class_uuid,
            },
        );
        board.nets.insert(
            net_b,
            Net {
                uuid: net_b,
                name: "B".into(),
                class: class_uuid,
            },
        );

        let track_a = Uuid::new_v4();
        let track_b = Uuid::new_v4();
        board.tracks.insert(
            track_a,
            Track {
                uuid: track_a,
                net: net_a,
                from: Point::new(0, 0),
                to: Point::new(10_000_000, 0),
                width: 200_000,
                layer: 1,
            },
        );
        board.tracks.insert(
            track_b,
            Track {
                uuid: track_b,
                net: net_b,
                from: Point::new(0, 100_000),
                to: Point::new(10_000_000, 100_000),
                width: 200_000,
                layer: 1,
            },
        );

        let report = run(&board, &[RuleType::ClearanceCopper]);
        assert!(!report.passed);
        assert_eq!(report.summary.errors, 1);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].code, "clearance_copper");
        let mut expected = vec![track_a, track_b];
        expected.sort();
        assert_eq!(report.violations[0].objects, expected);
    }

    #[test]
    fn track_width_check_reports_below_minimum_width() {
        let mut board = empty_board();
        let class_uuid = Uuid::new_v4();
        let net_uuid = Uuid::new_v4();
        board.net_classes.insert(
            class_uuid,
            NetClass {
                uuid: class_uuid,
                name: "default".into(),
                clearance: 100_000,
                track_width: 200_000,
                via_drill: 300_000,
                via_diameter: 600_000,
                diffpair_width: 0,
                diffpair_gap: 0,
            },
        );
        board.nets.insert(
            net_uuid,
            Net {
                uuid: net_uuid,
                name: "SIG".into(),
                class: class_uuid,
            },
        );
        let track_uuid = Uuid::new_v4();
        board.tracks.insert(
            track_uuid,
            Track {
                uuid: track_uuid,
                net: net_uuid,
                from: Point::new(0, 0),
                to: Point::new(10_000_000, 0),
                width: 100_000,
                layer: 1,
            },
        );

        let report = run(&board, &[RuleType::TrackWidth]);
        assert!(!report.passed);
        assert_eq!(report.summary.errors, 1);
        assert_eq!(report.violations[0].code, "track_width_below_min");
        assert_eq!(report.violations[0].objects, vec![track_uuid]);
    }

    #[test]
    fn via_checks_report_small_hole_and_annular_ring() {
        let mut board = empty_board();
        let class_uuid = Uuid::new_v4();
        let net_uuid = Uuid::new_v4();
        board.net_classes.insert(
            class_uuid,
            NetClass {
                uuid: class_uuid,
                name: "default".into(),
                clearance: 100_000,
                track_width: 200_000,
                via_drill: 200_000,
                via_diameter: 500_000,
                diffpair_width: 0,
                diffpair_gap: 0,
            },
        );
        board.nets.insert(
            net_uuid,
            Net {
                uuid: net_uuid,
                name: "SIG".into(),
                class: class_uuid,
            },
        );
        let via_uuid = Uuid::new_v4();
        board.vias.insert(
            via_uuid,
            Via {
                uuid: via_uuid,
                net: net_uuid,
                position: Point::new(1_000_000, 2_000_000),
                drill: 100_000,
                diameter: 200_000,
                from_layer: 1,
                to_layer: 2,
            },
        );

        let hole_report = run(&board, &[RuleType::ViaHole]);
        assert!(!hole_report.passed);
        assert_eq!(hole_report.summary.errors, 1);
        assert_eq!(hole_report.violations[0].code, "via_hole_out_of_range");

        let annular_report = run(&board, &[RuleType::ViaAnnularRing]);
        assert!(!annular_report.passed);
        assert_eq!(annular_report.summary.errors, 1);
        assert_eq!(annular_report.violations[0].code, "via_annular_below_min");
    }

    #[test]
    fn connectivity_reports_single_pin_unconnected_pin_violation() {
        let mut board = empty_board();
        let class_uuid = Uuid::new_v4();
        let net_uuid = Uuid::new_v4();
        board.net_classes.insert(
            class_uuid,
            NetClass {
                uuid: class_uuid,
                name: "default".into(),
                clearance: 100_000,
                track_width: 200_000,
                via_drill: 300_000,
                via_diameter: 600_000,
                diffpair_width: 0,
                diffpair_gap: 0,
            },
        );
        board.nets.insert(
            net_uuid,
            Net {
                uuid: net_uuid,
                name: "SIG".into(),
                class: class_uuid,
            },
        );
        let pkg = Uuid::new_v4();
        board.packages.insert(
            pkg,
            PlacedPackage {
                uuid: pkg,
                part: Uuid::new_v4(),
                package: Uuid::nil(),
                reference: "TP1".into(),
                value: "TP".into(),
                position: Point::new(10_000_000, 10_000_000),
                rotation: 0,
                layer: 1,
                locked: false,
            },
        );
        board.pads.insert(
            Uuid::new_v4(),
            crate::board::PlacedPad {
                uuid: Uuid::new_v4(),
                package: pkg,
                name: "1".into(),
                net: Some(net_uuid),
                position: Point::new(10_000_000, 10_000_000),
                layer: 1,
            },
        );

        let report = run(&board, &[RuleType::Connectivity]);
        assert!(!report.passed);
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.code == "connectivity_unconnected_pin")
        );
    }

    #[test]
    fn silk_clearance_reports_text_too_close_to_track() {
        let mut board = empty_board();
        let net_uuid = Uuid::new_v4();
        let class_uuid = Uuid::new_v4();
        board.net_classes.insert(
            class_uuid,
            NetClass {
                uuid: class_uuid,
                name: "default".into(),
                clearance: 100_000,
                track_width: 200_000,
                via_drill: 300_000,
                via_diameter: 600_000,
                diffpair_width: 0,
                diffpair_gap: 0,
            },
        );
        board.nets.insert(
            net_uuid,
            Net {
                uuid: net_uuid,
                name: "SIG".into(),
                class: class_uuid,
            },
        );

        let text_uuid = Uuid::new_v4();
        board.texts.push(crate::board::BoardText {
            uuid: text_uuid,
            text: "REF".into(),
            position: Point::new(10_000_000, 10_000_000),
            rotation: 0,
            layer: 37,
        });
        let track_uuid = Uuid::new_v4();
        board.tracks.insert(
            track_uuid,
            Track {
                uuid: track_uuid,
                net: net_uuid,
                from: Point::new(9_800_000, 10_000_000),
                to: Point::new(10_200_000, 10_000_000),
                width: 100_000,
                layer: 0,
            },
        );

        let report = run(&board, &[RuleType::SilkClearance]);
        assert!(!report.passed);
        assert_eq!(report.summary.errors, 1);
        assert_eq!(report.violations[0].code, "silk_clearance_copper");
        let mut expected = vec![text_uuid, track_uuid];
        expected.sort();
        assert_eq!(report.violations[0].objects, expected);
    }
}
