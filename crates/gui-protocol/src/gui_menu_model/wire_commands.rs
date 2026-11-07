//! Typed context configuration for the existing shared Move and polyline tools.
use eda_engine::api::native_write::schematic_wire_edit::WireRouting;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireCommand {
    Split,
    RemoveSpan,
    RemoveVertex,
    InsertVertex,
    ReconnectEnd,
    SetRouting(WireRouting),
    SwapCorner,
    RemoveLastVertex,
}
impl WireCommand {
    pub fn from_action(action: &str) -> Option<Self> {
        Some(match action {
            "schematic.wire.split" => Self::Split,
            "schematic.wire.remove_span" => Self::RemoveSpan,
            "schematic.wire.remove_vertex" => Self::RemoveVertex,
            "schematic.wire.insert_vertex" => Self::InsertVertex,
            "schematic.wire.reconnect_end" => Self::ReconnectEnd,
            "schematic.wire.route.free" => Self::SetRouting(WireRouting::Free),
            "schematic.wire.route.90" => Self::SetRouting(WireRouting::Orthogonal),
            "schematic.wire.route.45" => Self::SetRouting(WireRouting::Diagonal45),
            "schematic.wire.corner.swap" => Self::SwapCorner,
            "schematic.wire.vertex.undo" => Self::RemoveLastVertex,
            _ => return None,
        })
    }
    pub fn is_configuration(self) -> bool {
        matches!(
            self,
            Self::SetRouting(_) | Self::SwapCorner | Self::RemoveLastVertex
        )
    }
    pub fn unavailable_reason(self, state: &crate::ReviewWorkspaceState) -> Option<&'static str> {
        if state.ui.layout.focused_content() != crate::PaneContent::Schematic {
            return Some("Focus the schematic source pane");
        }
        if self == Self::RemoveLastVertex
            && (!state.schematic_authoring.wire || state.schematic_authoring.vertices.is_empty())
        {
            return Some("Place a draft wire vertex first");
        }
        None
    }
}
