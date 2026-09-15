use crate::{
    Action, AnchorResolution, CellContent, CoreLimitValues, CoreLimits, LimitError, LimitKind,
    ScreenAction, ScreenBuffer, ScreenError, TerminalCore, TerminalSize,
};

#[test]
fn primary_scrollback_preserves_logical_identity_across_reflow() {
    let mut core = core(4, 2, 32, 4_096, 4_096);
    print(&mut core, "abcdefghij");
    let history = core.state().history();
    assert_eq!(history.rows().len(), 1);
    assert_eq!(row_text(history.rows().next().unwrap().cells()), "abcd");
    let anchor = history.rows().next().unwrap().logical_start();
    assert_eq!(
        core.state().resolve_logical_point(anchor),
        AnchorResolution::History { row: 0, column: 0 }
    );

    core.resize(TerminalSize::new(2, 2, 0, 0).unwrap()).unwrap();
    assert!(core.state().contains_logical_point(anchor));
    assert!(matches!(
        core.state().resolve_logical_point(anchor),
        AnchorResolution::History { .. }
    ));
    assert_eq!(history_text(&core), "abcdef");
    assert_eq!(screen_text(&core), "ghij");

    core.resize(TerminalSize::new(5, 2, 0, 0).unwrap()).unwrap();
    assert!(core.state().contains_logical_point(anchor));
    assert!(matches!(
        core.state().resolve_logical_point(anchor),
        AnchorResolution::Screen { .. }
    ));
    assert_eq!(history_text(&core), "");
    assert_eq!(screen_text(&core), "abcdefghij");
}

#[test]
fn history_trims_complete_oldest_logical_lines_by_owner_limits() {
    let mut core = core(4, 2, 1, 4_096, 4_096);
    hard_line(&mut core, "one");
    hard_line(&mut core, "two");
    let trimmed_anchor = core
        .state()
        .history()
        .rows()
        .next()
        .unwrap()
        .logical_start();
    hard_line(&mut core, "three");
    print(&mut core, "four");

    let history = core.state().history();
    assert_eq!(history.rows().len(), 1);
    assert_eq!(row_text(history.rows().next().unwrap().cells()), "thre");
    assert_eq!(history.trimmed_rows(), 2);
    assert_eq!(
        core.state().resolve_logical_point(trimmed_anchor),
        AnchorResolution::Trimmed
    );
}

#[test]
fn history_byte_limit_evicts_whole_logical_lines_without_partial_rows() {
    let mut core = core(4, 2, 32, 3, 4_096);
    hard_line(&mut core, "one");
    hard_line(&mut core, "two");
    hard_line(&mut core, "tri");

    let history = core.state().history();
    assert_eq!(history.payload_bytes(), 3);
    assert_eq!(history.rows().len(), 1);
    assert_eq!(row_text(history.rows().next().unwrap().cells()), "two");
    assert_eq!(history.trimmed_rows(), 1);
}

#[test]
fn alternate_screen_never_contributes_to_primary_history() {
    let mut core = core(4, 2, 32, 4_096, 4_096);
    hard_line(&mut core, "one");
    hard_line(&mut core, "two");
    let before = core.state().history();

    core.reduce(ScreenAction::SwitchBuffer {
        buffer: ScreenBuffer::Alternate,
        clear: true,
        home: true,
    })
    .unwrap();
    print(&mut core, "abcdefghijklmnop");
    core.resize(TerminalSize::new(3, 3, 0, 0).unwrap()).unwrap();

    assert!(!history_text(&core).contains("abcd"));
    core.reduce(ScreenAction::SwitchBuffer {
        buffer: ScreenBuffer::Primary,
        clear: false,
        home: false,
    })
    .unwrap();
    assert_eq!(
        format!("{}{}", history_text(&core), screen_text(&core)),
        "onetwo"
    );
    assert_eq!(before.payload_bytes(), 3);
}

#[test]
fn resize_reflow_keeps_wide_clusters_atomic_and_cursor_logical() {
    let mut core = core(6, 2, 32, 4_096, 4_096);
    print(&mut core, "A界B👩‍💻C");
    let cursor_anchor = core
        .state()
        .logical_point_at(
            core.state().cursor().position.row.get(),
            core.state().cursor().position.column.get(),
        )
        .unwrap();

    core.resize(TerminalSize::new(3, 3, 0, 0).unwrap()).unwrap();
    assert!(core.state().contains_logical_point(cursor_anchor));
    core.snapshot().unwrap();
    for row in core.snapshot().unwrap().rows() {
        for (column, cell) in row.cells().iter().enumerate() {
            if matches!(cell.content, CellContent::Continuation { .. }) {
                assert!(column > 0);
            }
        }
    }
}

#[test]
fn resize_preserves_cursor_anchor_across_trailing_blank_cells() {
    let mut core = core(6, 2, 32, 4_096, 4_096);
    print(&mut core, "a");
    core.reduce(ScreenAction::SetCursor { row: 0, column: 3 })
        .unwrap();
    let anchor = core.state().logical_point_at(0, 3).unwrap();

    core.resize(TerminalSize::new(2, 3, 0, 0).unwrap()).unwrap();

    assert_eq!(
        core.state().resolve_logical_point(anchor),
        AnchorResolution::Screen { row: 1, column: 1 }
    );
    assert_eq!(core.state().cursor().position.row.get(), 1);
    assert_eq!(core.state().cursor().position.column.get(), 1);
}

#[test]
fn repeated_resize_reflow_is_deterministic_and_byte_preserving() {
    let mut core = core(5, 3, 64, 8_192, 8_192);
    let text = "abc界def👩‍💻ghiΩ";
    print(&mut core, text);

    for (columns, rows) in [(3, 4), (8, 2), (2, 6), (5, 3), (7, 2)] {
        core.resize(TerminalSize::new(columns, rows, 0, 0).unwrap())
            .unwrap();
        assert_eq!(
            format!("{}{}", history_text(&core), screen_text(&core)),
            text
        );
        core.snapshot().unwrap();
    }
}

#[test]
fn reset_invalidates_prior_history_anchors_without_identity_reuse() {
    let mut core = core(4, 2, 32, 4_096, 4_096);
    print(&mut core, "abcdefghij");
    let anchor = core
        .state()
        .history()
        .rows()
        .next()
        .unwrap()
        .logical_start();

    core.reduce(ScreenAction::Reset).unwrap();

    assert_eq!(
        core.state().resolve_logical_point(anchor),
        AnchorResolution::Trimmed
    );
    assert_eq!(core.state().history().rows().len(), 0);
}

#[test]
fn logical_anchor_resolution_remains_stable_while_output_arrives() {
    let mut core = core(4, 2, 32, 4_096, 4_096);
    hard_line(&mut core, "one");
    hard_line(&mut core, "two");
    let anchor = core
        .state()
        .history()
        .rows()
        .next()
        .unwrap()
        .logical_start();

    for line in ["tri", "for", "fiv", "six"] {
        hard_line(&mut core, line);
        assert!(matches!(
            core.state().resolve_logical_point(anchor),
            AnchorResolution::History { .. }
        ));
    }
}

#[test]
fn reflow_work_exhaustion_rejects_resize_without_mutation() {
    let mut core = core(4, 2, 32, 4_096, 1);
    print(&mut core, "abcdef");
    let before = core.snapshot().unwrap();

    assert_eq!(
        core.resize(TerminalSize::new(3, 2, 0, 0).unwrap()),
        Err(ScreenError::Limit(LimitError::Exceeded {
            kind: LimitKind::ReflowWork,
            requested: 8,
            maximum: 1,
        }))
    );
    assert_eq!(core.snapshot().unwrap(), before);
}

fn core(
    columns: u16,
    rows: u16,
    history_lines: usize,
    history_bytes: usize,
    reflow_work: usize,
) -> TerminalCore {
    let value = 4_096;
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
        clipboard_bytes: value,
        hyperlink_bytes: value,
        input_bytes: value,
        keyboard_stack: value,
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
        reflow_work,
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

fn hard_line(core: &mut TerminalCore, text: &str) {
    print(core, text);
    core.apply(Action::Execute(crate::ControlCode::new(b'\r').unwrap()))
        .unwrap();
    core.apply(Action::Execute(crate::ControlCode::new(b'\n').unwrap()))
        .unwrap();
}

fn row_text(cells: &[crate::Cell]) -> String {
    cells
        .iter()
        .filter_map(|cell| match &cell.content {
            CellContent::Cluster(cluster) => Some(cluster.text()),
            _ => None,
        })
        .collect()
}

fn history_text(core: &TerminalCore) -> String {
    core.state()
        .history()
        .rows()
        .map(|row| row_text(row.cells()))
        .collect()
}

fn screen_text(core: &TerminalCore) -> String {
    core.snapshot()
        .unwrap()
        .rows()
        .map(|row| row_text(row.cells()))
        .collect()
}
