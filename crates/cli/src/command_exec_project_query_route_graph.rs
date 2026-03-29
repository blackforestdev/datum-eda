use super::*;
use crate::cli_args::NativeRoutePathCandidateAuthoredCopperGraphPolicy;

pub(super) fn execute_native_project_route_path_candidate_authored_copper_graph_query(
    format: &OutputFormat,
    path: &Path,
    net: Uuid,
    from_anchor: Uuid,
    to_anchor: Uuid,
    policy: NativeRoutePathCandidateAuthoredCopperGraphPolicy,
) -> Result<(String, i32)> {
    let report = query_native_project_route_path_candidate_authored_copper_graph(
        path,
        net,
        from_anchor,
        to_anchor,
        policy,
    )?;
    let output = match format {
        OutputFormat::Text => render_native_project_route_path_candidate_authored_copper_graph_text(
            &report,
        ),
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_native_project_route_path_candidate_authored_copper_plus_one_gap_query(
    format: &OutputFormat,
    path: &Path,
    net: Uuid,
    from_anchor: Uuid,
    to_anchor: Uuid,
) -> Result<(String, i32)> {
    let report = query_native_project_route_path_candidate_authored_copper_plus_one_gap(
        path,
        net,
        from_anchor,
        to_anchor,
    )?;
    let output = match format {
        OutputFormat::Text => {
            render_native_project_route_path_candidate_authored_copper_plus_one_gap_text(&report)
        }
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_native_project_route_path_candidate_authored_copper_graph_explain_query(
    format: &OutputFormat,
    path: &Path,
    net: Uuid,
    from_anchor: Uuid,
    to_anchor: Uuid,
    policy: NativeRoutePathCandidateAuthoredCopperGraphPolicy,
) -> Result<(String, i32)> {
    let report = query_native_project_route_path_candidate_authored_copper_graph_explain(
        path,
        net,
        from_anchor,
        to_anchor,
        policy,
    )?;
    let output = match format {
        OutputFormat::Text => {
            render_native_project_route_path_candidate_authored_copper_graph_explain_text(&report)
        }
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn execute_native_project_route_path_candidate_authored_copper_graph_zone_aware_query(
    format: &OutputFormat,
    path: &Path,
    net: Uuid,
    from_anchor: Uuid,
    to_anchor: Uuid,
) -> Result<(String, i32)> {
    let report = query_native_project_route_path_candidate_authored_copper_graph_zone_aware(
        path,
        net,
        from_anchor,
        to_anchor,
    )?;
    let output = match format {
        OutputFormat::Text => {
            render_native_project_route_path_candidate_authored_copper_graph_zone_aware_text(
                &report,
            )
        }
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn
    execute_native_project_route_path_candidate_authored_copper_graph_zone_aware_explain_query(
        format: &OutputFormat,
        path: &Path,
        net: Uuid,
        from_anchor: Uuid,
        to_anchor: Uuid,
    ) -> Result<(String, i32)> {
    let report =
        query_native_project_route_path_candidate_authored_copper_graph_zone_aware_explain(
            path,
            net,
            from_anchor,
            to_anchor,
        )?;
    let output = match format {
        OutputFormat::Text => {
            render_native_project_route_path_candidate_authored_copper_graph_zone_aware_explain_text(
                &report,
            )
        }
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn
    execute_native_project_route_path_candidate_authored_copper_graph_obstacle_aware_query(
        format: &OutputFormat,
        path: &Path,
        net: Uuid,
        from_anchor: Uuid,
        to_anchor: Uuid,
    ) -> Result<(String, i32)> {
    let report = query_native_project_route_path_candidate_authored_copper_graph_obstacle_aware(
        path,
        net,
        from_anchor,
        to_anchor,
    )?;
    let output = match format {
        OutputFormat::Text => {
            render_native_project_route_path_candidate_authored_copper_graph_obstacle_aware_text(
                &report,
            )
        }
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn
    execute_native_project_route_path_candidate_authored_copper_graph_obstacle_aware_explain_query(
        format: &OutputFormat,
        path: &Path,
        net: Uuid,
        from_anchor: Uuid,
        to_anchor: Uuid,
    ) -> Result<(String, i32)> {
    let report =
        query_native_project_route_path_candidate_authored_copper_graph_obstacle_aware_explain(
            path,
            net,
            from_anchor,
            to_anchor,
        )?;
    let output = match format {
        OutputFormat::Text => {
            render_native_project_route_path_candidate_authored_copper_graph_obstacle_aware_explain_text(
                &report,
            )
        }
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn
    execute_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_query(
        format: &OutputFormat,
        path: &Path,
        net: Uuid,
        from_anchor: Uuid,
        to_anchor: Uuid,
    ) -> Result<(String, i32)> {
    let report =
        query_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware(
            path,
            net,
            from_anchor,
            to_anchor,
        )?;
    let output = match format {
        OutputFormat::Text => {
            render_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_text(
                &report,
            )
        }
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn
    execute_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_explain_query(
        format: &OutputFormat,
        path: &Path,
        net: Uuid,
        from_anchor: Uuid,
        to_anchor: Uuid,
    ) -> Result<(String, i32)> {
    let report =
        query_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_explain(
            path,
            net,
            from_anchor,
            to_anchor,
        )?;
    let output = match format {
        OutputFormat::Text => {
            render_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_explain_text(
                &report,
            )
        }
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn
    execute_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_query(
        format: &OutputFormat,
        path: &Path,
        net: Uuid,
        from_anchor: Uuid,
        to_anchor: Uuid,
    ) -> Result<(String, i32)> {
    let report =
        query_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware(
            path,
            net,
            from_anchor,
            to_anchor,
        )?;
    let output = match format {
        OutputFormat::Text => {
            render_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_text(
                &report,
            )
        }
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn
    execute_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_explain_query(
        format: &OutputFormat,
        path: &Path,
        net: Uuid,
        from_anchor: Uuid,
        to_anchor: Uuid,
    ) -> Result<(String, i32)> {
    let report =
        query_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_explain(
            path,
            net,
            from_anchor,
            to_anchor,
        )?;
    let output = match format {
        OutputFormat::Text => {
            render_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_explain_text(
                &report,
            )
        }
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn
    execute_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_query(
        format: &OutputFormat,
        path: &Path,
        net: Uuid,
        from_anchor: Uuid,
        to_anchor: Uuid,
    ) -> Result<(String, i32)> {
    let report =
        query_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware(
            path,
            net,
            from_anchor,
            to_anchor,
        )?;
    let output = match format {
        OutputFormat::Text => {
            render_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_text(
                &report,
            )
        }
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}

pub(super) fn
    execute_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_explain_query(
        format: &OutputFormat,
        path: &Path,
        net: Uuid,
        from_anchor: Uuid,
        to_anchor: Uuid,
    ) -> Result<(String, i32)> {
    let report =
        query_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_explain(
            path,
            net,
            from_anchor,
            to_anchor,
        )?;
    let output = match format {
        OutputFormat::Text => {
            render_native_project_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware_explain_text(
                &report,
            )
        }
        OutputFormat::Json => render_output(format, &report),
    };
    Ok((output, 0))
}
