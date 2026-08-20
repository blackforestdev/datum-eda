use crate::{
    Action, ControlCode, CoreLimitValues, CoreLimits, LimitError, LimitKind, LogicalPoint,
    ScreenAction, ScreenBuffer, Selection, SelectionError, SelectionScope, SelectionState,
    TerminalCore, TerminalSize,
};

#[test]
fn grapheme_and_word_selection_are_direction_independent() {
    let mut core = core(24, 2, 128, 16_384, 16_384);
    print(&mut core, "one two_2 👩‍💻!");
    let one = point(&core, 0, 0);
    let emoji = point(&core, 0, 10);
    core.set_selection(Selection::new(emoji, one, SelectionScope::Grapheme))
        .unwrap();
    assert_eq!(core.copy_selection().unwrap().as_str(), "one two_2 👩‍💻");

    let middle = point(&core, 0, 6);
    core.set_selection(Selection::new(middle, middle, SelectionScope::Word))
        .unwrap();
    assert_eq!(core.copy_selection().unwrap().as_str(), "two_2");
}

#[test]
fn soft_wrap_omits_newline_while_hard_line_includes_it() {
    let mut core = core(4, 3, 128, 16_384, 16_384);
    print(&mut core, "abcdef");
    select_cells(&mut core, (0, 0), (1, 1), SelectionScope::Grapheme);
    assert_eq!(core.copy_selection().unwrap().as_str(), "abcdef");

    hard_line(&mut core);
    print(&mut core, "xy");
    select_cells(&mut core, (0, 0), (2, 1), SelectionScope::Grapheme);
    assert_eq!(core.copy_selection().unwrap().as_str(), "abcdef\nxy");
}

#[test]
fn wrapped_logical_block_and_all_scopes_have_distinct_copy_rules() {
    let mut core = core(4, 4, 128, 16_384, 16_384);
    print(&mut core, "abcdef");
    hard_line(&mut core);
    print(&mut core, "xy z");

    select_cells(&mut core, (0, 1), (1, 0), SelectionScope::WrappedLine);
    assert_eq!(core.copy_selection().unwrap().as_str(), "abcd\nef");
    select_cells(&mut core, (0, 1), (1, 0), SelectionScope::LogicalLine);
    assert_eq!(core.copy_selection().unwrap().as_str(), "abcdef");
    select_cells(&mut core, (0, 1), (2, 2), SelectionScope::Block);
    assert_eq!(core.copy_selection().unwrap().as_str(), "bc\nf \ny ");
    select_cells(&mut core, (0, 0), (0, 0), SelectionScope::All);
    assert_eq!(core.copy_selection().unwrap().as_str(), "abcdef\nxy z");
}

#[test]
fn explicit_spaces_survive_while_unselected_padding_does_not() {
    let mut core = core(8, 2, 128, 16_384, 16_384);
    print(&mut core, "a  ");
    select_cells(&mut core, (0, 0), (0, 2), SelectionScope::Grapheme);
    assert_eq!(core.copy_selection().unwrap().as_str(), "a  ");
    select_cells(&mut core, (0, 0), (0, 0), SelectionScope::All);
    assert_eq!(core.copy_selection().unwrap().as_str(), "a  ");
}

#[test]
fn logical_endpoints_survive_reflow_then_report_history_trim() {
    let mut core = core(4, 2, 1, 16_384, 16_384);
    print(&mut core, "abcdefgh");
    let anchor = point(&core, 0, 0);
    let focus = point(&core, 1, 3);
    core.set_selection(Selection::new(anchor, focus, SelectionScope::Grapheme))
        .unwrap();
    core.resize(TerminalSize::new(2, 4, 0, 0).unwrap()).unwrap();
    assert_eq!(
        core.selection_state().unwrap(),
        Some(SelectionState::Active)
    );
    assert_eq!(core.copy_selection().unwrap().as_str(), "abcdefgh");

    for text in ["one", "two", "three"] {
        hard_line(&mut core);
        print(&mut core, text);
    }
    assert_eq!(
        core.selection_state().unwrap(),
        Some(SelectionState::Trimmed)
    );
    assert_eq!(core.copy_selection(), Err(SelectionError::Trimmed));
}

#[test]
fn alternate_buffer_isolated_and_buffer_switch_clears_selection() {
    let mut core = core(10, 2, 128, 16_384, 16_384);
    print(&mut core, "primary");
    select_cells(&mut core, (0, 0), (0, 6), SelectionScope::Grapheme);
    core.reduce(ScreenAction::SwitchBuffer {
        buffer: ScreenBuffer::Alternate,
        clear: true,
        home: true,
    })
    .unwrap();
    assert_eq!(core.selection(), None);
    print(&mut core, "alternate");
    select_cells(&mut core, (0, 0), (0, 8), SelectionScope::Grapheme);
    assert_eq!(core.copy_selection().unwrap().as_str(), "alternate");
    core.reduce(ScreenAction::SwitchBuffer {
        buffer: ScreenBuffer::Primary,
        clear: false,
        home: true,
    })
    .unwrap();
    assert_eq!(core.selection(), None);
}

#[test]
fn clipboard_limit_rejects_atomically_without_truncating() {
    let mut core = core(8, 2, 128, 16_384, 3);
    print(&mut core, "four");
    select_cells(&mut core, (0, 0), (0, 3), SelectionScope::Grapheme);
    assert_eq!(
        core.copy_selection(),
        Err(SelectionError::Limit(LimitError::Exceeded {
            kind: LimitKind::ClipboardBytes,
            requested: 4,
            maximum: 3,
        }))
    );
    assert_eq!(
        core.selection_state().unwrap(),
        Some(SelectionState::Active)
    );
}

#[test]
fn reset_and_explicit_clear_remove_selection() {
    let mut core = core(8, 2, 128, 16_384, 16_384);
    print(&mut core, "text");
    select_cells(&mut core, (0, 0), (0, 3), SelectionScope::Grapheme);
    core.clear_selection();
    assert_eq!(core.copy_selection(), Err(SelectionError::Missing));
    select_cells(&mut core, (0, 0), (0, 3), SelectionScope::Grapheme);
    core.reduce(ScreenAction::Reset).unwrap();
    assert_eq!(core.selection(), None);
}

fn core(
    columns: u16,
    rows: u16,
    history_lines: usize,
    history_bytes: usize,
    clipboard_bytes: usize,
) -> TerminalCore {
    let value = 65_536;
    let limits = CoreLimits::try_from(CoreLimitValues {
        parameter_count: value,
        parameter_digits: value,
        parameter_value: value,
        subparameter_count: value,
        intermediate_bytes: value,
        control_string_bytes: value,
        cluster_bytes: value,
        title_bytes: value,
        working_directory_bytes: value,
        clipboard_bytes,
        notification_bytes: value,
        reply_bytes: value,
        pending_events: value,
        pending_damage: value,
        history_lines,
        history_bytes,
        graphic_objects: value,
        graphic_pixels: value,
        graphic_decoded_bytes: value,
        graphic_frames: value,
        compression_ratio: value,
        parser_work: value,
        search_work: value,
        reflow_work: value,
        screen_cells: value,
        snapshot_cells: value,
    })
    .unwrap();
    TerminalCore::new(limits, TerminalSize::new(columns, rows, 0, 0).unwrap()).unwrap()
}

fn print(core: &mut TerminalCore, text: &str) {
    for character in text.chars() {
        core.apply(Action::Print(character)).unwrap();
    }
}

fn hard_line(core: &mut TerminalCore) {
    core.apply(Action::Execute(ControlCode::new(b'\r').unwrap()))
        .unwrap();
    core.apply(Action::Execute(ControlCode::new(b'\n').unwrap()))
        .unwrap();
}

fn point(core: &TerminalCore, row: u16, column: u16) -> LogicalPoint {
    core.state().logical_point_at(row, column).unwrap()
}

fn select_cells(
    core: &mut TerminalCore,
    anchor: (u16, u16),
    focus: (u16, u16),
    scope: SelectionScope,
) {
    let anchor = point(core, anchor.0, anchor.1);
    let focus = point(core, focus.0, focus.1);
    core.set_selection(Selection::new(anchor, focus, scope))
        .unwrap();
}
