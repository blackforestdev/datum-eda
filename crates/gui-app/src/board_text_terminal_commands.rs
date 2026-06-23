use datum_gui_protocol::{
    BOARD_TEXT_HEIGHT_MAX_NM, BOARD_TEXT_HEIGHT_MIN_NM, BOARD_TEXT_HEIGHT_STEP_PPM,
    BOARD_TEXT_LINE_SPACING_MAX_PPM, BOARD_TEXT_LINE_SPACING_MIN_PPM,
    BOARD_TEXT_LINE_SPACING_STEP_PPM, BoardTextAlignmentField, BoardTextBooleanField,
    BoardTextCycleField, BoardTextHeightStep, BoardTextLineSpacingStep, BoardTextPrimitive,
    BoardTextRotationStep,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BoardTextEditTerminalField {
    Content,
    Height,
    Rotation,
    LineSpacing,
    RenderIntent,
    Family,
    Alignment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BoardTextQuickEditTerminalAction {
    ToggleBoolean(BoardTextBooleanField),
    CycleField(BoardTextCycleField),
    CycleAlignment(BoardTextAlignmentField),
    StepLineSpacing(BoardTextLineSpacingStep),
    StepHeight(BoardTextHeightStep),
    StepRotation(BoardTextRotationStep),
}

pub(super) fn board_text_edit_terminal_command(
    text: &BoardTextPrimitive,
    field: BoardTextEditTerminalField,
) -> String {
    let base = board_text_edit_base(text);
    match field {
        BoardTextEditTerminalField::Content => {
            format!("{base} --value {}", shell_word(&text.text))
        }
        BoardTextEditTerminalField::Height => format!("{base} --height-nm {}", text.height_nm),
        BoardTextEditTerminalField::Rotation => {
            format!(
                "{base} --rotation-deg {}",
                text.rotation_degrees.rem_euclid(360)
            )
        }
        BoardTextEditTerminalField::LineSpacing => {
            format!(
                "{base} --line-spacing-ratio-ppm {}",
                text.line_spacing_ratio_ppm
            )
        }
        BoardTextEditTerminalField::RenderIntent => {
            format!("{base} --render-intent {}", shell_word(&text.render_intent))
        }
        BoardTextEditTerminalField::Family => {
            format!("{base} --family {}", shell_word(&text.family))
        }
        BoardTextEditTerminalField::Alignment => {
            format!(
                "{base} --h-align {} --v-align {}",
                shell_word(&text.h_align),
                shell_word(&text.v_align)
            )
        }
    }
}

pub(super) fn board_text_quick_edit_terminal_command(
    text: &BoardTextPrimitive,
    action: BoardTextQuickEditTerminalAction,
) -> String {
    let base = board_text_edit_base(text);
    match action {
        BoardTextQuickEditTerminalAction::ToggleBoolean(field) => {
            let (arg, value) = match field {
                BoardTextBooleanField::Mirrored => ("--mirrored", !text.mirrored),
                BoardTextBooleanField::KeepUpright => ("--keep-upright", !text.keep_upright),
                BoardTextBooleanField::Bold => ("--bold", !text.bold),
            };
            format!("{base} {arg} {value}")
        }
        BoardTextQuickEditTerminalAction::CycleField(BoardTextCycleField::RenderIntent) => {
            let next = next_render_intent(&text.render_intent);
            format!("{base} --render-intent {}", shell_word(next))
        }
        BoardTextQuickEditTerminalAction::CycleField(BoardTextCycleField::Family) => {
            let next = next_font_family(&text.family);
            format!("{base} --family {}", shell_word(next))
        }
        BoardTextQuickEditTerminalAction::CycleAlignment(BoardTextAlignmentField::Horizontal) => {
            let next = next_h_align(&text.h_align);
            format!("{base} --h-align {}", shell_word(next))
        }
        BoardTextQuickEditTerminalAction::CycleAlignment(BoardTextAlignmentField::Vertical) => {
            let next = next_v_align(&text.v_align);
            format!("{base} --v-align {}", shell_word(next))
        }
        BoardTextQuickEditTerminalAction::StepLineSpacing(step) => {
            let next = step_line_spacing(text.line_spacing_ratio_ppm, step);
            format!("{base} --line-spacing-ratio-ppm {next}")
        }
        BoardTextQuickEditTerminalAction::StepHeight(step) => {
            let (next_height, next_stroke) =
                step_height_and_stroke(text.height_nm, text.stroke_width_nm, step);
            format!("{base} --height-nm {next_height} --stroke-width-nm {next_stroke}")
        }
        BoardTextQuickEditTerminalAction::StepRotation(step) => {
            let next = step_rotation(text.rotation_degrees, step);
            format!("{base} --rotation-deg {next}")
        }
    }
}

fn board_text_edit_base(text: &BoardTextPrimitive) -> String {
    let base = format!(
        "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text {}",
        shell_word(&text.text_uuid),
    );
    base
}

fn next_render_intent(current: &str) -> &'static str {
    match current {
        "manufacturing" => "annotation",
        "annotation" => "branding",
        "branding" => "documentation",
        "documentation" => "ui_preview",
        "ui_preview" => "manufacturing",
        _ => "manufacturing",
    }
}

fn next_font_family(current: &str) -> &'static str {
    match current {
        "newstroke" => "inter",
        "inter" => "inter_display",
        "inter_display" => "ibm_plex_sans_condensed",
        "ibm_plex_sans_condensed" => "jetbrains_mono",
        "jetbrains_mono" => "newstroke",
        _ => "newstroke",
    }
}

fn next_h_align(current: &str) -> &'static str {
    match current {
        "left" => "center",
        "center" => "right",
        "right" => "left",
        _ => "left",
    }
}

fn next_v_align(current: &str) -> &'static str {
    match current {
        "top" => "center",
        "center" => "bottom",
        "bottom" => "top",
        _ => "bottom",
    }
}

fn step_line_spacing(current: i32, step: BoardTextLineSpacingStep) -> i32 {
    let next = match step {
        BoardTextLineSpacingStep::Decrease => current - BOARD_TEXT_LINE_SPACING_STEP_PPM,
        BoardTextLineSpacingStep::Increase => current + BOARD_TEXT_LINE_SPACING_STEP_PPM,
    };
    next.clamp(
        BOARD_TEXT_LINE_SPACING_MIN_PPM,
        BOARD_TEXT_LINE_SPACING_MAX_PPM,
    )
}

fn step_height_and_stroke(
    current_height: i64,
    current_stroke: i64,
    step: BoardTextHeightStep,
) -> (i64, i64) {
    let safe_height = current_height.max(1);
    let safe_stroke = current_stroke.max(1);
    let delta = ((safe_height as i128 * BOARD_TEXT_HEIGHT_STEP_PPM as i128) / 1_000_000_i128)
        .max(1_i128) as i64;
    let next_height = match step {
        BoardTextHeightStep::Decrease => safe_height - delta,
        BoardTextHeightStep::Increase => safe_height + delta,
    }
    .clamp(BOARD_TEXT_HEIGHT_MIN_NM, BOARD_TEXT_HEIGHT_MAX_NM);
    let next_stroke = ((safe_stroke as i128 * next_height as i128 + (safe_height as i128 / 2_i128))
        / safe_height as i128)
        .max(1_i128) as i64;
    (next_height, next_stroke)
}

fn step_rotation(current: i32, step: BoardTextRotationStep) -> i32 {
    let delta = match step {
        BoardTextRotationStep::CounterClockwise90 => -90,
        BoardTextRotationStep::Clockwise90 => 90,
    };
    (current + delta).rem_euclid(360)
}

fn shell_word(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::PointNm;

    fn fixture_text() -> BoardTextPrimitive {
        BoardTextPrimitive {
            object_id: "board_text:gain".to_string(),
            object_kind: "board_text".to_string(),
            text_uuid: "text-gain".to_string(),
            text: "GAIN STAGE A1".to_string(),
            layer_id: "F.SILKS".to_string(),
            position: PointNm {
                x: 1_000_000,
                y: 2_000_000,
            },
            rotation_degrees: -90,
            height_nm: 1_250_000,
            stroke_width_nm: 150_000,
            render_intent: "annotation".to_string(),
            family: "technical_sans".to_string(),
            style: "regular".to_string(),
            style_class: None,
            h_align: "center".to_string(),
            v_align: "center".to_string(),
            mirrored: false,
            keep_upright: true,
            line_spacing_ratio_ppm: 1_250_000,
            bold: false,
            italic: false,
        }
    }

    #[test]
    fn builds_board_text_edit_terminal_commands() {
        let text = fixture_text();

        assert_eq!(
            board_text_edit_terminal_command(&text, BoardTextEditTerminalField::Content),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --value 'GAIN STAGE A1'"
        );
        assert_eq!(
            board_text_edit_terminal_command(&text, BoardTextEditTerminalField::Height),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --height-nm 1250000"
        );
        assert_eq!(
            board_text_edit_terminal_command(&text, BoardTextEditTerminalField::Rotation),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --rotation-deg 270"
        );
        assert_eq!(
            board_text_edit_terminal_command(&text, BoardTextEditTerminalField::LineSpacing),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --line-spacing-ratio-ppm 1250000"
        );
        assert_eq!(
            board_text_edit_terminal_command(&text, BoardTextEditTerminalField::RenderIntent),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --render-intent 'annotation'"
        );
        assert_eq!(
            board_text_edit_terminal_command(&text, BoardTextEditTerminalField::Family),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --family 'technical_sans'"
        );
        assert_eq!(
            board_text_edit_terminal_command(&text, BoardTextEditTerminalField::Alignment),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --h-align 'center' --v-align 'center'"
        );
    }

    #[test]
    fn shell_quotes_board_text_values() {
        let mut text = fixture_text();
        text.text_uuid = "text-'quote".to_string();
        text.text = "Gain's stage".to_string();

        assert_eq!(
            board_text_edit_terminal_command(&text, BoardTextEditTerminalField::Content),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-'\\''quote' --value 'Gain'\\''s stage'"
        );
    }

    #[test]
    fn builds_board_text_quick_edit_terminal_commands() {
        let mut text = fixture_text();
        text.family = "inter".to_string();

        assert_eq!(
            board_text_quick_edit_terminal_command(
                &text,
                BoardTextQuickEditTerminalAction::ToggleBoolean(BoardTextBooleanField::Mirrored),
            ),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --mirrored true"
        );
        assert_eq!(
            board_text_quick_edit_terminal_command(
                &text,
                BoardTextQuickEditTerminalAction::ToggleBoolean(BoardTextBooleanField::KeepUpright),
            ),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --keep-upright false"
        );
        assert_eq!(
            board_text_quick_edit_terminal_command(
                &text,
                BoardTextQuickEditTerminalAction::ToggleBoolean(BoardTextBooleanField::Bold),
            ),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --bold true"
        );
        assert_eq!(
            board_text_quick_edit_terminal_command(
                &text,
                BoardTextQuickEditTerminalAction::CycleField(BoardTextCycleField::RenderIntent),
            ),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --render-intent 'branding'"
        );
        assert_eq!(
            board_text_quick_edit_terminal_command(
                &text,
                BoardTextQuickEditTerminalAction::CycleField(BoardTextCycleField::Family),
            ),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --family 'inter_display'"
        );
        assert_eq!(
            board_text_quick_edit_terminal_command(
                &text,
                BoardTextQuickEditTerminalAction::CycleAlignment(
                    BoardTextAlignmentField::Horizontal
                ),
            ),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --h-align 'right'"
        );
        assert_eq!(
            board_text_quick_edit_terminal_command(
                &text,
                BoardTextQuickEditTerminalAction::CycleAlignment(BoardTextAlignmentField::Vertical),
            ),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --v-align 'bottom'"
        );
        assert_eq!(
            board_text_quick_edit_terminal_command(
                &text,
                BoardTextQuickEditTerminalAction::StepLineSpacing(
                    BoardTextLineSpacingStep::Increase
                ),
            ),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --line-spacing-ratio-ppm 1350000"
        );
        assert_eq!(
            board_text_quick_edit_terminal_command(
                &text,
                BoardTextQuickEditTerminalAction::StepHeight(BoardTextHeightStep::Increase),
            ),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --height-nm 1375000 --stroke-width-nm 165000"
        );
        assert_eq!(
            board_text_quick_edit_terminal_command(
                &text,
                BoardTextQuickEditTerminalAction::StepRotation(BoardTextRotationStep::Clockwise90),
            ),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --rotation-deg 0"
        );
    }

    #[test]
    fn quick_edit_height_clamps_and_keeps_stroke_positive() {
        let mut text = fixture_text();
        text.height_nm = 1;
        text.stroke_width_nm = 0;

        assert_eq!(
            board_text_quick_edit_terminal_command(
                &text,
                BoardTextQuickEditTerminalAction::StepHeight(BoardTextHeightStep::Decrease),
            ),
            "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text 'text-gain' --height-nm 50000 --stroke-width-nm 50000"
        );
    }
}
