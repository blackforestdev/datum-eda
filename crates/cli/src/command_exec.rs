use super::*;
use crate::command_modify::{
    parse_apply_replacement_plan_arg, parse_apply_replacement_policy_arg,
    parse_apply_scoped_replacement_policy_arg, parse_assign_part_arg, parse_move_component_arg,
    parse_replace_component_arg, parse_rotate_component_arg, parse_set_net_class_arg,
    parse_set_package_arg, parse_set_package_with_part_arg, parse_set_reference_arg,
    parse_set_value_arg,
};
use eda_engine::schematic::{LabelKind, PortDirection};

fn parse_native_symbol_display_mode(
    value: NativeSymbolDisplayModeArg,
) -> eda_engine::schematic::SymbolDisplayMode {
    match value {
        NativeSymbolDisplayModeArg::LibraryDefault => {
            eda_engine::schematic::SymbolDisplayMode::LibraryDefault
        }
        NativeSymbolDisplayModeArg::ShowHiddenPins => {
            eda_engine::schematic::SymbolDisplayMode::ShowHiddenPins
        }
        NativeSymbolDisplayModeArg::HideOptionalPins => {
            eda_engine::schematic::SymbolDisplayMode::HideOptionalPins
        }
    }
}

fn parse_native_hidden_power_behavior(
    value: NativeHiddenPowerBehaviorArg,
) -> eda_engine::schematic::HiddenPowerBehavior {
    match value {
        NativeHiddenPowerBehaviorArg::SourceDefinedImplicit => {
            eda_engine::schematic::HiddenPowerBehavior::SourceDefinedImplicit
        }
        NativeHiddenPowerBehaviorArg::ExplicitPowerObject => {
            eda_engine::schematic::HiddenPowerBehavior::ExplicitPowerObject
        }
        NativeHiddenPowerBehaviorArg::PreservedAsImportedMetadata => {
            eda_engine::schematic::HiddenPowerBehavior::PreservedAsImportedMetadata
        }
    }
}

pub(super) fn execute_with_exit_code(cli: Cli) -> Result<(String, i32)> {
    match cli.command {
        Commands::Import { path } => {
            let report = import_path(&path)?;
            let view = ImportReportView::from(report);
            Ok((render_output(&cli.format, &view), 0))
        }
        Commands::Query { path, what } => match what {
            QueryCommands::Summary => {
                let summary = query_summary(&path)?;
                Ok((render_output(&cli.format, &summary), 0))
            }
            QueryCommands::Nets => {
                let nets = query_nets(&path)?;
                Ok((render_output(&cli.format, &nets), 0))
            }
            QueryCommands::Components => {
                let components = query_components(&path)?;
                Ok((render_output(&cli.format, &components), 0))
            }
            QueryCommands::Labels => {
                let labels = query_labels(&path)?;
                Ok((render_output(&cli.format, &labels), 0))
            }
            QueryCommands::Ports => {
                let ports = query_ports(&path)?;
                Ok((render_output(&cli.format, &ports), 0))
            }
            QueryCommands::Hierarchy => {
                let hierarchy = query_hierarchy(&path)?;
                Ok((render_output(&cli.format, &hierarchy), 0))
            }
            QueryCommands::Diagnostics => {
                let diagnostics = query_diagnostics(&path)?;
                Ok((render_output(&cli.format, &diagnostics), 0))
            }
            QueryCommands::Unrouted => {
                let airwires = query_unrouted(&path)?;
                Ok((render_output(&cli.format, &airwires), 0))
            }
            QueryCommands::DesignRules => {
                let rules = query_design_rules(&path)?;
                Ok((render_output(&cli.format, &rules), 0))
            }
            QueryCommands::PackageChangeCandidates { uuid, libraries } => {
                let report = query_package_change_candidates(&path, &uuid, &libraries)?;
                Ok((render_output(&cli.format, &report), 0))
            }
            QueryCommands::PartChangeCandidates { uuid, libraries } => {
                let report = query_part_change_candidates(&path, &uuid, &libraries)?;
                Ok((render_output(&cli.format, &report), 0))
            }
            QueryCommands::ComponentReplacementPlan { uuid, libraries } => {
                let report = query_component_replacement_plan(&path, &uuid, &libraries)?;
                Ok((render_output(&cli.format, &report), 0))
            }
            QueryCommands::ScopedReplacementPlan {
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
                    ReplacementPolicyArg::Package => {
                        ComponentReplacementPolicy::BestCompatiblePackage
                    }
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
                Ok((render_output(&cli.format, &report), 0))
            }
        },
        Commands::Drc { path } => {
            let report = run_drc(Path::new(&path))?;
            let output = match cli.format {
                OutputFormat::Text => render_drc_report_text(&report),
                OutputFormat::Json => render_output(&cli.format, &report),
            };
            let exit_code = if report.passed { 0 } else { 1 };
            Ok((output, exit_code))
        }
        Commands::Erc { path } => {
            let findings = run_erc(&path)?;
            let exit_code = if findings.iter().any(|finding| !finding.waived) {
                1
            } else {
                0
            };
            Ok((render_output(&cli.format, &findings), exit_code))
        }
        Commands::Check { path, fail_on } => {
            let report = run_check(&path)?;
            let output = match cli.format {
                OutputFormat::Text => render_check_report_text(&report),
                OutputFormat::Json => render_output(&cli.format, &report),
            };
            Ok((output, check_exit_code(&report, fail_on)))
        }
        Commands::Pool { action } => match action {
            PoolCommands::Search { query, libraries } => {
                let results = search_pool(&query, &libraries)?;
                Ok((render_output(&cli.format, &results), 0))
            }
        },
        Commands::Project { action } => match *action {
            ProjectCommands::New { path, name } => {
                let report = create_native_project(&path, name)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_create_report_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::Inspect { path } => {
                let report = inspect_native_project(&path)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_inspect_report_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::Query { path, what } => match what {
                NativeProjectQueryCommands::Summary => {
                    let report = query_native_project_summary(&path)?;
                    let output = match cli.format {
                        OutputFormat::Text => render_native_project_summary_text(&report),
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                NativeProjectQueryCommands::DesignRules => {
                    let report = query_native_project_rules(&path)?;
                    let output = match cli.format {
                        OutputFormat::Text => render_native_project_rules_text(&report),
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                NativeProjectQueryCommands::ForwardAnnotationAudit => {
                    let report = query_native_project_forward_annotation_audit(&path)?;
                    let output = match cli.format {
                        OutputFormat::Text => render_native_forward_annotation_audit_text(&report),
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                NativeProjectQueryCommands::ForwardAnnotationProposal => {
                    let report = query_native_project_forward_annotation_proposal(&path)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_forward_annotation_proposal_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                NativeProjectQueryCommands::ForwardAnnotationReview => {
                    let report = query_native_project_forward_annotation_review(&path)?;
                    let output = match cli.format {
                        OutputFormat::Text => render_native_forward_annotation_review_text(&report),
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                NativeProjectQueryCommands::Symbols => {
                    let report = query_native_project_symbols(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::SymbolFields { symbol } => {
                    let report = query_native_project_symbol_fields(&path, symbol)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::SymbolSemantics { symbol } => {
                    let report = query_native_project_symbol_semantics(&path, symbol)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::SymbolPins { symbol } => {
                    let report = query_native_project_symbol_pins(&path, symbol)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::Texts => {
                    let report = query_native_project_texts(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::Drawings => {
                    let report = query_native_project_drawings(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::Labels => {
                    let report = query_native_project_labels(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::Wires => {
                    let report = query_native_project_wires(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::Junctions => {
                    let report = query_native_project_junctions(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::Ports => {
                    let report = query_native_project_ports(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::Buses => {
                    let report = query_native_project_buses(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BusEntries => {
                    let report = query_native_project_bus_entries(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::Noconnects => {
                    let report = query_native_project_noconnects(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::Nets => {
                    let report = query_native_project_nets(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::Diagnostics => {
                    let report = query_native_project_diagnostics(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::Erc => {
                    let report = query_native_project_erc(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::Check => {
                    let report = query_native_project_check(&path)?;
                    let output = match cli.format {
                        OutputFormat::Text => render_check_report_text(&report),
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                NativeProjectQueryCommands::BoardTexts => {
                    let report = query_native_project_board_texts(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BoardKeepouts => {
                    let report = query_native_project_board_keepouts(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BoardOutline => {
                    let report = query_native_project_board_outline(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BoardStackup => {
                    let report = query_native_project_board_stackup(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BoardComponents => {
                    let report = query_native_project_board_components(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BoardTracks => {
                    let report = query_native_project_board_tracks(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BoardVias => {
                    let report = query_native_project_board_vias(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BoardZones => {
                    let report = query_native_project_board_zones(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BoardDiagnostics => {
                    let report = query_native_project_board_diagnostics(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BoardUnrouted => {
                    let report = query_native_project_board_unrouted(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BoardCheck => {
                    let report = query_native_project_board_check(&path)?;
                    let output = match cli.format {
                        OutputFormat::Text => render_check_report_text(&report),
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                NativeProjectQueryCommands::BoardPads => {
                    let report = query_native_project_board_pads(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BoardNets => {
                    let report = query_native_project_board_nets(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BoardNetClasses => {
                    let report = query_native_project_board_net_classes(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
                NativeProjectQueryCommands::BoardDimensions => {
                    let report = query_native_project_board_dimensions(&path)?;
                    Ok((render_output(&cli.format, &report), 0))
                }
            },
            ProjectCommands::ExportBom { path, out } => {
                let report = export_native_project_bom(&path, &out)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_bom_export_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::CompareBom { path, bom } => {
                let report = compare_native_project_bom(&path, &bom)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_bom_comparison_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ExportPnp { path, out } => {
                let report = export_native_project_pnp(&path, &out)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_pnp_export_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ComparePnp { path, pnp } => {
                let report = compare_native_project_pnp(&path, &pnp)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_pnp_comparison_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ExportDrill { path, out } => {
                let report = export_native_project_drill(&path, &out)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_drill_export_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ExportExcellonDrill { path, out } => {
                let report = export_native_project_excellon_drill(&path, &out)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_excellon_drill_export_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::InspectExcellonDrill { path } => {
                let report = inspect_excellon_drill(&path)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_excellon_drill_inspection_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::CompareExcellonDrill { path, drill } => {
                let report = compare_native_project_excellon_drill(&path, &drill)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_excellon_drill_comparison_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ReportDrillHoleClasses { path } => {
                let report = report_native_project_drill_hole_classes(&path)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_drill_hole_class_report_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ExportGerberOutline { path, out } => {
                let report = export_native_project_gerber_outline(&path, &out)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_gerber_outline_export_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ExportGerberCopperLayer { path, layer, out } => {
                let report = export_native_project_gerber_copper_layer(&path, layer, &out)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_gerber_copper_export_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ValidateGerberOutline { path, gerber } => {
                let report = validate_native_project_gerber_outline(&path, &gerber)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_gerber_outline_validation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                let exit_code = if report.matches_expected { 0 } else { 1 };
                Ok((output, exit_code))
            }
            ProjectCommands::ValidateGerberCopperLayer {
                path,
                layer,
                gerber,
            } => {
                let report = validate_native_project_gerber_copper_layer(&path, layer, &gerber)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_gerber_copper_validation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                let exit_code = if report.matches_expected { 0 } else { 1 };
                Ok((output, exit_code))
            }
            ProjectCommands::CompareGerberOutline { path, gerber } => {
                let report = compare_native_project_gerber_outline(&path, &gerber)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_gerber_outline_comparison_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::CompareGerberCopperLayer {
                path,
                layer,
                gerber,
            } => {
                let report = compare_native_project_gerber_copper_layer(&path, layer, &gerber)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_gerber_copper_comparison_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ValidateExcellonDrill { path, drill } => {
                let report = validate_native_project_excellon_drill(&path, &drill)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_excellon_drill_validation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                let exit_code = if report.matches_expected { 0 } else { 1 };
                Ok((output, exit_code))
            }
            ProjectCommands::PlanGerberExport { path, prefix } => {
                let report = plan_native_project_gerber_export(&path, prefix.as_deref())?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_gerber_plan_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::CompareGerberExportPlan {
                path,
                output_dir,
                prefix,
            } => {
                let report = compare_native_project_gerber_export_plan(
                    &path,
                    &output_dir,
                    prefix.as_deref(),
                )?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_gerber_plan_comparison_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceSymbol {
                path,
                sheet,
                reference,
                value,
                lib_id,
                x_nm,
                y_nm,
                rotation_deg,
                mirrored,
            } => {
                let report = place_native_project_symbol(
                    &path,
                    sheet,
                    reference,
                    value,
                    lib_id,
                    eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                    rotation_deg,
                    mirrored,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::MoveSymbol {
                path,
                symbol,
                x_nm,
                y_nm,
            } => {
                let report = move_native_project_symbol(
                    &path,
                    symbol,
                    eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::RotateSymbol {
                path,
                symbol,
                rotation_deg,
            } => {
                let report = rotate_native_project_symbol(&path, symbol, rotation_deg)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::MirrorSymbol { path, symbol } => {
                let report = mirror_native_project_symbol(&path, symbol)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteSymbol { path, symbol } => {
                let report = delete_native_project_symbol(&path, symbol)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetSymbolReference {
                path,
                symbol,
                reference,
            } => {
                let report = set_native_project_symbol_reference(&path, symbol, reference)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetSymbolValue {
                path,
                symbol,
                value,
            } => {
                let report = set_native_project_symbol_value(&path, symbol, value)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetSymbolLibId {
                path,
                symbol,
                lib_id,
            } => {
                let report = set_native_project_symbol_lib_id(&path, symbol, lib_id)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ClearSymbolLibId { path, symbol } => {
                let report = clear_native_project_symbol_lib_id(&path, symbol)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetSymbolEntity {
                path,
                symbol,
                entity_uuid,
            } => {
                let report = set_native_project_symbol_entity(&path, symbol, entity_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ClearSymbolEntity { path, symbol } => {
                let report = clear_native_project_symbol_entity(&path, symbol)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetSymbolPart {
                path,
                symbol,
                part_uuid,
            } => {
                let report = set_native_project_symbol_part(&path, symbol, part_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ClearSymbolPart { path, symbol } => {
                let report = clear_native_project_symbol_part(&path, symbol)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetSymbolUnit {
                path,
                symbol,
                unit_selection,
            } => {
                let report = set_native_project_symbol_unit(&path, symbol, unit_selection)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ClearSymbolUnit { path, symbol } => {
                let report = clear_native_project_symbol_unit(&path, symbol)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetSymbolGate {
                path,
                symbol,
                gate_uuid,
            } => {
                let report = set_native_project_symbol_gate(&path, symbol, gate_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ClearSymbolGate { path, symbol } => {
                let report = clear_native_project_symbol_gate(&path, symbol)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetSymbolDisplayMode {
                path,
                symbol,
                display_mode,
            } => {
                let report = set_native_project_symbol_display_mode(
                    &path,
                    symbol,
                    parse_native_symbol_display_mode(display_mode),
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetPinOverride {
                path,
                symbol,
                pin_uuid,
                visible,
                x_nm,
                y_nm,
            } => {
                let position = parse_native_field_position(x_nm, y_nm)?;
                let report = set_native_project_symbol_pin_override(
                    &path, symbol, pin_uuid, visible, position,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_pin_override_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetSymbolHiddenPowerBehavior {
                path,
                symbol,
                hidden_power_behavior,
            } => {
                let report = set_native_project_symbol_hidden_power_behavior(
                    &path,
                    symbol,
                    parse_native_hidden_power_behavior(hidden_power_behavior),
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ClearPinOverride {
                path,
                symbol,
                pin_uuid,
            } => {
                let report = clear_native_project_symbol_pin_override(&path, symbol, pin_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_pin_override_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::AddSymbolField {
                path,
                symbol,
                key,
                value,
                hidden,
                x_nm,
                y_nm,
            } => {
                let report = add_native_project_symbol_field(
                    &path,
                    symbol,
                    key,
                    value,
                    !hidden,
                    parse_native_field_position(x_nm, y_nm)?,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_field_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditSymbolField {
                path,
                field,
                key,
                value,
                visible,
                x_nm,
                y_nm,
            } => {
                let report = edit_native_project_symbol_field(
                    &path,
                    field,
                    key,
                    value,
                    visible,
                    parse_native_field_position(x_nm, y_nm)?,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_field_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteSymbolField { path, field } => {
                let report = delete_native_project_symbol_field(&path, field)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_symbol_field_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceText {
                path,
                sheet,
                text,
                x_nm,
                y_nm,
                rotation_deg,
            } => {
                let report = place_native_project_text(
                    &path,
                    sheet,
                    text,
                    eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                    rotation_deg,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_text_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditText {
                path,
                text,
                value,
                x_nm,
                y_nm,
                rotation_deg,
            } => {
                let position = match (x_nm, y_nm) {
                    (None, None) => None,
                    (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                    _ => bail!("text position requires both --x-nm and --y-nm"),
                };
                let report = edit_native_project_text(&path, text, value, position, rotation_deg)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_text_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteText { path, text } => {
                let report = delete_native_project_text(&path, text)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_text_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceDrawingLine {
                path,
                sheet,
                from_x_nm,
                from_y_nm,
                to_x_nm,
                to_y_nm,
            } => {
                let report = place_native_project_drawing_line(
                    &path,
                    sheet,
                    eda_engine::ir::geometry::Point {
                        x: from_x_nm,
                        y: from_y_nm,
                    },
                    eda_engine::ir::geometry::Point {
                        x: to_x_nm,
                        y: to_y_nm,
                    },
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceDrawingRect {
                path,
                sheet,
                min_x_nm,
                min_y_nm,
                max_x_nm,
                max_y_nm,
            } => {
                let report = place_native_project_drawing_rect(
                    &path,
                    sheet,
                    eda_engine::ir::geometry::Point {
                        x: min_x_nm,
                        y: min_y_nm,
                    },
                    eda_engine::ir::geometry::Point {
                        x: max_x_nm,
                        y: max_y_nm,
                    },
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceDrawingCircle {
                path,
                sheet,
                center_x_nm,
                center_y_nm,
                radius_nm,
            } => {
                let report = place_native_project_drawing_circle(
                    &path,
                    sheet,
                    eda_engine::ir::geometry::Point {
                        x: center_x_nm,
                        y: center_y_nm,
                    },
                    radius_nm,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceDrawingArc {
                path,
                sheet,
                center_x_nm,
                center_y_nm,
                radius_nm,
                start_angle_mdeg,
                end_angle_mdeg,
            } => {
                let report = place_native_project_drawing_arc(
                    &path,
                    sheet,
                    eda_engine::ir::geometry::Arc {
                        center: eda_engine::ir::geometry::Point {
                            x: center_x_nm,
                            y: center_y_nm,
                        },
                        radius: radius_nm,
                        start_angle: start_angle_mdeg,
                        end_angle: end_angle_mdeg,
                    },
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditDrawingLine {
                path,
                drawing,
                from_x_nm,
                from_y_nm,
                to_x_nm,
                to_y_nm,
            } => {
                let from = match (from_x_nm, from_y_nm) {
                    (None, None) => None,
                    (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                    _ => bail!("drawing start requires both --from-x-nm and --from-y-nm"),
                };
                let to = match (to_x_nm, to_y_nm) {
                    (None, None) => None,
                    (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                    _ => bail!("drawing end requires both --to-x-nm and --to-y-nm"),
                };
                let report = edit_native_project_drawing_line(&path, drawing, from, to)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditDrawingRect {
                path,
                drawing,
                min_x_nm,
                min_y_nm,
                max_x_nm,
                max_y_nm,
            } => {
                let min = match (min_x_nm, min_y_nm) {
                    (None, None) => None,
                    (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                    _ => bail!("rect min requires both --min-x-nm and --min-y-nm"),
                };
                let max = match (max_x_nm, max_y_nm) {
                    (None, None) => None,
                    (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                    _ => bail!("rect max requires both --max-x-nm and --max-y-nm"),
                };
                let report = edit_native_project_drawing_rect(&path, drawing, min, max)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditDrawingCircle {
                path,
                drawing,
                center_x_nm,
                center_y_nm,
                radius_nm,
            } => {
                let center = match (center_x_nm, center_y_nm) {
                    (None, None) => None,
                    (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                    _ => bail!("circle center requires both --center-x-nm and --center-y-nm"),
                };
                let report = edit_native_project_drawing_circle(&path, drawing, center, radius_nm)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditDrawingArc {
                path,
                drawing,
                center_x_nm,
                center_y_nm,
                radius_nm,
                start_angle_mdeg,
                end_angle_mdeg,
            } => {
                let center = match (center_x_nm, center_y_nm) {
                    (None, None) => None,
                    (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                    _ => bail!("arc center requires both --center-x-nm and --center-y-nm"),
                };
                let report = edit_native_project_drawing_arc(
                    &path,
                    drawing,
                    center,
                    radius_nm,
                    start_angle_mdeg,
                    end_angle_mdeg,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteDrawing { path, drawing } => {
                let report = delete_native_project_drawing(&path, drawing)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceLabel {
                path,
                sheet,
                name,
                kind,
                x_nm,
                y_nm,
            } => {
                let kind = match kind {
                    NativeLabelKindArg::Local => LabelKind::Local,
                    NativeLabelKindArg::Global => LabelKind::Global,
                    NativeLabelKindArg::Hierarchical => LabelKind::Hierarchical,
                    NativeLabelKindArg::Power => LabelKind::Power,
                };
                let report = place_native_project_label(
                    &path,
                    sheet,
                    name,
                    kind,
                    eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_label_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::RenameLabel { path, label, name } => {
                let report = rename_native_project_label(&path, label, name)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_label_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteLabel { path, label } => {
                let report = delete_native_project_label(&path, label)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_label_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DrawWire {
                path,
                sheet,
                from_x_nm,
                from_y_nm,
                to_x_nm,
                to_y_nm,
            } => {
                let report = draw_native_project_wire(
                    &path,
                    sheet,
                    eda_engine::ir::geometry::Point {
                        x: from_x_nm,
                        y: from_y_nm,
                    },
                    eda_engine::ir::geometry::Point {
                        x: to_x_nm,
                        y: to_y_nm,
                    },
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_wire_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteWire { path, wire } => {
                let report = delete_native_project_wire(&path, wire)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_wire_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceJunction {
                path,
                sheet,
                x_nm,
                y_nm,
            } => {
                let report = place_native_project_junction(
                    &path,
                    sheet,
                    eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_junction_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteJunction { path, junction } => {
                let report = delete_native_project_junction(&path, junction)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_junction_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlacePort {
                path,
                sheet,
                name,
                direction,
                x_nm,
                y_nm,
            } => {
                let direction = match direction {
                    NativePortDirectionArg::Input => PortDirection::Input,
                    NativePortDirectionArg::Output => PortDirection::Output,
                    NativePortDirectionArg::Bidirectional => PortDirection::Bidirectional,
                    NativePortDirectionArg::Passive => PortDirection::Passive,
                };
                let report = place_native_project_port(
                    &path,
                    sheet,
                    name,
                    direction,
                    eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_port_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditPort {
                path,
                port,
                name,
                direction,
                x_nm,
                y_nm,
            } => {
                let direction = direction.map(|value| match value {
                    NativePortDirectionArg::Input => PortDirection::Input,
                    NativePortDirectionArg::Output => PortDirection::Output,
                    NativePortDirectionArg::Bidirectional => PortDirection::Bidirectional,
                    NativePortDirectionArg::Passive => PortDirection::Passive,
                });
                let report = edit_native_project_port(&path, port, name, direction, x_nm, y_nm)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_port_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeletePort { path, port } => {
                let report = delete_native_project_port(&path, port)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_port_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::CreateBus {
                path,
                sheet,
                name,
                members,
            } => {
                let report = create_native_project_bus(&path, sheet, name, members)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_bus_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditBusMembers { path, bus, members } => {
                let report = edit_native_project_bus_members(&path, bus, members)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_bus_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceBusEntry {
                path,
                sheet,
                bus,
                wire,
                x_nm,
                y_nm,
            } => {
                let report = place_native_project_bus_entry(
                    &path,
                    sheet,
                    bus,
                    wire,
                    eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_bus_entry_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteBusEntry { path, bus_entry } => {
                let report = delete_native_project_bus_entry(&path, bus_entry)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_bus_entry_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceNoConnect {
                path,
                sheet,
                symbol,
                pin,
                x_nm,
                y_nm,
            } => {
                let report = place_native_project_noconnect(
                    &path,
                    sheet,
                    symbol,
                    pin,
                    eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_noconnect_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteNoConnect { path, noconnect } => {
                let report = delete_native_project_noconnect(&path, noconnect)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_noconnect_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceBoardText {
                path,
                text,
                x_nm,
                y_nm,
                rotation_deg,
                layer,
            } => {
                let report = place_native_project_board_text(
                    &path,
                    text,
                    eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                    rotation_deg,
                    layer,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_text_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditBoardText {
                path,
                text_uuid,
                value,
                x_nm,
                y_nm,
                rotation_deg,
                layer,
            } => {
                let report = edit_native_project_board_text(
                    &path,
                    text_uuid,
                    value,
                    x_nm,
                    y_nm,
                    rotation_deg,
                    layer,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_text_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteBoardText { path, text_uuid } => {
                let report = delete_native_project_board_text(&path, text_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_text_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceBoardKeepout {
                path,
                kind,
                layers,
                vertices,
            } => {
                let polygon = parse_native_polygon_vertices(&vertices)?;
                let report = place_native_project_board_keepout(&path, kind, layers, polygon)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_keepout_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditBoardKeepout {
                path,
                keepout_uuid,
                kind,
                layers,
                vertices,
            } => {
                let polygon = if vertices.is_empty() {
                    None
                } else {
                    Some(parse_native_polygon_vertices(&vertices)?)
                };
                let report =
                    edit_native_project_board_keepout(&path, keepout_uuid, kind, layers, polygon)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_keepout_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteBoardKeepout { path, keepout_uuid } => {
                let report = delete_native_project_board_keepout(&path, keepout_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_keepout_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetBoardOutline { path, vertices } => {
                let polygon = parse_native_polygon_vertices(&vertices)?;
                let report = set_native_project_board_outline(&path, polygon)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_outline_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetBoardStackup { path, layers } => {
                let stackup_layers = parse_native_stackup_layers(&layers)?;
                let report = set_native_project_board_stackup(&path, stackup_layers)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_stackup_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceBoardNet {
                path,
                name,
                class_uuid,
            } => {
                let report = place_native_project_board_net(&path, name, class_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_net_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceBoardComponent {
                path,
                part_uuid,
                package_uuid,
                reference,
                value,
                x_nm,
                y_nm,
                layer,
            } => {
                let report = place_native_project_board_component(
                    &path,
                    part_uuid,
                    package_uuid,
                    reference,
                    value,
                    eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                    layer,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_component_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceBoardNetClass {
                path,
                name,
                clearance_nm,
                track_width_nm,
                via_drill_nm,
                via_diameter_nm,
                diffpair_width_nm,
                diffpair_gap_nm,
            } => {
                let report = place_native_project_board_net_class(
                    &path,
                    name,
                    clearance_nm,
                    track_width_nm,
                    via_drill_nm,
                    via_diameter_nm,
                    diffpair_width_nm,
                    diffpair_gap_nm,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_net_class_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditBoardNetClass {
                path,
                net_class_uuid,
                name,
                clearance_nm,
                track_width_nm,
                via_drill_nm,
                via_diameter_nm,
                diffpair_width_nm,
                diffpair_gap_nm,
            } => {
                let report = edit_native_project_board_net_class(
                    &path,
                    net_class_uuid,
                    name,
                    clearance_nm,
                    track_width_nm,
                    via_drill_nm,
                    via_diameter_nm,
                    diffpair_width_nm,
                    diffpair_gap_nm,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_net_class_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditBoardNet {
                path,
                net_uuid,
                name,
                class_uuid,
            } => {
                let report = edit_native_project_board_net(&path, net_uuid, name, class_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_net_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::MoveBoardComponent {
                path,
                component_uuid,
                x_nm,
                y_nm,
            } => {
                let report = move_native_project_board_component(
                    &path,
                    component_uuid,
                    eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                )?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_component_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetBoardComponentPart {
                path,
                component_uuid,
                part_uuid,
            } => {
                let report =
                    set_native_project_board_component_part(&path, component_uuid, part_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_component_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetBoardComponentPackage {
                path,
                component_uuid,
                package_uuid,
            } => {
                let report = set_native_project_board_component_package(
                    &path,
                    component_uuid,
                    package_uuid,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_component_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::RotateBoardComponent {
                path,
                component_uuid,
                rotation_deg,
            } => {
                let report =
                    rotate_native_project_board_component(&path, component_uuid, rotation_deg)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_component_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetBoardComponentLocked {
                path,
                component_uuid,
            } => {
                let report =
                    set_native_project_board_component_locked(&path, component_uuid, true)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_component_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ClearBoardComponentLocked {
                path,
                component_uuid,
            } => {
                let report =
                    set_native_project_board_component_locked(&path, component_uuid, false)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_component_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteBoardComponent {
                path,
                component_uuid,
            } => {
                let report = delete_native_project_board_component(&path, component_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_component_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ApplyForwardAnnotationAction {
                path,
                action_id,
                package_uuid,
                part_uuid,
                x_nm,
                y_nm,
                layer,
            } => {
                let report = apply_native_project_forward_annotation_action(
                    &path,
                    &action_id,
                    package_uuid,
                    part_uuid,
                    x_nm,
                    y_nm,
                    layer,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_forward_annotation_apply_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ApplyForwardAnnotationReviewed { path } => {
                let report = apply_native_project_forward_annotation_reviewed(&path)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_forward_annotation_batch_apply_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ExportForwardAnnotationProposal { path, out } => {
                let report = export_native_project_forward_annotation_proposal(&path, &out)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_forward_annotation_export_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ExportForwardAnnotationProposalSelection {
                path,
                action_ids,
                out,
            } => {
                let report = export_native_project_forward_annotation_proposal_selection(
                    &path,
                    &action_ids,
                    &out,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_forward_annotation_export_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SelectForwardAnnotationProposalArtifact {
                artifact,
                action_ids,
                out,
            } => {
                let report =
                    select_forward_annotation_proposal_artifact(&artifact, &action_ids, &out)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_forward_annotation_export_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::InspectForwardAnnotationProposalArtifact { path } => {
                let report = inspect_forward_annotation_proposal_artifact(&path)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_forward_annotation_artifact_inspection_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::CompareForwardAnnotationProposalArtifact { path, artifact } => {
                let report = compare_forward_annotation_proposal_artifact(&path, &artifact)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_forward_annotation_artifact_comparison_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::FilterForwardAnnotationProposalArtifact {
                path,
                artifact,
                out,
            } => {
                let report = filter_forward_annotation_proposal_artifact(&path, &artifact, &out)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_forward_annotation_artifact_filter_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlanForwardAnnotationProposalArtifactApply { path, artifact } => {
                let report = plan_forward_annotation_proposal_artifact_apply(&path, &artifact)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_forward_annotation_artifact_apply_plan_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ApplyForwardAnnotationProposalArtifact { path, artifact } => {
                let report = apply_forward_annotation_proposal_artifact(&path, &artifact)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_forward_annotation_artifact_apply_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ImportForwardAnnotationArtifactReview { path, artifact } => {
                let report = import_forward_annotation_artifact_review(&path, &artifact)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_forward_annotation_artifact_review_import_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ReplaceForwardAnnotationArtifactReview { path, artifact } => {
                let report = replace_forward_annotation_artifact_review(&path, &artifact)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_forward_annotation_artifact_review_replace_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeferForwardAnnotationAction { path, action_id } => {
                let report =
                    record_native_project_forward_annotation_review(&path, &action_id, "deferred")?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_forward_annotation_review_report_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::RejectForwardAnnotationAction { path, action_id } => {
                let report =
                    record_native_project_forward_annotation_review(&path, &action_id, "rejected")?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_forward_annotation_review_report_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ClearForwardAnnotationActionReview { path, action_id } => {
                let report = clear_native_project_forward_annotation_review(&path, &action_id)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_forward_annotation_review_report_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DrawBoardTrack {
                path,
                net_uuid,
                from_x_nm,
                from_y_nm,
                to_x_nm,
                to_y_nm,
                width_nm,
                layer,
            } => {
                let report = place_native_project_board_track(
                    &path,
                    net_uuid,
                    eda_engine::ir::geometry::Point {
                        x: from_x_nm,
                        y: from_y_nm,
                    },
                    eda_engine::ir::geometry::Point {
                        x: to_x_nm,
                        y: to_y_nm,
                    },
                    width_nm,
                    layer,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_track_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteBoardTrack { path, track_uuid } => {
                let report = delete_native_project_board_track(&path, track_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_track_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceBoardVia {
                path,
                net_uuid,
                x_nm,
                y_nm,
                drill_nm,
                diameter_nm,
                from_layer,
                to_layer,
            } => {
                let report = place_native_project_board_via(
                    &path,
                    net_uuid,
                    eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                    drill_nm,
                    diameter_nm,
                    from_layer,
                    to_layer,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_via_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteBoardVia { path, via_uuid } => {
                let report = delete_native_project_board_via(&path, via_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_via_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceBoardZone {
                path,
                net_uuid,
                vertices,
                layer,
                priority,
                thermal_relief,
                thermal_gap_nm,
                thermal_spoke_width_nm,
            } => {
                let polygon = parse_native_polygon_vertices(&vertices)?;
                let report = place_native_project_board_zone(
                    &path,
                    net_uuid,
                    polygon,
                    layer,
                    priority,
                    thermal_relief,
                    thermal_gap_nm,
                    thermal_spoke_width_nm,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_zone_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteBoardZone { path, zone_uuid } => {
                let report = delete_native_project_board_zone(&path, zone_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_zone_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::SetBoardPadNet {
                path,
                pad_uuid,
                net_uuid,
            } => {
                let report = set_native_project_board_pad_net(&path, pad_uuid, Some(net_uuid))?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::ClearBoardPadNet { path, pad_uuid } => {
                let report = set_native_project_board_pad_net(&path, pad_uuid, None)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditBoardPad {
                path,
                pad_uuid,
                x_nm,
                y_nm,
                layer,
                diameter_nm,
            } => {
                let position = match (x_nm, y_nm) {
                    (None, None) => None,
                    (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                    _ => bail!("board pad position requires both --x-nm and --y-nm"),
                };
                let report =
                    edit_native_project_board_pad(&path, pad_uuid, position, layer, diameter_nm)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceBoardPad {
                path,
                package_uuid,
                name,
                x_nm,
                y_nm,
                layer,
                diameter_nm,
                net_uuid,
            } => {
                let report = place_native_project_board_pad(
                    &path,
                    package_uuid,
                    name,
                    eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                    layer,
                    diameter_nm,
                    net_uuid,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteBoardPad { path, pad_uuid } => {
                let report = delete_native_project_board_pad(&path, pad_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteBoardNetClass {
                path,
                net_class_uuid,
            } => {
                let report = delete_native_project_board_net_class(&path, net_class_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_net_class_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteBoardNet { path, net_uuid } => {
                let report = delete_native_project_board_net(&path, net_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => render_native_project_board_net_mutation_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::PlaceBoardDimension {
                path,
                from_x_nm,
                from_y_nm,
                to_x_nm,
                to_y_nm,
                text,
            } => {
                let report = place_native_project_board_dimension(
                    &path,
                    eda_engine::ir::geometry::Point {
                        x: from_x_nm,
                        y: from_y_nm,
                    },
                    eda_engine::ir::geometry::Point {
                        x: to_x_nm,
                        y: to_y_nm,
                    },
                    text,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_dimension_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::EditBoardDimension {
                path,
                dimension_uuid,
                from_x_nm,
                from_y_nm,
                to_x_nm,
                to_y_nm,
                text,
                clear_text,
            } => {
                let report = edit_native_project_board_dimension(
                    &path,
                    dimension_uuid,
                    from_x_nm,
                    from_y_nm,
                    to_x_nm,
                    to_y_nm,
                    text,
                    clear_text,
                )?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_dimension_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
            ProjectCommands::DeleteBoardDimension {
                path,
                dimension_uuid,
            } => {
                let report = delete_native_project_board_dimension(&path, dimension_uuid)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_native_project_board_dimension_mutation_text(&report)
                    }
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
        },
        Commands::Plan { action } => match action {
            PlanCommands::ExportScopedReplacementManifest {
                path,
                out,
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
                    ReplacementPolicyArg::Package => {
                        ComponentReplacementPolicy::BestCompatiblePackage
                    }
                    ReplacementPolicyArg::Part => ComponentReplacementPolicy::BestCompatiblePart,
                };
                let overrides = override_component
                    .iter()
                    .map(|value| parse_scoped_replacement_override_arg(value))
                    .collect::<Result<Vec<_>>>()?;
                let plan = query_scoped_component_replacement_plan(
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
                let manifest = scoped_replacement_manifest_from_parts(&path, &libraries, plan)?;
                let payload = serde_json::to_string_pretty(&manifest)
                    .expect("manifest serialization must succeed");
                std::fs::write(&out, payload)
                    .with_context(|| format!("failed to write manifest {}", out.display()))?;
                let output = match cli.format {
                    OutputFormat::Text => render_scoped_replacement_manifest_export_text(
                        &out,
                        &manifest.kind,
                        manifest.version,
                        manifest.plan.replacements.len(),
                    ),
                    OutputFormat::Json => render_output(
                        &cli.format,
                        &serde_json::json!({
                            "path": out.display().to_string(),
                            "kind": manifest.kind,
                            "version": manifest.version,
                            "replacements": manifest.plan.replacements.len(),
                        }),
                    ),
                };
                Ok((output, 0))
            }
            PlanCommands::InspectScopedReplacementManifest { path } => {
                let inspection = inspect_scoped_replacement_manifest(&path)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_scoped_replacement_manifest_inspection_text(&inspection)
                    }
                    OutputFormat::Json => render_output(&cli.format, &inspection),
                };
                Ok((output, 0))
            }
            PlanCommands::ValidateScopedReplacementManifest { paths } => {
                let summary = validate_scoped_replacement_manifest_inputs_batch(&paths)?;
                let output = match cli.format {
                    OutputFormat::Text => {
                        render_scoped_replacement_manifest_validation_text(&summary)
                    }
                    OutputFormat::Json => render_output(&cli.format, &summary),
                };
                let exit_code = if summary.manifests_failing == 0 { 0 } else { 1 };
                Ok((output, exit_code))
            }
            PlanCommands::UpgradeScopedReplacementManifest {
                path,
                out,
                in_place,
            } => {
                let output_path = match (out, in_place) {
                    (Some(out), false) => out,
                    (None, true) => path.clone(),
                    (Some(_), true) => {
                        bail!(
                            "plan upgrade-scoped-replacement-manifest accepts either --out or --in-place, not both"
                        );
                    }
                    (None, false) => {
                        bail!(
                            "plan upgrade-scoped-replacement-manifest requires either --out <path> or --in-place"
                        );
                    }
                };
                let report = upgrade_scoped_replacement_manifest(&path, &output_path)?;
                let output = match cli.format {
                    OutputFormat::Text => render_scoped_replacement_manifest_upgrade_text(&report),
                    OutputFormat::Json => render_output(&cli.format, &report),
                };
                Ok((output, 0))
            }
        },
        Commands::Modify {
            path,
            delete_track,
            delete_via,
            delete_component,
            libraries,
            move_component,
            rotate_component,
            set_value,
            assign_part,
            set_package,
            set_package_with_part,
            replace_component,
            apply_replacement_plan,
            apply_replacement_policy,
            apply_scoped_replacement_policy,
            apply_scoped_replacement_plan_file,
            apply_scoped_replacement_manifest,
            set_net_class,
            set_reference,
            undo,
            redo,
            save,
            set_clearance_min_nm,
            save_original,
        } => {
            let move_component = move_component
                .iter()
                .map(|value| parse_move_component_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let rotate_component = rotate_component
                .iter()
                .map(|value| parse_rotate_component_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let set_value = set_value
                .iter()
                .map(|value| parse_set_value_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let assign_part = assign_part
                .iter()
                .map(|value| parse_assign_part_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let set_package = set_package
                .iter()
                .map(|value| parse_set_package_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let set_package_with_part = set_package_with_part
                .iter()
                .map(|value| parse_set_package_with_part_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let replace_component = replace_component
                .iter()
                .map(|value| parse_replace_component_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let apply_replacement_plan = apply_replacement_plan
                .iter()
                .map(|value| parse_apply_replacement_plan_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let apply_replacement_policy = apply_replacement_policy
                .iter()
                .map(|value| parse_apply_replacement_policy_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let apply_scoped_replacement_policy = apply_scoped_replacement_policy
                .iter()
                .map(|value| parse_apply_scoped_replacement_policy_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let apply_scoped_replacement_plan = apply_scoped_replacement_plan_file
                .iter()
                .map(|path| {
                    let contents = std::fs::read_to_string(path).with_context(|| {
                        format!(
                            "failed to read scoped replacement plan file {}",
                            path.display()
                        )
                    })?;
                    serde_json::from_str::<ScopedComponentReplacementPlan>(&contents).with_context(
                        || {
                            format!(
                                "failed to parse scoped replacement plan file {}",
                                path.display()
                            )
                        },
                    )
                })
                .collect::<Result<Vec<_>>>()?;
            let scoped_replacement_manifests = apply_scoped_replacement_manifest
                .iter()
                .map(|manifest_path| {
                    let loaded = load_scoped_replacement_manifest_with_metadata(manifest_path)?;
                    validate_scoped_replacement_manifest(&loaded.manifest, &path)?;
                    Ok(loaded)
                })
                .collect::<Result<Vec<LoadedScopedReplacementManifest>>>()?;
            let mut libraries = libraries;
            for manifest in &scoped_replacement_manifests {
                for library in &manifest.manifest.libraries {
                    if !libraries.iter().any(|existing| existing == &library.path) {
                        libraries.push(library.path.clone());
                    }
                }
            }
            let mut apply_scoped_replacement_plan = apply_scoped_replacement_plan;
            apply_scoped_replacement_plan.extend(
                scoped_replacement_manifests
                    .iter()
                    .map(|loaded| loaded.manifest.plan.clone()),
            );
            let applied_scoped_replacement_manifests = scoped_replacement_manifests
                .iter()
                .map(|loaded| AppliedScopedReplacementManifestView {
                    path: loaded.manifest_path.display().to_string(),
                    source_version: loaded.source_version,
                    version: loaded.manifest.version,
                    migration_applied: loaded.source_version != loaded.manifest.version,
                    replacements: loaded.manifest.plan.replacements.len(),
                })
                .collect::<Vec<_>>();
            let set_net_class = set_net_class
                .iter()
                .map(|value| parse_set_net_class_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let set_reference = set_reference
                .iter()
                .map(|value| parse_set_reference_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let mut report = modify_board_with_plan(
                &path,
                &delete_track,
                &delete_via,
                &delete_component,
                &libraries,
                &move_component,
                &rotate_component,
                &set_value,
                &assign_part,
                &set_package,
                &set_package_with_part,
                &replace_component,
                &set_net_class,
                &set_reference,
                set_clearance_min_nm,
                undo,
                redo,
                save.as_deref(),
                save_original,
                &apply_replacement_plan,
                &apply_replacement_policy,
                &apply_scoped_replacement_policy,
                &apply_scoped_replacement_plan,
            )?;
            report.applied_scoped_replacement_manifests = applied_scoped_replacement_manifests;
            let output = match cli.format {
                OutputFormat::Text => render_modify_report_text(&report),
                OutputFormat::Json => render_output(&cli.format, &report),
            };
            Ok((output, 0))
        }
    }
}
