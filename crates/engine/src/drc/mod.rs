use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::Board;
use crate::ir::geometry::LayerId;
use crate::rules::ast::RuleType;
use crate::schematic::{CheckDomain, CheckWaiver, WaiverTarget};

mod checks;

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
    #[serde(default)]
    pub waived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrcSummary {
    pub errors: usize,
    pub warnings: usize,
    #[serde(default)]
    pub waived: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrcReport {
    pub passed: bool,
    pub violations: Vec<DrcViolation>,
    pub summary: DrcSummary,
}

pub fn run(board: &Board, selected_rules: &[RuleType]) -> DrcReport {
    run_with_waivers(board, selected_rules, &[])
}

pub fn run_with_waivers(
    board: &Board,
    selected_rules: &[RuleType],
    waivers: &[CheckWaiver],
) -> DrcReport {
    let run_all = selected_rules.is_empty();
    let mut violations = Vec::new();

    if run_all || selected_rules.contains(&RuleType::Connectivity) {
        violations.extend(checks::run_connectivity_checks(board));
    }
    if run_all || selected_rules.contains(&RuleType::ClearanceCopper) {
        violations.extend(checks::run_clearance_checks(board));
    }
    if run_all || selected_rules.contains(&RuleType::TrackWidth) {
        violations.extend(checks::run_track_width_checks(board));
    }
    if run_all || selected_rules.contains(&RuleType::ViaHole) {
        violations.extend(checks::run_via_hole_checks(board));
    }
    if run_all || selected_rules.contains(&RuleType::ViaAnnularRing) {
        violations.extend(checks::run_via_annular_checks(board));
    }
    if run_all || selected_rules.contains(&RuleType::SilkClearance) {
        violations.extend(checks::run_silk_clearance_checks(board));
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
        waived: 0,
    };
    apply_waivers(&mut violations, waivers);
    for violation in &violations {
        if violation.waived {
            summary.waived += 1;
            continue;
        }
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

fn apply_waivers(violations: &mut [DrcViolation], waivers: &[CheckWaiver]) {
    for violation in violations {
        violation.waived = waivers
            .iter()
            .any(|waiver| waiver_matches(waiver, violation));
    }
}

fn waiver_matches(waiver: &CheckWaiver, violation: &DrcViolation) -> bool {
    if !matches!(waiver.domain, CheckDomain::DRC) {
        return false;
    }

    match &waiver.target {
        WaiverTarget::Object(uuid) => violation.objects.contains(uuid),
        WaiverTarget::RuleObject { rule, object } => {
            *rule == violation.code && violation.objects.contains(object)
        }
        WaiverTarget::RuleObjects { rule, objects } => {
            if *rule != violation.code {
                return false;
            }
            let mut actual = violation.objects.clone();
            actual.sort();
            let mut expected = objects.clone();
            expected.sort();
            actual == expected
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::board::{
        Board, Keepout, PlacedPackage, Stackup, StackupLayer, StackupLayerType, Zone,
    };
    use crate::ir::geometry::{Point, Polygon};

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

    #[path = "mod_tests_connectivity_and_clearance.rs"]
    mod connectivity_and_clearance;

    #[path = "mod_tests_dimensions_and_silk.rs"]
    mod dimensions_and_silk;

    #[path = "mod_tests_waivers.rs"]
    mod waivers;
}
