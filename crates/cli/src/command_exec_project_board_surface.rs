use super::*;

pub(super) fn execute_project_board_surface_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::PlaceBoardText(ProjectPlaceBoardTextArgs {
            path,
            text,
            x_nm,
            y_nm,
            rotation_deg,
            height_nm,
            stroke_width_nm,
            layer,
        }) => {
            let report = place_native_project_board_text(
                &path,
                text,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                rotation_deg,
                height_nm,
                stroke_width_nm,
                layer,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_text_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditBoardText(ProjectEditBoardTextArgs {
            path,
            text_uuid,
            value,
            x_nm,
            y_nm,
            rotation_deg,
            height_nm,
            stroke_width_nm,
            layer,
        }) => {
            let report = edit_native_project_board_text(
                &path,
                text_uuid,
                value,
                x_nm,
                y_nm,
                rotation_deg,
                height_nm,
                stroke_width_nm,
                layer,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_text_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardText(ProjectDeleteBoardTextArgs { path, text_uuid }) => {
            let report = delete_native_project_board_text(&path, text_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_text_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceBoardKeepout(ProjectPlaceBoardKeepoutArgs {
            path,
            kind,
            layers,
            vertices,
        }) => {
            let polygon = parse_native_polygon_vertices(&vertices)?;
            let report = place_native_project_board_keepout(&path, kind, layers, polygon)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_keepout_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditBoardKeepout(ProjectEditBoardKeepoutArgs {
            path,
            keepout_uuid,
            kind,
            layers,
            vertices,
        }) => {
            let polygon = if vertices.is_empty() {
                None
            } else {
                Some(parse_native_polygon_vertices(&vertices)?)
            };
            let report =
                edit_native_project_board_keepout(&path, keepout_uuid, kind, layers, polygon)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_keepout_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardKeepout(ProjectDeleteBoardKeepoutArgs {
            path,
            keepout_uuid,
        }) => {
            let report = delete_native_project_board_keepout(&path, keepout_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_keepout_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetBoardOutline(ProjectSetBoardOutlineArgs { path, vertices }) => {
            let polygon = parse_native_polygon_vertices(&vertices)?;
            let report = set_native_project_board_outline(&path, polygon)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_outline_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
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
        }) => command_exec_board_net::execute_place_board_net(format, path, name, class_uuid),
        ProjectCommands::PlaceBoardComponent(ProjectPlaceBoardComponentArgs {
            path,
            part_uuid,
            package_uuid,
            reference,
            value,
            x_nm,
            y_nm,
            layer,
        }) => {
            let report = place_native_project_board_component(
                &path,
                part_uuid,
                package_uuid,
                reference,
                value,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                layer,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_component_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
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
        }) => {
            command_exec_board_net::execute_edit_board_net(format, path, net_uuid, name, class_uuid)
        }
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
        ProjectCommands::SetBoardComponentLayer(SetBoardComponentLayerArgs {
            path,
            component_uuid,
            layer,
        }) => command_exec_board_component::execute_set_board_component_layer(
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
        ProjectCommands::DeleteBoardComponent(ProjectDeleteBoardComponentArgs {
            path,
            component_uuid,
        }) => {
            let report = delete_native_project_board_component(&path, component_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_component_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DrawBoardTrack(ProjectDrawBoardTrackArgs {
            path,
            net_uuid,
            from_x_nm,
            from_y_nm,
            to_x_nm,
            to_y_nm,
            width_nm,
            layer,
        }) => {
            let report = place_native_project_board_track(
                &path,
                net_uuid,
                eda_engine::ir::geometry::Point {
                    x: from_x_nm,
                    y: from_y_nm,
                },
                eda_engine::ir::geometry::Point {
                    x: to_x_nm,
                    y: to_y_nm,
                },
                width_nm,
                layer,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_track_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardTrack(ProjectDeleteBoardTrackArgs { path, track_uuid }) => {
            let report = delete_native_project_board_track(&path, track_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_track_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceBoardVia(ProjectPlaceBoardViaArgs {
            path,
            net_uuid,
            x_nm,
            y_nm,
            drill_nm,
            diameter_nm,
            from_layer,
            to_layer,
        }) => {
            let report = place_native_project_board_via(
                &path,
                net_uuid,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                drill_nm,
                diameter_nm,
                from_layer,
                to_layer,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_via_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardVia(ProjectDeleteBoardViaArgs { path, via_uuid }) => {
            let report = delete_native_project_board_via(&path, via_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_via_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceBoardZone(ProjectPlaceBoardZoneArgs {
            path,
            net_uuid,
            vertices,
            layer,
            priority,
            thermal_relief,
            thermal_gap_nm,
            thermal_spoke_width_nm,
        }) => {
            let polygon = parse_native_polygon_vertices(&vertices)?;
            let report = place_native_project_board_zone(
                &path,
                net_uuid,
                polygon,
                layer,
                priority,
                thermal_relief,
                thermal_gap_nm,
                thermal_spoke_width_nm,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_zone_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardZone(ProjectDeleteBoardZoneArgs { path, zone_uuid }) => {
            let report = delete_native_project_board_zone(&path, zone_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_zone_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetBoardPadNet(ProjectSetBoardPadNetArgs {
            path,
            pad_uuid,
            net_uuid,
        }) => {
            let report = set_native_project_board_pad_net(&path, pad_uuid, Some(net_uuid))?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ClearBoardPadNet(ProjectClearBoardPadNetArgs { path, pad_uuid }) => {
            let report = set_native_project_board_pad_net(&path, pad_uuid, None)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditBoardPad(ProjectEditBoardPadArgs {
            path,
            pad_uuid,
            x_nm,
            y_nm,
            layer,
            shape,
            diameter_nm,
            width_nm,
            height_nm,
        }) => {
            let position = match (x_nm, y_nm) {
                (None, None) => None,
                (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                _ => bail!("board pad position requires both --x-nm and --y-nm"),
            };
            let report = edit_native_project_board_pad(
                &path,
                pad_uuid,
                position,
                layer,
                shape,
                diameter_nm,
                width_nm,
                height_nm,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceBoardPad(ProjectPlaceBoardPadArgs {
            path,
            package_uuid,
            name,
            x_nm,
            y_nm,
            layer,
            shape,
            diameter_nm,
            width_nm,
            height_nm,
            net_uuid,
        }) => {
            let report = place_native_project_board_pad(
                &path,
                package_uuid,
                name,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                layer,
                shape,
                diameter_nm,
                width_nm,
                height_nm,
                net_uuid,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardPad(ProjectDeleteBoardPadArgs { path, pad_uuid }) => {
            let report = delete_native_project_board_pad(&path, pad_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_pad_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardNetClass(ProjectDeleteBoardNetClassArgs {
            path,
            net_class_uuid,
        }) => command_exec_board_net::execute_delete_board_net_class(format, path, net_class_uuid),
        ProjectCommands::DeleteBoardNet(ProjectDeleteBoardNetArgs { path, net_uuid }) => {
            command_exec_board_net::execute_delete_board_net(format, path, net_uuid)
        }
        ProjectCommands::PlaceBoardDimension(args) => {
            let report = place_native_project_board_dimension(
                &args.path,
                eda_engine::ir::geometry::Point {
                    x: args.from_x_nm,
                    y: args.from_y_nm,
                },
                eda_engine::ir::geometry::Point {
                    x: args.to_x_nm,
                    y: args.to_y_nm,
                },
                args.layer,
                args.text,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_dimension_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditBoardDimension(args) => {
            let report = edit_native_project_board_dimension(
                &args.path,
                args.dimension_uuid,
                args.from_x_nm,
                args.from_y_nm,
                args.to_x_nm,
                args.to_y_nm,
                args.layer,
                args.text,
                args.clear_text,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_dimension_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBoardDimension(ProjectDeleteBoardDimensionArgs {
            path,
            dimension_uuid,
        }) => {
            let report = delete_native_project_board_dimension(&path, dimension_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_board_dimension_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        _ => unreachable!("board surface command should dispatch before board surface match"),
    }
}
