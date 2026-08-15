//! Workspace character shortcuts as one typed production dispatch table.

use crate::Runtime;
use datum_gui_protocol::{SessionCommand, WorkspaceTool};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WorkspaceCharacterAction {
    SetTool(WorkspaceTool),
    FitCamera,
    FitReviewTarget,
    TogglePaneZoom,
    CycleCrosshair,
    PreviousReview,
    NextReview,
    Consume,
}

pub(super) fn character_action(text: &str, control: bool) -> Option<WorkspaceCharacterAction> {
    let action = match text {
        value if value.eq_ignore_ascii_case("s") => {
            WorkspaceCharacterAction::SetTool(WorkspaceTool::Select)
        }
        value if value.eq_ignore_ascii_case("b") => {
            WorkspaceCharacterAction::SetTool(WorkspaceTool::PlaceBoardText)
        }
        value if value.eq_ignore_ascii_case("v") => {
            WorkspaceCharacterAction::SetTool(WorkspaceTool::PlaceBoardVia)
        }
        value if value.eq_ignore_ascii_case("m") => {
            WorkspaceCharacterAction::SetTool(WorkspaceTool::Move)
        }
        value if value.eq_ignore_ascii_case("x") => {
            WorkspaceCharacterAction::SetTool(WorkspaceTool::Delete)
        }
        value if value.eq_ignore_ascii_case("r") => {
            WorkspaceCharacterAction::SetTool(WorkspaceTool::DrawBoardTrack)
        }
        value if value.eq_ignore_ascii_case("f") => WorkspaceCharacterAction::FitCamera,
        value if value.eq_ignore_ascii_case("t") => WorkspaceCharacterAction::FitReviewTarget,
        value if value.eq_ignore_ascii_case("z") => WorkspaceCharacterAction::TogglePaneZoom,
        value if value.eq_ignore_ascii_case("c") && control => WorkspaceCharacterAction::Consume,
        value if value.eq_ignore_ascii_case("c") => WorkspaceCharacterAction::CycleCrosshair,
        "[" => WorkspaceCharacterAction::PreviousReview,
        "]" => WorkspaceCharacterAction::NextReview,
        _ => return None,
    };
    Some(action)
}

pub(super) fn apply(runtime: &mut Runtime, action: WorkspaceCharacterAction) -> bool {
    match action {
        WorkspaceCharacterAction::SetTool(tool) => runtime.set_workspace_tool(tool),
        WorkspaceCharacterAction::FitCamera => {
            runtime.fit_camera();
            true
        }
        WorkspaceCharacterAction::FitReviewTarget => runtime.fit_review_target(),
        WorkspaceCharacterAction::TogglePaneZoom => {
            runtime.pane_toggle_zoom();
            true
        }
        WorkspaceCharacterAction::CycleCrosshair => {
            runtime.cycle_crosshair_style();
            true
        }
        WorkspaceCharacterAction::PreviousReview => runtime
            .dispatch_session_command(SessionCommand::SelectPreviousReviewAction),
        WorkspaceCharacterAction::NextReview => runtime
            .dispatch_session_command(SessionCommand::SelectNextReviewAction),
        WorkspaceCharacterAction::Consume => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{WorkspaceCharacterAction, character_action};
    use datum_gui_protocol::WorkspaceTool;

    #[test]
    fn production_character_dispatch_table_covers_every_workspace_hotkey() {
        for (key, expected) in [
            ("s", WorkspaceCharacterAction::SetTool(WorkspaceTool::Select)),
            ("b", WorkspaceCharacterAction::SetTool(WorkspaceTool::PlaceBoardText)),
            ("v", WorkspaceCharacterAction::SetTool(WorkspaceTool::PlaceBoardVia)),
            ("m", WorkspaceCharacterAction::SetTool(WorkspaceTool::Move)),
            ("x", WorkspaceCharacterAction::SetTool(WorkspaceTool::Delete)),
            ("r", WorkspaceCharacterAction::SetTool(WorkspaceTool::DrawBoardTrack)),
            ("f", WorkspaceCharacterAction::FitCamera),
            ("t", WorkspaceCharacterAction::FitReviewTarget),
            ("z", WorkspaceCharacterAction::TogglePaneZoom),
            ("c", WorkspaceCharacterAction::CycleCrosshair),
            ("[", WorkspaceCharacterAction::PreviousReview),
            ("]", WorkspaceCharacterAction::NextReview),
        ] {
            assert_eq!(character_action(key, false), Some(expected));
            assert_eq!(character_action(&key.to_ascii_uppercase(), false), Some(expected));
        }
        assert_eq!(character_action("c", true), Some(WorkspaceCharacterAction::Consume));
        assert_eq!(character_action("q", false), None);
    }
}
