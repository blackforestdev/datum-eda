use super::*;

pub(crate) fn run_erc(path: &Path) -> Result<Vec<ErcFinding>> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("kicad_sch") {
        bail!(
            "erc currently only accepts KiCad .kicad_sch inputs in M1: {}",
            path.display()
        );
    }

    let mut engine = Engine::new().context("failed to initialize engine")?;
    engine
        .import(path)
        .with_context(|| format!("failed to import schematic {}", path.display()))?;
    engine
        .run_erc_prechecks()
        .with_context(|| format!("failed to run ERC on {}", path.display()))
}

pub(crate) fn run_drc(path: &Path) -> Result<DrcReport> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("kicad_pcb") {
        bail!(
            "drc currently only accepts KiCad .kicad_pcb inputs in M2 slice: {}",
            path.display()
        );
    }

    let mut engine = Engine::new().context("failed to initialize engine")?;
    engine
        .import(path)
        .with_context(|| format!("failed to import board {}", path.display()))?;
    engine
        .run_drc(&[
            RuleType::Connectivity,
            RuleType::ClearanceCopper,
            RuleType::TrackWidth,
            RuleType::ViaHole,
            RuleType::ViaAnnularRing,
            RuleType::SilkClearance,
        ])
        .with_context(|| format!("failed to run DRC on {}", path.display()))
}

pub(crate) fn run_check(path: &Path) -> Result<CheckReport> {
    let mut engine = Engine::new().context("failed to initialize engine")?;
    engine
        .import(path)
        .with_context(|| format!("failed to import design {}", path.display()))?;
    engine
        .get_check_report()
        .with_context(|| format!("failed to build check report for {}", path.display()))
}

pub(crate) fn check_exit_code(report: &CheckReport, fail_on: Option<FailOn>) -> i32 {
    let status = match report {
        CheckReport::Board { summary, .. } => summary.status,
        CheckReport::Combined { summary, .. } => summary.status,
        CheckReport::Schematic { summary, .. } => summary.status,
    };

    let threshold = fail_on.unwrap_or(FailOn::Error);
    if status_rank(status) >= fail_on_rank(threshold) {
        1
    } else {
        0
    }
}

fn status_rank(status: CheckStatus) -> u8 {
    match status {
        CheckStatus::Ok => 0,
        CheckStatus::Info => 1,
        CheckStatus::Warning => 2,
        CheckStatus::Error => 3,
    }
}

fn fail_on_rank(level: FailOn) -> u8 {
    match level {
        FailOn::Info => 1,
        FailOn::Warning => 2,
        FailOn::Error => 3,
    }
}

pub(crate) fn render_check_report_text(report: &CheckReport) -> String {
    match report {
        CheckReport::Board {
            summary,
            diagnostics,
        } => {
            let mut lines = vec![format!(
                "board check: status={} errors={} warnings={} infos={} waived={}",
                render_status(summary.status),
                summary.errors,
                summary.warnings,
                summary.infos,
                summary.waived
            )];
            if !summary.by_code.is_empty() {
                lines.push("counts:".into());
                for entry in &summary.by_code {
                    lines.push(format!("  {} x{}", entry.code, entry.count));
                }
            }
            if !diagnostics.is_empty() {
                lines.push("diagnostics:".into());
                for diagnostic in diagnostics {
                    lines.push(format!(
                        "  [{}] {}",
                        diagnostic.severity, diagnostic.message
                    ));
                }
            }
            lines.join("\n")
        }
        CheckReport::Combined {
            summary,
            diagnostics,
            erc,
            drc,
        } => {
            let mut lines = vec![format!(
                "combined check: status={} errors={} warnings={} infos={} waived={}",
                render_status(summary.status),
                summary.errors,
                summary.warnings,
                summary.infos,
                summary.waived
            )];
            if !summary.by_code.is_empty() {
                lines.push("counts:".into());
                for entry in &summary.by_code {
                    lines.push(format!("  {} x{}", entry.code, entry.count));
                }
            }
            if !diagnostics.is_empty() {
                lines.push("diagnostics:".into());
                for diagnostic in diagnostics {
                    lines.push(format!(
                        "  [{}] {}",
                        diagnostic.severity, diagnostic.message
                    ));
                }
            }
            if !erc.is_empty() {
                lines.push("erc:".into());
                for finding in erc {
                    let waived = if finding.waived { " (waived)" } else { "" };
                    lines.push(format!(
                        "  [{}] {}: {}{}",
                        render_erc_severity(&finding.severity),
                        finding.code,
                        finding.message,
                        waived
                    ));
                }
            }
            if !drc.is_empty() {
                lines.push("drc:".into());
                for violation in drc {
                    let location = violation
                        .location
                        .as_ref()
                        .map(|loc| format!(" @({}, {}) L{:?}", loc.x_nm, loc.y_nm, loc.layer))
                        .unwrap_or_default();
                    let waived = if violation.waived { " (waived)" } else { "" };
                    lines.push(format!(
                        "  [{}] {}: {}{}{}",
                        render_drc_severity(violation.severity),
                        violation.code,
                        violation.message,
                        location,
                        waived
                    ));
                }
            }
            lines.join("\n")
        }
        CheckReport::Schematic {
            summary,
            diagnostics,
            erc,
            drc,
        } => {
            let mut lines = vec![format!(
                "schematic check: status={} errors={} warnings={} infos={} waived={}",
                render_status(summary.status),
                summary.errors,
                summary.warnings,
                summary.infos,
                summary.waived
            )];
            if !summary.by_code.is_empty() {
                lines.push("counts:".into());
                for entry in &summary.by_code {
                    lines.push(format!("  {} x{}", entry.code, entry.count));
                }
            }
            if !diagnostics.is_empty() {
                lines.push("diagnostics:".into());
                for diagnostic in diagnostics {
                    lines.push(format!(
                        "  [{}] {}",
                        diagnostic.severity, diagnostic.message
                    ));
                }
            }
            if !erc.is_empty() {
                lines.push("erc:".into());
                for finding in erc {
                    let waived = if finding.waived { " (waived)" } else { "" };
                    lines.push(format!(
                        "  [{}] {}: {}{}",
                        render_erc_severity(&finding.severity),
                        finding.code,
                        finding.message,
                        waived
                    ));
                }
            }
            if !drc.is_empty() {
                lines.push("drc:".into());
                for violation in drc {
                    let location = violation
                        .location
                        .as_ref()
                        .map(|loc| format!(" @({}, {}) L{:?}", loc.x_nm, loc.y_nm, loc.layer))
                        .unwrap_or_default();
                    let waived = if violation.waived { " (waived)" } else { "" };
                    lines.push(format!(
                        "  [{}] {}: {}{}{}",
                        render_drc_severity(violation.severity),
                        violation.code,
                        violation.message,
                        location,
                        waived
                    ));
                }
            }
            lines.join("\n")
        }
    }
}

fn render_status(status: CheckStatus) -> &'static str {
    match status {
        CheckStatus::Ok => "ok",
        CheckStatus::Info => "info",
        CheckStatus::Warning => "warning",
        CheckStatus::Error => "error",
    }
}

fn render_erc_severity(severity: &eda_engine::erc::ErcSeverity) -> &'static str {
    match severity {
        eda_engine::erc::ErcSeverity::Error => "error",
        eda_engine::erc::ErcSeverity::Warning => "warning",
        eda_engine::erc::ErcSeverity::Info => "info",
    }
}

pub(crate) fn render_drc_report_text(report: &DrcReport) -> String {
    let mut lines = vec![format!(
        "drc: passed={} errors={} warnings={} waived={}",
        report.passed, report.summary.errors, report.summary.warnings, report.summary.waived
    )];
    if !report.violations.is_empty() {
        lines.push("violations:".into());
        for violation in &report.violations {
            let location = violation
                .location
                .as_ref()
                .map(|loc| format!(" @({}, {}) L{:?}", loc.x_nm, loc.y_nm, loc.layer))
                .unwrap_or_default();
            let waived = if violation.waived { " (waived)" } else { "" };
            lines.push(format!(
                "  [{}] {}: {}{}{}",
                render_drc_severity(violation.severity),
                violation.code,
                violation.message,
                location,
                waived
            ));
        }
    }
    lines.join("\n")
}

fn render_drc_severity(severity: eda_engine::drc::DrcSeverity) -> &'static str {
    match severity {
        eda_engine::drc::DrcSeverity::Error => "error",
        eda_engine::drc::DrcSeverity::Warning => "warning",
    }
}

pub(crate) fn render_output<T: Serialize>(format: &OutputFormat, value: &T) -> String {
    match format {
        OutputFormat::Text => render_text(value),
        OutputFormat::Json => {
            serde_json::to_string_pretty(value).expect("CLI JSON serialization must succeed")
        }
    }
}

fn render_text<T: Serialize>(value: &T) -> String {
    let json = serde_json::to_value(value).expect("CLI text formatting serialization must succeed");
    match json {
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(|item| {
                serde_json::to_string_pretty(&item)
                    .expect("CLI text formatting item serialization must succeed")
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => serde_json::to_string_pretty(value)
            .expect("CLI text formatting serialization must succeed"),
    }
}
