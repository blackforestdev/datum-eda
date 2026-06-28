use datum_gui_protocol::TerminalLaneState;

const CELL_WIDTH_PX: u16 = 8;
const CELL_HEIGHT_PX: u16 = 16;

pub(super) fn cursor_position_report(
    cursor_row: usize,
    cursor_col: usize,
    private: bool,
) -> Vec<u8> {
    let row = cursor_row.saturating_add(1);
    let col = cursor_col + 1;
    if private {
        format!("\x1b[?{row};{col}R").into_bytes()
    } else {
        format!("\x1b[{row};{col}R").into_bytes()
    }
}

pub(super) fn window_report_response(param: usize, state: &TerminalLaneState) -> Option<Vec<u8>> {
    match param {
        11 => Some(b"\x1b[1t".to_vec()),
        13 => Some(b"\x1b[3;0;0t".to_vec()),
        14 => Some(
            format!(
                "\x1b[4;{};{}t",
                state.rows.max(1).saturating_mul(CELL_HEIGHT_PX),
                state.columns.max(1).saturating_mul(CELL_WIDTH_PX)
            )
            .into_bytes(),
        ),
        15 => Some(
            format!(
                "\x1b[5;{};{}t",
                state.rows.max(1).saturating_mul(CELL_HEIGHT_PX),
                state.columns.max(1).saturating_mul(CELL_WIDTH_PX)
            )
            .into_bytes(),
        ),
        16 => Some(format!("\x1b[6;{};{}t", CELL_HEIGHT_PX, CELL_WIDTH_PX).into_bytes()),
        18 => Some(format!("\x1b[8;{};{}t", state.rows.max(1), state.columns.max(1)).into_bytes()),
        19 => Some(format!("\x1b[9;{};{}t", state.rows.max(1), state.columns.max(1)).into_bytes()),
        20 => Some(format!("\x1b]L{}\x1b\\", sanitized_title(state)).into_bytes()),
        21 => Some(format!("\x1b]l{}\x1b\\", sanitized_title(state)).into_bytes()),
        _ => None,
    }
}

fn sanitized_title(state: &TerminalLaneState) -> String {
    state
        .title
        .as_deref()
        .unwrap_or_default()
        .chars()
        .filter(|ch| *ch >= ' ' && *ch != '\u{7f}')
        .collect()
}
