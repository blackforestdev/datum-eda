use super::*;
use eda_engine::schematic::{LabelKind, PortDirection};

pub(super) fn execute_project_schematic_connectivity_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::CreateSheet(ProjectCreateSheetArgs { path, name, sheet }) => {
            let report = create_native_project_sheet(&path, name, sheet)?;
            let output = match format {
                OutputFormat::Text => render_native_project_sheet_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteSheet(ProjectDeleteSheetArgs { path, sheet }) => {
            let report = delete_native_project_sheet(&path, sheet)?;
            let output = match format {
                OutputFormat::Text => render_native_project_sheet_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::RenameSheet(ProjectRenameSheetArgs { path, sheet, name }) => {
            let report = rename_native_project_sheet(&path, sheet, name)?;
            let output = match format {
                OutputFormat::Text => render_native_project_sheet_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::CreateSheetDefinition(ProjectCreateSheetDefinitionArgs {
            path,
            root_sheet,
            name,
            definition,
        }) => {
            let report =
                create_native_project_sheet_definition(&path, root_sheet, name, definition)?;
            let output = match format {
                OutputFormat::Text => render_native_project_sheet_definition_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::CreateSheetInstance(ProjectCreateSheetInstanceArgs {
            path,
            definition,
            parent_sheet,
            name,
            x_nm,
            y_nm,
            instance,
        }) => {
            let report = create_native_project_sheet_instance(
                &path,
                definition,
                parent_sheet,
                name,
                x_nm,
                y_nm,
                instance,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_sheet_instance_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteSheetInstance(ProjectDeleteSheetInstanceArgs { path, instance }) => {
            let report = delete_native_project_sheet_instance(&path, instance)?;
            let output = match format {
                OutputFormat::Text => render_native_project_sheet_instance_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::MoveSheetInstance(ProjectMoveSheetInstanceArgs {
            path,
            instance,
            x_nm,
            y_nm,
        }) => {
            let report = move_native_project_sheet_instance(&path, instance, x_nm, y_nm)?;
            let output = match format {
                OutputFormat::Text => render_native_project_sheet_instance_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::BindSheetInstancePort(ProjectBindSheetInstancePortArgs {
            path,
            instance,
            port,
        }) => {
            let report = bind_native_project_sheet_instance_port(&path, instance, port)?;
            let output = match format {
                OutputFormat::Text => render_native_project_sheet_instance_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::UnbindSheetInstancePort(ProjectUnbindSheetInstancePortArgs {
            path,
            instance,
            port,
        }) => {
            let report = unbind_native_project_sheet_instance_port(&path, instance, port)?;
            let output = match format {
                OutputFormat::Text => render_native_project_sheet_instance_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceLabel(ProjectPlaceLabelArgs {
            path,
            sheet,
            name,
            kind,
            x_nm,
            y_nm,
        }) => {
            let kind = match kind {
                NativeLabelKindArg::Local => LabelKind::Local,
                NativeLabelKindArg::Global => LabelKind::Global,
                NativeLabelKindArg::Hierarchical => LabelKind::Hierarchical,
                NativeLabelKindArg::Power => LabelKind::Power,
            };
            let report = place_native_project_label(
                &path,
                sheet,
                name,
                kind,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_label_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::RenameLabel(ProjectRenameLabelArgs { path, label, name }) => {
            let report = rename_native_project_label(&path, label, name)?;
            let output = match format {
                OutputFormat::Text => render_native_project_label_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteLabel(ProjectDeleteLabelArgs { path, label }) => {
            let report = delete_native_project_label(&path, label)?;
            let output = match format {
                OutputFormat::Text => render_native_project_label_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DrawWire(ProjectDrawWireArgs {
            path,
            sheet,
            from_x_nm,
            from_y_nm,
            to_x_nm,
            to_y_nm,
        }) => {
            let report = draw_native_project_wire(
                &path,
                sheet,
                eda_engine::ir::geometry::Point {
                    x: from_x_nm,
                    y: from_y_nm,
                },
                eda_engine::ir::geometry::Point {
                    x: to_x_nm,
                    y: to_y_nm,
                },
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_wire_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteWire(ProjectDeleteWireArgs { path, wire }) => {
            let report = delete_native_project_wire(&path, wire)?;
            let output = match format {
                OutputFormat::Text => render_native_project_wire_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceJunction(ProjectPlaceJunctionArgs {
            path,
            sheet,
            x_nm,
            y_nm,
        }) => {
            let report = place_native_project_junction(
                &path,
                sheet,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_junction_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteJunction(ProjectDeleteJunctionArgs { path, junction }) => {
            let report = delete_native_project_junction(&path, junction)?;
            let output = match format {
                OutputFormat::Text => render_native_project_junction_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlacePort(ProjectPlacePortArgs {
            path,
            sheet,
            name,
            direction,
            x_nm,
            y_nm,
        }) => {
            let direction = match direction {
                NativePortDirectionArg::Input => PortDirection::Input,
                NativePortDirectionArg::Output => PortDirection::Output,
                NativePortDirectionArg::Bidirectional => PortDirection::Bidirectional,
                NativePortDirectionArg::Passive => PortDirection::Passive,
            };
            let report = place_native_project_port(
                &path,
                sheet,
                name,
                direction,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_port_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditPort(ProjectEditPortArgs {
            path,
            port,
            name,
            direction,
            x_nm,
            y_nm,
        }) => {
            let direction = direction.map(|value| match value {
                NativePortDirectionArg::Input => PortDirection::Input,
                NativePortDirectionArg::Output => PortDirection::Output,
                NativePortDirectionArg::Bidirectional => PortDirection::Bidirectional,
                NativePortDirectionArg::Passive => PortDirection::Passive,
            });
            let report = edit_native_project_port(&path, port, name, direction, x_nm, y_nm)?;
            let output = match format {
                OutputFormat::Text => render_native_project_port_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeletePort(ProjectDeletePortArgs { path, port }) => {
            let report = delete_native_project_port(&path, port)?;
            let output = match format {
                OutputFormat::Text => render_native_project_port_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::CreateBus(ProjectCreateBusArgs {
            path,
            sheet,
            name,
            members,
        }) => {
            let report = create_native_project_bus(&path, sheet, name, members)?;
            let output = match format {
                OutputFormat::Text => render_native_project_bus_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditBusMembers(ProjectEditBusMembersArgs { path, bus, members }) => {
            let report = edit_native_project_bus_members(&path, bus, members)?;
            let output = match format {
                OutputFormat::Text => render_native_project_bus_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBus(ProjectDeleteBusArgs { path, bus }) => {
            let report = delete_native_project_bus(&path, bus)?;
            let output = match format {
                OutputFormat::Text => render_native_project_bus_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceBusEntry(ProjectPlaceBusEntryArgs {
            path,
            sheet,
            bus,
            wire,
            x_nm,
            y_nm,
        }) => {
            let report = place_native_project_bus_entry(
                &path,
                sheet,
                bus,
                wire,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_bus_entry_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteBusEntry(ProjectDeleteBusEntryArgs { path, bus_entry }) => {
            let report = delete_native_project_bus_entry(&path, bus_entry)?;
            let output = match format {
                OutputFormat::Text => render_native_project_bus_entry_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceNoConnect(ProjectPlaceNoConnectArgs {
            path,
            sheet,
            symbol,
            pin,
            x_nm,
            y_nm,
        }) => {
            let report = place_native_project_noconnect(
                &path,
                sheet,
                symbol,
                pin,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_noconnect_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteNoConnect(ProjectDeleteNoConnectArgs { path, noconnect }) => {
            let report = delete_native_project_noconnect(&path, noconnect)?;
            let output = match format {
                OutputFormat::Text => render_native_project_noconnect_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        _ => {
            unreachable!("schematic connectivity command should dispatch before connectivity match")
        }
    }
}
