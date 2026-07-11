//! Runtime board-text edit helpers (decomposition of the gui-app monolith,
//! decision 021 / source-size governance): selection lookup plus the terminal
//! command-handoff prefill and quick-edit entry points for the selected board
//! text object. Split out of `main.rs`'s `impl Runtime`; behavior unchanged. A
//! child module of the crate root, so it sees `Runtime`'s private fields/methods
//! and the crate's board-text command helpers via `use super::*` exactly as the
//! inline impl did.

use super::*;

impl Runtime {
    pub(super) fn selected_board_text(&self) -> Option<&datum_gui_protocol::BoardTextPrimitive> {
        let datum_gui_protocol::SelectionTarget::AuthoredObject(object_id) =
            &self.workspace().selection
        else {
            return None;
        };
        self.workspace()
            .scene
            .board_texts
            .iter()
            .find(|text| &text.object_id == object_id)
    }

    pub(super) fn begin_selected_board_text_content_edit(&mut self) -> bool {
        let Some(command) = self.selected_board_text().map(|text| {
            board_text_edit_terminal_command(text, BoardTextEditTerminalField::Content)
        }) else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(command, "editing selected board text content")
    }

    pub(super) fn begin_selected_board_text_height_edit(&mut self) -> bool {
        let Some(command) = self
            .selected_board_text()
            .map(|text| board_text_edit_terminal_command(text, BoardTextEditTerminalField::Height))
        else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(command, "editing selected board text height")
    }

    pub(super) fn begin_selected_board_text_rotation_edit(&mut self) -> bool {
        let Some(command) = self.selected_board_text().map(|text| {
            board_text_edit_terminal_command(text, BoardTextEditTerminalField::Rotation)
        }) else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(command, "editing selected board text rotation")
    }

    pub(super) fn begin_selected_board_text_line_spacing_edit(&mut self) -> bool {
        let Some(command) = self.selected_board_text().map(|text| {
            board_text_edit_terminal_command(text, BoardTextEditTerminalField::LineSpacing)
        }) else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(
            command,
            "editing selected board text line spacing",
        )
    }

    pub(super) fn begin_selected_board_text_render_intent_edit(&mut self) -> bool {
        let Some(command) = self.selected_board_text().map(|text| {
            board_text_edit_terminal_command(text, BoardTextEditTerminalField::RenderIntent)
        }) else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(
            command,
            "editing selected board text render intent",
        )
    }

    pub(super) fn begin_selected_board_text_family_edit(&mut self) -> bool {
        let Some(command) = self
            .selected_board_text()
            .map(|text| board_text_edit_terminal_command(text, BoardTextEditTerminalField::Family))
        else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(command, "editing selected board text font")
    }

    pub(super) fn begin_selected_board_text_alignment_edit(&mut self) -> bool {
        let Some(command) = self.selected_board_text().map(|text| {
            board_text_edit_terminal_command(text, BoardTextEditTerminalField::Alignment)
        }) else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(
            command,
            "editing selected board text alignment",
        )
    }

    pub(super) fn begin_selected_board_text_command_edit(
        &mut self,
        command: String,
        event: impl Into<String>,
    ) -> bool {
        self.set_active_dock(DockTab::Terminal);
        if let Err(err) = record_manual_terminal_command_handoff(
            self.terminal_sessions.active(),
            "board_text_terminal_command",
            "datum.gui.board_text.edit_prefill",
            "prefill",
            &command,
        ) {
            self.push_terminal_line(format!("terminal handoff event write failed: {err}"));
        }
        self.write_terminal_bytes(command.as_bytes());
        self.invalidate_frame();
        self.log_review_event(event.into());
        true
    }

    pub(super) fn toggle_selected_board_text_boolean(&mut self, field: BoardTextBooleanField) -> bool {
        let field_label = match field {
            BoardTextBooleanField::Mirrored => "mirrored",
            BoardTextBooleanField::KeepUpright => "keep-upright",
            BoardTextBooleanField::Bold => "bold",
        };
        self.begin_selected_board_text_quick_edit(
            BoardTextQuickEditTerminalAction::ToggleBoolean(field),
            format!("editing selected board text {field_label}"),
        )
    }

    pub(super) fn cycle_selected_board_text_field(&mut self, field: BoardTextCycleField) -> bool {
        let field_label = match field {
            BoardTextCycleField::RenderIntent => "render intent",
            BoardTextCycleField::Family => "font family",
        };
        self.begin_selected_board_text_quick_edit(
            BoardTextQuickEditTerminalAction::CycleField(field),
            format!("editing selected board text {field_label}"),
        )
    }

    pub(super) fn cycle_selected_board_text_alignment(&mut self, field: BoardTextAlignmentField) -> bool {
        let field_label = match field {
            BoardTextAlignmentField::Horizontal => "horizontal align",
            BoardTextAlignmentField::Vertical => "vertical align",
        };
        self.begin_selected_board_text_quick_edit(
            BoardTextQuickEditTerminalAction::CycleAlignment(field),
            format!("editing selected board text {field_label}"),
        )
    }

    pub(super) fn step_selected_board_text_line_spacing(&mut self, step: BoardTextLineSpacingStep) -> bool {
        self.begin_selected_board_text_quick_edit(
            BoardTextQuickEditTerminalAction::StepLineSpacing(step),
            "editing selected board text line spacing".to_string(),
        )
    }

    pub(super) fn step_selected_board_text_height(&mut self, step: BoardTextHeightStep) -> bool {
        self.begin_selected_board_text_quick_edit(
            BoardTextQuickEditTerminalAction::StepHeight(step),
            "editing selected board text height".to_string(),
        )
    }

    pub(super) fn step_selected_board_text_rotation(&mut self, step: BoardTextRotationStep) -> bool {
        self.begin_selected_board_text_quick_edit(
            BoardTextQuickEditTerminalAction::StepRotation(step),
            "editing selected board text rotation".to_string(),
        )
    }

    pub(super) fn begin_selected_board_text_quick_edit(
        &mut self,
        action: BoardTextQuickEditTerminalAction,
        event: String,
    ) -> bool {
        let Some(command) = self
            .selected_board_text()
            .map(|text| board_text_quick_edit_terminal_command(text, action))
        else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(command, event)
    }
}
