use super::command_exec_project_query_route_graph::*;
use super::*;
use anyhow::anyhow;

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
        NativeProjectQueryCommands::RoutingSubstrate => {
            let report = query_native_project_routing_substrate(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_project_routing_substrate_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectQueryCommands::RoutePreflight { net } => {
            let report = query_native_project_route_preflight(&path, net)?;
            let output = match format {
                OutputFormat::Text => render_native_project_route_preflight_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectQueryCommands::RouteCorridor { net } => {
            let report = query_native_project_route_corridor(&path, net)?;
            let output = match format {
                OutputFormat::Text => render_native_project_route_corridor_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectQueryCommands::RoutePathCandidate {
            net,
            from_anchor,
            to_anchor,
            candidate,
            policy,
        } => execute_native_project_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            candidate,
            policy,
        ),
        NativeProjectQueryCommands::RoutePathCandidateVia {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateVia,
            None,
            "route-path-candidate --candidate route-path-candidate-via",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateTwoVia {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateTwoVia,
            None,
            "route-path-candidate --candidate route-path-candidate-two-via",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateThreeVia {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateThreeVia,
            None,
            "route-path-candidate --candidate route-path-candidate-three-via",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateFourVia {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateFourVia,
            None,
            "route-path-candidate --candidate route-path-candidate-four-via",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateFiveVia {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateFiveVia,
            None,
            "route-path-candidate --candidate route-path-candidate-five-via",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateSixVia {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateSixVia,
            None,
            "route-path-candidate --candidate route-path-candidate-six-via",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredViaChain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateAuthoredViaChain,
            None,
            "route-path-candidate --candidate route-path-candidate-authored-via-chain",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredViaChainExplain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateAuthoredViaChain,
            None,
            "route-path-candidate-explain --candidate route-path-candidate-authored-via-chain",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperGraph {
            net,
            from_anchor,
            to_anchor,
            policy,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            Some(policy),
            "route-path-candidate --candidate authored-copper-graph --policy <policy>",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperPlusOneGap {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperPlusOneGap,
            None,
            "route-path-candidate --candidate authored-copper-plus-one-gap",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperPlusOneGapExplain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperPlusOneGap,
            None,
            "route-path-candidate-explain --candidate authored-copper-plus-one-gap",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperGraphExplain {
            net,
            from_anchor,
            to_anchor,
            policy,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            Some(policy),
            "route-path-candidate-explain --candidate authored-copper-graph --policy <policy>",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperGraphZoneAware {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware),
            "route-path-candidate --candidate authored-copper-graph --policy zone_aware",
            Some((
                "m5_route_path_candidate_authored_copper_graph_policy_v1",
                "m5_route_path_candidate_authored_copper_graph_zone_aware_v1",
            )),
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperGraphZoneAwareExplain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware),
            "route-path-candidate-explain --candidate authored-copper-graph --policy zone_aware",
            Some((
                "m5_route_path_candidate_authored_copper_graph_policy_explain_v1",
                "m5_route_path_candidate_authored_copper_graph_zone_aware_explain_v1",
            )),
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperGraphZoneObstacleAware {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware),
            "route-path-candidate --candidate authored-copper-graph --policy zone_obstacle_aware",
            Some((
                "m5_route_path_candidate_authored_copper_graph_policy_v1",
                "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_v1",
            )),
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareExplain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware),
            "route-path-candidate-explain --candidate authored-copper-graph --policy zone_obstacle_aware",
            Some((
                "m5_route_path_candidate_authored_copper_graph_policy_explain_v1",
                "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_explain_v1",
            )),
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAware {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware),
            "route-path-candidate --candidate authored-copper-graph --policy zone_obstacle_topology_aware",
            Some((
                "m5_route_path_candidate_authored_copper_graph_policy_v1",
                "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_v1",
            )),
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareExplain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware),
            "route-path-candidate-explain --candidate authored-copper-graph --policy zone_obstacle_topology_aware",
            Some((
                "m5_route_path_candidate_authored_copper_graph_policy_explain_v1",
                "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_explain_v1",
            )),
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAware {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            Some(
                NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware,
            ),
            "route-path-candidate --candidate authored-copper-graph --policy zone_obstacle_topology_layer_balance_aware",
            Some((
                "m5_route_path_candidate_authored_copper_graph_policy_v1",
                "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_v1",
            )),
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareTopologyAwareLayerBalanceAwareExplain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            Some(
                NativeRoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware,
            ),
            "route-path-candidate-explain --candidate authored-copper-graph --policy zone_obstacle_topology_layer_balance_aware",
            Some((
                "m5_route_path_candidate_authored_copper_graph_policy_explain_v1",
                "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_explain_v1",
            )),
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperGraphObstacleAware {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware),
            "route-path-candidate --candidate authored-copper-graph --policy obstacle_aware",
            Some((
                "m5_route_path_candidate_authored_copper_graph_policy_v1",
                "m5_route_path_candidate_authored_copper_graph_obstacle_aware_v1",
            )),
        ),
        NativeProjectQueryCommands::RoutePathCandidateAuthoredCopperGraphObstacleAwareExplain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph,
            Some(NativeRoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware),
            "route-path-candidate-explain --candidate authored-copper-graph --policy obstacle_aware",
            Some((
                "m5_route_path_candidate_authored_copper_graph_policy_explain_v1",
                "m5_route_path_candidate_authored_copper_graph_obstacle_aware_explain_v1",
            )),
        ),
        NativeProjectQueryCommands::RoutePathCandidateSixViaExplain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateSixVia,
            None,
            "route-path-candidate-explain --candidate route-path-candidate-six-via",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateFiveViaExplain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateFiveVia,
            None,
            "route-path-candidate-explain --candidate route-path-candidate-five-via",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateFourViaExplain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateFourVia,
            None,
            "route-path-candidate-explain --candidate route-path-candidate-four-via",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateThreeViaExplain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateThreeVia,
            None,
            "route-path-candidate-explain --candidate route-path-candidate-three-via",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateTwoViaExplain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateTwoVia,
            None,
            "route-path-candidate-explain --candidate route-path-candidate-two-via",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateViaExplain {
            net,
            from_anchor,
            to_anchor,
        } => execute_legacy_route_path_candidate_explain_wrapper(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            NativeProjectRouteApplyCandidateArg::RoutePathCandidateVia,
            None,
            "route-path-candidate-explain --candidate route-path-candidate-via",
            None,
        ),
        NativeProjectQueryCommands::RoutePathCandidateExplain {
            net,
            from_anchor,
            to_anchor,
            candidate,
            policy,
        } => execute_native_project_route_path_candidate_explain_query(
            format,
            &path,
            net,
            from_anchor,
            to_anchor,
            candidate,
            policy,
        ),
        NativeProjectQueryCommands::BoardComponents => {
            let report = query_native_project_board_component_views(&path)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardComponent(args) => {
            let report = query_native_project_board_component_view(&path, args.component_uuid)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardComponentModels3d(args) => {
            let report =
                query_native_project_board_component_models_3d(&path, args.component_uuid)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardComponentPads(args) => {
            let report = query_native_project_board_component_pads(&path, args.component_uuid)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardComponentSilkscreen(args) => {
            let report =
                query_native_project_board_component_silkscreen(&path, args.component_uuid)?;
            Ok((render_output(format, &report), 0))
        }
        NativeProjectQueryCommands::BoardComponentMechanical(args) => {
            let report =
                query_native_project_board_component_mechanical(&path, args.component_uuid)?;
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

fn annotate_legacy_route_query_result(
    format: &OutputFormat,
    result: Result<(String, i32)>,
    replacement: &str,
    contract_rewrite: Option<(&str, &str)>,
) -> Result<(String, i32)> {
    result.map(|(output, code)| {
        (
            annotate_legacy_route_query_output(format, output, replacement, contract_rewrite),
            code,
        )
    })
}

fn annotate_legacy_route_query_output(
    format: &OutputFormat,
    output: String,
    replacement: &str,
    contract_rewrite: Option<(&str, &str)>,
) -> String {
    let output = match contract_rewrite {
        Some((from, to)) => output.replace(from, to),
        None => output,
    };
    match format {
        OutputFormat::Text => format!(
            "{output}\nnote: deprecated compatibility wrapper; prefer `project query <dir> {replacement}`"
        ),
        OutputFormat::Json => output,
    }
}

fn execute_legacy_route_path_candidate_query(
    format: &OutputFormat,
    path: &Path,
    net: Uuid,
    from_anchor: Uuid,
    to_anchor: Uuid,
    candidate: NativeProjectRouteApplyCandidateArg,
    policy: Option<NativeRoutePathCandidateAuthoredCopperGraphPolicy>,
    replacement: &str,
    contract_rewrite: Option<(&str, &str)>,
) -> Result<(String, i32)> {
    annotate_legacy_route_query_result(
        format,
        execute_native_project_route_path_candidate_query(
            format,
            path,
            net,
            from_anchor,
            to_anchor,
            candidate,
            policy,
        ),
        replacement,
        contract_rewrite,
    )
}

fn execute_legacy_route_path_candidate_explain_wrapper(
    format: &OutputFormat,
    path: &Path,
    net: Uuid,
    from_anchor: Uuid,
    to_anchor: Uuid,
    candidate: NativeProjectRouteApplyCandidateArg,
    policy: Option<NativeRoutePathCandidateAuthoredCopperGraphPolicy>,
    replacement: &str,
    contract_rewrite: Option<(&str, &str)>,
) -> Result<(String, i32)> {
    annotate_legacy_route_query_result(
        format,
        execute_native_project_route_path_candidate_explain_query(
            format,
            path,
            net,
            from_anchor,
            to_anchor,
            candidate,
            policy,
        ),
        replacement,
        contract_rewrite,
    )
}

fn execute_native_project_route_path_candidate_query(
    format: &OutputFormat,
    path: &Path,
    net: Uuid,
    from_anchor: Uuid,
    to_anchor: Uuid,
    candidate: NativeProjectRouteApplyCandidateArg,
    policy: Option<NativeRoutePathCandidateAuthoredCopperGraphPolicy>,
) -> Result<(String, i32)> {
    match candidate {
        NativeProjectRouteApplyCandidateArg::RoutePathCandidate => {
            let report =
                query_native_project_route_path_candidate(path, net, from_anchor, to_anchor)?;
            let output = match format {
                OutputFormat::Text => render_native_project_route_path_candidate_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateVia => {
            let report =
                query_native_project_route_path_candidate_via(path, net, from_anchor, to_anchor)?;
            let output = match format {
                OutputFormat::Text => render_native_project_route_path_candidate_via_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateTwoVia => {
            let report = query_native_project_route_path_candidate_two_via(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_two_via_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateThreeVia => {
            let report = query_native_project_route_path_candidate_three_via(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_three_via_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFourVia => {
            let report = query_native_project_route_path_candidate_four_via(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_four_via_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFiveVia => {
            let report = query_native_project_route_path_candidate_five_via(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_five_via_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateSixVia => {
            let report = query_native_project_route_path_candidate_six_via(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_six_via_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateAuthoredViaChain => {
            let report = query_native_project_route_path_candidate_authored_via_chain(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_authored_via_chain_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalDogleg => {
            let report = query_native_project_route_path_candidate_orthogonal_dogleg(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_dogleg_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalTwoBend => {
            let report = query_native_project_route_path_candidate_orthogonal_two_bend(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_two_bend_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraph => {
            let report = query_native_project_route_path_candidate_orthogonal_graph(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_graph_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphVia => {
            let report = query_native_project_route_path_candidate_orthogonal_graph_via(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_graph_via_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphTwoVia => {
            let report = query_native_project_route_path_candidate_orthogonal_graph_two_via(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_graph_two_via_text(
                        &report,
                    )
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphThreeVia => {
            let report = query_native_project_route_path_candidate_orthogonal_graph_three_via(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_graph_three_via_text(
                        &report,
                    )
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFourVia => {
            let report = query_native_project_route_path_candidate_orthogonal_graph_four_via(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_graph_four_via_text(
                        &report,
                    )
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFiveVia => {
            let report = query_native_project_route_path_candidate_orthogonal_graph_five_via(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_graph_five_via_text(
                        &report,
                    )
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphSixVia => {
            let report = query_native_project_route_path_candidate_orthogonal_graph_six_via(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_graph_six_via_text(
                        &report,
                    )
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::AuthoredCopperPlusOneGap => {
            execute_native_project_route_path_candidate_authored_copper_plus_one_gap_query(
                format,
                path,
                net,
                from_anchor,
                to_anchor,
            )
        }
        NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph => {
            let policy = policy.ok_or_else(|| {
                anyhow!(
                    "`--policy` is required when `--candidate authored-copper-graph` is selected"
                )
            })?;
            execute_native_project_route_path_candidate_authored_copper_graph_query(
                format,
                path,
                net,
                from_anchor,
                to_anchor,
                policy,
            )
        }
    }
}

fn execute_native_project_route_path_candidate_explain_query(
    format: &OutputFormat,
    path: &Path,
    net: Uuid,
    from_anchor: Uuid,
    to_anchor: Uuid,
    candidate: NativeProjectRouteApplyCandidateArg,
    policy: Option<NativeRoutePathCandidateAuthoredCopperGraphPolicy>,
) -> Result<(String, i32)> {
    match candidate {
        NativeProjectRouteApplyCandidateArg::RoutePathCandidate => {
            let report = query_native_project_route_path_candidate_explain(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_explain_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateVia => {
            let report = query_native_project_route_path_candidate_via_explain(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_via_explain_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateTwoVia => {
            let report = query_native_project_route_path_candidate_two_via_explain(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_two_via_explain_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateThreeVia => {
            let report = query_native_project_route_path_candidate_three_via_explain(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_three_via_explain_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFourVia => {
            let report = query_native_project_route_path_candidate_four_via_explain(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_four_via_explain_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateFiveVia => {
            let report = query_native_project_route_path_candidate_five_via_explain(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_five_via_explain_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateSixVia => {
            let report = query_native_project_route_path_candidate_six_via_explain(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_six_via_explain_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateAuthoredViaChain => {
            let report = query_native_project_route_path_candidate_authored_via_chain_explain(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_authored_via_chain_explain_text(
                        &report,
                    )
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalDogleg => {
            let report = query_native_project_route_path_candidate_orthogonal_dogleg_explain(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_dogleg_explain_text(
                        &report,
                    )
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalTwoBend => {
            let report = query_native_project_route_path_candidate_orthogonal_two_bend_explain(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_two_bend_explain_text(
                        &report,
                    )
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraph => {
            let report = query_native_project_route_path_candidate_orthogonal_graph_explain(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_graph_explain_text(
                        &report,
                    )
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphVia => {
            let report = query_native_project_route_path_candidate_orthogonal_graph_via_explain(
                path,
                net,
                from_anchor,
                to_anchor,
            )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_graph_via_explain_text(
                        &report,
                    )
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphTwoVia => {
            let report =
                query_native_project_route_path_candidate_orthogonal_graph_two_via_explain(
                    path,
                    net,
                    from_anchor,
                    to_anchor,
                )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_graph_two_via_explain_text(
                        &report,
                    )
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphThreeVia => {
            let report =
                query_native_project_route_path_candidate_orthogonal_graph_three_via_explain(
                    path,
                    net,
                    from_anchor,
                    to_anchor,
                )?;
            let output = match format {
                OutputFormat::Text => render_native_project_route_path_candidate_orthogonal_graph_three_via_explain_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFourVia => {
            let report =
                query_native_project_route_path_candidate_orthogonal_graph_four_via_explain(
                    path,
                    net,
                    from_anchor,
                    to_anchor,
                )?;
            let output = match format {
                OutputFormat::Text => render_native_project_route_path_candidate_orthogonal_graph_four_via_explain_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphFiveVia => {
            let report =
                query_native_project_route_path_candidate_orthogonal_graph_five_via_explain(
                    path,
                    net,
                    from_anchor,
                    to_anchor,
                )?;
            let output = match format {
                OutputFormat::Text => render_native_project_route_path_candidate_orthogonal_graph_five_via_explain_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::RoutePathCandidateOrthogonalGraphSixVia => {
            let report =
                query_native_project_route_path_candidate_orthogonal_graph_six_via_explain(
                    path,
                    net,
                    from_anchor,
                    to_anchor,
                )?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_project_route_path_candidate_orthogonal_graph_six_via_explain_text(
                        &report,
                    )
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        NativeProjectRouteApplyCandidateArg::AuthoredCopperPlusOneGap => {
            execute_native_project_route_path_candidate_authored_copper_plus_one_gap_explain_query(
                format,
                path,
                net,
                from_anchor,
                to_anchor,
            )
        }
        NativeProjectRouteApplyCandidateArg::AuthoredCopperGraph => {
            let policy = policy.ok_or_else(|| {
                anyhow!(
                    "`--policy` is required when `--candidate authored-copper-graph` is selected"
                )
            })?;
            execute_native_project_route_path_candidate_authored_copper_graph_explain_query(
                format,
                path,
                net,
                from_anchor,
                to_anchor,
                policy,
            )
        }
    }
}
