use super::*;

pub(crate) fn execute_project_import_or_part_binding_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::ImportKicadFootprint(_)
        | ProjectCommands::ImportKicadBoard(_)
        | ProjectCommands::ImportKicadSchematic(_)
        | ProjectCommands::ImportEagleLibrary(_) => execute_project_import_command(format, command),
        ProjectCommands::SetPoolPartBindings(_) => {
            execute_project_library_part_bindings_command(format, command)
        }
        ProjectCommands::SetPoolSymbolPinAnchor(_) => {
            execute_project_library_symbol_pin_command(format, command)
        }
        _ => unreachable!("unsupported project import or part binding command"),
    }
}

pub(crate) fn execute_project_library_part_bindings_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::SetPoolPartBindings(ProjectSetPoolPartBindingsArgs {
            path,
            pool,
            part_uuid,
            default_footprint,
            clear_default_footprint,
            default_pin_pad_map,
            clear_default_pin_pad_map,
        }) => Ok((
            render_output(
                format,
                &set_native_project_pool_part_bindings(
                    &path,
                    &pool,
                    part_uuid,
                    default_footprint,
                    clear_default_footprint,
                    default_pin_pad_map,
                    clear_default_pin_pad_map,
                )?,
            ),
            0,
        )),
        _ => unreachable!("unsupported project library part binding command"),
    }
}
