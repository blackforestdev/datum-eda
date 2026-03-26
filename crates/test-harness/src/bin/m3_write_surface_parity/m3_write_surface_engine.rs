use super::*;

#[path = "../m3_write_surface_engine/m3_write_surface_engine_basic.rs"]
mod m3_write_surface_engine_basic;
#[path = "../m3_write_surface_engine/m3_write_surface_engine_replacements.rs"]
mod m3_write_surface_engine_replacements;

pub(super) fn engine_surface_result(cli: &Cli) -> Result<String> {
    let track = m3_write_surface_engine_basic::engine_track_surface_result(cli)?;
    let via = m3_write_surface_engine_basic::engine_via_surface_result(cli)?;
    let component = m3_write_surface_engine_basic::engine_component_surface_result(cli)?;
    let rule = m3_write_surface_engine_basic::engine_rule_surface_result(cli)?;
    let moved = m3_write_surface_engine_basic::engine_move_surface_result(cli)?;
    let rotate = m3_write_surface_engine_basic::engine_rotate_surface_result(cli)?;
    let value = m3_write_surface_engine_basic::engine_value_surface_result(cli)?;
    let reference = m3_write_surface_engine_basic::engine_reference_surface_result(cli)?;
    let assign = m3_write_surface_engine_replacements::engine_assign_part_surface_result(cli)?;
    let package = m3_write_surface_engine_replacements::engine_set_package_surface_result(cli)?;
    let net_class =
        m3_write_surface_engine_replacements::engine_set_net_class_surface_result(cli)?;

    Ok(format!(
        "{track}, {via}, {component}, {rule}, {moved}, {rotate}, {value}, {reference}, {assign}, {package}, {net_class}"
    ))
}
