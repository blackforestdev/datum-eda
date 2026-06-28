use datum_gui_protocol::{TerminalLaneState, TerminalStyledLine};

pub(super) fn apply_escape_intermediate(byte: u8, marker: &[u8], state: &mut TerminalLaneState) {
    if marker != b"#" || byte != b'8' {
        return;
    }
    let row = "E".repeat(state.columns.max(1) as usize);
    state.lines = vec![row.clone(); state.rows.max(1) as usize];
    state.styled_lines = state
        .lines
        .iter()
        .map(|text| TerminalStyledLine {
            text: text.clone(),
            spans: Vec::new(),
        })
        .collect();
}
