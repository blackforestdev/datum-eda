use super::*;

pub(super) fn execute_route_proposal_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::RouteStrategyReport(ProjectRouteStrategyReportArgs {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            objective,
        }) => {
            let report = report_native_project_route_strategy(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                objective,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_strategy_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::RouteStrategyCompare(ProjectRouteStrategyCompareArgs {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        }) => {
            let report = compare_native_project_route_strategy(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_strategy_compare_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::RouteStrategyDelta(ProjectRouteStrategyDeltaArgs {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        }) => {
            let report = report_native_project_route_strategy_delta(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_strategy_delta_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::RouteStrategyBatchEvaluate(ProjectRouteStrategyBatchEvaluateArgs {
            requests,
        }) => {
            let report = evaluate_native_project_route_strategy_batch(&requests)?;
            let output = match format {
                OutputFormat::Text => render_native_route_strategy_batch_evaluate_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::InspectRouteStrategyBatchResult(
            ProjectInspectRouteStrategyBatchResultArgs { path },
        ) => {
            let report = inspect_route_strategy_batch_result(&path)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_route_strategy_batch_result_inspection_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ValidateRouteStrategyBatchResult(
            ProjectValidateRouteStrategyBatchResultArgs { path },
        ) => {
            let report = validate_route_strategy_batch_result(&path)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_route_strategy_batch_result_validation_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::RouteProposal(ProjectRouteProposalArgs {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
        }) => {
            let report = select_native_project_route_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                profile,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_selection_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::RouteProposalExplain(ProjectRouteProposalExplainArgs {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
        }) => {
            let report = explain_native_project_route_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                profile,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_explain_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRouteProposal(ProjectExportRouteProposalArgs {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
            out,
        }) => {
            let report = export_selected_native_project_route_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                profile,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_selected_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathProposal(ProjectExportRoutePathProposalArgs {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate,
            policy,
            out,
        }) => {
            let report = export_native_project_route_path_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                candidate,
                policy,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::InspectRouteProposalArtifact(
            ProjectInspectRouteProposalArtifactArgs { path },
        ) => {
            let report = inspect_route_proposal_artifact(&path)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_route_proposal_artifact_inspection_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::RevalidateRouteProposalArtifact(
            ProjectRevalidateRouteProposalArtifactArgs { path, artifact },
        ) => {
            let report = revalidate_route_proposal_artifact(&path, &artifact)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_route_proposal_artifact_revalidation_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ApplyRouteProposalArtifact(ProjectApplyRouteProposalArtifactArgs {
            path,
            artifact,
        }) => {
            let report = apply_route_proposal_artifact(&path, &artifact)?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_artifact_apply_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::RouteApplySelected(ProjectRouteApplySelectedArgs {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            profile,
        }) => {
            let report = apply_selected_native_project_route(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                profile,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_apply_selected_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::RouteApply(ProjectRouteApplyArgs {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate,
            policy,
        }) => {
            let report = apply_native_project_route(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                candidate,
                policy,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_apply_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        _ => unreachable!("non-route-proposal command passed to dispatcher"),
    }
}
