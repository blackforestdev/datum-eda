use super::command_exec_native_support::{
    parse_native_hidden_power_behavior, parse_native_symbol_display_mode,
};
use super::*;
use crate::command_modify::{
    parse_apply_replacement_plan_arg, parse_apply_replacement_policy_arg,
    parse_apply_scoped_replacement_policy_arg, parse_assign_part_arg, parse_move_component_arg,
    parse_replace_component_arg, parse_rotate_component_arg, parse_set_net_class_arg,
    parse_set_package_arg, parse_set_package_with_part_arg, parse_set_reference_arg,
    parse_set_value_arg,
};
use eda_engine::schematic::{LabelKind, PortDirection};

pub(crate) fn execute_with_exit_code(cli: Cli) -> Result<(String, i32)> {
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
        Commands::Project { action } => {
            let action = *action;
            if command_exec_inventory::is_inventory_command(&action) {
                return command_exec_inventory::execute_inventory_command(&cli.format, action);
            }
            match action {
                ProjectCommands::New(ProjectNewArgs { path, name }) => {
                    let report = create_native_project(&path, name)?;
                    let output = match cli.format {
                        OutputFormat::Text => render_native_project_create_report_text(&report),
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                ProjectCommands::Inspect(ProjectInspectArgs { path }) => {
                    let report = inspect_native_project(&path)?;
                    let output = match cli.format {
                        OutputFormat::Text => render_native_project_inspect_report_text(&report),
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                ProjectCommands::Query(ProjectQueryArgs { path, what }) => {
                    command_exec_project_query::execute_native_project_query_command(
                        &cli.format,
                        path,
                        what,
                    )
                }
                command @ ProjectCommands::ExportDrill(_)
                | command @ ProjectCommands::ValidateDrill(_)
                | command @ ProjectCommands::CompareDrill(_)
                | command @ ProjectCommands::ExportExcellonDrill(_)
                | command @ ProjectCommands::InspectDrill(_)
                | command @ ProjectCommands::CompareExcellonDrill(_)
                | command @ ProjectCommands::ValidateExcellonDrill(_)
                | command @ ProjectCommands::ReportDrillHoleClasses(_) => {
                    command_exec_drill::execute_drill_command(&cli.format, command)
                }
                ProjectCommands::InspectExcellonDrill(ProjectInspectExcellonDrillArgs { path }) => {
                    command_exec_project_inspect::execute_project_excellon_drill_inspection(
                        &cli.format,
                        &path,
                    )
                }
                ProjectCommands::InspectGerber(ProjectInspectGerberArgs { path }) => {
                    command_exec_project_inspect::execute_project_gerber_inspection(
                        &cli.format,
                        &path,
                    )
                }
                ProjectCommands::ExportGerberOutline(ProjectExportGerberOutlineArgs {
                    path,
                    out,
                }) => {
                    let report = export_native_project_gerber_outline(&path, &out)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_outline_export_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                ProjectCommands::ExportGerberCopperLayer(ProjectExportGerberCopperLayerArgs {
                    path,
                    layer,
                    out,
                }) => {
                    let report = export_native_project_gerber_copper_layer(&path, layer, &out)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_copper_export_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                ProjectCommands::ExportGerberSoldermaskLayer(
                    ProjectExportGerberSoldermaskLayerArgs { path, layer, out },
                ) => {
                    let report = export_native_project_gerber_soldermask_layer(&path, layer, &out)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_soldermask_export_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                ProjectCommands::ExportGerberSilkscreenLayer(
                    ProjectExportGerberSilkscreenLayerArgs { path, layer, out },
                ) => {
                    let report = export_native_project_gerber_silkscreen_layer(&path, layer, &out)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_silkscreen_export_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                ProjectCommands::ExportGerberPasteLayer(ProjectExportGerberPasteLayerArgs {
                    path,
                    layer,
                    out,
                }) => {
                    let report = export_native_project_gerber_paste_layer(&path, layer, &out)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_paste_export_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                ProjectCommands::ExportGerberMechanicalLayer(
                    ProjectExportGerberMechanicalLayerArgs { path, layer, out },
                ) => {
                    let report = export_native_project_gerber_mechanical_layer(&path, layer, &out)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_mechanical_export_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                ProjectCommands::ValidateGerberOutline(ProjectValidateGerberOutlineArgs {
                    path,
                    gerber,
                }) => {
                    let report = validate_native_project_gerber_outline(&path, &gerber)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_outline_validation_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, if report.matches_expected { 0 } else { 1 }))
                }
                ProjectCommands::ValidateGerberCopperLayer(
                    ProjectValidateGerberCopperLayerArgs {
                        path,
                        layer,
                        gerber,
                    },
                ) => {
                    let report =
                        validate_native_project_gerber_copper_layer(&path, layer, &gerber)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_copper_validation_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    let exit_code = if report.matches_expected { 0 } else { 1 };
                    Ok((output, exit_code))
                }
                ProjectCommands::ValidateGerberSoldermaskLayer(
                    ProjectValidateGerberSoldermaskLayerArgs {
                        path,
                        layer,
                        gerber,
                    },
                ) => {
                    let report =
                        validate_native_project_gerber_soldermask_layer(&path, layer, &gerber)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_soldermask_validation_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    let exit_code = if report.matches_expected { 0 } else { 1 };
                    Ok((output, exit_code))
                }
                ProjectCommands::ValidateGerberSilkscreenLayer(
                    ProjectValidateGerberSilkscreenLayerArgs {
                        path,
                        layer,
                        gerber,
                    },
                ) => {
                    let report =
                        validate_native_project_gerber_silkscreen_layer(&path, layer, &gerber)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_silkscreen_validation_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    let exit_code = if report.matches_expected { 0 } else { 1 };
                    Ok((output, exit_code))
                }
                ProjectCommands::ValidateGerberPasteLayer(
                    ProjectValidateGerberPasteLayerArgs {
                        path,
                        layer,
                        gerber,
                    },
                ) => {
                    let report = validate_native_project_gerber_paste_layer(&path, layer, &gerber)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_paste_validation_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    let exit_code = if report.matches_expected { 0 } else { 1 };
                    Ok((output, exit_code))
                }
                ProjectCommands::ValidateGerberMechanicalLayer(
                    ProjectValidateGerberMechanicalLayerArgs {
                        path,
                        layer,
                        gerber,
                    },
                ) => {
                    let report =
                        validate_native_project_gerber_mechanical_layer(&path, layer, &gerber)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_mechanical_validation_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    let exit_code = if report.matches_expected { 0 } else { 1 };
                    Ok((output, exit_code))
                }
                ProjectCommands::CompareGerberOutline(ProjectCompareGerberOutlineArgs {
                    path,
                    gerber,
                }) => {
                    let report = compare_native_project_gerber_outline(&path, &gerber)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_outline_comparison_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                ProjectCommands::CompareGerberCopperLayer(
                    ProjectCompareGerberCopperLayerArgs {
                        path,
                        layer,
                        gerber,
                    },
                ) => {
                    let report = compare_native_project_gerber_copper_layer(&path, layer, &gerber)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_copper_comparison_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                ProjectCommands::CompareGerberSoldermaskLayer(
                    ProjectCompareGerberSoldermaskLayerArgs {
                        path,
                        layer,
                        gerber,
                    },
                ) => {
                    let report =
                        compare_native_project_gerber_soldermask_layer(&path, layer, &gerber)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_soldermask_comparison_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                ProjectCommands::CompareGerberSilkscreenLayer(
                    ProjectCompareGerberSilkscreenLayerArgs {
                        path,
                        layer,
                        gerber,
                    },
                ) => {
                    let report =
                        compare_native_project_gerber_silkscreen_layer(&path, layer, &gerber)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_silkscreen_comparison_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                ProjectCommands::CompareGerberPasteLayer(ProjectCompareGerberPasteLayerArgs {
                    path,
                    layer,
                    gerber,
                }) => {
                    let report = compare_native_project_gerber_paste_layer(&path, layer, &gerber)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_paste_comparison_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                ProjectCommands::CompareGerberMechanicalLayer(
                    ProjectCompareGerberMechanicalLayerArgs {
                        path,
                        layer,
                        gerber,
                    },
                ) => {
                    let report =
                        compare_native_project_gerber_mechanical_layer(&path, layer, &gerber)?;
                    let output = match cli.format {
                        OutputFormat::Text => {
                            render_native_project_gerber_mechanical_comparison_text(&report)
                        }
                        OutputFormat::Json => render_output(&cli.format, &report),
                    };
                    Ok((output, 0))
                }
                command @ ProjectCommands::PlanGerberExport(_)
                | command @ ProjectCommands::ExportGerberSet(_)
                | command @ ProjectCommands::ValidateGerberSet(_)
                | command @ ProjectCommands::CompareGerberSet(_)
                | command @ ProjectCommands::CompareGerberExportPlan(_) => {
                    command_exec_gerber_plan::execute_gerber_workflow_command(&cli.format, command)
                }
                command @ ProjectCommands::ReportManufacturing(_)
                | command @ ProjectCommands::ExportManufacturingSet(_)
                | command @ ProjectCommands::InspectManufacturingSet(_)
                | command @ ProjectCommands::ValidateManufacturingSet(_)
                | command @ ProjectCommands::CompareManufacturingSet(_)
                | command @ ProjectCommands::ManifestManufacturingSet(_) => {
                    command_exec_manufacturing::execute_manufacturing_command(&cli.format, command)
                }
                command @ ProjectCommands::ExportForwardAnnotationAudit(
                    ProjectExportForwardAnnotationAuditArgs { .. },
                )
                | command @ ProjectCommands::ForwardAnnotationAudit(
                    ProjectForwardAnnotationAuditArgs { .. },
                )
                | command @ ProjectCommands::ExportForwardAnnotationProposal(
                    ProjectExportForwardAnnotationProposalArgs { .. },
                )
                | command @ ProjectCommands::ApplyForwardAnnotationAction(
                    ProjectApplyForwardAnnotationActionArgs { .. },
                )
                | command @ ProjectCommands::ApplyForwardAnnotationReviewed(
                    ProjectApplyForwardAnnotationReviewedArgs { .. },
                )
                | command @ ProjectCommands::ExportForwardAnnotationProposalSelection(
                    ProjectExportForwardAnnotationProposalSelectionArgs { .. },
                )
                | command @ ProjectCommands::SelectForwardAnnotationProposalArtifact(
                    ProjectSelectForwardAnnotationProposalArtifactArgs { .. },
                )
                | command @ ProjectCommands::InspectForwardAnnotationProposalArtifact(
                    ProjectInspectForwardAnnotationProposalArtifactArgs { .. },
                )
                | command @ ProjectCommands::ValidateForwardAnnotationProposalArtifact(
                    ProjectValidateForwardAnnotationProposalArtifactArgs { .. },
                )
                | command @ ProjectCommands::CompareForwardAnnotationProposalArtifact(
                    ProjectCompareForwardAnnotationProposalArtifactArgs { .. },
                )
                | command @ ProjectCommands::FilterForwardAnnotationProposalArtifact(
                    ProjectFilterForwardAnnotationProposalArtifactArgs { .. },
                )
                | command @ ProjectCommands::PlanForwardAnnotationProposalArtifactApply(
                    ProjectPlanForwardAnnotationProposalArtifactApplyArgs { .. },
                )
                | command @ ProjectCommands::ApplyForwardAnnotationProposalArtifact(
                    ProjectApplyForwardAnnotationProposalArtifactArgs { .. },
                )
                | command @ ProjectCommands::ImportForwardAnnotationArtifactReview(
                    ProjectImportForwardAnnotationArtifactReviewArgs { .. },
                )
                | command @ ProjectCommands::ReplaceForwardAnnotationArtifactReview(
                    ProjectReplaceForwardAnnotationArtifactReviewArgs { .. },
                )
                | command @ ProjectCommands::DeferForwardAnnotationAction(
                    ProjectDeferForwardAnnotationActionArgs { .. },
                )
                | command @ ProjectCommands::RejectForwardAnnotationAction(
                    ProjectRejectForwardAnnotationActionArgs { .. },
                )
                | command @ ProjectCommands::ClearForwardAnnotationActionReview(
                    ProjectClearForwardAnnotationActionReviewArgs { .. },
                ) => command_exec_forward_annotation::execute_forward_annotation_command(
                    &cli.format,
                    command,
                ),
                command @ ProjectCommands::ExportRoutePathProposal(
                    ProjectExportRoutePathProposalArgs { .. },
                )
                | command @ ProjectCommands::InspectRouteProposalArtifact(
                    ProjectInspectRouteProposalArtifactArgs { .. },
                )
                | command @ ProjectCommands::ApplyRouteProposalArtifact(
                    ProjectApplyRouteProposalArtifactArgs { .. },
                )
                | command @ ProjectCommands::RouteApply(ProjectRouteApplyArgs { .. }) => {
                    command_exec_route_proposal::execute_route_proposal_command(
                        &cli.format,
                        command,
                    )
                }
                command => {
                    command_exec_project_command::execute_project_command(&cli.format, command)
                }
            }
        }
        Commands::Plan { action } => command_exec_plan::execute_plan_command(&cli.format, action),
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
                .map(|plan_path| {
                    let plan_text = std::fs::read_to_string(plan_path).with_context(|| {
                        format!(
                            "failed to read scoped replacement plan file {}",
                            plan_path.display()
                        )
                    })?;
                    serde_json::from_str::<ScopedComponentReplacementPlan>(&plan_text).with_context(
                        || {
                            format!(
                                "failed to parse scoped replacement plan file {}",
                                plan_path.display()
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
