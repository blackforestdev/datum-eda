use super::*;

pub(super) fn run_erc(path: &Path) -> Result<Vec<ErcFinding>> {
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

pub(super) fn run_drc(path: &Path) -> Result<DrcReport> {
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

pub(super) fn run_check(path: &Path) -> Result<CheckReport> {
    let mut engine = Engine::new().context("failed to initialize engine")?;
    engine
        .import(path)
        .with_context(|| format!("failed to import design {}", path.display()))?;
    engine
        .get_check_report()
        .with_context(|| format!("failed to build check report for {}", path.display()))
}

pub(super) fn check_exit_code(report: &CheckReport, fail_on: Option<FailOn>) -> i32 {
    let status = match report {
        CheckReport::Board { summary, .. } => summary.status,
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

pub(super) fn render_check_report_text(report: &CheckReport) -> String {
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
        CheckReport::Schematic {
            summary,
            diagnostics,
            erc,
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

pub(super) fn render_drc_report_text(report: &DrcReport) -> String {
    let mut lines = vec![format!(
        "drc: passed={} errors={} warnings={}",
        report.passed, report.summary.errors, report.summary.warnings
    )];
    if !report.violations.is_empty() {
        lines.push("violations:".into());
        for violation in &report.violations {
            let location = violation
                .location
                .as_ref()
                .map(|loc| format!(" @({}, {}) L{:?}", loc.x_nm, loc.y_nm, loc.layer))
                .unwrap_or_default();
            lines.push(format!(
                "  [{}] {}: {}{}",
                render_drc_severity(violation.severity),
                violation.code,
                violation.message,
                location
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

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(super) enum SummaryView {
    Board {
        name: String,
        layers: usize,
        components: usize,
        nets: usize,
    },
    Schematic {
        sheets: usize,
        symbols: usize,
        labels: usize,
        ports: usize,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(super) enum NetListView {
    Board { nets: Vec<BoardNetInfo> },
    Schematic { nets: Vec<SchematicNetInfo> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(super) enum ComponentListView {
    Board { components: Vec<ComponentInfo> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(super) enum LabelListView {
    Schematic { labels: Vec<LabelInfo> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(super) enum PortListView {
    Schematic { ports: Vec<PortInfo> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(super) enum HierarchyView {
    Schematic { hierarchy: HierarchyInfo },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(super) enum DiagnosticsView {
    Board {
        diagnostics: Vec<ConnectivityDiagnosticInfo>,
    },
    Schematic {
        diagnostics: Vec<ConnectivityDiagnosticInfo>,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(super) enum UnroutedView {
    Board { airwires: Vec<Airwire> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub(super) enum DesignRuleListView {
    Board { rules: Vec<Rule> },
}

pub(super) fn import_design_for_query(path: &Path) -> Result<Engine> {
    import_design_for_query_with_libraries(path, &[])
}

fn import_design_for_query_with_libraries(path: &Path, libraries: &[PathBuf]) -> Result<Engine> {
    let mut engine = Engine::new().context("failed to initialize engine")?;
    for library in libraries {
        engine
            .import_eagle_library(library)
            .with_context(|| format!("failed to import library {}", library.display()))?;
    }
    engine
        .import(path)
        .with_context(|| format!("failed to import design {}", path.display()))?;
    Ok(engine)
}

pub(super) fn query_summary(path: &Path) -> Result<SummaryView> {
    let engine = import_design_for_query(path)?;
    match engine.get_board_summary() {
        Ok(summary) => Ok(SummaryView::Board {
            name: summary.name,
            layers: summary.layer_count,
            components: summary.component_count,
            nets: summary.net_count,
        }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => {
            let summary = engine.get_schematic_summary()?;
            Ok(SummaryView::Schematic {
                sheets: summary.sheet_count,
                symbols: summary.symbol_count,
                labels: summary.net_label_count,
                ports: summary.port_count,
            })
        }
        Err(err) => Err(err.into()),
    }
}

pub(super) fn query_nets(path: &Path) -> Result<NetListView> {
    let engine = import_design_for_query(path)?;
    match engine.get_net_info() {
        Ok(nets) => Ok(NetListView::Board { nets }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => Ok(NetListView::Schematic {
            nets: engine.get_schematic_net_info()?,
        }),
        Err(err) => Err(err.into()),
    }
}

pub(super) fn query_components(path: &Path) -> Result<ComponentListView> {
    let engine = import_design_for_query(path)?;
    match engine.get_components() {
        Ok(components) => Ok(ComponentListView::Board { components }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "component query is currently only implemented for boards in M1: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

fn require_schematic(engine: &Engine, path: &Path) -> Result<()> {
    match engine.get_schematic_summary() {
        Ok(_) => Ok(()),
        Err(EngineError::NotFound {
            object_type: "schematic",
            ..
        }) => bail!(
            "query is currently only implemented for schematics for this subcommand in M1: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

pub(super) fn query_labels(path: &Path) -> Result<LabelListView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(LabelListView::Schematic {
        labels: engine.get_labels(None)?,
    })
}

pub(super) fn query_ports(path: &Path) -> Result<PortListView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(PortListView::Schematic {
        ports: engine.get_ports(None)?,
    })
}

pub(super) fn query_hierarchy(path: &Path) -> Result<HierarchyView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(HierarchyView::Schematic {
        hierarchy: engine.get_hierarchy()?,
    })
}

pub(super) fn query_diagnostics(path: &Path) -> Result<DiagnosticsView> {
    let engine = import_design_for_query(path)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(DiagnosticsView::Board {
            diagnostics: engine.get_connectivity_diagnostics()?,
        }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => {
            require_schematic(&engine, path)?;
            Ok(DiagnosticsView::Schematic {
                diagnostics: engine.get_connectivity_diagnostics()?,
            })
        }
        Err(err) => Err(err.into()),
    }
}

pub(super) fn query_unrouted(path: &Path) -> Result<UnroutedView> {
    let engine = import_design_for_query(path)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(UnroutedView::Board {
            airwires: engine.get_unrouted()?,
        }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query unrouted is currently only implemented for boards in M1: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

pub(super) fn query_design_rules(path: &Path) -> Result<DesignRuleListView> {
    let engine = import_design_for_query(path)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(DesignRuleListView::Board {
            rules: engine.get_design_rules()?,
        }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query design-rules is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

pub(super) fn query_package_change_candidates(
    path: &Path,
    uuid: &Uuid,
    libraries: &[PathBuf],
) -> Result<PackageChangeCompatibilityReport> {
    let engine = import_design_for_query_with_libraries(path, libraries)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(engine.get_package_change_candidates(uuid)?),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query package-change-candidates is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

pub(super) fn query_part_change_candidates(
    path: &Path,
    uuid: &Uuid,
    libraries: &[PathBuf],
) -> Result<PartChangeCompatibilityReport> {
    let engine = import_design_for_query_with_libraries(path, libraries)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(engine.get_part_change_candidates(uuid)?),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query part-change-candidates is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

pub(super) fn query_component_replacement_plan(
    path: &Path,
    uuid: &Uuid,
    libraries: &[PathBuf],
) -> Result<ComponentReplacementPlan> {
    let engine = import_design_for_query_with_libraries(path, libraries)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(engine.get_component_replacement_plan(uuid)?),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query component-replacement-plan is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

pub(super) fn query_scoped_component_replacement_plan(
    path: &Path,
    input: ScopedComponentReplacementPolicyInput,
    edit: ScopedComponentReplacementPlanEdit,
    libraries: &[PathBuf],
) -> Result<ScopedComponentReplacementPlan> {
    let engine = import_design_for_query_with_libraries(path, libraries)?;
    match engine.get_board_summary() {
        Ok(_) => {
            let plan = engine.get_scoped_component_replacement_plan(input)?;
            if edit.exclude_component_uuids.is_empty() && edit.overrides.is_empty() {
                Ok(plan)
            } else {
                Ok(engine.edit_scoped_component_replacement_plan(plan, edit)?)
            }
        }
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query scoped-replacement-plan is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

pub(super) fn render_output<T: Serialize>(format: &OutputFormat, value: &T) -> String {
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
