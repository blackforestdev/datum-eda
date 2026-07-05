// commands/dispatch.rs — the single exhaustive ProjectCommands router.
//
// Wave 3 of the CLI reorganization: replaces the legacy three-layer router
// chain (command_exec_dispatch.rs Project arm + command_exec_project_command.rs
// + command_exec_project_board_surface.rs) with ONE compiler-enforced
// exhaustive match (no `_ =>` arm). Every arm still calls the existing
// execute_* functions in the command_exec_* files; converting those to run()
// impls is the next wave.

use super::*;
use crate::command_exec::command_exec_surface::{
    execute_add_default_top_stackup, execute_delete_board_net, execute_delete_board_net_class,
    execute_drill_command, execute_edit_board_net, execute_edit_board_net_class,
    execute_forward_annotation_command, execute_generate_board_components,
    execute_gerber_workflow_command, execute_inventory_command, execute_manufacturing_command,
    execute_move_board_component, execute_native_project_query_command,
    execute_place_board_component, execute_place_board_net, execute_place_board_net_class,
    execute_project_excellon_drill_inspection, execute_project_gerber_inspection,
    execute_project_import_or_part_binding_command, execute_project_library_command,
    execute_project_library_footprint_command, execute_project_library_pin_pad_map_command,
    execute_project_proposal_lifecycle_command, execute_project_schematic_connectivity_command,
    execute_project_schematic_symbols_command, execute_rotate_board_component,
    execute_route_proposal_command, execute_set_board_component_layer,
    execute_set_board_component_locked, execute_set_board_component_package,
    execute_set_board_component_part, execute_set_board_component_reference,
    execute_set_board_component_value, execute_set_board_name, execute_set_board_outline,
    execute_set_board_stackup,
};

pub(crate) fn execute_project_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::New(ProjectNewArgs { path, name }) => {
            let report = create_native_project(&path, name)?;
            let output = match format {
                OutputFormat::Text => render_native_project_create_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::Inspect(ProjectInspectArgs { path }) => {
            let report = inspect_native_project(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_project_inspect_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::Validate(ProjectValidateArgs { path }) => {
            let report = validate_native_project(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_project_validation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, if report.valid { 0 } else { 1 }))
        }
        ProjectCommands::Query(ProjectQueryArgs { path, what }) => {
            execute_native_project_query_command(format, path, what)
        }
        command @ ProjectCommands::ExportDrill(_)
        | command @ ProjectCommands::ValidateDrill(_)
        | command @ ProjectCommands::CompareDrill(_)
        | command @ ProjectCommands::ExportExcellonDrill(_)
        | command @ ProjectCommands::InspectDrill(_)
        | command @ ProjectCommands::CompareExcellonDrill(_)
        | command @ ProjectCommands::ValidateExcellonDrill(_)
        | command @ ProjectCommands::ReportDrillHoleClasses(_) => {
            execute_drill_command(format, command)
        }
        ProjectCommands::InspectExcellonDrill(ProjectInspectExcellonDrillArgs { path }) => {
            execute_project_excellon_drill_inspection(format, &path)
        }
        ProjectCommands::InspectGerber(ProjectInspectGerberArgs { path }) => {
            execute_project_gerber_inspection(format, &path)
        }
        ProjectCommands::ExportGerberOutline(ProjectExportGerberOutlineArgs { path, out }) => {
            let report = export_native_project_gerber_outline(&path, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_project_gerber_outline_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportGerberCopperLayer(ProjectExportGerberCopperLayerArgs {
            path,
            layer,
            out,
        }) => {
            let report = export_native_project_gerber_copper_layer(&path, layer, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_project_gerber_copper_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportGerberSoldermaskLayer(ProjectExportGerberSoldermaskLayerArgs {
            path,
            layer,
            out,
        }) => {
            let report = export_native_project_gerber_soldermask_layer(&path, layer, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_project_gerber_soldermask_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportGerberSilkscreenLayer(ProjectExportGerberSilkscreenLayerArgs {
            path,
            layer,
            out,
        }) => {
            let report = export_native_project_gerber_silkscreen_layer(&path, layer, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_project_gerber_silkscreen_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportGerberPasteLayer(ProjectExportGerberPasteLayerArgs {
            path,
            layer,
            out,
        }) => {
            let report = export_native_project_gerber_paste_layer(&path, layer, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_project_gerber_paste_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportGerberMechanicalLayer(ProjectExportGerberMechanicalLayerArgs {
            path,
            layer,
            out,
        }) => {
            let report = export_native_project_gerber_mechanical_layer(&path, layer, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_project_gerber_mechanical_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ValidateGerberOutline(ProjectValidateGerberOutlineArgs {
            path,
            gerber,
        }) => {
            let report = validate_native_project_gerber_outline(&path, &gerber)?;
            let output = match format {
                OutputFormat::Text => render_native_project_gerber_outline_validation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, if report.matches_expected { 0 } else { 1 }))
        }
        ProjectCommands::ValidateGerberCopperLayer(ProjectValidateGerberCopperLayerArgs {
            path,
            layer,
            gerber,
        }) => {
            let report = validate_native_project_gerber_copper_layer(&path, layer, &gerber)?;
            let output = match format {
                OutputFormat::Text => render_native_project_gerber_copper_validation_text(&report),
                OutputFormat::Json => render_output(format, &report),
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
            let report = validate_native_project_gerber_soldermask_layer(&path, layer, &gerber)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_gerber_soldermask_validation_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
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
            let report = validate_native_project_gerber_silkscreen_layer(&path, layer, &gerber)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_gerber_silkscreen_validation_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            let exit_code = if report.matches_expected { 0 } else { 1 };
            Ok((output, exit_code))
        }
        ProjectCommands::ValidateGerberPasteLayer(ProjectValidateGerberPasteLayerArgs {
            path,
            layer,
            gerber,
        }) => {
            let report = validate_native_project_gerber_paste_layer(&path, layer, &gerber)?;
            let output = match format {
                OutputFormat::Text => render_native_project_gerber_paste_validation_text(&report),
                OutputFormat::Json => render_output(format, &report),
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
            let report = validate_native_project_gerber_mechanical_layer(&path, layer, &gerber)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_gerber_mechanical_validation_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            let exit_code = if report.matches_expected { 0 } else { 1 };
            Ok((output, exit_code))
        }
        ProjectCommands::CompareGerberOutline(ProjectCompareGerberOutlineArgs { path, gerber }) => {
            let report = compare_native_project_gerber_outline(&path, &gerber)?;
            let output = match format {
                OutputFormat::Text => render_native_project_gerber_outline_comparison_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::CompareGerberCopperLayer(ProjectCompareGerberCopperLayerArgs {
            path,
            layer,
            gerber,
        }) => {
            let report = compare_native_project_gerber_copper_layer(&path, layer, &gerber)?;
            let output = match format {
                OutputFormat::Text => render_native_project_gerber_copper_comparison_text(&report),
                OutputFormat::Json => render_output(format, &report),
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
            let report = compare_native_project_gerber_soldermask_layer(&path, layer, &gerber)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_gerber_soldermask_comparison_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
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
            let report = compare_native_project_gerber_silkscreen_layer(&path, layer, &gerber)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_gerber_silkscreen_comparison_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::CompareGerberPasteLayer(ProjectCompareGerberPasteLayerArgs {
            path,
            layer,
            gerber,
        }) => {
            let report = compare_native_project_gerber_paste_layer(&path, layer, &gerber)?;
            let output = match format {
                OutputFormat::Text => render_native_project_gerber_paste_comparison_text(&report),
                OutputFormat::Json => render_output(format, &report),
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
            let report = compare_native_project_gerber_mechanical_layer(&path, layer, &gerber)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_gerber_mechanical_comparison_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        command @ ProjectCommands::PlanGerberExport(_)
        | command @ ProjectCommands::ExportGerberSet(_)
        | command @ ProjectCommands::ValidateGerberSet(_)
        | command @ ProjectCommands::CompareGerberSet(_)
        | command @ ProjectCommands::CompareGerberExportPlan(_) => {
            execute_gerber_workflow_command(format, command)
        }
        command @ ProjectCommands::ReportManufacturing(_)
        | command @ ProjectCommands::ExportManufacturingSet(_)
        | command @ ProjectCommands::InspectManufacturingSet(_)
        | command @ ProjectCommands::ValidateManufacturingSet(_)
        | command @ ProjectCommands::CompareManufacturingSet(_)
        | command @ ProjectCommands::ManifestManufacturingSet(_) => {
            execute_manufacturing_command(format, command)
        }
        command @ ProjectCommands::ExportForwardAnnotationAudit(
            ProjectExportForwardAnnotationAuditArgs { .. },
        )
        | command @ ProjectCommands::ForwardAnnotationAudit(ProjectForwardAnnotationAuditArgs {
            ..
        })
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
        ) => execute_forward_annotation_command(format, command),
        command @ ProjectCommands::RouteProposal(ProjectRouteProposalArgs { .. })
        | command @ ProjectCommands::ReviewRouteProposal(ProjectReviewRouteProposalArgs {
            ..
        })
        | command @ ProjectCommands::RouteStrategyReport(ProjectRouteStrategyReportArgs {
            ..
        })
        | command @ ProjectCommands::RouteStrategyCompare(ProjectRouteStrategyCompareArgs {
            ..
        })
        | command @ ProjectCommands::RouteStrategyDelta(ProjectRouteStrategyDeltaArgs { .. })
        | command @ ProjectCommands::WriteRouteStrategyCuratedFixtureSuite(
            ProjectWriteRouteStrategyCuratedFixtureSuiteArgs { .. },
        )
        | command @ ProjectCommands::CaptureRouteStrategyCuratedBaseline(
            ProjectCaptureRouteStrategyCuratedBaselineArgs { .. },
        )
        | command @ ProjectCommands::RouteStrategyBatchEvaluate(
            ProjectRouteStrategyBatchEvaluateArgs { .. },
        )
        | command @ ProjectCommands::InspectRouteStrategyBatchResult(
            ProjectInspectRouteStrategyBatchResultArgs { .. },
        )
        | command @ ProjectCommands::ValidateRouteStrategyBatchResult(
            ProjectValidateRouteStrategyBatchResultArgs { .. },
        )
        | command @ ProjectCommands::CompareRouteStrategyBatchResult(
            ProjectCompareRouteStrategyBatchResultArgs { .. },
        )
        | command @ ProjectCommands::GateRouteStrategyBatchResult(
            ProjectGateRouteStrategyBatchResultArgs { .. },
        )
        | command @ ProjectCommands::SummarizeRouteStrategyBatchResults(
            ProjectSummarizeRouteStrategyBatchResultsArgs { .. },
        )
        | command @ ProjectCommands::RouteProposalExplain(ProjectRouteProposalExplainArgs {
            ..
        })
        | command @ ProjectCommands::ExportRouteProposal(ProjectExportRouteProposalArgs {
            ..
        })
        | command @ ProjectCommands::ExportRoutePathProposal(ProjectExportRoutePathProposalArgs {
            ..
        })
        | command @ ProjectCommands::InspectRouteProposalArtifact(
            ProjectInspectRouteProposalArtifactArgs { .. },
        )
        | command @ ProjectCommands::RevalidateRouteProposalArtifact(
            ProjectRevalidateRouteProposalArtifactArgs { .. },
        )
        | command @ ProjectCommands::ApplyRouteProposalArtifact(
            ProjectApplyRouteProposalArtifactArgs { .. },
        )
        | command @ ProjectCommands::RouteApplySelected(ProjectRouteApplySelectedArgs { .. })
        | command @ ProjectCommands::RouteApply(ProjectRouteApplyArgs { .. }) => {
            execute_route_proposal_command(format, command)
        }
        command @ (ProjectCommands::ExportBom(_)
        | ProjectCommands::CompareBom(_)
        | ProjectCommands::ValidateBom(_)
        | ProjectCommands::InspectBom(_)
        | ProjectCommands::ExportPnp(_)
        | ProjectCommands::ComparePnp(_)
        | ProjectCommands::ValidatePnp(_)
        | ProjectCommands::InspectPnp(_)) => execute_inventory_command(format, command),
        command @ (ProjectCommands::ReviewProposal(_)
        | ProjectCommands::ShowProposal(_)
        | ProjectCommands::ValidateProposal(_)
        | ProjectCommands::DeferProposal(_)
        | ProjectCommands::ApplyProposal(_)) => {
            execute_project_proposal_lifecycle_command(format, command)
        }
        command @ (ProjectCommands::ImportKicadFootprint(_)
        | ProjectCommands::ImportKicadBoard(_)
        | ProjectCommands::ImportKicadSchematic(_)
        | ProjectCommands::ImportEagleLibrary(_)
        | ProjectCommands::SetPoolPartBindings(_)
        | ProjectCommands::SetPoolSymbolPinAnchor(_)) => {
            execute_project_import_or_part_binding_command(format, command)
        }
        command @ (ProjectCommands::CreatePoolFootprint(_)
        | ProjectCommands::GenerateIpc7351bTwoTerminalChip(_)
        | ProjectCommands::SetPoolFootprintPad(_)
        | ProjectCommands::SetPoolFootprintCourtyardRect(_)
        | ProjectCommands::SetPoolFootprintCourtyardPolygon(_)
        | ProjectCommands::AddPoolFootprintSilkscreenLine(_)
        | ProjectCommands::AddPoolFootprintSilkscreenRect(_)
        | ProjectCommands::AddPoolFootprintSilkscreenCircle(_)
        | ProjectCommands::AddPoolFootprintSilkscreenPolygon(_)) => {
            execute_project_library_footprint_command(format, command)
        }
        command @ (ProjectCommands::CreatePoolPinPadMap(_)
        | ProjectCommands::SetPoolPinPadMap(_)) => {
            execute_project_library_pin_pad_map_command(format, command)
        }
        command @ ProjectCommands::CreatePoolLibraryObject(_)
        | command @ ProjectCommands::CreatePoolUnit(_)
        | command @ ProjectCommands::SetPoolUnitPin(_)
        | command @ ProjectCommands::CreatePoolSymbol(_)
        | command @ ProjectCommands::AddPoolSymbolLine(_)
        | command @ ProjectCommands::AddPoolSymbolPolygon(_)
        | command @ ProjectCommands::AddPoolSymbolRect(_)
        | command @ ProjectCommands::AddPoolSymbolCircle(_)
        | command @ ProjectCommands::AddPoolSymbolText(_)
        | command @ ProjectCommands::AddPoolSymbolArc(_)
        | command @ ProjectCommands::CreatePoolEntity(_)
        | command @ ProjectCommands::CreatePoolPadstack(_)
        | command @ ProjectCommands::CreatePoolPackage(_)
        | command @ ProjectCommands::SetPoolPackagePad(_)
        | command @ ProjectCommands::SetPoolPackageCourtyardRect(_)
        | command @ ProjectCommands::SetPoolPackageCourtyardPolygon(_)
        | command @ ProjectCommands::AddPoolPackageSilkscreenLine(_)
        | command @ ProjectCommands::AddPoolPackageSilkscreenRect(_)
        | command @ ProjectCommands::AddPoolPackageSilkscreenCircle(_)
        | command @ ProjectCommands::AddPoolPackageSilkscreenArc(_)
        | command @ ProjectCommands::AddPoolPackageSilkscreenPolygon(_)
        | command @ ProjectCommands::AddPoolPackageSilkscreenText(_)
        | command @ ProjectCommands::AddPoolPackageModel3d(_)
        | command @ ProjectCommands::SetPoolPackageBodyHeights(_)
        | command @ ProjectCommands::CreatePoolPart(_)
        | command @ ProjectCommands::SetPoolPartMetadata(_)
        | command @ ProjectCommands::SetPoolPartParametric(_)
        | command @ ProjectCommands::SetPoolPartOrderableMpns(_)
        | command @ ProjectCommands::SetPoolPartPackagingOptions(_)
        | command @ ProjectCommands::SetPoolPartBehaviouralModels(_)
        | command @ ProjectCommands::AttachPoolPartModel(_)
        | command @ ProjectCommands::DetachPoolPartModel(_)
        | command @ ProjectCommands::GcPoolModels(_)
        | command @ ProjectCommands::SetPoolPartThermal(_)
        | command @ ProjectCommands::SetPoolPartSupplyChain(_)
        | command @ ProjectCommands::SetPoolPartTags(_)
        | command @ ProjectCommands::SetPoolPartPadMap(_)
        | command @ ProjectCommands::SetPoolPartPadMapEntry(_)
        | command @ ProjectCommands::SetPoolLibraryObject(_)
        | command @ ProjectCommands::DeletePoolLibraryObject(_) => {
            execute_project_library_command(format, command)
        }
        ProjectCommands::SetProjectName(ProjectSetProjectNameArgs { path, name }) => {
            let report = set_native_project_name(&path, name)?;
            let output = match format {
                OutputFormat::Text => render_native_project_name_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetProjectRules(ProjectSetProjectRulesArgs { path, rules_file }) => {
            let report = set_native_project_rules(&path, &rules_file)?;
            let output = match format {
                OutputFormat::Text => render_native_project_rules_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::CreateProjectRule(ProjectCreateProjectRuleArgs { path, rule_file }) => {
            let report = create_native_project_rule(&path, &rule_file)?;
            let output = match format {
                OutputFormat::Text => render_native_project_rules_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetProjectRule(ProjectSetProjectRuleArgs { path, rule_file }) => {
            let report = set_native_project_rule(&path, &rule_file)?;
            let output = match format {
                OutputFormat::Text => render_native_project_rules_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteProjectRule(ProjectDeleteProjectRuleArgs { path, rule_uuid }) => {
            let report = delete_native_project_rule(&path, rule_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_rules_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::Undo(args) => execute_native_project_journal_undo(
            format,
            &args.path,
            args.expected_model_revision.as_deref(),
            args.expected_tip_transaction,
        ),
        ProjectCommands::Redo(args) => execute_native_project_journal_redo(
            format,
            &args.path,
            args.expected_model_revision.as_deref(),
            args.expected_tip_transaction,
        ),
        ProjectCommands::GenerateStandardsRepairProposals(
            ProjectGenerateStandardsRepairProposalsArgs { path },
        ) => Ok((
            render_output(
                format,
                &generate_native_project_standards_repair_proposals(&path)?,
            ),
            0,
        )),
        ProjectCommands::WaiveFinding(ProjectWaiveFindingArgs {
            path,
            fingerprint,
            rationale,
            created_by,
        }) => Ok((
            render_output(
                format,
                &waive_native_project_finding(&path, &fingerprint, &rationale, created_by)?,
            ),
            0,
        )),
        ProjectCommands::AcceptDeviation(ProjectAcceptDeviationArgs {
            path,
            fingerprint,
            rationale,
            accepted_by,
        }) => Ok((
            render_output(
                format,
                &accept_native_project_deviation(&path, &fingerprint, &rationale, accepted_by)?,
            ),
            0,
        )),
        ProjectCommands::CreateGerberOutputJob(ProjectCreateGerberOutputJobArgs {
            path,
            prefix,
            output_dir,
            name,
            manufacturing_plan,
            variant,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_create_native_project_output_job(
                        &path,
                        &prefix,
                        output_dir.as_deref(),
                        "gerber-set",
                        name.as_deref(),
                        manufacturing_plan,
                        variant,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(create_native_project_gerber_set_output_job(
                        &path,
                        &prefix,
                        output_dir.as_deref(),
                        name.as_deref(),
                        manufacturing_plan,
                        variant,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::CreateOutputJob(ProjectCreateOutputJobArgs {
            path,
            prefix,
            output_dir,
            include,
            name,
            manufacturing_plan,
            variant,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_create_native_project_output_job(
                        &path,
                        &prefix,
                        output_dir.as_deref(),
                        &include,
                        name.as_deref(),
                        manufacturing_plan,
                        variant,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(create_native_project_output_job(
                        &path,
                        &prefix,
                        output_dir.as_deref(),
                        &include,
                        name.as_deref(),
                        manufacturing_plan,
                        variant,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::UpdateOutputJob(ProjectUpdateOutputJobArgs {
            path,
            output_job,
            name,
            output_dir,
            manufacturing_plan,
            variant,
            clear_manufacturing_plan,
            clear_variant,
            clear_output_dir,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_update_native_project_output_job(
                        &path,
                        output_job,
                        name.as_deref(),
                        output_dir.as_deref(),
                        manufacturing_plan,
                        variant,
                        clear_manufacturing_plan,
                        clear_variant,
                        clear_output_dir,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(update_native_project_output_job(
                        &path,
                        output_job,
                        name.as_deref(),
                        output_dir.as_deref(),
                        manufacturing_plan,
                        variant,
                        clear_manufacturing_plan,
                        clear_variant,
                        clear_output_dir,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::RunOutputJob(ProjectRunOutputJobArgs {
            path,
            output_job,
            output_dir,
        }) => {
            let report = run_native_project_output_job(&path, output_job, output_dir.as_deref())?;
            let exit_code = report.exit_code;
            Ok((render_output(format, &report), exit_code))
        }
        ProjectCommands::StartOutputJobRun(ProjectStartOutputJobRunArgs { path, output_job }) => {
            Ok((
                render_output(
                    format,
                    &start_native_project_output_job_run(&path, output_job)?,
                ),
                0,
            ))
        }
        ProjectCommands::CancelOutputJobRun(ProjectCancelOutputJobRunArgs { path, run }) => Ok((
            render_output(format, &cancel_native_project_output_job_run(&path, run)?),
            0,
        )),
        ProjectCommands::DeleteOutputJob(ProjectDeleteOutputJobArgs {
            path,
            output_job,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_delete_native_project_output_job(
                        &path,
                        output_job,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(delete_native_project_output_job(&path, output_job)?)?
                },
            ),
            0,
        )),
        ProjectCommands::CreateManufacturingPlan(ProjectCreateManufacturingPlanArgs {
            path,
            prefix,
            name,
            variant,
            panel_projection,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_create_native_project_manufacturing_plan(
                        &path,
                        &prefix,
                        name.as_deref(),
                        variant,
                        panel_projection,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(create_native_project_manufacturing_plan(
                        &path,
                        &prefix,
                        name.as_deref(),
                        variant,
                        panel_projection,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::UpdateManufacturingPlan(ProjectUpdateManufacturingPlanArgs {
            path,
            manufacturing_plan,
            name,
            prefix,
            variant,
            clear_variant,
            panel_projection,
            clear_panel_projection,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_update_native_project_manufacturing_plan(
                        &path,
                        manufacturing_plan,
                        name.as_deref(),
                        prefix.as_deref(),
                        variant,
                        clear_variant,
                        panel_projection,
                        clear_panel_projection,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(update_native_project_manufacturing_plan(
                        &path,
                        manufacturing_plan,
                        name.as_deref(),
                        prefix.as_deref(),
                        variant,
                        clear_variant,
                        panel_projection,
                        clear_panel_projection,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::DeleteManufacturingPlan(ProjectDeleteManufacturingPlanArgs {
            path,
            manufacturing_plan,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_delete_native_project_manufacturing_plan(
                        &path,
                        manufacturing_plan,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(delete_native_project_manufacturing_plan(
                        &path,
                        manufacturing_plan,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::CreatePanelProjection(ProjectCreatePanelProjectionArgs {
            path,
            key,
            name,
            board,
            x_nm,
            y_nm,
            rotation_deg,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_create_native_project_panel_projection(
                        &path,
                        &key,
                        name.as_deref(),
                        board,
                        x_nm,
                        y_nm,
                        rotation_deg,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(create_native_project_panel_projection(
                        &path,
                        &key,
                        name.as_deref(),
                        board,
                        x_nm,
                        y_nm,
                        rotation_deg,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::UpdatePanelProjection(ProjectUpdatePanelProjectionArgs {
            path,
            panel_projection,
            name,
            board,
            x_nm,
            y_nm,
            rotation_deg,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_update_native_project_panel_projection(
                        &path,
                        panel_projection,
                        name.as_deref(),
                        board,
                        x_nm,
                        y_nm,
                        rotation_deg,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(update_native_project_panel_projection(
                        &path,
                        panel_projection,
                        name.as_deref(),
                        board,
                        x_nm,
                        y_nm,
                        rotation_deg,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::DeletePanelProjection(ProjectDeletePanelProjectionArgs {
            path,
            panel_projection,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_delete_native_project_panel_projection(
                        &path,
                        panel_projection,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(delete_native_project_panel_projection(
                        &path,
                        panel_projection,
                    )?)?
                },
            ),
            0,
        )),
        command @ ProjectCommands::CreateSheet(ProjectCreateSheetArgs { .. })
        | command @ ProjectCommands::DeleteSheet(ProjectDeleteSheetArgs { .. })
        | command @ ProjectCommands::RenameSheet(ProjectRenameSheetArgs { .. })
        | command @ ProjectCommands::CreateSheetDefinition(ProjectCreateSheetDefinitionArgs {
            ..
        })
        | command @ ProjectCommands::CreateSheetInstance(ProjectCreateSheetInstanceArgs {
            ..
        })
        | command @ ProjectCommands::DeleteSheetInstance(ProjectDeleteSheetInstanceArgs {
            ..
        })
        | command @ ProjectCommands::MoveSheetInstance(ProjectMoveSheetInstanceArgs { .. })
        | command @ ProjectCommands::BindSheetInstancePort(ProjectBindSheetInstancePortArgs {
            ..
        })
        | command @ ProjectCommands::UnbindSheetInstancePort(ProjectUnbindSheetInstancePortArgs {
            ..
        })
        | command @ ProjectCommands::PlaceLabel(ProjectPlaceLabelArgs { .. })
        | command @ ProjectCommands::RenameLabel(ProjectRenameLabelArgs { .. })
        | command @ ProjectCommands::DeleteLabel(ProjectDeleteLabelArgs { .. })
        | command @ ProjectCommands::DrawWire(ProjectDrawWireArgs { .. })
        | command @ ProjectCommands::DeleteWire(ProjectDeleteWireArgs { .. })
        | command @ ProjectCommands::PlaceJunction(ProjectPlaceJunctionArgs { .. })
        | command @ ProjectCommands::DeleteJunction(ProjectDeleteJunctionArgs { .. })
        | command @ ProjectCommands::PlacePort(ProjectPlacePortArgs { .. })
        | command @ ProjectCommands::EditPort(ProjectEditPortArgs { .. })
        | command @ ProjectCommands::DeletePort(ProjectDeletePortArgs { .. })
        | command @ ProjectCommands::CreateBus(ProjectCreateBusArgs { .. })
        | command @ ProjectCommands::EditBusMembers(ProjectEditBusMembersArgs { .. })
        | command @ ProjectCommands::DeleteBus(ProjectDeleteBusArgs { .. })
        | command @ ProjectCommands::PlaceBusEntry(ProjectPlaceBusEntryArgs { .. })
        | command @ ProjectCommands::DeleteBusEntry(ProjectDeleteBusEntryArgs { .. })
        | command @ ProjectCommands::PlaceNoConnect(ProjectPlaceNoConnectArgs { .. })
        | command @ ProjectCommands::DeleteNoConnect(ProjectDeleteNoConnectArgs { .. }) => {
            execute_project_schematic_connectivity_command(format, command)
        }
        command @ ProjectCommands::PlaceSymbol(ProjectPlaceSymbolArgs { .. })
        | command @ ProjectCommands::MoveSymbol(ProjectMoveSymbolArgs { .. })
        | command @ ProjectCommands::RotateSymbol(ProjectRotateSymbolArgs { .. })
        | command @ ProjectCommands::MirrorSymbol(ProjectMirrorSymbolArgs { .. })
        | command @ ProjectCommands::DeleteSymbol(ProjectDeleteSymbolArgs { .. })
        | command @ ProjectCommands::SetSymbolReference(ProjectSetSymbolReferenceArgs { .. })
        | command @ ProjectCommands::SetSymbolValue(ProjectSetSymbolValueArgs { .. })
        | command @ ProjectCommands::SetSymbolLibId(ProjectSetSymbolLibIdArgs { .. })
        | command @ ProjectCommands::ClearSymbolLibId(ProjectClearSymbolLibIdArgs { .. })
        | command @ ProjectCommands::SetSymbolEntity(ProjectSetSymbolEntityArgs { .. })
        | command @ ProjectCommands::ClearSymbolEntity(ProjectClearSymbolEntityArgs { .. })
        | command @ ProjectCommands::SetSymbolPart(ProjectSetSymbolPartArgs { .. })
        | command @ ProjectCommands::ClearSymbolPart(ProjectClearSymbolPartArgs { .. })
        | command @ ProjectCommands::SetSymbolUnit(ProjectSetSymbolUnitArgs { .. })
        | command @ ProjectCommands::ClearSymbolUnit(ProjectClearSymbolUnitArgs { .. })
        | command @ ProjectCommands::SetSymbolGate(ProjectSetSymbolGateArgs { .. })
        | command @ ProjectCommands::ClearSymbolGate(ProjectClearSymbolGateArgs { .. })
        | command @ ProjectCommands::SetSymbolDisplayMode(ProjectSetSymbolDisplayModeArgs {
            ..
        })
        | command @ ProjectCommands::SetSymbolHiddenPowerBehavior(
            ProjectSetSymbolHiddenPowerBehaviorArgs { .. },
        )
        | command @ ProjectCommands::SetPinOverride(ProjectSetPinOverrideArgs { .. })
        | command @ ProjectCommands::ClearPinOverride(ProjectClearPinOverrideArgs { .. })
        | command @ ProjectCommands::AddSymbolField(ProjectAddSymbolFieldArgs { .. })
        | command @ ProjectCommands::EditSymbolField(ProjectEditSymbolFieldArgs { .. })
        | command @ ProjectCommands::DeleteSymbolField(ProjectDeleteSymbolFieldArgs { .. })
        | command @ ProjectCommands::PlaceText(ProjectPlaceTextArgs { .. })
        | command @ ProjectCommands::EditText(ProjectEditTextArgs { .. })
        | command @ ProjectCommands::DeleteText(ProjectDeleteTextArgs { .. })
        | command @ ProjectCommands::PlaceDrawingLine(ProjectPlaceDrawingLineArgs { .. })
        | command @ ProjectCommands::PlaceDrawingRect(ProjectPlaceDrawingRectArgs { .. })
        | command @ ProjectCommands::PlaceDrawingCircle(ProjectPlaceDrawingCircleArgs { .. })
        | command @ ProjectCommands::PlaceDrawingArc(ProjectPlaceDrawingArcArgs { .. })
        | command @ ProjectCommands::EditDrawingLine(ProjectEditDrawingLineArgs { .. })
        | command @ ProjectCommands::EditDrawingRect(ProjectEditDrawingRectArgs { .. })
        | command @ ProjectCommands::EditDrawingCircle(ProjectEditDrawingCircleArgs { .. })
        | command @ ProjectCommands::EditDrawingArc(ProjectEditDrawingArcArgs { .. })
        | command @ ProjectCommands::DeleteDrawing(ProjectDeleteDrawingArgs { .. }) => {
            execute_project_schematic_symbols_command(format, command)
        }
        ProjectCommands::PlaceBoardText(ProjectPlaceBoardTextArgs {
            path,
            text,
            x_nm,
            y_nm,
            rotation_deg,
            height_nm,
            stroke_width_nm,
            render_intent,
            family,
            style,
            style_class,
            h_align,
            v_align,
            mirrored,
            keep_upright,
            line_spacing_ratio_ppm,
            bold,
            italic,
            layer,
        }) => {
            let report = place_native_project_board_text(
                &path,
                text,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                rotation_deg,
                height_nm,
                stroke_width_nm,
                render_intent,
                family,
                style,
                style_class,
                h_align,
                v_align,
                mirrored,
                keep_upright,
                line_spacing_ratio_ppm,
                bold,
                italic,
                layer,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_text_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditBoardText(ProjectEditBoardTextArgs {
            path,
            text_uuid,
            value,
            x_nm,
            y_nm,
            rotation_deg,
            height_nm,
            stroke_width_nm,
            render_intent,
            family,
            style,
            style_class,
            h_align,
            v_align,
            mirrored,
            keep_upright,
            line_spacing_ratio_ppm,
            bold,
            italic,
            layer,
        }) => {
            let report = edit_native_project_board_text(
                &path,
                text_uuid,
                value,
                x_nm,
                y_nm,
                rotation_deg,
                height_nm,
                stroke_width_nm,
                render_intent,
                family,
                style,
                style_class,
                h_align,
                v_align,
                mirrored,
                keep_upright,
                line_spacing_ratio_ppm,
                bold,
                italic,
                layer,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_text_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardText(ProjectDeleteBoardTextArgs { path, text_uuid }) => {
            let report = delete_native_project_board_text(&path, text_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_text_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceBoardKeepout(ProjectPlaceBoardKeepoutArgs {
            path,
            kind,
            layers,
            vertices,
        }) => {
            let polygon = parse_native_polygon_vertices(&vertices)?;
            let report = place_native_project_board_keepout(&path, kind, layers, polygon)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_keepout_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditBoardKeepout(ProjectEditBoardKeepoutArgs {
            path,
            keepout_uuid,
            kind,
            layers,
            vertices,
        }) => {
            let polygon = if vertices.is_empty() {
                None
            } else {
                Some(parse_native_polygon_vertices(&vertices)?)
            };
            let report =
                edit_native_project_board_keepout(&path, keepout_uuid, kind, layers, polygon)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_keepout_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardKeepout(ProjectDeleteBoardKeepoutArgs {
            path,
            keepout_uuid,
        }) => {
            let report = delete_native_project_board_keepout(&path, keepout_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_keepout_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetBoardOutline(ProjectSetBoardOutlineArgs { path, vertices }) => {
            execute_set_board_outline(format, path, vertices)
        }
        ProjectCommands::SetBoardName(ProjectSetBoardNameArgs { path, name }) => {
            execute_set_board_name(format, path, name)
        }
        ProjectCommands::PlaceBoardComponent(ProjectPlaceBoardComponentArgs {
            path,
            part_uuid,
            package_uuid,
            reference,
            value,
            x_nm,
            y_nm,
            layer,
        }) => execute_place_board_component(
            format,
            path,
            part_uuid,
            package_uuid,
            reference,
            value,
            x_nm,
            y_nm,
            layer,
        ),
        ProjectCommands::GenerateBoardComponents(args) => {
            execute_generate_board_components(format, args)
        }
        ProjectCommands::DeleteBoardComponent(ProjectDeleteBoardComponentArgs {
            path,
            component_uuid,
        }) => {
            let report = delete_native_project_board_component(&path, component_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_component_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DrawBoardTrack(ProjectDrawBoardTrackArgs {
            path,
            net_uuid,
            from_x_nm,
            from_y_nm,
            to_x_nm,
            to_y_nm,
            width_nm,
            layer,
        }) => {
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
            let output = match format {
                OutputFormat::Text => render_native_project_board_track_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardTrack(ProjectDeleteBoardTrackArgs { path, track_uuid }) => {
            let report = delete_native_project_board_track(&path, track_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_track_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditBoardTrack(ProjectEditBoardTrackArgs {
            path,
            track_uuid,
            net_uuid,
            from_x_nm,
            from_y_nm,
            to_x_nm,
            to_y_nm,
            width_nm,
            layer,
        }) => {
            let from = match (from_x_nm, from_y_nm) {
                (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                (None, None) => None,
                _ => bail!("editing a board track start requires both --from-x-nm and --from-y-nm"),
            };
            let to = match (to_x_nm, to_y_nm) {
                (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                (None, None) => None,
                _ => bail!("editing a board track end requires both --to-x-nm and --to-y-nm"),
            };
            let report = edit_native_project_board_track(
                &path, track_uuid, net_uuid, from, to, width_nm, layer,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_track_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceBoardVia(ProjectPlaceBoardViaArgs {
            path,
            net_uuid,
            x_nm,
            y_nm,
            drill_nm,
            diameter_nm,
            from_layer,
            to_layer,
        }) => {
            let report = place_native_project_board_via(
                &path,
                net_uuid,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                drill_nm,
                diameter_nm,
                from_layer,
                to_layer,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_via_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardVia(ProjectDeleteBoardViaArgs { path, via_uuid }) => {
            let report = delete_native_project_board_via(&path, via_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_via_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditBoardVia(ProjectEditBoardViaArgs {
            path,
            via_uuid,
            net_uuid,
            x_nm,
            y_nm,
            drill_nm,
            diameter_nm,
            from_layer,
            to_layer,
        }) => {
            let position = match (x_nm, y_nm) {
                (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                (None, None) => None,
                _ => bail!("editing a board via position requires both --x-nm and --y-nm"),
            };
            let report = edit_native_project_board_via(
                &path,
                via_uuid,
                net_uuid,
                position,
                drill_nm,
                diameter_nm,
                from_layer,
                to_layer,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_via_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceBoardZone(ProjectPlaceBoardZoneArgs {
            path,
            net_uuid,
            vertices,
            layer,
            priority,
            thermal_relief,
            thermal_gap_nm,
            thermal_spoke_width_nm,
        }) => {
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
            let output = match format {
                OutputFormat::Text => render_native_project_board_zone_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditBoardZone(ProjectEditBoardZoneArgs {
            path,
            zone_uuid,
            net_uuid,
            vertices,
            layer,
            priority,
            thermal_relief,
            thermal_gap_nm,
            thermal_spoke_width_nm,
        }) => {
            let polygon = if vertices.is_empty() {
                None
            } else {
                Some(parse_native_polygon_vertices(&vertices)?)
            };
            let report = edit_native_project_board_zone(
                &path,
                zone_uuid,
                net_uuid,
                polygon,
                layer,
                priority,
                thermal_relief,
                thermal_gap_nm,
                thermal_spoke_width_nm,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_zone_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::FillZones(ProjectFillZonesArgs {
            path,
            zone_uuid,
            net_uuid,
        }) => Ok((
            render_output(
                format,
                &fill_native_project_zones(&path, zone_uuid, net_uuid)?,
            ),
            0,
        )),
        ProjectCommands::DeleteBoardZone(ProjectDeleteBoardZoneArgs { path, zone_uuid }) => {
            let report = delete_native_project_board_zone(&path, zone_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_zone_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetBoardPadNet(ProjectSetBoardPadNetArgs {
            path,
            pad_uuid,
            net_uuid,
        }) => {
            let report = set_native_project_board_pad_net(&path, pad_uuid, Some(net_uuid))?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ClearBoardPadNet(ProjectClearBoardPadNetArgs { path, pad_uuid }) => {
            let report = set_native_project_board_pad_net(&path, pad_uuid, None)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditBoardPad(ProjectEditBoardPadArgs {
            path,
            pad_uuid,
            x_nm,
            y_nm,
            layer,
            shape,
            diameter_nm,
            width_nm,
            height_nm,
        }) => {
            let position = match (x_nm, y_nm) {
                (None, None) => None,
                (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                _ => bail!("board pad position requires both --x-nm and --y-nm"),
            };
            let report = edit_native_project_board_pad(
                &path,
                pad_uuid,
                position,
                layer,
                shape,
                diameter_nm,
                width_nm,
                height_nm,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceBoardPad(ProjectPlaceBoardPadArgs {
            path,
            package_uuid,
            name,
            x_nm,
            y_nm,
            layer,
            shape,
            diameter_nm,
            width_nm,
            height_nm,
            net_uuid,
        }) => {
            let report = place_native_project_board_pad(
                &path,
                package_uuid,
                name,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                layer,
                shape,
                diameter_nm,
                width_nm,
                height_nm,
                net_uuid,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardPad(ProjectDeleteBoardPadArgs { path, pad_uuid }) => {
            let report = delete_native_project_board_pad(&path, pad_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceBoardDimension(args) => {
            let report = place_native_project_board_dimension(
                &args.path,
                eda_engine::ir::geometry::Point {
                    x: args.from_x_nm,
                    y: args.from_y_nm,
                },
                eda_engine::ir::geometry::Point {
                    x: args.to_x_nm,
                    y: args.to_y_nm,
                },
                args.layer,
                args.text,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_dimension_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditBoardDimension(args) => {
            let report = edit_native_project_board_dimension(
                &args.path,
                args.dimension_uuid,
                args.from_x_nm,
                args.from_y_nm,
                args.to_x_nm,
                args.to_y_nm,
                args.layer,
                args.text,
                args.clear_text,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_dimension_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardDimension(ProjectDeleteBoardDimensionArgs {
            path,
            dimension_uuid,
        }) => {
            let report = delete_native_project_board_dimension(&path, dimension_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_dimension_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetBoardStackup(ProjectSetBoardStackupArgs { path, layers }) => {
            execute_set_board_stackup(format, path, layers)
        }
        ProjectCommands::AddDefaultTopStackup(ProjectAddDefaultTopStackupArgs { path }) => {
            execute_add_default_top_stackup(format, path)
        }
        ProjectCommands::PlaceBoardNet(ProjectPlaceBoardNetArgs {
            path,
            name,
            class_uuid,
            impedance_target_ohms,
            impedance_tolerance_pct,
            controlled_dielectric_layer,
        }) => execute_place_board_net(
            format,
            path,
            name,
            class_uuid,
            impedance_target_ohms,
            impedance_tolerance_pct,
            controlled_dielectric_layer,
        ),
        ProjectCommands::PlaceBoardNetClass(ProjectPlaceBoardNetClassArgs {
            path,
            name,
            clearance_nm,
            track_width_nm,
            via_drill_nm,
            via_diameter_nm,
            diffpair_width_nm,
            diffpair_gap_nm,
        }) => execute_place_board_net_class(
            format,
            path,
            name,
            clearance_nm,
            track_width_nm,
            via_drill_nm,
            via_diameter_nm,
            diffpair_width_nm,
            diffpair_gap_nm,
        ),
        ProjectCommands::EditBoardNetClass(ProjectEditBoardNetClassArgs {
            path,
            net_class_uuid,
            name,
            clearance_nm,
            track_width_nm,
            via_drill_nm,
            via_diameter_nm,
            diffpair_width_nm,
            diffpair_gap_nm,
        }) => execute_edit_board_net_class(
            format,
            path,
            net_class_uuid,
            name,
            clearance_nm,
            track_width_nm,
            via_drill_nm,
            via_diameter_nm,
            diffpair_width_nm,
            diffpair_gap_nm,
        ),
        ProjectCommands::EditBoardNet(ProjectEditBoardNetArgs {
            path,
            net_uuid,
            name,
            class_uuid,
            impedance_target_ohms,
            impedance_tolerance_pct,
            controlled_dielectric_layer,
            clear_controlled_impedance,
        }) => execute_edit_board_net(
            format,
            path,
            net_uuid,
            name,
            class_uuid,
            impedance_target_ohms,
            impedance_tolerance_pct,
            controlled_dielectric_layer,
            clear_controlled_impedance,
        ),
        ProjectCommands::MoveBoardComponent(ProjectMoveBoardComponentArgs {
            path,
            component_uuid,
            x_nm,
            y_nm,
        }) => execute_move_board_component(format, path, component_uuid, x_nm, y_nm),
        ProjectCommands::SetBoardComponentPart(SetBoardComponentPartArgs {
            path,
            component_uuid,
            part_uuid,
        }) => execute_set_board_component_part(format, path, component_uuid, part_uuid),
        ProjectCommands::SetBoardComponentPackage(SetBoardComponentPackageArgs {
            path,
            component_uuid,
            package_uuid,
        }) => execute_set_board_component_package(format, path, component_uuid, package_uuid),
        ProjectCommands::SetBoardComponentLayer(SetBoardComponentLayerArgs {
            path,
            component_uuid,
            layer,
        })
        | ProjectCommands::FlipBoardComponent(SetBoardComponentLayerArgs {
            path,
            component_uuid,
            layer,
        }) => execute_set_board_component_layer(format, path, component_uuid, layer),
        ProjectCommands::SetBoardComponentReference(SetBoardComponentReferenceArgs {
            path,
            component_uuid,
            reference,
        }) => execute_set_board_component_reference(format, path, component_uuid, reference),
        ProjectCommands::SetBoardComponentValue(SetBoardComponentValueArgs {
            path,
            component_uuid,
            value,
        }) => execute_set_board_component_value(format, path, component_uuid, value),
        ProjectCommands::RotateBoardComponent(ProjectRotateBoardComponentArgs {
            path,
            component_uuid,
            rotation_deg,
        }) => execute_rotate_board_component(format, path, component_uuid, rotation_deg),
        ProjectCommands::SetBoardComponentLocked(ProjectSetBoardComponentLockedArgs {
            path,
            component_uuid,
        }) => execute_set_board_component_locked(format, path, component_uuid, true),
        ProjectCommands::ClearBoardComponentLocked(ProjectClearBoardComponentLockedArgs {
            path,
            component_uuid,
        }) => execute_set_board_component_locked(format, path, component_uuid, false),
        ProjectCommands::BindComponentInstance(args) => {
            let view = bind_native_project_component_instance(
                &args.path,
                args.component_instance,
                args.symbols,
                args.package,
                args.part,
                args.symbol_roles,
                args.package_roles,
            )?;
            Ok((render_output(format, &view), 0))
        }
        ProjectCommands::SetComponentInstance(args) => {
            let view = set_native_project_component_instance(
                &args.path,
                args.component_instance,
                args.symbols,
                args.package,
                args.part,
                args.symbol_roles,
                args.package_roles,
            )?;
            Ok((render_output(format, &view), 0))
        }
        ProjectCommands::DeleteComponentInstance(ProjectDeleteComponentInstanceArgs {
            path,
            component_instance,
        }) => Ok((
            render_output(
                format,
                &delete_native_project_component_instance(&path, component_instance)?,
            ),
            0,
        )),
        ProjectCommands::DeleteBoardNetClass(ProjectDeleteBoardNetClassArgs {
            path,
            net_class_uuid,
        }) => execute_delete_board_net_class(format, path, net_class_uuid),
        ProjectCommands::DeleteBoardNet(ProjectDeleteBoardNetArgs { path, net_uuid }) => {
            execute_delete_board_net(format, path, net_uuid)
        }
    }
}
