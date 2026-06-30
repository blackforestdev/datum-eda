use super::*;
pub(super) fn execute_project_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        command @ (ProjectCommands::ReviewProposal(_)
        | ProjectCommands::ShowProposal(_)
        | ProjectCommands::ValidateProposal(_)
        | ProjectCommands::DeferProposal(_)
        | ProjectCommands::ApplyProposal(_)) => {
            execute_project_proposal_lifecycle_command(format, command)
        }
        command @ (ProjectCommands::ImportKicadFootprint(_) | ProjectCommands::ImportKicadBoard(_) | ProjectCommands::ImportKicadSchematic(_) | ProjectCommands::ImportEagleLibrary(_) | ProjectCommands::SetPoolPartBindings(_) | ProjectCommands::SetPoolSymbolPinAnchor(_)) => execute_project_import_or_part_binding_command(format, command),
        command @ (ProjectCommands::CreatePoolFootprint(_)
        | ProjectCommands::SetPoolFootprintPad(_)
        | ProjectCommands::SetPoolFootprintCourtyardRect(_)
        | ProjectCommands::SetPoolFootprintCourtyardPolygon(_)
        | ProjectCommands::AddPoolFootprintSilkscreenLine(_)
        | ProjectCommands::AddPoolFootprintSilkscreenRect(_)
        | ProjectCommands::AddPoolFootprintSilkscreenCircle(_)
        | ProjectCommands::AddPoolFootprintSilkscreenPolygon(_)) => {
            execute_project_library_footprint_command(format, command)
        }
        command @ (ProjectCommands::CreatePoolPinPadMap(_) | ProjectCommands::SetPoolPinPadMap(_)) => execute_project_library_pin_pad_map_command(format, command),
        command @ ProjectCommands::CreatePoolLibraryObject(_) | command @ ProjectCommands::CreatePoolUnit(_) | command @ ProjectCommands::SetPoolUnitPin(_) | command @ ProjectCommands::CreatePoolSymbol(_) | command @ ProjectCommands::AddPoolSymbolLine(_) | command @ ProjectCommands::AddPoolSymbolPolygon(_) | command @ ProjectCommands::AddPoolSymbolRect(_) | command @ ProjectCommands::AddPoolSymbolCircle(_) | command @ ProjectCommands::AddPoolSymbolText(_) | command @ ProjectCommands::AddPoolSymbolArc(_) | command @ ProjectCommands::CreatePoolEntity(_) | command @ ProjectCommands::CreatePoolPadstack(_) | command @ ProjectCommands::CreatePoolPackage(_) | command @ ProjectCommands::SetPoolPackagePad(_) | command @ ProjectCommands::SetPoolPackageCourtyardRect(_) | command @ ProjectCommands::SetPoolPackageCourtyardPolygon(_) | command @ ProjectCommands::AddPoolPackageSilkscreenLine(_) | command @ ProjectCommands::AddPoolPackageSilkscreenRect(_) | command @ ProjectCommands::AddPoolPackageSilkscreenCircle(_) | command @ ProjectCommands::AddPoolPackageSilkscreenArc(_) | command @ ProjectCommands::AddPoolPackageSilkscreenPolygon(_) | command @ ProjectCommands::AddPoolPackageSilkscreenText(_) | command @ ProjectCommands::AddPoolPackageModel3d(_) | command @ ProjectCommands::SetPoolPackageBodyHeights(_) | command @ ProjectCommands::CreatePoolPart(_) | command @ ProjectCommands::SetPoolPartMetadata(_) | command @ ProjectCommands::SetPoolPartParametric(_) | command @ ProjectCommands::SetPoolPartOrderableMpns(_) | command @ ProjectCommands::SetPoolPartPackagingOptions(_) | command @ ProjectCommands::SetPoolPartBehaviouralModels(_) | command @ ProjectCommands::AttachPoolPartModel(_) | command @ ProjectCommands::DetachPoolPartModel(_) | command @ ProjectCommands::GcPoolModels(_) | command @ ProjectCommands::SetPoolPartThermal(_) | command @ ProjectCommands::SetPoolPartSupplyChain(_) | command @ ProjectCommands::SetPoolPartTags(_) | command @ ProjectCommands::SetPoolPartPadMap(_) | command @ ProjectCommands::SetPoolPartPadMapEntry(_) | command @ ProjectCommands::SetPoolLibraryObject(_) | command @ ProjectCommands::DeletePoolLibraryObject(_) => execute_project_library_command(format, command),
        ProjectCommands::SetProjectName(ProjectSetProjectNameArgs { path, name }) => {
            let report = set_native_project_name(&path, name)?;
            let output = match format {
                OutputFormat::Text => render_native_project_name_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetProjectRules(ProjectSetProjectRulesArgs { path, rules_file }) => {
            let report = set_native_project_rules(&path, &rules_file)?;
            let output = match format {
                OutputFormat::Text => render_native_project_rules_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::CreateProjectRule(ProjectCreateProjectRuleArgs { path, rule_file }) => {
            let report = create_native_project_rule(&path, &rule_file)?;
            let output = match format {
                OutputFormat::Text => render_native_project_rules_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetProjectRule(ProjectSetProjectRuleArgs { path, rule_file }) => {
            let report = set_native_project_rule(&path, &rule_file)?;
            let output = match format {
                OutputFormat::Text => render_native_project_rules_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteProjectRule(ProjectDeleteProjectRuleArgs { path, rule_uuid }) => {
            let report = delete_native_project_rule(&path, rule_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_rules_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::Undo(args) => execute_native_project_journal_undo(format, &args.path, args.expected_model_revision.as_deref(), args.expected_tip_transaction),
        ProjectCommands::Redo(args) => execute_native_project_journal_redo(format, &args.path, args.expected_model_revision.as_deref(), args.expected_tip_transaction),
        ProjectCommands::GenerateStandardsRepairProposals(
            ProjectGenerateStandardsRepairProposalsArgs { path },
        ) => Ok((
            render_output(
                format,
                &generate_native_project_standards_repair_proposals(&path)?,
            ),
            0,
        )),
        ProjectCommands::WaiveFinding(ProjectWaiveFindingArgs {
            path,
            fingerprint,
            rationale,
            created_by,
        }) => Ok((
            render_output(
                format,
                &waive_native_project_finding(&path, &fingerprint, &rationale, created_by)?,
            ),
            0,
        )),
        ProjectCommands::AcceptDeviation(ProjectAcceptDeviationArgs {
            path,
            fingerprint,
            rationale,
            accepted_by,
        }) => Ok((
            render_output(
                format,
                &accept_native_project_deviation(&path, &fingerprint, &rationale, accepted_by)?,
            ),
            0,
        )),
        ProjectCommands::CreateGerberOutputJob(ProjectCreateGerberOutputJobArgs {
            path,
            prefix,
            output_dir,
            name,
            manufacturing_plan,
            variant,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_create_native_project_output_job(
                        &path,
                        &prefix,
                        output_dir.as_deref(),
                        "gerber-set",
                        name.as_deref(),
                        manufacturing_plan,
                        variant,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(create_native_project_gerber_set_output_job(
                        &path,
                        &prefix,
                        output_dir.as_deref(),
                        name.as_deref(),
                        manufacturing_plan,
                        variant,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::CreateOutputJob(ProjectCreateOutputJobArgs {
            path,
            prefix,
            output_dir,
            include,
            name,
            manufacturing_plan,
            variant,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_create_native_project_output_job(
                        &path,
                        &prefix,
                        output_dir.as_deref(),
                        &include,
                        name.as_deref(),
                        manufacturing_plan,
                        variant,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(create_native_project_output_job(
                        &path,
                        &prefix,
                        output_dir.as_deref(),
                        &include,
                        name.as_deref(),
                        manufacturing_plan,
                        variant,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::UpdateOutputJob(ProjectUpdateOutputJobArgs {
            path,
            output_job,
            name,
            output_dir,
            manufacturing_plan,
            variant,
            clear_manufacturing_plan,
            clear_variant,
            clear_output_dir,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_update_native_project_output_job(
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
                    )?)?
                } else {
                    serde_json::to_value(update_native_project_output_job(
                        &path,
                        output_job,
                        name.as_deref(),
                        output_dir.as_deref(),
                        manufacturing_plan,
                        variant,
                        clear_manufacturing_plan,
                        clear_variant,
                        clear_output_dir,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::RunOutputJob(ProjectRunOutputJobArgs {
            path,
            output_job,
            output_dir,
        }) => {
            let report = run_native_project_output_job(&path, output_job, output_dir.as_deref())?;
            let exit_code = report.exit_code;
            Ok((render_output(format, &report), exit_code))
        }
        ProjectCommands::StartOutputJobRun(ProjectStartOutputJobRunArgs { path, output_job }) => {
            Ok((
                render_output(
                    format,
                    &start_native_project_output_job_run(&path, output_job)?,
                ),
                0,
            ))
        }
        ProjectCommands::CancelOutputJobRun(ProjectCancelOutputJobRunArgs { path, run }) => Ok((
            render_output(format, &cancel_native_project_output_job_run(&path, run)?),
            0,
        )),
        ProjectCommands::DeleteOutputJob(ProjectDeleteOutputJobArgs {
            path,
            output_job,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_delete_native_project_output_job(
                        &path,
                        output_job,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(delete_native_project_output_job(&path, output_job)?)?
                },
            ),
            0,
        )),
        ProjectCommands::CreateManufacturingPlan(ProjectCreateManufacturingPlanArgs {
            path,
            prefix,
            name,
            variant,
            panel_projection,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_create_native_project_manufacturing_plan(
                        &path,
                        &prefix,
                        name.as_deref(),
                        variant,
                        panel_projection,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(create_native_project_manufacturing_plan(
                        &path,
                        &prefix,
                        name.as_deref(),
                        variant,
                        panel_projection,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::UpdateManufacturingPlan(ProjectUpdateManufacturingPlanArgs {
            path,
            manufacturing_plan,
            name,
            prefix,
            variant,
            clear_variant,
            panel_projection,
            clear_panel_projection,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_update_native_project_manufacturing_plan(
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
                    )?)?
                } else {
                    serde_json::to_value(update_native_project_manufacturing_plan(
                        &path,
                        manufacturing_plan,
                        name.as_deref(),
                        prefix.as_deref(),
                        variant,
                        clear_variant,
                        panel_projection,
                        clear_panel_projection,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::DeleteManufacturingPlan(ProjectDeleteManufacturingPlanArgs {
            path,
            manufacturing_plan,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_delete_native_project_manufacturing_plan(
                        &path,
                        manufacturing_plan,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(delete_native_project_manufacturing_plan(
                        &path,
                        manufacturing_plan,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::CreatePanelProjection(ProjectCreatePanelProjectionArgs {
            path,
            key,
            name,
            board,
            x_nm,
            y_nm,
            rotation_deg,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_create_native_project_panel_projection(
                        &path,
                        &key,
                        name.as_deref(),
                        board,
                        x_nm,
                        y_nm,
                        rotation_deg,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(create_native_project_panel_projection(
                        &path,
                        &key,
                        name.as_deref(),
                        board,
                        x_nm,
                        y_nm,
                        rotation_deg,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::UpdatePanelProjection(ProjectUpdatePanelProjectionArgs {
            path,
            panel_projection,
            name,
            board,
            x_nm,
            y_nm,
            rotation_deg,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_update_native_project_panel_projection(
                        &path,
                        panel_projection,
                        name.as_deref(),
                        board,
                        x_nm,
                        y_nm,
                        rotation_deg,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(update_native_project_panel_projection(
                        &path,
                        panel_projection,
                        name.as_deref(),
                        board,
                        x_nm,
                        y_nm,
                        rotation_deg,
                    )?)?
                },
            ),
            0,
        )),
        ProjectCommands::DeletePanelProjection(ProjectDeletePanelProjectionArgs {
            path,
            panel_projection,
            as_proposal,
            proposal,
            rationale,
        }) => Ok((
            render_output(
                format,
                &if as_proposal {
                    serde_json::to_value(propose_delete_native_project_panel_projection(
                        &path,
                        panel_projection,
                        proposal,
                        rationale.as_deref(),
                    )?)?
                } else {
                    serde_json::to_value(delete_native_project_panel_projection(
                        &path,
                        panel_projection,
                    )?)?
                },
            ),
            0,
        )),
        command @ ProjectCommands::CreateSheet(ProjectCreateSheetArgs { .. })
        | command @ ProjectCommands::DeleteSheet(ProjectDeleteSheetArgs { .. })
        | command @ ProjectCommands::RenameSheet(ProjectRenameSheetArgs { .. }) | command @ ProjectCommands::CreateSheetDefinition(ProjectCreateSheetDefinitionArgs { .. })
        | command @ ProjectCommands::CreateSheetInstance(ProjectCreateSheetInstanceArgs { .. }) | command @ ProjectCommands::DeleteSheetInstance(ProjectDeleteSheetInstanceArgs { .. }) | command @ ProjectCommands::MoveSheetInstance(ProjectMoveSheetInstanceArgs { .. }) | command @ ProjectCommands::BindSheetInstancePort(ProjectBindSheetInstancePortArgs { .. }) | command @ ProjectCommands::UnbindSheetInstancePort(ProjectUnbindSheetInstancePortArgs { .. }) | command @ ProjectCommands::PlaceLabel(ProjectPlaceLabelArgs { .. })
        | command @ ProjectCommands::RenameLabel(ProjectRenameLabelArgs { .. })
        | command @ ProjectCommands::DeleteLabel(ProjectDeleteLabelArgs { .. })
        | command @ ProjectCommands::DrawWire(ProjectDrawWireArgs { .. })
        | command @ ProjectCommands::DeleteWire(ProjectDeleteWireArgs { .. })
        | command @ ProjectCommands::PlaceJunction(ProjectPlaceJunctionArgs { .. })
        | command @ ProjectCommands::DeleteJunction(ProjectDeleteJunctionArgs { .. })
        | command @ ProjectCommands::PlacePort(ProjectPlacePortArgs { .. })
        | command @ ProjectCommands::EditPort(ProjectEditPortArgs { .. })
        | command @ ProjectCommands::DeletePort(ProjectDeletePortArgs { .. })
        | command @ ProjectCommands::CreateBus(ProjectCreateBusArgs { .. })
        | command @ ProjectCommands::EditBusMembers(ProjectEditBusMembersArgs { .. })
        | command @ ProjectCommands::DeleteBus(ProjectDeleteBusArgs { .. })
        | command @ ProjectCommands::PlaceBusEntry(ProjectPlaceBusEntryArgs { .. })
        | command @ ProjectCommands::DeleteBusEntry(ProjectDeleteBusEntryArgs { .. })
        | command @ ProjectCommands::PlaceNoConnect(ProjectPlaceNoConnectArgs { .. })
        | command @ ProjectCommands::DeleteNoConnect(ProjectDeleteNoConnectArgs { .. }) => {
            command_exec_project_schematic_connectivity::execute_project_schematic_connectivity_command(
                format,
                command,
            )
        }
        command @ ProjectCommands::PlaceSymbol(ProjectPlaceSymbolArgs { .. })
        | command @ ProjectCommands::MoveSymbol(ProjectMoveSymbolArgs { .. })
        | command @ ProjectCommands::RotateSymbol(ProjectRotateSymbolArgs { .. })
        | command @ ProjectCommands::MirrorSymbol(ProjectMirrorSymbolArgs { .. })
        | command @ ProjectCommands::DeleteSymbol(ProjectDeleteSymbolArgs { .. })
        | command @ ProjectCommands::SetSymbolReference(ProjectSetSymbolReferenceArgs { .. })
        | command @ ProjectCommands::SetSymbolValue(ProjectSetSymbolValueArgs { .. })
        | command @ ProjectCommands::SetSymbolLibId(ProjectSetSymbolLibIdArgs { .. })
        | command @ ProjectCommands::ClearSymbolLibId(ProjectClearSymbolLibIdArgs { .. })
        | command @ ProjectCommands::SetSymbolEntity(ProjectSetSymbolEntityArgs { .. })
        | command @ ProjectCommands::ClearSymbolEntity(ProjectClearSymbolEntityArgs { .. })
        | command @ ProjectCommands::SetSymbolPart(ProjectSetSymbolPartArgs { .. })
        | command @ ProjectCommands::ClearSymbolPart(ProjectClearSymbolPartArgs { .. })
        | command @ ProjectCommands::SetSymbolUnit(ProjectSetSymbolUnitArgs { .. })
        | command @ ProjectCommands::ClearSymbolUnit(ProjectClearSymbolUnitArgs { .. })
        | command @ ProjectCommands::SetSymbolGate(ProjectSetSymbolGateArgs { .. })
        | command @ ProjectCommands::ClearSymbolGate(ProjectClearSymbolGateArgs { .. })
        | command @ ProjectCommands::SetSymbolDisplayMode(ProjectSetSymbolDisplayModeArgs { .. })
        | command @ ProjectCommands::SetSymbolHiddenPowerBehavior(ProjectSetSymbolHiddenPowerBehaviorArgs { .. })
        | command @ ProjectCommands::SetPinOverride(ProjectSetPinOverrideArgs { .. })
        | command @ ProjectCommands::ClearPinOverride(ProjectClearPinOverrideArgs { .. })
        | command @ ProjectCommands::AddSymbolField(ProjectAddSymbolFieldArgs { .. })
        | command @ ProjectCommands::EditSymbolField(ProjectEditSymbolFieldArgs { .. })
        | command @ ProjectCommands::DeleteSymbolField(ProjectDeleteSymbolFieldArgs { .. })
        | command @ ProjectCommands::PlaceText(ProjectPlaceTextArgs { .. })
        | command @ ProjectCommands::EditText(ProjectEditTextArgs { .. })
        | command @ ProjectCommands::DeleteText(ProjectDeleteTextArgs { .. })
        | command @ ProjectCommands::PlaceDrawingLine(ProjectPlaceDrawingLineArgs { .. })
        | command @ ProjectCommands::PlaceDrawingRect(ProjectPlaceDrawingRectArgs { .. })
        | command @ ProjectCommands::PlaceDrawingCircle(ProjectPlaceDrawingCircleArgs { .. })
        | command @ ProjectCommands::PlaceDrawingArc(ProjectPlaceDrawingArcArgs { .. })
        | command @ ProjectCommands::EditDrawingLine(ProjectEditDrawingLineArgs { .. })
        | command @ ProjectCommands::EditDrawingRect(ProjectEditDrawingRectArgs { .. })
        | command @ ProjectCommands::EditDrawingCircle(ProjectEditDrawingCircleArgs { .. })
        | command @ ProjectCommands::EditDrawingArc(ProjectEditDrawingArcArgs { .. })
        | command @ ProjectCommands::DeleteDrawing(ProjectDeleteDrawingArgs { .. }) => {
            command_exec_project_schematic_symbols::execute_project_schematic_symbols_command(
                format,
                command,
            )
        }
        command @ ProjectCommands::PlaceBoardText(ProjectPlaceBoardTextArgs { .. })
        | command @ ProjectCommands::EditBoardText(ProjectEditBoardTextArgs { .. })
        | command @ ProjectCommands::DeleteBoardText(ProjectDeleteBoardTextArgs { .. })
        | command @ ProjectCommands::PlaceBoardKeepout(ProjectPlaceBoardKeepoutArgs { .. })
        | command @ ProjectCommands::EditBoardKeepout(ProjectEditBoardKeepoutArgs { .. })
        | command @ ProjectCommands::DeleteBoardKeepout(ProjectDeleteBoardKeepoutArgs { .. })
        | command @ ProjectCommands::SetBoardOutline(ProjectSetBoardOutlineArgs { .. })
        | command @ ProjectCommands::SetBoardName(ProjectSetBoardNameArgs { .. })
        | command @ ProjectCommands::PlaceBoardComponent(ProjectPlaceBoardComponentArgs { .. })
        | command @ ProjectCommands::DeleteBoardComponent(ProjectDeleteBoardComponentArgs { .. })
        | command @ ProjectCommands::DrawBoardTrack(ProjectDrawBoardTrackArgs { .. })
        | command @ ProjectCommands::EditBoardTrack(ProjectEditBoardTrackArgs { .. })
        | command @ ProjectCommands::DeleteBoardTrack(ProjectDeleteBoardTrackArgs { .. })
        | command @ ProjectCommands::PlaceBoardVia(ProjectPlaceBoardViaArgs { .. })
        | command @ ProjectCommands::EditBoardVia(ProjectEditBoardViaArgs { .. })
        | command @ ProjectCommands::DeleteBoardVia(ProjectDeleteBoardViaArgs { .. })
        | command @ ProjectCommands::PlaceBoardZone(ProjectPlaceBoardZoneArgs { .. }) | command @ ProjectCommands::EditBoardZone(ProjectEditBoardZoneArgs { .. }) | command @ ProjectCommands::FillZones(ProjectFillZonesArgs { .. })
        | command @ ProjectCommands::DeleteBoardZone(ProjectDeleteBoardZoneArgs { .. })
        | command @ ProjectCommands::SetBoardPadNet(ProjectSetBoardPadNetArgs { .. })
        | command @ ProjectCommands::ClearBoardPadNet(ProjectClearBoardPadNetArgs { .. })
        | command @ ProjectCommands::EditBoardPad(ProjectEditBoardPadArgs { .. })
        | command @ ProjectCommands::PlaceBoardPad(ProjectPlaceBoardPadArgs { .. })
        | command @ ProjectCommands::DeleteBoardPad(ProjectDeleteBoardPadArgs { .. })
        | command @ ProjectCommands::PlaceBoardDimension(_)
        | command @ ProjectCommands::EditBoardDimension(_)
        | command @ ProjectCommands::DeleteBoardDimension(ProjectDeleteBoardDimensionArgs { .. }) => {
            command_exec_project_board_surface::execute_project_board_surface_command(
                format,
                command,
            )
        }
        ProjectCommands::SetBoardStackup(ProjectSetBoardStackupArgs { path, layers }) => {
            command_exec_board_stackup::execute_set_board_stackup(format, path, layers)
        }
        ProjectCommands::AddDefaultTopStackup(ProjectAddDefaultTopStackupArgs { path }) => {
            command_exec_board_stackup::execute_add_default_top_stackup(format, path)
        }
        ProjectCommands::PlaceBoardNet(ProjectPlaceBoardNetArgs {
            path,
            name,
            class_uuid,
            impedance_target_ohms,
            impedance_tolerance_pct,
            controlled_dielectric_layer,
        }) => command_exec_board_net::execute_place_board_net(
            format,
            path,
            name,
            class_uuid,
            impedance_target_ohms,
            impedance_tolerance_pct,
            controlled_dielectric_layer,
        ),
        ProjectCommands::PlaceBoardNetClass(ProjectPlaceBoardNetClassArgs {
            path,
            name,
            clearance_nm,
            track_width_nm,
            via_drill_nm,
            via_diameter_nm,
            diffpair_width_nm,
            diffpair_gap_nm,
        }) => command_exec_board_net::execute_place_board_net_class(
            format,
            path,
            name,
            clearance_nm,
            track_width_nm,
            via_drill_nm,
            via_diameter_nm,
            diffpair_width_nm,
            diffpair_gap_nm,
        ),
        ProjectCommands::EditBoardNetClass(ProjectEditBoardNetClassArgs {
            path,
            net_class_uuid,
            name,
            clearance_nm,
            track_width_nm,
            via_drill_nm,
            via_diameter_nm,
            diffpair_width_nm,
            diffpair_gap_nm,
        }) => command_exec_board_net::execute_edit_board_net_class(
            format,
            path,
            net_class_uuid,
            name,
            clearance_nm,
            track_width_nm,
            via_drill_nm,
            via_diameter_nm,
            diffpair_width_nm,
            diffpair_gap_nm,
        ),
        ProjectCommands::EditBoardNet(ProjectEditBoardNetArgs {
            path,
            net_uuid,
            name,
            class_uuid,
            impedance_target_ohms,
            impedance_tolerance_pct,
            controlled_dielectric_layer,
            clear_controlled_impedance,
        }) => command_exec_board_net::execute_edit_board_net(
            format,
            path,
            net_uuid,
            name,
            class_uuid,
            impedance_target_ohms,
            impedance_tolerance_pct,
            controlled_dielectric_layer,
            clear_controlled_impedance,
        ),
        ProjectCommands::MoveBoardComponent(ProjectMoveBoardComponentArgs {
            path,
            component_uuid,
            x_nm,
            y_nm,
        }) => command_exec_board_component::execute_move_board_component(
            format,
            path,
            component_uuid,
            x_nm,
            y_nm,
        ),
        ProjectCommands::SetBoardComponentPart(SetBoardComponentPartArgs {
            path,
            component_uuid,
            part_uuid,
        }) => command_exec_board_component::execute_set_board_component_part(
            format,
            path,
            component_uuid,
            part_uuid,
        ),
        ProjectCommands::SetBoardComponentPackage(SetBoardComponentPackageArgs {
            path,
            component_uuid,
            package_uuid,
        }) => command_exec_board_component::execute_set_board_component_package(
            format,
            path,
            component_uuid,
            package_uuid,
        ),
        ProjectCommands::SetBoardComponentLayer(SetBoardComponentLayerArgs { path, component_uuid, layer })
        | ProjectCommands::FlipBoardComponent(SetBoardComponentLayerArgs { path, component_uuid, layer }) => command_exec_board_component::execute_set_board_component_layer(
            format,
            path,
            component_uuid,
            layer,
        ),
        ProjectCommands::SetBoardComponentReference(SetBoardComponentReferenceArgs {
            path,
            component_uuid,
            reference,
        }) => command_exec_board_component::execute_set_board_component_reference(
            format,
            path,
            component_uuid,
            reference,
        ),
        ProjectCommands::SetBoardComponentValue(SetBoardComponentValueArgs {
            path,
            component_uuid,
            value,
        }) => command_exec_board_component::execute_set_board_component_value(
            format,
            path,
            component_uuid,
            value,
        ),
        ProjectCommands::RotateBoardComponent(ProjectRotateBoardComponentArgs {
            path,
            component_uuid,
            rotation_deg,
        }) => command_exec_board_component::execute_rotate_board_component(
            format,
            path,
            component_uuid,
            rotation_deg,
        ),
        ProjectCommands::SetBoardComponentLocked(ProjectSetBoardComponentLockedArgs {
            path,
            component_uuid,
        }) => command_exec_board_component::execute_set_board_component_locked(
            format,
            path,
            component_uuid,
            true,
        ),
        ProjectCommands::ClearBoardComponentLocked(ProjectClearBoardComponentLockedArgs {
            path,
            component_uuid,
        }) => command_exec_board_component::execute_set_board_component_locked(
            format,
            path,
            component_uuid,
            false,
        ),
        ProjectCommands::BindComponentInstance(args) => {
            let view = bind_native_project_component_instance(
                &args.path,
                args.component_instance,
                args.symbols,
                args.package,
                args.part,
                args.symbol_roles,
                args.package_roles,
            )?;
            Ok((render_output(format, &view), 0))
        }
        ProjectCommands::SetComponentInstance(args) => {
            let view = set_native_project_component_instance(
                &args.path,
                args.component_instance,
                args.symbols,
                args.package,
                args.part,
                args.symbol_roles,
                args.package_roles,
            )?;
            Ok((render_output(format, &view), 0))
        }
        ProjectCommands::DeleteComponentInstance(ProjectDeleteComponentInstanceArgs {
            path,
            component_instance,
        }) => Ok((
            render_output(
                format,
                &delete_native_project_component_instance(&path, component_instance)?,
            ),
            0,
        )),
        ProjectCommands::DeleteBoardNetClass(ProjectDeleteBoardNetClassArgs { path, net_class_uuid }) => {
            command_exec_board_net::execute_delete_board_net_class(format, path, net_class_uuid)
        }
        ProjectCommands::DeleteBoardNet(ProjectDeleteBoardNetArgs { path, net_uuid }) => {
            command_exec_board_net::execute_delete_board_net(format, path, net_uuid)
        }
        _ => unreachable!("inventory command should dispatch before project match"),
    }
}
