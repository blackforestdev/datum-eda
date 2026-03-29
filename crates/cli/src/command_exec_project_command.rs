use super::*;

pub(super) fn execute_project_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        command @ ProjectCommands::PlaceLabel(ProjectPlaceLabelArgs { .. })
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
        | command @ ProjectCommands::PlaceBoardComponent(ProjectPlaceBoardComponentArgs { .. })
        | command @ ProjectCommands::DeleteBoardComponent(ProjectDeleteBoardComponentArgs { .. })
        | command @ ProjectCommands::DrawBoardTrack(ProjectDrawBoardTrackArgs { .. })
        | command @ ProjectCommands::DeleteBoardTrack(ProjectDeleteBoardTrackArgs { .. })
        | command @ ProjectCommands::PlaceBoardVia(ProjectPlaceBoardViaArgs { .. })
        | command @ ProjectCommands::DeleteBoardVia(ProjectDeleteBoardViaArgs { .. })
        | command @ ProjectCommands::PlaceBoardZone(ProjectPlaceBoardZoneArgs { .. })
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
        ProjectCommands::PlaceBoardNet(ProjectPlaceBoardNetArgs { path, name, class_uuid }) => {
            command_exec_board_net::execute_place_board_net(format, path, name, class_uuid)
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
        }) => command_exec_board_net::execute_edit_board_net(format, path, net_uuid, name, class_uuid),
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
        ProjectCommands::DeleteBoardNetClass(ProjectDeleteBoardNetClassArgs { path, net_class_uuid }) => {
            command_exec_board_net::execute_delete_board_net_class(format, path, net_class_uuid)
        }
        ProjectCommands::DeleteBoardNet(ProjectDeleteBoardNetArgs { path, net_uuid }) => {
            command_exec_board_net::execute_delete_board_net(format, path, net_uuid)
        }
        _ => unreachable!("inventory command should dispatch before project match"),
    }
}
