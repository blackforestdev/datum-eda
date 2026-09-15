use super::*;

mod query_check;
mod query_ops;
mod query_views;

pub(crate) use query_check::{
    check_exit_code, render_check_report_text, render_drc_report_text, render_output, run_check,
    run_drc, run_erc,
};
pub(crate) use query_ops::{
    query_bus_entries, query_buses, query_component_replacement_plan, query_components,
    query_design_rules, query_diagnostics, query_hierarchy, query_labels, query_netlist,
    query_nets, query_noconnects, query_package_change_candidates, query_part_change_candidates,
    query_ports, query_schematic_nets, query_scoped_component_replacement_plan, query_sheets,
    query_summary, query_symbols, query_unrouted,
};
pub(crate) use query_views::{
    BusEntryListView, BusListView, ComponentListView, DesignRuleListView, DiagnosticsView,
    HierarchyView, LabelListView, NetListView, NetlistView, NoConnectListView, PortListView,
    SheetListView, SummaryView, SymbolListView, UnroutedView,
};

// Phase 5: the `query` sub-enum router and imported-design query drivers,
// absorbed from the dissolved command_exec_query.rs (QueryPathArgs is shared
// across variants, so this stays a family fn rather than per-variant run()
// impls).
pub(crate) fn execute_query_command(
    format: &OutputFormat,
    action: QueryCommands,
) -> Result<(String, i32)> {
    match action {
        QueryCommands::Summary(QueryPathArgs { path }) => {
            let report = query_native_project_summary(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_project_summary_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        QueryCommands::Relationships(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_relationships(&path)?),
            0,
        )),
        QueryCommands::Variants(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_variants(&path)?),
            0,
        )),
        QueryCommands::ImportMap(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_import_map(&path)?),
            0,
        )),
        QueryCommands::ComponentInstances(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_component_instances(&path)?),
            0,
        )),
        QueryCommands::Sheets(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_sheets(&path)?),
            0,
        )),
        QueryCommands::Symbols(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_symbols(&path)?),
            0,
        )),
        QueryCommands::Labels(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_labels(&path)?),
            0,
        )),
        QueryCommands::Ports(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_ports(&path)?),
            0,
        )),
        QueryCommands::Buses(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_buses(&path)?),
            0,
        )),
        QueryCommands::BusEntries(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_bus_entries(&path)?),
            0,
        )),
        QueryCommands::Noconnects(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_noconnects(&path)?),
            0,
        )),
        QueryCommands::Hierarchy(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_hierarchy(&path)?),
            0,
        )),
        QueryCommands::SchematicNets(QueryPathArgs { path }) => {
            Ok((render_output(format, &query_native_project_nets(&path)?), 0))
        }
        QueryCommands::ConnectivityDiagnostics(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_diagnostics(&path)?),
            0,
        )),
        QueryCommands::ZoneFills(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_zone_fills(&path)?),
            0,
        )),
        QueryCommands::PanelProjections(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_panel_projections(&path)?),
            0,
        )),
        QueryCommands::ManufacturingPlans(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_manufacturing_plans(&path)?),
            0,
        )),
        QueryCommands::OutputJobs(QueryPathArgs { path }) => Ok((
            render_output(format, &query_native_project_output_jobs(&path)?),
            0,
        )),
        QueryCommands::Imported { path, what } => execute_imported_query(format, path, what),
        QueryCommands::LegacyImported(args) => execute_legacy_imported_query(format, args),
    }
}

fn execute_legacy_imported_query(
    format: &OutputFormat,
    args: Vec<std::ffi::OsString>,
) -> Result<(String, i32)> {
    if args.len() < 2 {
        bail!("legacy query compatibility expects: query <path> <what>");
    }
    let path = PathBuf::from(&args[0]);
    let mut parser_args = vec![std::ffi::OsString::from("query")];
    parser_args.extend(args.into_iter().skip(1));
    let parsed = ImportedQueryCommandParser::try_parse_from(parser_args)?;
    execute_imported_query(format, path, parsed.what)
}

fn execute_imported_query(
    format: &OutputFormat,
    path: PathBuf,
    what: ImportedQueryCommands,
) -> Result<(String, i32)> {
    match what {
        ImportedQueryCommands::Summary => {
            let summary = query_summary(&path)?;
            Ok((render_output(format, &summary), 0))
        }
        ImportedQueryCommands::Netlist => {
            let netlist = query_netlist(&path)?;
            Ok((render_output(format, &netlist), 0))
        }
        ImportedQueryCommands::Nets => {
            let nets = query_nets(&path)?;
            Ok((render_output(format, &nets), 0))
        }
        ImportedQueryCommands::SchematicNets => {
            let nets = query_schematic_nets(&path)?;
            Ok((render_output(format, &nets), 0))
        }
        ImportedQueryCommands::Components => {
            let components = query_components(&path)?;
            Ok((render_output(format, &components), 0))
        }
        ImportedQueryCommands::Sheets => {
            let sheets = query_sheets(&path)?;
            Ok((render_output(format, &sheets), 0))
        }
        ImportedQueryCommands::Symbols => {
            let symbols = query_symbols(&path)?;
            Ok((render_output(format, &symbols), 0))
        }
        ImportedQueryCommands::Labels => {
            let labels = query_labels(&path)?;
            Ok((render_output(format, &labels), 0))
        }
        ImportedQueryCommands::Ports => {
            let ports = query_ports(&path)?;
            Ok((render_output(format, &ports), 0))
        }
        ImportedQueryCommands::Buses => {
            let buses = query_buses(&path)?;
            Ok((render_output(format, &buses), 0))
        }
        ImportedQueryCommands::BusEntries => {
            let entries = query_bus_entries(&path)?;
            Ok((render_output(format, &entries), 0))
        }
        ImportedQueryCommands::Noconnects => {
            let noconnects = query_noconnects(&path)?;
            Ok((render_output(format, &noconnects), 0))
        }
        ImportedQueryCommands::Hierarchy => {
            let hierarchy = query_hierarchy(&path)?;
            Ok((render_output(format, &hierarchy), 0))
        }
        ImportedQueryCommands::Diagnostics => {
            let diagnostics = query_diagnostics(&path)?;
            Ok((render_output(format, &diagnostics), 0))
        }
        ImportedQueryCommands::Unrouted => {
            let airwires = query_unrouted(&path)?;
            Ok((render_output(format, &airwires), 0))
        }
        ImportedQueryCommands::DesignRules => {
            let rules = query_design_rules(&path)?;
            Ok((render_output(format, &rules), 0))
        }
        ImportedQueryCommands::PackageChangeCandidates { uuid, libraries } => {
            let report = query_package_change_candidates(&path, &uuid, &libraries)?;
            Ok((render_output(format, &report), 0))
        }
        ImportedQueryCommands::PartChangeCandidates { uuid, libraries } => {
            let report = query_part_change_candidates(&path, &uuid, &libraries)?;
            Ok((render_output(format, &report), 0))
        }
        ImportedQueryCommands::ComponentReplacementPlan { uuid, libraries } => {
            let report = query_component_replacement_plan(&path, &uuid, &libraries)?;
            Ok((render_output(format, &report), 0))
        }
        ImportedQueryCommands::ScopedReplacementPlan {
            policy,
            ref_prefix,
            value,
            package_uuid,
            part_uuid,
            exclude_component,
            override_component,
            libraries,
        } => {
            let policy = match policy {
                ReplacementPolicyArg::Package => ComponentReplacementPolicy::BestCompatiblePackage,
                ReplacementPolicyArg::Part => ComponentReplacementPolicy::BestCompatiblePart,
            };
            let overrides = override_component
                .iter()
                .map(|value| parse_scoped_replacement_override_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let report = query_scoped_component_replacement_plan(
                &path,
                ScopedComponentReplacementPolicyInput {
                    scope: ComponentReplacementScope {
                        reference_prefix: ref_prefix,
                        value_equals: value,
                        current_package_uuid: package_uuid,
                        current_part_uuid: part_uuid,
                    },
                    policy,
                },
                ScopedComponentReplacementPlanEdit {
                    exclude_component_uuids: exclude_component,
                    overrides,
                },
                &libraries,
            )?;
            Ok((render_output(format, &report), 0))
        }
    }
}
