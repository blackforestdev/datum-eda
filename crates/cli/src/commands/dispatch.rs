// commands/dispatch.rs — the single exhaustive ProjectCommands router.
//
// Phase 5 endgame: ONE compiler-enforced exhaustive match (no `_ =>` arm).
// The command_exec_* forwarding layer is dissolved — arms either run their
// family's `args.run(format)` inherent method (impls live in the owning
// commands/<family>/ files) or inline the short destructure-and-render body
// directly.

use super::*;
use crate::command_modify::{
    parse_apply_replacement_plan_arg, parse_apply_replacement_policy_arg,
    parse_apply_scoped_replacement_policy_arg, parse_assign_part_arg, parse_move_component_arg,
    parse_replace_component_arg, parse_rotate_component_arg, parse_set_net_class_arg,
    parse_set_package_arg, parse_set_package_with_part_arg, parse_set_reference_arg,
    parse_set_value_arg,
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
        ProjectCommands::Query(args) => args.run(format),
        ProjectCommands::ExportDrill(args) => args.run(format),
        ProjectCommands::ValidateDrill(args) => args.run(format),
        ProjectCommands::CompareDrill(args) => args.run(format),
        ProjectCommands::ExportExcellonDrill(args) => args.run(format),
        ProjectCommands::InspectDrill(args) => args.run(format),
        ProjectCommands::CompareExcellonDrill(args) => args.run(format),
        ProjectCommands::ValidateExcellonDrill(args) => args.run(format),
        ProjectCommands::ReportDrillHoleClasses(args) => args.run(format),
        ProjectCommands::InspectExcellonDrill(args) => args.run(format),
        ProjectCommands::InspectGerber(args) => args.run(format),
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
        ProjectCommands::PlanGerberExport(args) => args.run(format),
        ProjectCommands::ExportGerberSet(args) => args.run(format),
        ProjectCommands::ValidateGerberSet(args) => args.run(format),
        ProjectCommands::CompareGerberSet(args) => args.run(format),
        ProjectCommands::CompareGerberExportPlan(args) => args.run(format),
        ProjectCommands::ReportManufacturing(args) => args.run(format),
        ProjectCommands::ExportManufacturingSet(args) => args.run(format),
        ProjectCommands::InspectManufacturingSet(args) => args.run(format),
        ProjectCommands::ValidateManufacturingSet(args) => args.run(format),
        ProjectCommands::CompareManufacturingSet(args) => args.run(format),
        ProjectCommands::ManifestManufacturingSet(args) => args.run(format),
        ProjectCommands::ExportForwardAnnotationAudit(args) => args.run(format),
        ProjectCommands::ForwardAnnotationAudit(args) => args.run(format),
        ProjectCommands::ExportForwardAnnotationProposal(args) => args.run(format),
        ProjectCommands::ApplyForwardAnnotationAction(args) => args.run(format),
        ProjectCommands::ApplyForwardAnnotationReviewed(args) => args.run(format),
        ProjectCommands::ExportForwardAnnotationProposalSelection(args) => args.run(format),
        ProjectCommands::SelectForwardAnnotationProposalArtifact(args) => args.run(format),
        ProjectCommands::InspectForwardAnnotationProposalArtifact(args) => args.run(format),
        ProjectCommands::ValidateForwardAnnotationProposalArtifact(args) => args.run(format),
        ProjectCommands::CompareForwardAnnotationProposalArtifact(args) => args.run(format),
        ProjectCommands::FilterForwardAnnotationProposalArtifact(args) => args.run(format),
        ProjectCommands::PlanForwardAnnotationProposalArtifactApply(args) => args.run(format),
        ProjectCommands::ApplyForwardAnnotationProposalArtifact(args) => args.run(format),
        ProjectCommands::ImportForwardAnnotationArtifactReview(args) => args.run(format),
        ProjectCommands::ReplaceForwardAnnotationArtifactReview(args) => args.run(format),
        ProjectCommands::DeferForwardAnnotationAction(args) => args.run(format),
        ProjectCommands::RejectForwardAnnotationAction(args) => args.run(format),
        ProjectCommands::ClearForwardAnnotationActionReview(args) => args.run(format),
        ProjectCommands::RouteProposal(args) => args.run(format),
        ProjectCommands::ReviewRouteProposal(args) => args.run(format),
        ProjectCommands::RouteStrategyReport(args) => args.run(format),
        ProjectCommands::RouteStrategyCompare(args) => args.run(format),
        ProjectCommands::RouteStrategyDelta(args) => args.run(format),
        ProjectCommands::WriteRouteStrategyCuratedFixtureSuite(args) => args.run(format),
        ProjectCommands::CaptureRouteStrategyCuratedBaseline(args) => args.run(format),
        ProjectCommands::RouteStrategyBatchEvaluate(args) => args.run(format),
        ProjectCommands::InspectRouteStrategyBatchResult(args) => args.run(format),
        ProjectCommands::ValidateRouteStrategyBatchResult(args) => args.run(format),
        ProjectCommands::CompareRouteStrategyBatchResult(args) => args.run(format),
        ProjectCommands::GateRouteStrategyBatchResult(args) => args.run(format),
        ProjectCommands::SummarizeRouteStrategyBatchResults(args) => args.run(format),
        ProjectCommands::RouteProposalExplain(args) => args.run(format),
        ProjectCommands::ExportRouteProposal(args) => args.run(format),
        ProjectCommands::ExportRoutePathProposal(args) => args.run(format),
        ProjectCommands::InspectRouteProposalArtifact(args) => args.run(format),
        ProjectCommands::RevalidateRouteProposalArtifact(args) => args.run(format),
        ProjectCommands::ApplyRouteProposalArtifact(args) => args.run(format),
        ProjectCommands::RouteApplySelected(args) => args.run(format),
        ProjectCommands::RouteApply(args) => args.run(format),
        ProjectCommands::ExportBom(args) => args.run(format),
        ProjectCommands::CompareBom(args) => args.run(format),
        ProjectCommands::ValidateBom(args) => args.run(format),
        ProjectCommands::InspectBom(args) => args.run(format),
        ProjectCommands::ExportPnp(args) => args.run(format),
        ProjectCommands::ComparePnp(args) => args.run(format),
        ProjectCommands::ValidatePnp(args) => args.run(format),
        ProjectCommands::InspectPnp(args) => args.run(format),
        ProjectCommands::ReviewProposal(args) => args.run(format),
        ProjectCommands::ShowProposal(args) => args.run(format),
        ProjectCommands::ValidateProposal(args) => args.run(format),
        ProjectCommands::DeferProposal(args) => args.run(format),
        ProjectCommands::ApplyProposal(args) => args.run(format),
        ProjectCommands::ImportKicadFootprint(args) => args.run(format),
        ProjectCommands::ImportKicadBoard(args) => args.run(format),
        ProjectCommands::ImportKicadSchematic(args) => args.run(format),
        ProjectCommands::ImportEagleLibrary(args) => args.run(format),
        ProjectCommands::SetPoolPartBindings(args) => args.run(format),
        ProjectCommands::SetPoolSymbolPinAnchor(args) => args.run(format),
        ProjectCommands::CreatePoolFootprint(args) => args.run(format),
        ProjectCommands::GenerateIpc7351bTwoTerminalChip(args) => args.run(format),
        ProjectCommands::SetPoolFootprintPad(args) => args.run(format),
        ProjectCommands::SetPoolFootprintCourtyardRect(args) => args.run(format),
        ProjectCommands::SetPoolFootprintCourtyardPolygon(args) => args.run(format),
        ProjectCommands::AddPoolFootprintSilkscreenLine(args) => args.run(format),
        ProjectCommands::AddPoolFootprintSilkscreenRect(args) => args.run(format),
        ProjectCommands::AddPoolFootprintSilkscreenCircle(args) => args.run(format),
        ProjectCommands::AddPoolFootprintSilkscreenPolygon(args) => args.run(format),
        ProjectCommands::CreatePoolPinPadMap(args) => args.run(format),
        ProjectCommands::SetPoolPinPadMap(args) => args.run(format),
        ProjectCommands::CreatePoolLibraryObject(args) => args.run(format),
        ProjectCommands::CreatePoolUnit(args) => args.run(format),
        ProjectCommands::SetPoolUnitPin(args) => args.run(format),
        ProjectCommands::CreatePoolSymbol(args) => args.run(format),
        ProjectCommands::AddPoolSymbolLine(args) => args.run(format),
        ProjectCommands::AddPoolSymbolPolygon(args) => args.run(format),
        ProjectCommands::AddPoolSymbolRect(args) => args.run(format),
        ProjectCommands::AddPoolSymbolCircle(args) => args.run(format),
        ProjectCommands::AddPoolSymbolText(args) => args.run(format),
        ProjectCommands::AddPoolSymbolArc(args) => args.run(format),
        ProjectCommands::CreatePoolEntity(args) => args.run(format),
        ProjectCommands::CreatePoolPadstack(args) => args.run(format),
        ProjectCommands::CreatePoolPackage(args) => args.run(format),
        ProjectCommands::SetPoolPackagePad(args) => args.run(format),
        ProjectCommands::SetPoolPackageCourtyardRect(args) => args.run(format),
        ProjectCommands::SetPoolPackageCourtyardPolygon(args) => args.run(format),
        ProjectCommands::AddPoolPackageSilkscreenLine(args) => args.run(format),
        ProjectCommands::AddPoolPackageSilkscreenRect(args) => args.run(format),
        ProjectCommands::AddPoolPackageSilkscreenCircle(args) => args.run(format),
        ProjectCommands::AddPoolPackageSilkscreenArc(args) => args.run(format),
        ProjectCommands::AddPoolPackageSilkscreenPolygon(args) => args.run(format),
        ProjectCommands::AddPoolPackageSilkscreenText(args) => args.run(format),
        ProjectCommands::AddPoolPackageModel3d(args) => args.run(format),
        ProjectCommands::SetPoolPackageBodyHeights(args) => args.run(format),
        ProjectCommands::CreatePoolPart(args) => args.run(format),
        ProjectCommands::SetPoolPartMetadata(args) => args.run(format),
        ProjectCommands::SetPoolPartParametric(args) => args.run(format),
        ProjectCommands::SetPoolPartOrderableMpns(args) => args.run(format),
        ProjectCommands::SetPoolPartPackagingOptions(args) => args.run(format),
        ProjectCommands::SetPoolPartBehaviouralModels(args) => args.run(format),
        ProjectCommands::AttachPoolPartModel(args) => args.run(format),
        ProjectCommands::DetachPoolPartModel(args) => args.run(format),
        ProjectCommands::GcPoolModels(args) => args.run(format),
        ProjectCommands::SetPoolPartThermal(args) => args.run(format),
        ProjectCommands::SetPoolPartSupplyChain(args) => args.run(format),
        ProjectCommands::SetPoolPartTags(args) => args.run(format),
        ProjectCommands::SetPoolPartPadMap(args) => args.run(format),
        ProjectCommands::SetPoolPartPadMapEntry(args) => args.run(format),
        ProjectCommands::SetPoolLibraryObject(args) => args.run(format),
        ProjectCommands::DeletePoolLibraryObject(args) => args.run(format),
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
        ProjectCommands::CreateSheet(args) => args.run(format),
        ProjectCommands::DeleteSheet(args) => args.run(format),
        ProjectCommands::RenameSheet(args) => args.run(format),
        ProjectCommands::CreateSheetDefinition(args) => args.run(format),
        ProjectCommands::CreateSheetInstance(args) => args.run(format),
        ProjectCommands::DeleteSheetInstance(args) => args.run(format),
        ProjectCommands::MoveSheetInstance(args) => args.run(format),
        ProjectCommands::BindSheetInstancePort(args) => args.run(format),
        ProjectCommands::UnbindSheetInstancePort(args) => args.run(format),
        ProjectCommands::PlaceLabel(args) => args.run(format),
        ProjectCommands::RenameLabel(args) => args.run(format),
        ProjectCommands::DeleteLabel(args) => args.run(format),
        ProjectCommands::DrawWire(args) => args.run(format),
        ProjectCommands::DeleteWire(args) => args.run(format),
        ProjectCommands::PlaceJunction(args) => args.run(format),
        ProjectCommands::DeleteJunction(args) => args.run(format),
        ProjectCommands::PlacePort(args) => args.run(format),
        ProjectCommands::EditPort(args) => args.run(format),
        ProjectCommands::DeletePort(args) => args.run(format),
        ProjectCommands::CreateBus(args) => args.run(format),
        ProjectCommands::EditBusMembers(args) => args.run(format),
        ProjectCommands::DeleteBus(args) => args.run(format),
        ProjectCommands::PlaceBusEntry(args) => args.run(format),
        ProjectCommands::DeleteBusEntry(args) => args.run(format),
        ProjectCommands::PlaceNoConnect(args) => args.run(format),
        ProjectCommands::DeleteNoConnect(args) => args.run(format),
        ProjectCommands::PlaceSymbol(args) => args.run(format),
        ProjectCommands::MoveSymbol(args) => args.run(format),
        ProjectCommands::RotateSymbol(args) => args.run(format),
        ProjectCommands::MirrorSymbol(args) => args.run(format),
        ProjectCommands::DeleteSymbol(args) => args.run(format),
        ProjectCommands::SetSymbolReference(args) => args.run(format),
        ProjectCommands::SetSymbolValue(args) => args.run(format),
        ProjectCommands::SetSymbolLibId(args) => args.run(format),
        ProjectCommands::ClearSymbolLibId(args) => args.run(format),
        ProjectCommands::SetSymbolEntity(args) => args.run(format),
        ProjectCommands::ClearSymbolEntity(args) => args.run(format),
        ProjectCommands::SetSymbolPart(args) => args.run(format),
        ProjectCommands::ClearSymbolPart(args) => args.run(format),
        ProjectCommands::SetSymbolUnit(args) => args.run(format),
        ProjectCommands::ClearSymbolUnit(args) => args.run(format),
        ProjectCommands::SetSymbolGate(args) => args.run(format),
        ProjectCommands::ClearSymbolGate(args) => args.run(format),
        ProjectCommands::SetSymbolDisplayMode(args) => args.run(format),
        ProjectCommands::SetSymbolHiddenPowerBehavior(args) => args.run(format),
        ProjectCommands::SetPinOverride(args) => args.run(format),
        ProjectCommands::ClearPinOverride(args) => args.run(format),
        ProjectCommands::AddSymbolField(args) => args.run(format),
        ProjectCommands::EditSymbolField(args) => args.run(format),
        ProjectCommands::DeleteSymbolField(args) => args.run(format),
        ProjectCommands::PlaceText(args) => args.run(format),
        ProjectCommands::EditText(args) => args.run(format),
        ProjectCommands::DeleteText(args) => args.run(format),
        ProjectCommands::PlaceDrawingLine(args) => args.run(format),
        ProjectCommands::PlaceDrawingRect(args) => args.run(format),
        ProjectCommands::PlaceDrawingCircle(args) => args.run(format),
        ProjectCommands::PlaceDrawingArc(args) => args.run(format),
        ProjectCommands::EditDrawingLine(args) => args.run(format),
        ProjectCommands::EditDrawingRect(args) => args.run(format),
        ProjectCommands::EditDrawingCircle(args) => args.run(format),
        ProjectCommands::EditDrawingArc(args) => args.run(format),
        ProjectCommands::DeleteDrawing(args) => args.run(format),
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
        ProjectCommands::SetBoardOutline(args) => args.run(format),
        ProjectCommands::SetBoardName(args) => args.run(format),
        ProjectCommands::PlaceBoardComponent(args) => args.run(format),
        ProjectCommands::GenerateBoardComponents(args) => args.run(format),
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
        ProjectCommands::SetBoardStackup(args) => args.run(format),
        ProjectCommands::AddDefaultTopStackup(args) => args.run(format),
        ProjectCommands::PlaceBoardNet(args) => args.run(format),
        ProjectCommands::PlaceBoardNetClass(args) => args.run(format),
        ProjectCommands::EditBoardNetClass(args) => args.run(format),
        ProjectCommands::EditBoardNet(args) => args.run(format),
        ProjectCommands::MoveBoardComponent(args) => args.run(format),
        ProjectCommands::SetBoardComponentPart(args) => args.run(format),
        ProjectCommands::SetBoardComponentPackage(args) => args.run(format),
        ProjectCommands::SetBoardComponentLayer(args)
        | ProjectCommands::FlipBoardComponent(args) => args.run(format),
        ProjectCommands::SetBoardComponentReference(args) => args.run(format),
        ProjectCommands::SetBoardComponentValue(args) => args.run(format),
        ProjectCommands::RotateBoardComponent(args) => args.run(format),
        ProjectCommands::SetBoardComponentLocked(args) => args.run(format),
        ProjectCommands::ClearBoardComponentLocked(args) => args.run(format),
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
        ProjectCommands::DeleteBoardNetClass(args) => args.run(format),
        ProjectCommands::DeleteBoardNet(args) => args.run(format),
    }
}

/// Top-level Commands router (Phase 5 fold of command_exec_dispatch.rs):
/// every subcommand family dispatches to `args.run(format)` inherent
/// methods or the family fn that owns its sub-enum.
pub(crate) fn execute_with_exit_code(cli: Cli) -> Result<(String, i32)> {
    let format = &cli.format;
    match cli.command {
        Commands::Context { action } => match action {
            ContextCommands::Get(args) => {
                Ok((render_output(format, &query_context_envelope(&args)?), 0))
            }
            ContextCommands::Refresh(args) => {
                Ok((render_output(format, &refresh_context_envelope(&args)?), 0))
            }
            ContextCommands::SessionEvents(args) => Ok((
                render_output(format, &query_context_session_events(&args)?),
                0,
            )),
            ContextCommands::SessionActivity(args) => Ok((
                render_output(format, &query_context_session_activity(&args)?),
                0,
            )),
        },
        Commands::Import { path } => {
            let report = import_path(&path)?;
            let view = ImportReportView::from(report);
            Ok((render_output(format, &view), 0))
        }
        Commands::Query { action } => execute_query_command(format, action),
        Commands::Drc { path } => {
            let report = run_drc(Path::new(&path))?;
            let output = match *format {
                OutputFormat::Text => render_drc_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
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
            Ok((render_output(format, &findings), exit_code))
        }
        Commands::Check { action } => match action {
            CheckCommands::Run(args) => args.run(format),
            CheckCommands::List(args) => args.run(format),
            CheckCommands::Show(args) => args.run(format),
            CheckCommands::Profiles(args) => args.run(format),
            CheckCommands::FillZones(args) => args.run(format),
            CheckCommands::RepairStandards(args) => args.run(format),
            CheckCommands::Waive(args) => args.run(format),
            CheckCommands::AcceptDeviation(args) => args.run(format),
            CheckCommands::Imported(args) => args.run(format),
        },
        Commands::Pool { action } => match action {
            PoolCommands::Search { query, libraries } => {
                let results = search_pool(&query, &libraries)?;
                Ok((render_output(format, &results), 0))
            }
        },
        Commands::Proposal { action } => match action {
            ProposalCommands::Create(args) => args.run(format),
            ProposalCommands::CreatePlaceLabel(args) => args.run(format),
            ProposalCommands::CreatePlaceSymbol(args) => args.run(format),
            ProposalCommands::CreateDrawWire(args) => args.run(format),
            ProposalCommands::CreateBoardComponentReplacement(args) => args.run(format),
            ProposalCommands::CreateBoardComponentReplacements(args) => args.run(format),
            ProposalCommands::CreateBoardComponentReplacementPlan(args) => args.run(format),
            ProposalCommands::CreatePoolLibraryObject(args) => args.run(format),
            ProposalCommands::CreatePoolUnit(args) => args.run(format),
            ProposalCommands::CreatePoolSymbol(args) => args.run(format),
            ProposalCommands::CreatePoolEntity(args) => args.run(format),
            ProposalCommands::CreatePoolPadstack(args) => args.run(format),
            ProposalCommands::CreatePoolPackage(args) => args.run(format),
            ProposalCommands::CreatePoolFootprint(args) => args.run(format),
            ProposalCommands::GenerateIpc7351bTwoTerminalChip(args) => args.run(format),
            ProposalCommands::CreatePoolPinPadMap(args) => args.run(format),
            ProposalCommands::SetPoolPinPadMap(args) => args.run(format),
            ProposalCommands::SetPoolFootprintPad(args) => args.run(format),
            ProposalCommands::SetPoolFootprintCourtyardRect(args) => args.run(format),
            ProposalCommands::SetPoolFootprintCourtyardPolygon(args) => args.run(format),
            ProposalCommands::AddPoolFootprintSilkscreenLine(args) => args.run(format),
            ProposalCommands::AddPoolFootprintSilkscreenRect(args) => args.run(format),
            ProposalCommands::AddPoolFootprintSilkscreenCircle(args) => args.run(format),
            ProposalCommands::AddPoolFootprintSilkscreenPolygon(args) => args.run(format),
            ProposalCommands::SetPoolPackagePad(args) => args.run(format),
            ProposalCommands::SetPoolPackageCourtyardRect(args) => args.run(format),
            ProposalCommands::SetPoolPackageCourtyardPolygon(args) => args.run(format),
            ProposalCommands::CreateOutputJob(args) => args.run(format),
            ProposalCommands::UpdateOutputJob(args) => args.run(format),
            ProposalCommands::DeleteOutputJob(args) => args.run(format),
            ProposalCommands::CreateManufacturingPlan(args) => args.run(format),
            ProposalCommands::UpdateManufacturingPlan(args) => args.run(format),
            ProposalCommands::DeleteManufacturingPlan(args) => args.run(format),
            ProposalCommands::CreatePanelProjection(args) => args.run(format),
            ProposalCommands::UpdatePanelProjection(args) => args.run(format),
            ProposalCommands::DeletePanelProjection(args) => args.run(format),
            ProposalCommands::Preview(args) => args.run(format),
            ProposalCommands::List(args) => args.run(format),
            ProposalCommands::Show(args) => args.run(format),
            ProposalCommands::Validate(args) => args.run(format),
            ProposalCommands::Review(args) => args.run(format),
            ProposalCommands::Defer(args) => args.run(format),
            ProposalCommands::Reject(args) => args.run(format),
            ProposalCommands::AcceptApply(args) => args.run_accept_apply(format),
            ProposalCommands::Apply(args) => args.run(format),
        },
        Commands::Journal { action } => match action {
            JournalCommands::List(args) => args.run(format),
            JournalCommands::Show(args) => args.run(format),
            JournalCommands::Undo(args) => args.run(format),
            JournalCommands::Redo(args) => args.run(format),
        },
        Commands::Artifact { action } => match action {
            ArtifactCommands::Generate(args) => args.run(format),
            ArtifactCommands::StartOutputJobRun(args) => args.run(format),
            ArtifactCommands::CancelOutputJobRun(args) => args.run(format),
            ArtifactCommands::List(args) => args.run(format),
            ArtifactCommands::Show(args) => args.run(format),
            ArtifactCommands::Files(args) => args.run(format),
            ArtifactCommands::Preview(args) => args.run(format),
            ArtifactCommands::Compare(args) => args.run(format),
            ArtifactCommands::Validate(args) => args.run(format),
            ArtifactCommands::ExportManufacturingSet(args) => args.run(format),
            ArtifactCommands::ValidateManufacturingSet(args) => args.run(format),
        },
        Commands::Project { action } => execute_project_command(format, *action),
        Commands::Plan { action } => execute_plan_command(format, action),
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
            let output = match *format {
                OutputFormat::Text => render_modify_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
    }
}
