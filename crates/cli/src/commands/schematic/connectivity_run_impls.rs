use super::*;
use super::connectivity_mutations::*;

// Phase 5: exec-layer dissolution — variant run() impls (the former
// command_exec destructure-and-forward glue, now inherent methods on the
// clap args structs).

impl ProjectCreateSheetArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, name, sheet } = self;
        let report = create_native_project_sheet(&path, name, sheet)?;
        let output = render_report(format, &report, render_native_project_sheet_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDeleteSheetArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, sheet } = self;
        let report = delete_native_project_sheet(&path, sheet)?;
        let output = render_report(format, &report, render_native_project_sheet_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectRenameSheetArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, sheet, name } = self;
        let report = rename_native_project_sheet(&path, sheet, name)?;
        let output = render_report(format, &report, render_native_project_sheet_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectCreateSheetDefinitionArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            root_sheet,
            name,
            definition,
        } = self;
        let report = create_native_project_sheet_definition(&path, root_sheet, name, definition)?;
        let output = render_report(
            format,
            &report,
            render_native_project_sheet_definition_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectCreateSheetInstanceArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            definition,
            parent_sheet,
            name,
            x_nm,
            y_nm,
            instance,
        } = self;
        let report = create_native_project_sheet_instance(
            &path,
            definition,
            parent_sheet,
            name,
            x_nm,
            y_nm,
            instance,
        )?;
        let output = render_report(
            format,
            &report,
            render_native_project_sheet_instance_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectDeleteSheetInstanceArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, instance } = self;
        let report = delete_native_project_sheet_instance(&path, instance)?;
        let output = render_report(
            format,
            &report,
            render_native_project_sheet_instance_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectMoveSheetInstanceArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            instance,
            x_nm,
            y_nm,
        } = self;
        let report = move_native_project_sheet_instance(&path, instance, x_nm, y_nm)?;
        let output = render_report(
            format,
            &report,
            render_native_project_sheet_instance_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectBindSheetInstancePortArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            instance,
            port,
        } = self;
        let report = bind_native_project_sheet_instance_port(&path, instance, port)?;
        let output = render_report(
            format,
            &report,
            render_native_project_sheet_instance_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectUnbindSheetInstancePortArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            instance,
            port,
        } = self;
        let report = unbind_native_project_sheet_instance_port(&path, instance, port)?;
        let output = render_report(
            format,
            &report,
            render_native_project_sheet_instance_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectPlaceLabelArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            name,
            kind,
            x_nm,
            y_nm,
        } = self;
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
        let output = render_report(format, &report, render_native_project_label_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectRenameLabelArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, label, name } = self;
        let report = rename_native_project_label(&path, label, name)?;
        let output = render_report(format, &report, render_native_project_label_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDeleteLabelArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, label } = self;
        let report = delete_native_project_label(&path, label)?;
        let output = render_report(format, &report, render_native_project_label_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDrawWireArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            from_x_nm,
            from_y_nm,
            to_x_nm,
            to_y_nm,
        } = self;
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
        let output = render_report(format, &report, render_native_project_wire_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDeleteWireArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, wire } = self;
        let report = delete_native_project_wire(&path, wire)?;
        let output = render_report(format, &report, render_native_project_wire_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectPlaceJunctionArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            x_nm,
            y_nm,
        } = self;
        let report = place_native_project_junction(
            &path,
            sheet,
            eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
        )?;
        let output = render_report(
            format,
            &report,
            render_native_project_junction_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectDeleteJunctionArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, junction } = self;
        let report = delete_native_project_junction(&path, junction)?;
        let output = render_report(
            format,
            &report,
            render_native_project_junction_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectPlacePortArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            name,
            direction,
            x_nm,
            y_nm,
        } = self;
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
        let output = render_report(format, &report, render_native_project_port_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectEditPortArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            port,
            name,
            direction,
            x_nm,
            y_nm,
        } = self;
        let direction = direction.map(|value| match value {
            NativePortDirectionArg::Input => PortDirection::Input,
            NativePortDirectionArg::Output => PortDirection::Output,
            NativePortDirectionArg::Bidirectional => PortDirection::Bidirectional,
            NativePortDirectionArg::Passive => PortDirection::Passive,
        });
        let report = edit_native_project_port(&path, port, name, direction, x_nm, y_nm)?;
        let output = render_report(format, &report, render_native_project_port_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDeletePortArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, port } = self;
        let report = delete_native_project_port(&path, port)?;
        let output = render_report(format, &report, render_native_project_port_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectCreateBusArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            name,
            members,
        } = self;
        let report = create_native_project_bus(&path, sheet, name, members)?;
        let output = render_report(format, &report, render_native_project_bus_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectEditBusMembersArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, bus, members } = self;
        let report = edit_native_project_bus_members(&path, bus, members)?;
        let output = render_report(format, &report, render_native_project_bus_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDeleteBusArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, bus } = self;
        let report = delete_native_project_bus(&path, bus)?;
        let output = render_report(format, &report, render_native_project_bus_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectPlaceBusEntryArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            bus,
            wire,
            x_nm,
            y_nm,
        } = self;
        let report = place_native_project_bus_entry(
            &path,
            sheet,
            bus,
            wire,
            eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
        )?;
        let output = render_report(
            format,
            &report,
            render_native_project_bus_entry_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectDeleteBusEntryArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, bus_entry } = self;
        let report = delete_native_project_bus_entry(&path, bus_entry)?;
        let output = render_report(
            format,
            &report,
            render_native_project_bus_entry_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectPlaceNoConnectArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            symbol,
            pin,
            x_nm,
            y_nm,
        } = self;
        let report = place_native_project_noconnect(
            &path,
            sheet,
            symbol,
            pin,
            eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
        )?;
        let output = render_report(
            format,
            &report,
            render_native_project_noconnect_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectDeleteNoConnectArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, noconnect } = self;
        let report = delete_native_project_noconnect(&path, noconnect)?;
        let output = render_report(
            format,
            &report,
            render_native_project_noconnect_mutation_text,
        );
        Ok((output, 0))
    }
}
