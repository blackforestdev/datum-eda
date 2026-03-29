use super::command_exec_native_support::{
    parse_native_hidden_power_behavior, parse_native_symbol_display_mode,
};
use super::*;

pub(super) fn execute_project_schematic_symbols_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::PlaceSymbol(ProjectPlaceSymbolArgs {
            path,
            sheet,
            reference,
            value,
            lib_id,
            x_nm,
            y_nm,
            rotation_deg,
            mirrored,
        }) => {
            let report = place_native_project_symbol(
                &path,
                sheet,
                reference,
                value,
                lib_id,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                rotation_deg,
                mirrored,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::MoveSymbol(ProjectMoveSymbolArgs {
            path,
            symbol,
            x_nm,
            y_nm,
        }) => {
            let report = move_native_project_symbol(
                &path,
                symbol,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::RotateSymbol(ProjectRotateSymbolArgs {
            path,
            symbol,
            rotation_deg,
        }) => {
            let report = rotate_native_project_symbol(&path, symbol, rotation_deg)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::MirrorSymbol(ProjectMirrorSymbolArgs { path, symbol }) => {
            let report = mirror_native_project_symbol(&path, symbol)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteSymbol(ProjectDeleteSymbolArgs { path, symbol }) => {
            let report = delete_native_project_symbol(&path, symbol)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetSymbolReference(ProjectSetSymbolReferenceArgs {
            path,
            symbol,
            reference,
        }) => {
            let report = set_native_project_symbol_reference(&path, symbol, reference)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetSymbolValue(ProjectSetSymbolValueArgs {
            path,
            symbol,
            value,
        }) => {
            let report = set_native_project_symbol_value(&path, symbol, value)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetSymbolLibId(ProjectSetSymbolLibIdArgs {
            path,
            symbol_uuid,
            lib_id,
        }) => {
            let report = set_native_project_symbol_lib_id(&path, symbol_uuid, lib_id)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ClearSymbolLibId(ProjectClearSymbolLibIdArgs { path, symbol_uuid }) => {
            let report = clear_native_project_symbol_lib_id(&path, symbol_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetSymbolEntity(ProjectSetSymbolEntityArgs {
            path,
            symbol,
            entity_uuid,
        }) => {
            let report = set_native_project_symbol_entity(&path, symbol, entity_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ClearSymbolEntity(ProjectClearSymbolEntityArgs { path, symbol }) => {
            let report = clear_native_project_symbol_entity(&path, symbol)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetSymbolPart(ProjectSetSymbolPartArgs {
            path,
            symbol,
            part_uuid,
        }) => {
            let report = set_native_project_symbol_part(&path, symbol, part_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ClearSymbolPart(ProjectClearSymbolPartArgs { path, symbol }) => {
            let report = clear_native_project_symbol_part(&path, symbol)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetSymbolUnit(ProjectSetSymbolUnitArgs {
            path,
            symbol,
            unit_selection,
        }) => {
            let report = set_native_project_symbol_unit(&path, symbol, unit_selection)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ClearSymbolUnit(ProjectClearSymbolUnitArgs { path, symbol }) => {
            let report = clear_native_project_symbol_unit(&path, symbol)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetSymbolGate(ProjectSetSymbolGateArgs {
            path,
            symbol,
            gate_uuid,
        }) => {
            let report = set_native_project_symbol_gate(&path, symbol, gate_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ClearSymbolGate(ProjectClearSymbolGateArgs { path, symbol }) => {
            let report = clear_native_project_symbol_gate(&path, symbol)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetSymbolDisplayMode(ProjectSetSymbolDisplayModeArgs {
            path,
            symbol,
            display_mode,
        }) => {
            let report = set_native_project_symbol_display_mode(
                &path,
                symbol,
                parse_native_symbol_display_mode(display_mode),
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetPinOverride(ProjectSetPinOverrideArgs {
            path,
            symbol,
            pin_uuid,
            visible,
            x_nm,
            y_nm,
        }) => {
            let position = parse_native_field_position(x_nm, y_nm)?;
            let report =
                set_native_project_symbol_pin_override(&path, symbol, pin_uuid, visible, position)?;
            let output = match format {
                OutputFormat::Text => render_native_project_pin_override_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SetSymbolHiddenPowerBehavior(
            ProjectSetSymbolHiddenPowerBehaviorArgs {
                path,
                symbol,
                hidden_power_behavior,
            },
        ) => {
            let report = set_native_project_symbol_hidden_power_behavior(
                &path,
                symbol,
                parse_native_hidden_power_behavior(hidden_power_behavior),
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ClearPinOverride(ProjectClearPinOverrideArgs {
            path,
            symbol,
            pin_uuid,
        }) => {
            let report = clear_native_project_symbol_pin_override(&path, symbol, pin_uuid)?;
            let output = match format {
                OutputFormat::Text => render_native_project_pin_override_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::AddSymbolField(ProjectAddSymbolFieldArgs {
            path,
            symbol,
            key,
            value,
            hidden,
            x_nm,
            y_nm,
        }) => {
            let report = add_native_project_symbol_field(
                &path,
                symbol,
                key,
                value,
                !hidden,
                parse_native_field_position(x_nm, y_nm)?,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_field_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditSymbolField(ProjectEditSymbolFieldArgs {
            path,
            field,
            key,
            value,
            visible,
            x_nm,
            y_nm,
            ..
        }) => {
            let report = edit_native_project_symbol_field(
                &path,
                field,
                key,
                value,
                visible,
                parse_native_field_position(x_nm, y_nm)?,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_field_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteSymbolField(ProjectDeleteSymbolFieldArgs { path, field }) => {
            let report = delete_native_project_symbol_field(&path, field)?;
            let output = match format {
                OutputFormat::Text => render_native_project_symbol_field_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceText(ProjectPlaceTextArgs {
            path,
            sheet,
            text,
            x_nm,
            y_nm,
            rotation_deg,
        }) => {
            let report = place_native_project_text(
                &path,
                sheet,
                text,
                eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
                rotation_deg,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_text_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditText(ProjectEditTextArgs {
            path,
            text,
            value,
            x_nm,
            y_nm,
            rotation_deg,
        }) => {
            let position = match (x_nm, y_nm) {
                (None, None) => None,
                (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                _ => bail!("text position requires both --x-nm and --y-nm"),
            };
            let report = edit_native_project_text(&path, text, value, position, rotation_deg)?;
            let output = match format {
                OutputFormat::Text => render_native_project_text_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteText(ProjectDeleteTextArgs { path, text }) => {
            let report = delete_native_project_text(&path, text)?;
            let output = match format {
                OutputFormat::Text => render_native_project_text_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceDrawingLine(ProjectPlaceDrawingLineArgs {
            path,
            sheet,
            from_x_nm,
            from_y_nm,
            to_x_nm,
            to_y_nm,
        }) => {
            let report = place_native_project_drawing_line(
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
                OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceDrawingRect(ProjectPlaceDrawingRectArgs {
            path,
            sheet,
            min_x_nm,
            min_y_nm,
            max_x_nm,
            max_y_nm,
        }) => {
            let report = place_native_project_drawing_rect(
                &path,
                sheet,
                eda_engine::ir::geometry::Point {
                    x: min_x_nm,
                    y: min_y_nm,
                },
                eda_engine::ir::geometry::Point {
                    x: max_x_nm,
                    y: max_y_nm,
                },
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceDrawingCircle(ProjectPlaceDrawingCircleArgs {
            path,
            sheet,
            center_x_nm,
            center_y_nm,
            radius_nm,
        }) => {
            let report = place_native_project_drawing_circle(
                &path,
                sheet,
                eda_engine::ir::geometry::Point {
                    x: center_x_nm,
                    y: center_y_nm,
                },
                radius_nm,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlaceDrawingArc(ProjectPlaceDrawingArcArgs {
            path,
            sheet,
            center_x_nm,
            center_y_nm,
            radius_nm,
            start_angle_mdeg,
            end_angle_mdeg,
        }) => {
            let report = place_native_project_drawing_arc(
                &path,
                sheet,
                eda_engine::ir::geometry::Arc {
                    center: eda_engine::ir::geometry::Point {
                        x: center_x_nm,
                        y: center_y_nm,
                    },
                    radius: radius_nm,
                    start_angle: start_angle_mdeg,
                    end_angle: end_angle_mdeg,
                },
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditDrawingLine(ProjectEditDrawingLineArgs {
            path,
            drawing,
            from_x_nm,
            from_y_nm,
            to_x_nm,
            to_y_nm,
        }) => {
            let from = match (from_x_nm, from_y_nm) {
                (None, None) => None,
                (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                _ => bail!("drawing start requires both --from-x-nm and --from-y-nm"),
            };
            let to = match (to_x_nm, to_y_nm) {
                (None, None) => None,
                (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                _ => bail!("drawing end requires both --to-x-nm and --to-y-nm"),
            };
            let report = edit_native_project_drawing_line(&path, drawing, from, to)?;
            let output = match format {
                OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditDrawingRect(ProjectEditDrawingRectArgs {
            path,
            drawing,
            min_x_nm,
            min_y_nm,
            max_x_nm,
            max_y_nm,
        }) => {
            let min = match (min_x_nm, min_y_nm) {
                (None, None) => None,
                (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                _ => bail!("rect min requires both --min-x-nm and --min-y-nm"),
            };
            let max = match (max_x_nm, max_y_nm) {
                (None, None) => None,
                (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                _ => bail!("rect max requires both --max-x-nm and --max-y-nm"),
            };
            let report = edit_native_project_drawing_rect(&path, drawing, min, max)?;
            let output = match format {
                OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditDrawingCircle(ProjectEditDrawingCircleArgs {
            path,
            drawing,
            center_x_nm,
            center_y_nm,
            radius_nm,
        }) => {
            let center = match (center_x_nm, center_y_nm) {
                (None, None) => None,
                (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                _ => bail!("circle center requires both --center-x-nm and --center-y-nm"),
            };
            let report = edit_native_project_drawing_circle(&path, drawing, center, radius_nm)?;
            let output = match format {
                OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::EditDrawingArc(ProjectEditDrawingArcArgs {
            path,
            drawing,
            center_x_nm,
            center_y_nm,
            radius_nm,
            start_angle_mdeg,
            end_angle_mdeg,
        }) => {
            let center = match (center_x_nm, center_y_nm) {
                (None, None) => None,
                (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
                _ => bail!("arc center requires both --center-x-nm and --center-y-nm"),
            };
            let report = edit_native_project_drawing_arc(
                &path,
                drawing,
                center,
                radius_nm,
                start_angle_mdeg,
                end_angle_mdeg,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeleteDrawing(ProjectDeleteDrawingArgs { path, drawing }) => {
            let report = delete_native_project_drawing(&path, drawing)?;
            let output = match format {
                OutputFormat::Text => render_native_project_drawing_mutation_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        _ => unreachable!("schematic symbol command should dispatch before symbol match"),
    }
}
