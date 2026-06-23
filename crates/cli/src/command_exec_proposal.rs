use super::*;

pub(crate) fn execute_proposal_command(
    format: &OutputFormat,
    action: ProposalCommands,
) -> Result<(String, i32)> {
    match action {
        ProposalCommands::Create(ProjectCreateProposalArgs {
            path,
            batch,
            rationale,
            proposal,
            source,
            checks_run,
            finding_fingerprints,
        }) => Ok((
            render_output(
                format,
                &create_native_project_proposal(
                    &path,
                    &batch,
                    rationale,
                    proposal,
                    source,
                    checks_run,
                    finding_fingerprints,
                )?,
            ),
            0,
        )),
        ProposalCommands::CreatePlaceLabel(ProposalPlaceLabelArgs {
            path,
            sheet,
            name,
            kind,
            x_nm,
            y_nm,
            proposal,
            rationale,
        }) => {
            let kind = match kind {
                NativeLabelKindArg::Local => eda_engine::schematic::LabelKind::Local,
                NativeLabelKindArg::Global => eda_engine::schematic::LabelKind::Global,
                NativeLabelKindArg::Hierarchical => eda_engine::schematic::LabelKind::Hierarchical,
                NativeLabelKindArg::Power => eda_engine::schematic::LabelKind::Power,
            };
            Ok((
                render_output(
                    format,
                    &propose_place_native_project_label(
                        &path,
                        sheet,
                        name,
                        kind,
                        Point { x: x_nm, y: y_nm },
                        proposal,
                        rationale.as_deref(),
                    )?,
                ),
                0,
            ))
        }
        ProposalCommands::CreatePlaceSymbol(ProposalPlaceSymbolArgs {
            path,
            sheet,
            reference,
            value,
            lib_id,
            x_nm,
            y_nm,
            rotation_deg,
            mirrored,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &propose_place_native_project_symbol(
                    &path,
                    sheet,
                    reference,
                    value,
                    lib_id,
                    Point { x: x_nm, y: y_nm },
                    rotation_deg,
                    mirrored,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        )),
        ProposalCommands::CreateDrawWire(ProposalDrawWireArgs {
            path,
            sheet,
            from_x_nm,
            from_y_nm,
            to_x_nm,
            to_y_nm,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &propose_draw_native_project_wire(
                    &path,
                    sheet,
                    Point {
                        x: from_x_nm,
                        y: from_y_nm,
                    },
                    Point {
                        x: to_x_nm,
                        y: to_y_nm,
                    },
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        )),
        ProposalCommands::CreateOutputJob(ProposalCreateOutputJobArgs {
            path,
            prefix,
            include,
            output_dir,
            name,
            manufacturing_plan,
            variant,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &propose_create_native_project_output_job(
                    &path,
                    &prefix,
                    output_dir.as_deref(),
                    &include,
                    name.as_deref(),
                    manufacturing_plan,
                    variant,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        )),
        ProposalCommands::UpdateOutputJob(ProposalUpdateOutputJobArgs {
            path,
            output_job,
            name,
            output_dir,
            manufacturing_plan,
            variant,
            clear_manufacturing_plan,
            clear_variant,
            clear_output_dir,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &propose_update_native_project_output_job(
                    &path,
                    output_job,
                    name.as_deref(),
                    output_dir.as_deref(),
                    manufacturing_plan,
                    variant,
                    clear_manufacturing_plan,
                    clear_variant,
                    clear_output_dir,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        )),
        ProposalCommands::DeleteOutputJob(ProposalDeleteOutputJobArgs {
            path,
            output_job,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &propose_delete_native_project_output_job(
                    &path,
                    output_job,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        )),
        ProposalCommands::CreateManufacturingPlan(ProposalCreateManufacturingPlanArgs {
            path,
            prefix,
            name,
            variant,
            panel_projection,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &propose_create_native_project_manufacturing_plan(
                    &path,
                    &prefix,
                    name.as_deref(),
                    variant,
                    panel_projection,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        )),
        ProposalCommands::UpdateManufacturingPlan(ProposalUpdateManufacturingPlanArgs {
            path,
            manufacturing_plan,
            name,
            prefix,
            variant,
            clear_variant,
            panel_projection,
            clear_panel_projection,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &propose_update_native_project_manufacturing_plan(
                    &path,
                    manufacturing_plan,
                    name.as_deref(),
                    prefix.as_deref(),
                    variant,
                    clear_variant,
                    panel_projection,
                    clear_panel_projection,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        )),
        ProposalCommands::DeleteManufacturingPlan(ProposalDeleteManufacturingPlanArgs {
            path,
            manufacturing_plan,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &propose_delete_native_project_manufacturing_plan(
                    &path,
                    manufacturing_plan,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        )),
        ProposalCommands::CreatePanelProjection(ProposalCreatePanelProjectionArgs {
            path,
            key,
            name,
            board,
            x_nm,
            y_nm,
            rotation_deg,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &propose_create_native_project_panel_projection(
                    &path,
                    &key,
                    name.as_deref(),
                    board,
                    x_nm,
                    y_nm,
                    rotation_deg,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        )),
        ProposalCommands::UpdatePanelProjection(ProposalUpdatePanelProjectionArgs {
            path,
            panel_projection,
            name,
            board,
            x_nm,
            y_nm,
            rotation_deg,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &propose_update_native_project_panel_projection(
                    &path,
                    panel_projection,
                    name.as_deref(),
                    board,
                    x_nm,
                    y_nm,
                    rotation_deg,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        )),
        ProposalCommands::DeletePanelProjection(ProposalDeletePanelProjectionArgs {
            path,
            panel_projection,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &propose_delete_native_project_panel_projection(
                    &path,
                    panel_projection,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        )),
        ProposalCommands::Preview(ProjectPreviewProposalArgs { path, proposal }) => Ok((
            render_output(format, &preview_native_project_proposal(&path, proposal)?),
            0,
        )),
        ProposalCommands::List(ProjectProposalListArgs { path }) => Ok((
            render_output(format, &query_native_project_proposals(&path)?),
            0,
        )),
        ProposalCommands::Show(ProjectShowProposalArgs { path, proposal }) => Ok((
            render_output(format, &show_native_project_proposal(&path, proposal)?),
            0,
        )),
        ProposalCommands::Validate(ProjectValidateProposalArgs { path, proposal }) => Ok((
            render_output(format, &validate_native_project_proposal(&path, proposal)?),
            0,
        )),
        ProposalCommands::Review(ProjectReviewProposalArgs {
            path,
            proposal,
            status,
        }) => review_proposal(format, &path, proposal, status),
        ProposalCommands::Defer(ProjectDeferProposalArgs { path, proposal }) => Ok((
            render_output(format, &defer_native_project_proposal(&path, proposal)?),
            0,
        )),
        ProposalCommands::Reject(ProjectRejectProposalArgs { path, proposal }) => {
            review_proposal(format, &path, proposal, ProposalReviewStatusArg::Rejected)
        }
        ProposalCommands::AcceptApply(ProjectApplyProposalArgs { path, proposal }) => Ok((
            render_output(
                format,
                &accept_and_apply_native_project_proposal(&path, proposal)?,
            ),
            0,
        )),
        ProposalCommands::Apply(ProjectApplyProposalArgs { path, proposal }) => Ok((
            render_output(format, &apply_native_project_proposal(&path, proposal)?),
            0,
        )),
    }
}

fn review_proposal(
    format: &OutputFormat,
    path: &Path,
    proposal: Uuid,
    status: ProposalReviewStatusArg,
) -> Result<(String, i32)> {
    let status = match status {
        ProposalReviewStatusArg::Accepted => eda_engine::substrate::ProposalStatus::Accepted,
        ProposalReviewStatusArg::Deferred => eda_engine::substrate::ProposalStatus::Deferred,
        ProposalReviewStatusArg::Rejected => eda_engine::substrate::ProposalStatus::Rejected,
    };
    Ok((
        render_output(
            format,
            &review_native_project_proposal(path, proposal, status)?,
        ),
        0,
    ))
}
