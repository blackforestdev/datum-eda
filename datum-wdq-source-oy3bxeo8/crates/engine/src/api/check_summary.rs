use super::*;

pub(super) fn summarize_diagnostics(diagnostics: &[ConnectivityDiagnosticInfo]) -> CheckSummary {
    let mut summary = CheckSummary {
        status: CheckStatus::Ok,
        errors: 0,
        warnings: 0,
        infos: 0,
        waived: 0,
        by_code: summarize_diagnostic_codes(diagnostics),
    };

    for diagnostic in diagnostics {
        match diagnostic.severity.as_str() {
            "error" => summary.errors += 1,
            "warning" => summary.warnings += 1,
            _ => summary.infos += 1,
        }
    }

    summary.status = derive_status(summary.errors, summary.warnings, summary.infos);
    summary
}

pub(super) fn summarize_schematic_checks(
    diagnostics: &[ConnectivityDiagnosticInfo],
    erc_findings: &[ErcFinding],
) -> CheckSummary {
    let mut summary = summarize_diagnostics(diagnostics);

    for finding in erc_findings {
        if finding.waived {
            summary.waived += 1;
            continue;
        }
        match finding.severity {
            erc::ErcSeverity::Error => summary.errors += 1,
            erc::ErcSeverity::Warning => summary.warnings += 1,
            erc::ErcSeverity::Info => summary.infos += 1,
        }
    }

    for (code, count) in summarize_erc_codes(erc_findings) {
        if let Some(existing) = summary.by_code.iter_mut().find(|entry| entry.code == code) {
            existing.count += count;
        } else {
            summary.by_code.push(CheckCodeCount { code, count });
        }
    }
    summary.by_code.sort_by(|a, b| a.code.cmp(&b.code));

    summary.status = derive_status(summary.errors, summary.warnings, summary.infos);
    summary
}

fn derive_status(errors: usize, warnings: usize, infos: usize) -> CheckStatus {
    if errors > 0 {
        CheckStatus::Error
    } else if warnings > 0 {
        CheckStatus::Warning
    } else if infos > 0 {
        CheckStatus::Info
    } else {
        CheckStatus::Ok
    }
}

fn summarize_diagnostic_codes(diagnostics: &[ConnectivityDiagnosticInfo]) -> Vec<CheckCodeCount> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for diagnostic in diagnostics {
        *counts.entry(diagnostic.kind.clone()).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(code, count)| CheckCodeCount { code, count })
        .collect()
}

fn summarize_erc_codes(findings: &[ErcFinding]) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for finding in findings {
        *counts.entry(finding.code.to_string()).or_default() += 1;
    }
    counts.into_iter().collect()
}

pub(super) fn erc_suggestion(code: &str) -> &'static str {
    match code {
        "output_to_output_conflict" => {
            "Ensure only one active output drives the net or add isolation between outputs."
        }
        "undriven_input_pin" => {
            "Connect the input to a valid driver or mark it intentionally unused."
        }
        "input_without_explicit_driver" => {
            "If intentional analog biasing is present, keep the passive network documented or add an explicit driver."
        }
        "power_in_without_source" => {
            "Add a valid power source pin on this net or connect to a driven power rail."
        }
        "noconnect_connected" => "Remove the no-connect marker or disconnect the pin from the net.",
        "unconnected_component_pin" => {
            "Wire the pin to a valid net or add a no-connect marker if intentionally left open."
        }
        "unconnected_interface_port" => {
            "Connect the hierarchical port to a net or remove the unused interface port."
        }
        "undriven_power_net" | "undriven_named_net" => {
            "Add a driving source or connect this net to its intended source rail."
        }
        _ => "Review net intent and either fix connectivity or apply a justified waiver.",
    }
}

pub(super) fn drc_suggestion(code: &str) -> &'static str {
    match code {
        "connectivity_no_copper" | "connectivity_unrouted_net" => {
            "Route the remaining airwires so all required pins on the net are electrically connected."
        }
        "connectivity_unconnected_pin" => {
            "Route copper from this pin to the intended net or remove the unintended net assignment."
        }
        "clearance_copper" => {
            "Increase spacing between copper features or relax the applicable clearance rule."
        }
        "track_width_below_min" => {
            "Increase track width to meet the assigned net class or adjust rule constraints."
        }
        "via_hole_out_of_range" | "via_annular_below_min" => {
            "Use a via size compliant with the current drill and annular-ring rule limits."
        }
        "silk_clearance_copper" => {
            "Move or resize silkscreen text to satisfy copper clearance requirements."
        }
        _ => "Adjust geometry or rules so the reported objects satisfy constraint checks.",
    }
}
