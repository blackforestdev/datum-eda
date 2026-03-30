use super::*;

pub(super) fn execute_route_proposal_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::ExportRouteProposal(ProjectExportRouteProposalArgs {
            path,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            out,
        }) => {
            let report = export_native_project_route_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
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
        ProjectCommands::ExportRoutePathCandidateProposal(
            ProjectExportRoutePathCandidateProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                out,
            },
        ) => {
            let report = export_native_project_route_path_candidate_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathCandidateViaProposal(
            ProjectExportRoutePathCandidateViaProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                out,
            },
        ) => {
            let report = export_native_project_route_path_candidate_via_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathCandidateTwoViaProposal(
            ProjectExportRoutePathCandidateTwoViaProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                out,
            },
        ) => {
            let report = export_native_project_route_path_candidate_two_via_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathCandidateThreeViaProposal(
            ProjectExportRoutePathCandidateThreeViaProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                out,
            },
        ) => {
            let report = export_native_project_route_path_candidate_three_via_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathCandidateFourViaProposal(
            ProjectExportRoutePathCandidateFourViaProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                out,
            },
        ) => {
            let report = export_native_project_route_path_candidate_four_via_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathCandidateFiveViaProposal(
            ProjectExportRoutePathCandidateFiveViaProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                out,
            },
        ) => {
            let report = export_native_project_route_path_candidate_five_via_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathCandidateSixViaProposal(
            ProjectExportRoutePathCandidateSixViaProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                out,
            },
        ) => {
            let report = export_native_project_route_path_candidate_six_via_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathCandidateAuthoredViaChainProposal(
            ProjectExportRoutePathCandidateAuthoredViaChainProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                out,
            },
        ) => {
            let report = export_native_project_route_path_candidate_authored_via_chain_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathCandidateAuthoredCopperGraphZoneAwareProposal(
            ProjectExportRoutePathCandidateAuthoredCopperGraphZoneAwareProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                out,
            },
        ) => {
            let report =
                export_native_project_route_path_candidate_authored_copper_graph_zone_aware_proposal(
                    &path,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    &out,
                )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathCandidateAuthoredCopperGraphZoneObstacleAwareProposal(
            ProjectExportRoutePathCandidateAuthoredCopperGraphZoneObstacleAwareProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                out,
            },
        ) => {
            let report = export_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareProposal(
            ProjectExportRoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                out,
            },
        ) => {
            let report = export_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareProposal(
            ProjectExportRoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                out,
            },
        ) => {
            let report = export_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathCandidateAuthoredCopperGraphObstacleAwareProposal(
            ProjectExportRoutePathCandidateAuthoredCopperGraphObstacleAwareProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                out,
            },
        ) => {
            let report =
                export_native_project_route_path_candidate_authored_copper_graph_obstacle_aware_proposal(
                    &path,
                    net_uuid,
                    from_anchor_pad_uuid,
                    to_anchor_pad_uuid,
                    &out,
                )?;
            let output = match format {
                OutputFormat::Text => render_native_route_proposal_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportRoutePathCandidateAuthoredCopperGraphProposal(
            ProjectExportRoutePathCandidateAuthoredCopperGraphProposalArgs {
                path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
                policy,
                out,
            },
        ) => {
            let report = export_native_project_route_path_candidate_authored_copper_graph_proposal(
                &path,
                net_uuid,
                from_anchor_pad_uuid,
                to_anchor_pad_uuid,
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
