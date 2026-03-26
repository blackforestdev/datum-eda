use super::*;

#[path = "../m3_write_surface_cli/m3_write_surface_cli_assign_package/mod.rs"]
mod m3_write_surface_cli_assign_package;
#[path = "../m3_write_surface_cli/m3_write_surface_cli_core.rs"]
mod m3_write_surface_cli_core;
#[path = "../m3_write_surface_cli/m3_write_surface_cli_motion_netclass.rs"]
mod m3_write_surface_cli_motion_netclass;

pub(super) fn check_cli_surface(cli: &Cli) -> SurfaceCheck {
    match (
        m3_write_surface_cli_core::cli_surface_result(cli),
        m3_write_surface_cli_core::cli_via_surface_result(cli),
        m3_write_surface_cli_core::cli_component_surface_result(cli),
        m3_write_surface_cli_motion_netclass::cli_move_surface_result(cli),
        m3_write_surface_cli_motion_netclass::cli_rotate_surface_result(cli),
        m3_write_surface_cli_core::cli_rule_surface_result(cli),
        m3_write_surface_cli_core::cli_value_surface_result(cli),
        m3_write_surface_cli_core::cli_reference_surface_result(cli),
        m3_write_surface_cli_assign_package::cli_assign_part_surface_result(cli),
        m3_write_surface_cli_assign_package::cli_assign_part_remap_surface_result(cli),
        m3_write_surface_cli_assign_package::cli_set_package_surface_result(cli),
        m3_write_surface_cli_assign_package::cli_set_package_remap_surface_result(cli),
        m3_write_surface_cli_assign_package::cli_set_package_with_part_surface_result(cli),
        m3_write_surface_cli_motion_netclass::cli_set_net_class_surface_result(cli),
    ) {
        (
            Ok(track_evidence),
            Ok(via_evidence),
            Ok(component_evidence),
            Ok(move_evidence),
            Ok(rotate_evidence),
            Ok(rule_evidence),
            Ok(value_evidence),
            Ok(reference_evidence),
            Ok(assign_part_evidence),
            Ok(assign_part_remap_evidence),
            Ok(set_package_evidence),
            Ok(set_package_remap_evidence),
            Ok(set_package_with_part_evidence),
            Ok(net_class_evidence),
        ) => SurfaceCheck {
            surface: "cli_modify_surface".to_string(),
            status: Status::Passed,
            evidence: format!(
                "{track_evidence}; {via_evidence}; {component_evidence}; {move_evidence}; {rotate_evidence}; {rule_evidence}; {value_evidence}; {reference_evidence}; {assign_part_evidence}; {assign_part_remap_evidence}; {set_package_evidence}; {set_package_remap_evidence}; {set_package_with_part_evidence}; {net_class_evidence}"
            ),
        },
        (Err(err), _, _, _, _, _, _, _, _, _, _, _, _, _)
        | (_, Err(err), _, _, _, _, _, _, _, _, _, _, _, _)
        | (_, _, Err(err), _, _, _, _, _, _, _, _, _, _, _)
        | (_, _, _, Err(err), _, _, _, _, _, _, _, _, _, _)
        | (_, _, _, _, Err(err), _, _, _, _, _, _, _, _, _)
        | (_, _, _, _, _, Err(err), _, _, _, _, _, _, _, _)
        | (_, _, _, _, _, _, Err(err), _, _, _, _, _, _, _)
        | (_, _, _, _, _, _, _, Err(err), _, _, _, _, _, _)
        | (_, _, _, _, _, _, _, _, Err(err), _, _, _, _, _)
        | (_, _, _, _, _, _, _, _, _, Err(err), _, _, _, _)
        | (_, _, _, _, _, _, _, _, _, _, Err(err), _, _, _)
        | (_, _, _, _, _, _, _, _, _, _, _, Err(err), _, _)
        | (_, _, _, _, _, _, _, _, _, _, _, _, Err(err), _)
        | (_, _, _, _, _, _, _, _, _, _, _, _, _, Err(err)) => SurfaceCheck {
            surface: "cli_modify_surface".to_string(),
            status: Status::Failed,
            evidence: err.to_string(),
        },
    }
}
