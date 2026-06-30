use super::*;

pub(crate) fn execute_project_library_pin_pad_map_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::CreatePoolPinPadMap(ProjectCreatePoolPinPadMapArgs {
            path,
            pool,
            map_uuid,
            part_uuid,
            footprint_uuid,
            entries,
            set_default,
        }) => Ok((
            render_output(
                format,
                &create_native_project_pool_pin_pad_map(
                    &path,
                    &pool,
                    map_uuid,
                    part_uuid,
                    footprint_uuid,
                    entries,
                    set_default,
                )?,
            ),
            0,
        )),
        ProjectCommands::SetPoolPinPadMap(ProjectSetPoolPinPadMapArgs {
            path,
            pool,
            map_uuid,
            mode,
            entries,
        }) => Ok((
            render_output(
                format,
                &set_native_project_pool_pin_pad_map(&path, &pool, map_uuid, mode, entries)?,
            ),
            0,
        )),
        _ => unreachable!("unsupported project library PinPadMap command"),
    }
}
