use super::*;

pub(super) fn execute_native_project_query_command(
    format: &OutputFormat,
    path: PathBuf,
    what: NativeProjectQueryCommands,
) -> Result<(String, i32)> {
    match what {
        NativeProjectQueryCommands::Summary => {
            let report = query_native_project_summary(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_project_summary_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectQueryCommands::Pools => {
            let report = query_native_project_pools(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::DesignRules => {
            let report = query_native_project_rules(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_project_rules_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectQueryCommands::ForwardAnnotationAudit => {
            let report = query_native_project_forward_annotation_audit(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_forward_annotation_audit_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectQueryCommands::ForwardAnnotationProposal => {
            let report = query_native_project_forward_annotation_proposal(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_forward_annotation_proposal_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectQueryCommands::ForwardAnnotationReview => {
            let report = query_native_project_forward_annotation_review(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_forward_annotation_review_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectQueryCommands::Symbols => {
            let report = query_native_project_symbols(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::SymbolFields { symbol } => {
            let report = query_native_project_symbol_fields(&path, symbol)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::SymbolSemantics { symbol } => {
            let report = query_native_project_symbol_semantics(&path, symbol)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::SymbolPins { symbol } => {
            let report = query_native_project_symbol_pins(&path, symbol)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::Texts => {
            let report = query_native_project_texts(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::Drawings => {
            let report = query_native_project_drawings(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::Labels => {
            let report = query_native_project_labels(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::Wires => {
            let report = query_native_project_wires(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::Junctions => {
            let report = query_native_project_junctions(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::Ports => {
            let report = query_native_project_ports(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::Buses => {
            let report = query_native_project_buses(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BusEntries => {
            let report = query_native_project_bus_entries(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::Noconnects => {
            let report = query_native_project_noconnects(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::Nets => {
            let report = query_native_project_nets(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::Diagnostics => {
            let report = query_native_project_diagnostics(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::Erc => {
            let report = query_native_project_erc(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::Check => {
            let report = query_native_project_check(&path)?;
            let output = match format {
                OutputFormat::Text => render_check_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectQueryCommands::BoardTexts => {
            let report = query_native_project_board_texts(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardKeepouts => {
            let report = query_native_project_board_keepouts(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardOutline => {
            let report = query_native_project_board_outline(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardStackup => {
            let report = query_native_project_board_stackup(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardComponents => {
            let report = query_native_project_board_component_views(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardComponent(args) => {
            let report = query_native_project_board_component_view(&path, args.component_uuid)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardComponentModels3d(args) => {
            let report = query_native_project_board_component_models_3d(&path, args.component_uuid)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardComponentPads(args) => {
            let report = query_native_project_board_component_pads(&path, args.component_uuid)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardComponentSilkscreen(args) => {
            let report = query_native_project_board_component_silkscreen(&path, args.component_uuid)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardComponentMechanical(args) => {
            let report = query_native_project_board_component_mechanical(&path, args.component_uuid)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardTracks => {
            let report = query_native_project_board_tracks(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardVias => {
            let report = query_native_project_board_vias(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardZones => {
            let report = query_native_project_board_zones(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardDiagnostics => {
            let report = query_native_project_board_diagnostics(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardUnrouted => {
            let report = query_native_project_board_unrouted(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardCheck => {
            let report = query_native_project_board_check(&path)?;
            let output = match format {
                OutputFormat::Text => render_check_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectQueryCommands::BoardPads => {
            let report = query_native_project_board_pads(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardNets => {
            let report = query_native_project_board_nets(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardNet { net } => {
            let report = query_native_project_board_net(&path, net)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardNetClasses => {
            let report = query_native_project_board_net_classes(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardNetClass { net_class } => {
            let report = query_native_project_board_net_class(&path, net_class)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardDimensions => {
            let report = query_native_project_board_dimensions(&path)?;
            Ok((render_output(format, &report), 0))
        }
    }
}
