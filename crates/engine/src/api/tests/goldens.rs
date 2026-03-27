use super::*;
use crate::drc::DrcReport;
use crate::erc::ErcFinding;
use crate::ir::serialization::to_json_deterministic;
use std::fs;
use std::path::PathBuf;

#[test]
fn erc_golden_simple_demo_matches_checked_in_fixture() {
    assert_erc_matches_golden("simple-demo.kicad_sch");
}

#[test]
fn erc_golden_analog_input_demo_matches_checked_in_fixture() {
    assert_erc_matches_golden("analog-input-demo.kicad_sch");
}

#[test]
fn erc_golden_analog_input_bias_demo_matches_checked_in_fixture() {
    assert_erc_matches_golden("analog-input-bias-demo.kicad_sch");
}

#[test]
fn erc_golden_coverage_demo_matches_checked_in_fixture() {
    assert_erc_matches_golden("erc-coverage-demo.kicad_sch");
}

#[test]
fn erc_golden_hierarchy_mismatch_demo_matches_checked_in_fixture() {
    assert_erc_matches_golden("hierarchy-mismatch-demo.kicad_sch");
}

#[test]
fn drc_golden_simple_demo_matches_checked_in_fixture() {
    assert_drc_matches_golden("simple-demo.kicad_pcb");
}

#[test]
fn drc_golden_partial_route_demo_matches_checked_in_fixture() {
    assert_drc_matches_golden("partial-route-demo.kicad_pcb");
}

#[test]
fn drc_golden_clearance_violation_demo_matches_checked_in_fixture() {
    assert_drc_matches_golden("clearance-violation-demo.kicad_pcb");
}

#[test]
fn drc_golden_coverage_demo_matches_checked_in_fixture() {
    assert_drc_matches_golden("drc-coverage-demo.kicad_pcb");
}

#[test]
fn drc_golden_silk_clearance_demo_matches_checked_in_fixture() {
    assert_drc_matches_golden("silk-clearance-demo.kicad_pcb");
}

#[test]
fn drc_golden_airwire_demo_matches_checked_in_fixture() {
    assert_drc_matches_golden("airwire-demo.kicad_pcb");
}

#[test]
fn erc_golden_corpus_covers_required_m2_codes_for_current_implementation_slice() {
    let fixtures = [
        "simple-demo.kicad_sch",
        "analog-input-demo.kicad_sch",
        "erc-coverage-demo.kicad_sch",
        "hierarchy-mismatch-demo.kicad_sch",
    ];
    let mut seen = std::collections::BTreeSet::<String>::new();
    for fixture in fixtures {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path(fixture))
            .unwrap_or_else(|err| panic!("fixture should import: {err}"));
        for finding in engine
            .run_erc_prechecks()
            .unwrap_or_else(|err| panic!("ERC should run: {err}"))
        {
            seen.insert(finding.code.to_string());
        }
    }
    for required in [
        "output_to_output_conflict",
        "undriven_input_pin",
        "input_without_explicit_driver",
        "power_in_without_source",
        "unconnected_component_pin",
        "undriven_power_net",
        "noconnect_connected",
        "hierarchical_connectivity_mismatch",
    ] {
        assert!(
            seen.contains(required),
            "ERC golden corpus missing required code: {required}"
        );
    }
}

fn assert_erc_matches_golden(fixture: &str) {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path(fixture))
        .unwrap_or_else(|err| panic!("fixture should import: {err}"));
    let findings = engine
        .run_erc_prechecks()
        .unwrap_or_else(|err| panic!("ERC should run: {err}"));
    let normalized = normalize_erc_findings(&findings);
    let actual = to_json_deterministic(&normalized)
        .unwrap_or_else(|err| panic!("failed to serialize ERC findings: {err}"));

    let golden = golden_path_for_erc_fixture(fixture);
    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        if let Some(parent) = golden.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|err| {
                panic!(
                    "failed to create golden directory {}: {err}",
                    parent.display()
                )
            });
        }
        fs::write(&golden, &actual)
            .unwrap_or_else(|err| panic!("failed to write golden {}: {err}", golden.display()));
        return;
    }

    let expected = fs::read_to_string(&golden).unwrap_or_else(|err| {
        panic!(
            "failed to read golden {} (set UPDATE_GOLDENS=1 to regenerate): {err}",
            golden.display()
        )
    });
    assert_eq!(
        actual, expected,
        "ERC golden mismatch for fixture {}",
        fixture
    );
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct ErcGoldenFinding {
    code: String,
    severity: String,
    message: String,
    net_name: Option<String>,
    component: Option<String>,
    pin: Option<String>,
    objects: Vec<(String, String)>,
    waived: bool,
}

fn normalize_erc_findings(findings: &[ErcFinding]) -> Vec<ErcGoldenFinding> {
    let mut normalized: Vec<_> = findings
        .iter()
        .map(|finding| ErcGoldenFinding {
            code: finding.code.to_string(),
            severity: format!("{:?}", finding.severity),
            message: finding.message.clone(),
            net_name: finding.net_name.as_ref().map(|net| {
                if net.starts_with("N$") {
                    "N$<anon>".to_string()
                } else {
                    net.clone()
                }
            }),
            component: finding.component.clone(),
            pin: finding.pin.clone(),
            objects: finding
                .objects
                .iter()
                .map(|obj| (obj.kind.to_string(), obj.key.clone()))
                .collect(),
            waived: finding.waived,
        })
        .collect();
    normalized.sort_by(|a, b| {
        a.code
            .cmp(&b.code)
            .then_with(|| a.net_name.cmp(&b.net_name))
            .then_with(|| a.component.cmp(&b.component))
            .then_with(|| a.pin.cmp(&b.pin))
            .then_with(|| a.message.cmp(&b.message))
    });
    normalized
}

fn assert_drc_matches_golden(fixture: &str) {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path(fixture))
        .unwrap_or_else(|err| panic!("fixture should import: {err}"));
    let report = engine
        .run_drc(&[
            RuleType::Connectivity,
            RuleType::ClearanceCopper,
            RuleType::TrackWidth,
            RuleType::ViaHole,
            RuleType::ViaAnnularRing,
            RuleType::SilkClearance,
        ])
        .unwrap_or_else(|err| panic!("DRC should run: {err}"));
    let normalized = normalize_drc_report(&report);
    let actual = to_json_deterministic(&normalized)
        .unwrap_or_else(|err| panic!("failed to serialize DRC report: {err}"));

    let golden = golden_path_for_drc_fixture(fixture);
    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        if let Some(parent) = golden.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|err| {
                panic!(
                    "failed to create golden directory {}: {err}",
                    parent.display()
                )
            });
        }
        fs::write(&golden, &actual)
            .unwrap_or_else(|err| panic!("failed to write golden {}: {err}", golden.display()));
        return;
    }

    let expected = fs::read_to_string(&golden).unwrap_or_else(|err| {
        panic!(
            "failed to read golden {} (set UPDATE_GOLDENS=1 to regenerate): {err}",
            golden.display()
        )
    });
    assert_eq!(
        actual, expected,
        "DRC golden mismatch for fixture {}",
        fixture
    );
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct DrcGoldenReport {
    passed: bool,
    summary: (usize, usize),
    violations: Vec<DrcGoldenViolation>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct DrcGoldenViolation {
    code: String,
    rule_type: String,
    severity: String,
    message: String,
    location: Option<(i64, i64, Option<i32>)>,
    objects: Vec<String>,
}

fn normalize_drc_report(report: &DrcReport) -> DrcGoldenReport {
    let mut violations: Vec<_> = report
        .violations
        .iter()
        .map(|violation| DrcGoldenViolation {
            code: violation.code.clone(),
            rule_type: format!("{:?}", violation.rule_type),
            severity: match violation.severity {
                crate::drc::DrcSeverity::Error => "error".to_string(),
                crate::drc::DrcSeverity::Warning => "warning".to_string(),
            },
            message: violation.message.clone(),
            location: violation
                .location
                .as_ref()
                .map(|loc| (loc.x_nm, loc.y_nm, loc.layer)),
            objects: violation
                .objects
                .iter()
                .map(|uuid| uuid.to_string())
                .collect(),
        })
        .collect();
    violations.sort_by(|a, b| {
        a.code
            .cmp(&b.code)
            .then_with(|| a.message.cmp(&b.message))
            .then_with(|| a.objects.cmp(&b.objects))
    });
    DrcGoldenReport {
        passed: report.passed,
        summary: (report.summary.errors, report.summary.warnings),
        violations,
    }
}

fn golden_path_for_erc_fixture(fixture: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/golden/erc")
        .join(format!("{fixture}.json"))
}

fn golden_path_for_drc_fixture(fixture: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/golden/drc")
        .join(format!("{fixture}.json"))
}
