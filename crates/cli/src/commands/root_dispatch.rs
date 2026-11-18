use super::*;

/// Top-level Commands router (Phase 5 fold of command_exec_dispatch.rs):
/// every subcommand family dispatches to `args.run(format)` inherent
/// methods or the family fn that owns its sub-enum.
pub(crate) fn execute_with_exit_code(cli: Cli) -> Result<(String, i32)> {
    let format = &cli.format;
    match cli.command {
        Commands::Agent { action } => execute_agent_command(format, action),
        Commands::Mcp { action } => execute_mcp_command(format, action),
        Commands::Context { action } => execute_context_command(format, action),
        Commands::Revision { action } => execute_revision_command(format, action),
        Commands::Units { action } => execute_units_command(format, action),
        Commands::Preferences { action } => execute_preferences_command(format, action),
        Commands::Import { path } => {
            let report = import_path(&path)?;
            let view = ImportReportView::from(report);
            Ok((render_output(format, &view), 0))
        }
        Commands::Query { action } => execute_query_command(format, action),
        Commands::Drc { path } => {
            let report = run_drc(Path::new(&path))?;
            let output = match *format {
                OutputFormat::Text => render_drc_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            let exit_code = if report.passed { 0 } else { 1 };
            Ok((output, exit_code))
        }
        Commands::Erc { path } => {
            let findings = run_erc(&path)?;
            let exit_code = if findings.iter().any(|finding| !finding.waived) {
                1
            } else {
                0
            };
            Ok((render_output(format, &findings), exit_code))
        }
        Commands::Check { action } => match action {
            CheckCommands::Run(args) => args.run(format),
            CheckCommands::List(args) => args.run(format),
            CheckCommands::Show(args) => args.run(format),
            CheckCommands::Profiles(args) => args.run(format),
            CheckCommands::FillZones(args) => args.run(format),
            CheckCommands::RepairStandards(args) => args.run(format),
            CheckCommands::Waive(args) => args.run(format),
            CheckCommands::AcceptDeviation(args) => args.run(format),
            CheckCommands::Imported(args) => args.run(format),
        },
        Commands::Pool { action } => action.run(format),
        Commands::Proposal { action } => match action {
            ProposalCommands::Create(args) => args.run(format),
            ProposalCommands::CreatePlaceLabel(args) => args.run(format),
            ProposalCommands::CreatePlaceSymbol(args) => args.run(format),
            ProposalCommands::CreateDrawWire(args) => args.run(format),
            ProposalCommands::CreateBoardComponentReplacement(args) => args.run(format),
            ProposalCommands::CreateBoardComponentReplacements(args) => args.run(format),
            ProposalCommands::CreateBoardComponentReplacementPlan(args) => args.run(format),
            ProposalCommands::BindComponentInstance(args) => args.run(format),
            ProposalCommands::SetComponentInstance(args) => args.run(format),
            ProposalCommands::DeleteComponentInstance(args) => args.run(format),
            ProposalCommands::CreatePoolLibraryObject(args) => args.run(format),
            ProposalCommands::CreatePoolUnit(args) => args.run(format),
            ProposalCommands::CreatePoolSymbol(args) => args.run(format),
            ProposalCommands::CreatePoolEntity(args) => args.run(format),
            ProposalCommands::CreatePoolPadstack(args) => args.run(format),
            ProposalCommands::CreatePoolPackage(args) => args.run(format),
            ProposalCommands::CreatePoolFootprint(args) => args.run(format),
            ProposalCommands::GenerateIpc7351bTwoTerminalChip(args) => args.run(format),
            ProposalCommands::GenerateIpc7351bSoic(args) => args.run(format),
            ProposalCommands::CreatePoolPinPadMap(args) => args.run(format),
            ProposalCommands::SetPoolPinPadMap(args) => args.run(format),
            ProposalCommands::SetPoolFootprintPad(args) => args.run(format),
            ProposalCommands::SetPoolFootprintCourtyardRect(args) => args.run(format),
            ProposalCommands::SetPoolFootprintCourtyardPolygon(args) => args.run(format),
            ProposalCommands::AddPoolFootprintSilkscreenLine(args) => args.run(format),
            ProposalCommands::AddPoolFootprintSilkscreenRect(args) => args.run(format),
            ProposalCommands::AddPoolFootprintSilkscreenCircle(args) => args.run(format),
            ProposalCommands::AddPoolFootprintSilkscreenPolygon(args) => args.run(format),
            ProposalCommands::SetPoolPackagePad(args) => args.run(format),
            ProposalCommands::SetPoolPackageCourtyardRect(args) => args.run(format),
            ProposalCommands::SetPoolPackageCourtyardPolygon(args) => args.run(format),
            ProposalCommands::CreateOutputJob(args) => args.run(format),
            ProposalCommands::UpdateOutputJob(args) => args.run(format),
            ProposalCommands::DeleteOutputJob(args) => args.run(format),
            ProposalCommands::CreateManufacturingPlan(args) => args.run(format),
            ProposalCommands::UpdateManufacturingPlan(args) => args.run(format),
            ProposalCommands::DeleteManufacturingPlan(args) => args.run(format),
            ProposalCommands::CreatePanelProjection(args) => args.run(format),
            ProposalCommands::UpdatePanelProjection(args) => args.run(format),
            ProposalCommands::DeletePanelProjection(args) => args.run(format),
            ProposalCommands::Preview(args) => args.run(format),
            ProposalCommands::List(args) => args.run(format),
            ProposalCommands::Show(args) => args.run(format),
            ProposalCommands::Validate(args) => args.run(format),
            ProposalCommands::Review(args) => args.run(format),
            ProposalCommands::Defer(args) => args.run(format),
            ProposalCommands::Reject(args) => args.run(format),
            ProposalCommands::AcceptApply(args) => args.run_accept_apply(format),
            ProposalCommands::Apply(args) => args.run(format),
        },
        Commands::Journal { action } => match action {
            JournalCommands::List(args) => args.run(format),
            JournalCommands::Show(args) => args.run(format),
            JournalCommands::Undo(args) => args.run(format),
            JournalCommands::Redo(args) => args.run(format),
        },
        Commands::Artifact { action } => match action {
            ArtifactCommands::ExportLibraryCandidate { path, output_dir } => Ok((
                render_output(
                    format,
                    &eda_engine::api::Engine::export_library_candidate(&path, &output_dir)?,
                ),
                0,
            )),
            ArtifactCommands::Generate(args) => args.run(format),
            ArtifactCommands::StartOutputJobRun(args) => args.run(format),
            ArtifactCommands::CancelOutputJobRun(args) => args.run(format),
            ArtifactCommands::List(args) => args.run(format),
            ArtifactCommands::Show(args) => args.run(format),
            ArtifactCommands::Files(args) => args.run(format),
            ArtifactCommands::Preview(args) => args.run(format),
            ArtifactCommands::Compare(args) => args.run(format),
            ArtifactCommands::Validate(args) => args.run(format),
            ArtifactCommands::ExportManufacturingSet(args) => args.run(format),
            ArtifactCommands::ValidateManufacturingSet(args) => args.run(format),
        },
        Commands::Project { action } => execute_project_command(format, *action),
        Commands::Plan { action } => execute_plan_command(format, action),
        Commands::Modify {
            path,
            delete_track,
            delete_via,
            delete_component,
            libraries,
            move_component,
            rotate_component,
            set_value,
            assign_part,
            set_package,
            set_package_with_part,
            replace_component,
            apply_replacement_plan,
            apply_replacement_policy,
            apply_scoped_replacement_policy,
            apply_scoped_replacement_plan_file,
            apply_scoped_replacement_manifest,
            set_net_class,
            set_reference,
            undo,
            redo,
            save,
            set_clearance_min_nm,
            save_original,
        } => {
            let move_component = move_component
                .iter()
                .map(|value| parse_move_component_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let rotate_component = rotate_component
                .iter()
                .map(|value| parse_rotate_component_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let set_value = set_value
                .iter()
                .map(|value| parse_set_value_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let assign_part = assign_part
                .iter()
                .map(|value| parse_assign_part_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let set_package = set_package
                .iter()
                .map(|value| parse_set_package_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let set_package_with_part = set_package_with_part
                .iter()
                .map(|value| parse_set_package_with_part_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let replace_component = replace_component
                .iter()
                .map(|value| parse_replace_component_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let apply_replacement_plan = apply_replacement_plan
                .iter()
                .map(|value| parse_apply_replacement_plan_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let apply_replacement_policy = apply_replacement_policy
                .iter()
                .map(|value| parse_apply_replacement_policy_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let apply_scoped_replacement_policy = apply_scoped_replacement_policy
                .iter()
                .map(|value| parse_apply_scoped_replacement_policy_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let apply_scoped_replacement_plan = apply_scoped_replacement_plan_file
                .iter()
                .map(|plan_path| {
                    let plan_text = std::fs::read_to_string(plan_path).with_context(|| {
                        format!(
                            "failed to read scoped replacement plan file {}",
                            plan_path.display()
                        )
                    })?;
                    serde_json::from_str::<ScopedComponentReplacementPlan>(&plan_text).with_context(
                        || {
                            format!(
                                "failed to parse scoped replacement plan file {}",
                                plan_path.display()
                            )
                        },
                    )
                })
                .collect::<Result<Vec<_>>>()?;
            let scoped_replacement_manifests = apply_scoped_replacement_manifest
                .iter()
                .map(|manifest_path| {
                    let loaded = load_scoped_replacement_manifest_with_metadata(manifest_path)?;
                    validate_scoped_replacement_manifest(&loaded.manifest, &path)?;
                    Ok(loaded)
                })
                .collect::<Result<Vec<LoadedScopedReplacementManifest>>>()?;
            let mut libraries = libraries;
            for manifest in &scoped_replacement_manifests {
                for library in &manifest.manifest.libraries {
                    if !libraries.iter().any(|existing| existing == &library.path) {
                        libraries.push(library.path.clone());
                    }
                }
            }
            let mut apply_scoped_replacement_plan = apply_scoped_replacement_plan;
            apply_scoped_replacement_plan.extend(
                scoped_replacement_manifests
                    .iter()
                    .map(|loaded| loaded.manifest.plan.clone()),
            );
            let applied_scoped_replacement_manifests = scoped_replacement_manifests
                .iter()
                .map(|loaded| AppliedScopedReplacementManifestView {
                    path: loaded.manifest_path.display().to_string(),
                    source_version: loaded.source_version,
                    version: loaded.manifest.version,
                    migration_applied: loaded.source_version != loaded.manifest.version,
                    replacements: loaded.manifest.plan.replacements.len(),
                })
                .collect::<Vec<_>>();
            let set_net_class = set_net_class
                .iter()
                .map(|value| parse_set_net_class_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let set_reference = set_reference
                .iter()
                .map(|value| parse_set_reference_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let mut report = modify_board_with_plan(
                &path,
                &delete_track,
                &delete_via,
                &delete_component,
                &libraries,
                &move_component,
                &rotate_component,
                &set_value,
                &assign_part,
                &set_package,
                &set_package_with_part,
                &replace_component,
                &set_net_class,
                &set_reference,
                set_clearance_min_nm,
                undo,
                redo,
                save.as_deref(),
                save_original,
                &apply_replacement_plan,
                &apply_replacement_policy,
                &apply_scoped_replacement_policy,
                &apply_scoped_replacement_plan,
            )?;
            report.applied_scoped_replacement_manifests = applied_scoped_replacement_manifests;
            let output = match *format {
                OutputFormat::Text => render_modify_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
    }
}
