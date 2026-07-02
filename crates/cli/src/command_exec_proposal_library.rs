use super::*;

pub(super) fn execute_create_pool_library_object_proposal(
    format: &OutputFormat,
    args: ProposalCreatePoolLibraryObjectArgs,
) -> Result<(String, i32)> {
    let ProposalCreatePoolLibraryObjectArgs {
        path,
        pool,
        kind,
        object,
        from_json,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_create_native_project_pool_library_object(
                &path,
                &pool,
                &kind,
                object,
                &from_json,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_create_pool_unit_proposal(
    format: &OutputFormat,
    args: ProposalCreatePoolUnitArgs,
) -> Result<(String, i32)> {
    let ProposalCreatePoolUnitArgs {
        path,
        pool,
        unit,
        name,
        manufacturer,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_create_native_project_pool_unit(
                &path,
                &pool,
                unit,
                name,
                manufacturer,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_create_pool_symbol_proposal(
    format: &OutputFormat,
    args: ProposalCreatePoolSymbolArgs,
) -> Result<(String, i32)> {
    let ProposalCreatePoolSymbolArgs {
        path,
        pool,
        symbol,
        unit,
        name,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_create_native_project_pool_symbol(
                &path,
                &pool,
                symbol,
                unit,
                name,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_create_pool_entity_proposal(
    format: &OutputFormat,
    args: ProposalCreatePoolEntityArgs,
) -> Result<(String, i32)> {
    let ProposalCreatePoolEntityArgs {
        path,
        pool,
        entity,
        gate,
        unit,
        symbol,
        name,
        prefix,
        manufacturer,
        gate_name,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_create_native_project_pool_entity(
                &path,
                &pool,
                entity,
                gate,
                unit,
                symbol,
                name,
                prefix,
                manufacturer,
                gate_name,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_create_pool_padstack_proposal(
    format: &OutputFormat,
    args: ProposalCreatePoolPadstackArgs,
) -> Result<(String, i32)> {
    let ProposalCreatePoolPadstackArgs {
        path,
        pool,
        padstack,
        name,
        aperture,
        diameter_nm,
        width_nm,
        height_nm,
        drill_nm,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_create_native_project_pool_padstack(
                &path,
                &pool,
                padstack,
                name,
                aperture,
                diameter_nm,
                width_nm,
                height_nm,
                drill_nm,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_create_pool_package_proposal(
    format: &OutputFormat,
    args: ProposalCreatePoolPackageArgs,
) -> Result<(String, i32)> {
    let ProposalCreatePoolPackageArgs {
        path,
        pool,
        package,
        name,
        pad,
        padstack,
        pad_name,
        x_nm,
        y_nm,
        layer,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_create_native_project_pool_package(
                &path,
                &pool,
                package,
                name,
                pad,
                padstack,
                pad_name,
                x_nm,
                y_nm,
                layer,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_create_pool_footprint_proposal(
    format: &OutputFormat,
    args: ProposalCreatePoolFootprintArgs,
) -> Result<(String, i32)> {
    let ProposalCreatePoolFootprintArgs {
        path,
        pool,
        footprint,
        package,
        name,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_create_native_project_pool_footprint(
                &path,
                &pool,
                footprint,
                package,
                name,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_generate_ipc7351b_two_terminal_chip_proposal(
    format: &OutputFormat,
    args: ProposalGenerateIpc7351bTwoTerminalChipArgs,
) -> Result<(String, i32)> {
    let ProposalGenerateIpc7351bTwoTerminalChipArgs {
        path,
        pool,
        footprint,
        package,
        padstack,
        pad_a,
        pad_b,
        name,
        metric_code,
        body_length_nm,
        body_width_nm,
        terminal_length_nm,
        terminal_width_nm,
        density,
        mask_expansion_nm,
        paste_reduction_nm,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_generate_native_project_ipc7351b_two_terminal_chip(
                &path,
                &pool,
                footprint,
                package,
                padstack,
                pad_a,
                pad_b,
                name,
                metric_code,
                body_length_nm,
                body_width_nm,
                terminal_length_nm,
                terminal_width_nm,
                density,
                mask_expansion_nm,
                paste_reduction_nm,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_create_pool_pin_pad_map_proposal(
    format: &OutputFormat,
    args: ProposalCreatePoolPinPadMapArgs,
) -> Result<(String, i32)> {
    let ProposalCreatePoolPinPadMapArgs {
        path,
        pool,
        map_uuid,
        part_uuid,
        footprint_uuid,
        entries,
        set_default,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_create_native_project_pool_pin_pad_map(
                &path,
                &pool,
                map_uuid,
                part_uuid,
                footprint_uuid,
                entries,
                set_default,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_set_pool_pin_pad_map_proposal(
    format: &OutputFormat,
    args: ProposalSetPoolPinPadMapArgs,
) -> Result<(String, i32)> {
    let ProposalSetPoolPinPadMapArgs {
        path,
        pool,
        map_uuid,
        mode,
        entries,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_set_native_project_pool_pin_pad_map(
                &path,
                &pool,
                map_uuid,
                mode,
                entries,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_set_pool_footprint_pad_proposal(
    format: &OutputFormat,
    args: ProposalSetPoolFootprintPadArgs,
) -> Result<(String, i32)> {
    let ProposalSetPoolFootprintPadArgs {
        path,
        pool,
        footprint,
        pad,
        padstack,
        pad_name,
        x_nm,
        y_nm,
        layer,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_set_native_project_pool_footprint_pad(
                &path,
                &pool,
                footprint,
                pad,
                padstack,
                pad_name,
                x_nm,
                y_nm,
                layer,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_set_pool_package_pad_proposal(
    format: &OutputFormat,
    args: ProposalSetPoolPackagePadArgs,
) -> Result<(String, i32)> {
    let ProposalSetPoolPackagePadArgs {
        path,
        pool,
        package,
        pad,
        padstack,
        pad_name,
        x_nm,
        y_nm,
        layer,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_set_native_project_pool_package_pad(
                &path,
                &pool,
                package,
                pad,
                padstack,
                pad_name,
                x_nm,
                y_nm,
                layer,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_set_pool_footprint_courtyard_rect_proposal(
    format: &OutputFormat,
    args: ProposalSetPoolFootprintCourtyardRectArgs,
) -> Result<(String, i32)> {
    let ProposalSetPoolFootprintCourtyardRectArgs {
        path,
        pool,
        footprint,
        min_x_nm,
        min_y_nm,
        max_x_nm,
        max_y_nm,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_set_native_project_pool_footprint_courtyard_rect(
                &path,
                &pool,
                footprint,
                min_x_nm,
                min_y_nm,
                max_x_nm,
                max_y_nm,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_set_pool_footprint_courtyard_polygon_proposal(
    format: &OutputFormat,
    args: ProposalSetPoolFootprintCourtyardPolygonArgs,
) -> Result<(String, i32)> {
    let ProposalSetPoolFootprintCourtyardPolygonArgs {
        path,
        pool,
        footprint,
        vertices,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_set_native_project_pool_footprint_courtyard_polygon(
                &path,
                &pool,
                footprint,
                &vertices,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_add_pool_footprint_silkscreen_line_proposal(
    format: &OutputFormat,
    args: ProposalAddPoolFootprintSilkscreenLineArgs,
) -> Result<(String, i32)> {
    let ProposalAddPoolFootprintSilkscreenLineArgs {
        path,
        pool,
        footprint,
        from_x_nm,
        from_y_nm,
        to_x_nm,
        to_y_nm,
        width_nm,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_add_native_project_pool_footprint_silkscreen_line(
                &path,
                &pool,
                footprint,
                from_x_nm,
                from_y_nm,
                to_x_nm,
                to_y_nm,
                width_nm,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_add_pool_footprint_silkscreen_rect_proposal(
    format: &OutputFormat,
    args: ProposalAddPoolFootprintSilkscreenRectArgs,
) -> Result<(String, i32)> {
    let ProposalAddPoolFootprintSilkscreenRectArgs {
        path,
        pool,
        footprint,
        min_x_nm,
        min_y_nm,
        max_x_nm,
        max_y_nm,
        width_nm,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_add_native_project_pool_footprint_silkscreen_rect(
                &path,
                &pool,
                footprint,
                min_x_nm,
                min_y_nm,
                max_x_nm,
                max_y_nm,
                width_nm,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_add_pool_footprint_silkscreen_circle_proposal(
    format: &OutputFormat,
    args: ProposalAddPoolFootprintSilkscreenCircleArgs,
) -> Result<(String, i32)> {
    let ProposalAddPoolFootprintSilkscreenCircleArgs {
        path,
        pool,
        footprint,
        center_x_nm,
        center_y_nm,
        radius_nm,
        width_nm,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_add_native_project_pool_footprint_silkscreen_circle(
                &path,
                &pool,
                footprint,
                center_x_nm,
                center_y_nm,
                radius_nm,
                width_nm,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_add_pool_footprint_silkscreen_polygon_proposal(
    format: &OutputFormat,
    args: ProposalAddPoolFootprintSilkscreenPolygonArgs,
) -> Result<(String, i32)> {
    let ProposalAddPoolFootprintSilkscreenPolygonArgs {
        path,
        pool,
        footprint,
        vertices,
        closed,
        width_nm,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_add_native_project_pool_footprint_silkscreen_polygon(
                &path,
                &pool,
                footprint,
                &vertices,
                closed,
                width_nm,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_set_pool_package_courtyard_rect_proposal(
    format: &OutputFormat,
    args: ProposalSetPoolPackageCourtyardRectArgs,
) -> Result<(String, i32)> {
    let ProposalSetPoolPackageCourtyardRectArgs {
        path,
        pool,
        package,
        min_x_nm,
        min_y_nm,
        max_x_nm,
        max_y_nm,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_set_native_project_pool_package_courtyard_rect(
                &path,
                &pool,
                package,
                min_x_nm,
                min_y_nm,
                max_x_nm,
                max_y_nm,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}

pub(super) fn execute_set_pool_package_courtyard_polygon_proposal(
    format: &OutputFormat,
    args: ProposalSetPoolPackageCourtyardPolygonArgs,
) -> Result<(String, i32)> {
    let ProposalSetPoolPackageCourtyardPolygonArgs {
        path,
        pool,
        package,
        vertices,
        proposal,
        rationale,
    } = args;
    Ok((
        render_output(
            format,
            &propose_set_native_project_pool_package_courtyard_polygon(
                &path,
                &pool,
                package,
                &vertices,
                proposal,
                rationale.as_deref(),
            )?,
        ),
        0,
    ))
}
