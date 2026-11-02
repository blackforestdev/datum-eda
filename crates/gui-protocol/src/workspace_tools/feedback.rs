//! Shared presentation of the current command; this does not own tool state.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTool {
    Select,
    Move,
    Wire,
    Bus,
    BusEntry,
    Field,
    Symbol,
    Power,
    Label,
    Port,
    Junction,
    NoConnect,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interface_port_feedback_preserves_command_identity_through_form_and_preview() {
        let mut state = crate::load_fixture_workspace_state();
        state.ui.layout = crate::WorkspaceLayout::board_schematic();
        state.ui.layout.focused = PaneId(1);
        for direction in [
            eda_engine::schematic::PortDirection::Input,
            eda_engine::schematic::PortDirection::Output,
            eda_engine::schematic::PortDirection::Bidirectional,
            eda_engine::schematic::PortDirection::Passive,
        ] {
            state.schematic_authoring.label =
                Some(crate::schematic_authoring::LabelDraft::port(direction));
            for picker in [true, false] {
                state.schematic_authoring.picker = picker;
                let tool = EditorTool::active(&state);
                assert_eq!(tool, EditorTool::Port);
                assert_eq!(tool.label(), "Port");
                assert_eq!(tool.menu_entry(), Some(("Place", "Port")));
            }
            state.schematic_authoring.cancel();
            assert_eq!(EditorTool::active(&state), EditorTool::Select);
        }
    }
    #[test]
    fn live_schematic_command_overrides_stale_generic_select_tool() {
        let mut state = crate::load_fixture_workspace_state();
        state.ui.layout = crate::WorkspaceLayout::board_schematic();
        state.ui.layout.focused = PaneId(1);
        assert_eq!(EditorTool::active(&state), EditorTool::Select);
        state.schematic_authoring.wire = true;
        assert_eq!(EditorTool::active(&state), EditorTool::Wire);
        state.schematic_authoring.cancel();
        state.schematic_authoring.label = Some(crate::schematic_authoring::LabelDraft::new(
            eda_engine::schematic::LabelKind::Power,
        ));
        assert_eq!(EditorTool::active(&state), EditorTool::Power);
        state.schematic_authoring.cancel();
        state.schematic_authoring.picker = true;
        assert_eq!(EditorTool::active(&state), EditorTool::Symbol);
        state.schematic_authoring.cancel();
        assert_eq!(EditorTool::active(&state), EditorTool::Select);
        for (kind, tool) in [
            (
                crate::schematic_authoring::MarkerKind::Junction,
                EditorTool::Junction,
            ),
            (
                crate::schematic_authoring::MarkerKind::NoConnect,
                EditorTool::NoConnect,
            ),
        ] {
            state.schematic_authoring.marker = Some(kind);
            assert_eq!(EditorTool::active(&state), tool);
            assert_eq!(tool.label(), kind.label());
            assert_eq!(tool.menu_entry(), Some(("Place", kind.label())));
            state.schematic_authoring.cancel();
            assert_eq!(EditorTool::active(&state), EditorTool::Select);
        }
        state.tool = WorkspaceTool::Move;
        assert_eq!(EditorTool::active(&state), EditorTool::Move);
    }
}

impl EditorTool {
    pub fn active(state: &ReviewWorkspaceState) -> Self {
        if state.ui.layout.focused_content() == PaneContent::Schematic {
            let schematic = &state.schematic_authoring;
            if schematic.field_placement().is_some() {
                return Self::Field;
            }
            if schematic.bus_entry.is_some() {
                return Self::BusEntry;
            }
            if schematic.bus {
                return Self::Bus;
            }
            if schematic.wire {
                return Self::Wire;
            }
            if let Some(marker) = schematic.marker {
                return match marker {
                    crate::schematic_authoring::MarkerKind::Junction => Self::Junction,
                    crate::schematic_authoring::MarkerKind::NoConnect => Self::NoConnect,
                };
            }
            if let Some(label) = &schematic.label {
                return if label.port_direction.is_some() {
                    Self::Port
                } else if label.kind == eda_engine::schematic::LabelKind::Power {
                    Self::Power
                } else {
                    Self::Label
                };
            }
            if schematic.picker || schematic.chosen.is_some() || schematic.editing.is_some() {
                return Self::Symbol;
            }
        }
        if state.tool == WorkspaceTool::Move {
            Self::Move
        } else {
            Self::Select
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Move => "Move",
            Self::Wire => "Wire",
            Self::Bus => "Bus",
            Self::BusEntry => "Bus Entry",
            Self::Field => "Field text",
            Self::Symbol => "Symbol",
            Self::Power => "Power",
            Self::Label => "Label",
            Self::Port => "Port",
            Self::Junction => "Junction",
            Self::NoConnect => "No-Connect",
        }
    }

    /// Existing command entry shared with the menu, including its readiness gate.
    pub fn menu_entry(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Wire => Some(("Place", "Wire")),
            Self::Bus => Some(("Place", "Bus")),
            Self::BusEntry => Some(("Place", "Bus Entry")),
            Self::Symbol => Some(("Place", "Symbol")),
            Self::Power => Some(("Place", "Power Port")),
            Self::Label => Some(("Place", "Net Label")),
            Self::Port => Some(("Place", "Port")),
            Self::Junction => Some(("Place", "Junction")),
            Self::NoConnect => Some(("Place", "No-Connect")),
            Self::Select | Self::Move | Self::Field => None,
        }
    }
}
