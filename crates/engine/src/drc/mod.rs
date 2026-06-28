use crate::board::Board;
use crate::rules::ast::RuleType;
use crate::schematic::{CheckDomain, CheckWaiver, WaiverTarget};

mod checks;
mod fingerprint;
mod types;
mod zone_fill_projection;
use fingerprint::attach_drc_violation_fingerprints;
pub(crate) use fingerprint::drc_violation_fingerprint;
pub use types::{DrcLocation, DrcReport, DrcSeverity, DrcSummary, DrcViolation};
pub use zone_fill_projection::{run_with_zone_fills, run_with_zone_fills_and_waivers};

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
    if run_all || selected_rules.contains(&RuleType::ProcessAperture) {
        violations.extend(checks::run_process_aperture_checks(board));
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
    attach_drc_violation_fingerprints(&mut violations);
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
        WaiverTarget::Fingerprint(fingerprint) => {
            fingerprint == &drc_violation_fingerprint(violation)
        }
    }
}

#[cfg(test)]
mod tests {
    #[path = "support.rs"]
    mod support;
    use support::empty_board;

    #[path = "mod_tests_connectivity_and_clearance.rs"]
    mod connectivity_and_clearance;

    #[path = "mod_tests_dimensions_and_silk.rs"]
    mod dimensions_and_silk;

    #[path = "mod_tests_waivers.rs"]
    mod waivers;

    #[path = "mod_tests_zone_fill_projection.rs"]
    mod zone_fill_projection;
}
