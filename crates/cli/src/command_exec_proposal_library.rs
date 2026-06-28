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
