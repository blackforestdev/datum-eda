use super::*;

pub(crate) fn execute_project_library_symbol_pin_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::SetPoolSymbolPinAnchor(ProjectSetPoolSymbolPinAnchorArgs {
            path,
            pool,
            symbol_uuid,
            pin_uuid,
            x_nm,
            y_nm,
            orientation,
            length_nm,
            decoration,
        }) => Ok((
            render_output(
                format,
                &set_native_project_pool_symbol_pin_anchor(
                    &path,
                    &pool,
                    symbol_uuid,
                    pin_uuid,
                    x_nm,
                    y_nm,
                    orientation,
                    length_nm,
                    decoration,
                )?,
            ),
            0,
        )),
        _ => unreachable!("unsupported project library symbol pin command"),
    }
}
