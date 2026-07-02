use super::*;

pub(crate) fn execute_project_library_footprint_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::CreatePoolFootprint(ProjectCreatePoolFootprintArgs {
            path,
            pool,
            footprint_uuid,
            package_uuid,
            name,
        }) => Ok((
            render_output(
                format,
                &create_native_project_pool_footprint(
                    &path,
                    &pool,
                    footprint_uuid,
                    package_uuid,
                    name,
                )?,
            ),
            0,
        )),
        ProjectCommands::GenerateIpc7351bTwoTerminalChip(
            ProjectGenerateIpc7351bTwoTerminalChipArgs {
                path,
                pool,
                footprint_uuid,
                package_uuid,
                padstack_uuid,
                pad_a_uuid,
                pad_b_uuid,
                name,
                metric_code,
                body_length_nm,
                body_width_nm,
                terminal_length_nm,
                terminal_width_nm,
                density,
                mask_expansion_nm,
                paste_reduction_nm,
            },
        ) => Ok((
            render_output(
                format,
                &generate_native_project_ipc7351b_two_terminal_chip(
                    &path,
                    &pool,
                    footprint_uuid,
                    package_uuid,
                    padstack_uuid,
                    pad_a_uuid,
                    pad_b_uuid,
                    name,
                    metric_code,
                    body_length_nm,
                    body_width_nm,
                    terminal_length_nm,
                    terminal_width_nm,
                    density,
                    mask_expansion_nm,
                    paste_reduction_nm,
                )?,
            ),
            0,
        )),
        ProjectCommands::SetPoolFootprintPad(ProjectSetPoolFootprintPadArgs {
            path,
            pool,
            footprint_uuid,
            pad_uuid,
            padstack_uuid,
            pad_name,
            x_nm,
            y_nm,
            layer,
        }) => Ok((
            render_output(
                format,
                &set_native_project_pool_footprint_pad(
                    &path,
                    &pool,
                    footprint_uuid,
                    pad_uuid,
                    padstack_uuid,
                    pad_name,
                    x_nm,
                    y_nm,
                    layer,
                )?,
            ),
            0,
        )),
        ProjectCommands::SetPoolFootprintCourtyardRect(
            ProjectSetPoolFootprintCourtyardRectArgs {
                path,
                pool,
                footprint_uuid,
                min_x_nm,
                min_y_nm,
                max_x_nm,
                max_y_nm,
            },
        ) => Ok((
            render_output(
                format,
                &set_native_project_pool_footprint_courtyard_rect(
                    &path,
                    &pool,
                    footprint_uuid,
                    min_x_nm,
                    min_y_nm,
                    max_x_nm,
                    max_y_nm,
                )?,
            ),
            0,
        )),
        ProjectCommands::SetPoolFootprintCourtyardPolygon(
            ProjectSetPoolFootprintCourtyardPolygonArgs {
                path,
                pool,
                footprint_uuid,
                vertices,
            },
        ) => Ok((
            render_output(
                format,
                &set_native_project_pool_footprint_courtyard_polygon(
                    &path,
                    &pool,
                    footprint_uuid,
                    &vertices,
                )?,
            ),
            0,
        )),
        ProjectCommands::AddPoolFootprintSilkscreenLine(
            ProjectAddPoolFootprintSilkscreenLineArgs {
                path,
                pool,
                footprint_uuid,
                from_x_nm,
                from_y_nm,
                to_x_nm,
                to_y_nm,
                width_nm,
            },
        ) => Ok((
            render_output(
                format,
                &add_native_project_pool_footprint_silkscreen_line(
                    &path,
                    &pool,
                    footprint_uuid,
                    from_x_nm,
                    from_y_nm,
                    to_x_nm,
                    to_y_nm,
                    width_nm,
                )?,
            ),
            0,
        )),
        ProjectCommands::AddPoolFootprintSilkscreenRect(
            ProjectAddPoolFootprintSilkscreenRectArgs {
                path,
                pool,
                footprint_uuid,
                min_x_nm,
                min_y_nm,
                max_x_nm,
                max_y_nm,
                width_nm,
            },
        ) => Ok((
            render_output(
                format,
                &add_native_project_pool_footprint_silkscreen_rect(
                    &path,
                    &pool,
                    footprint_uuid,
                    min_x_nm,
                    min_y_nm,
                    max_x_nm,
                    max_y_nm,
                    width_nm,
                )?,
            ),
            0,
        )),
        ProjectCommands::AddPoolFootprintSilkscreenCircle(
            ProjectAddPoolFootprintSilkscreenCircleArgs {
                path,
                pool,
                footprint_uuid,
                center_x_nm,
                center_y_nm,
                radius_nm,
                width_nm,
            },
        ) => Ok((
            render_output(
                format,
                &add_native_project_pool_footprint_silkscreen_circle(
                    &path,
                    &pool,
                    footprint_uuid,
                    center_x_nm,
                    center_y_nm,
                    radius_nm,
                    width_nm,
                )?,
            ),
            0,
        )),
        ProjectCommands::AddPoolFootprintSilkscreenPolygon(
            ProjectAddPoolFootprintSilkscreenPolygonArgs {
                path,
                pool,
                footprint_uuid,
                vertices,
                closed,
                width_nm,
            },
        ) => Ok((
            render_output(
                format,
                &add_native_project_pool_footprint_silkscreen_polygon(
                    &path,
                    &pool,
                    footprint_uuid,
                    &vertices,
                    closed,
                    width_nm,
                )?,
            ),
            0,
        )),
        _ => unreachable!("unsupported project library Footprint command"),
    }
}
