use super::*;

fn raw_limits(value: usize) -> CoreLimitValues {
    CoreLimitValues {
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
        input_bytes: value,
        keyboard_stack: value,
        notification_bytes: value,
        reply_bytes: value,
        pending_events: value,
        pending_damage: value,
        history_lines: value,
        history_bytes: value,
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
    }
}

fn core(columns: u16, rows: u16) -> TerminalCore {
    TerminalCore::new(
        CoreLimits::try_from(raw_limits(4_096)).unwrap(),
        TerminalSize::new(columns, rows, 0, 0).unwrap(),
    )
    .unwrap()
}

fn cluster(text: &str, width: CellWidth) -> Cluster {
    Cluster::new(text, width, ClusterBytesLimit::new(64).unwrap()).unwrap()
}

fn print(core: &mut TerminalCore, text: &str) {
    for character in text.chars() {
        core.reduce(ScreenAction::Print(cluster(
            &character.to_string(),
            CellWidth::One,
        )))
        .unwrap();
    }
}

fn row_text(core: &TerminalCore, row: usize) -> String {
    core.snapshot()
        .unwrap()
        .rows()
        .nth(row)
        .unwrap()
        .cells()
        .iter()
        .map(|cell| match &cell.content {
            CellContent::Cluster(cluster) => cluster.text().to_owned(),
            CellContent::Empty | CellContent::Continuation { .. } => " ".to_owned(),
        })
        .collect()
}

#[test]
fn both_screen_buffers_are_admitted_as_one_checked_resource() {
    let size = TerminalSize::new(4, 2, 0, 0).unwrap();
    let mut raw = raw_limits(64);
    raw.screen_cells = 15;
    assert!(matches!(
        TerminalCore::new(CoreLimits::try_from(raw).unwrap(), size),
        Err(ScreenError::Limit(LimitError::Exceeded {
            kind: LimitKind::ScreenCells,
            requested: 16,
            maximum: 15,
        }))
    ));
}

#[test]
fn delayed_wrap_scroll_and_hard_soft_line_identity_are_deterministic() {
    let mut core = core(4, 2);
    print(&mut core, "abcd");
    assert!(core.state().cursor().pending_wrap);
    assert_eq!(core.state().cursor().position.column.get(), 3);

    print(&mut core, "e");
    assert_eq!(row_text(&core, 0), "abcd");
    assert_eq!(row_text(&core, 1), "e   ");
    assert!(
        core.snapshot()
            .unwrap()
            .rows()
            .next()
            .unwrap()
            .soft_wrapped()
    );

    core.reduce(ScreenAction::CarriageReturn).unwrap();
    core.reduce(ScreenAction::LineFeed).unwrap();
    print(&mut core, "z");
    assert_eq!(row_text(&core, 0), "e   ");
    assert_eq!(row_text(&core, 1), "z   ");
}

#[test]
fn primary_and_alternate_buffers_are_isolated_and_reset_is_total() {
    let mut core = core(5, 2);
    print(&mut core, "main");
    core.reduce(ScreenAction::SwitchBuffer {
        buffer: ScreenBuffer::Alternate,
        clear: true,
        home: true,
    })
    .unwrap();
    print(&mut core, "alt");
    assert_eq!(row_text(&core, 0), "alt  ");

    core.reduce(ScreenAction::SwitchBuffer {
        buffer: ScreenBuffer::Primary,
        clear: false,
        home: true,
    })
    .unwrap();
    assert_eq!(row_text(&core, 0), "main ");

    core.reduce(ScreenAction::Reset).unwrap();
    assert_eq!(row_text(&core, 0), "     ");
    assert_eq!(core.state().active_buffer(), ScreenBuffer::Primary);
    assert!(core.state().modes().auto_wrap);
}

#[test]
fn wide_clusters_remain_atomic_across_overwrite_insert_delete_and_erase() {
    let mut core = core(6, 2);
    core.reduce(ScreenAction::Print(cluster("界", CellWidth::Two)))
        .unwrap();
    core.snapshot().unwrap();
    core.reduce(ScreenAction::SetCursor { row: 0, column: 1 })
        .unwrap();
    print(&mut core, "x");
    assert!(matches!(
        core.state().cell(0, 0).unwrap().content,
        CellContent::Empty
    ));
    assert!(matches!(
        core.state().cell(0, 1).unwrap().content,
        CellContent::Cluster(_)
    ));

    core.reduce(ScreenAction::SetCursor { row: 0, column: 2 })
        .unwrap();
    core.reduce(ScreenAction::Print(cluster("語", CellWidth::Two)))
        .unwrap();
    core.reduce(ScreenAction::SetCursor { row: 0, column: 3 })
        .unwrap();
    core.reduce(ScreenAction::InsertCells(1)).unwrap();
    core.snapshot().unwrap();
    core.reduce(ScreenAction::DeleteCells(2)).unwrap();
    core.snapshot().unwrap();
    core.reduce(ScreenAction::EraseLine {
        mode: EraseLine::All,
        selective: false,
    })
    .unwrap();
    core.snapshot().unwrap();
}

#[test]
fn selective_erase_preserves_protected_clusters_without_orphans() {
    let mut core = core(5, 1);
    core.reduce(ScreenAction::SetProtection(true)).unwrap();
    core.reduce(ScreenAction::Print(cluster("界", CellWidth::Two)))
        .unwrap();
    core.reduce(ScreenAction::SetProtection(false)).unwrap();
    print(&mut core, "x");
    core.reduce(ScreenAction::EraseDisplay {
        mode: EraseDisplay::All,
        selective: true,
    })
    .unwrap();
    assert!(matches!(
        core.state().cell(0, 0).unwrap().content,
        CellContent::Cluster(_)
    ));
    assert!(matches!(
        core.state().cell(0, 1).unwrap().content,
        CellContent::Continuation { .. }
    ));
    assert!(matches!(
        core.state().cell(0, 2).unwrap().content,
        CellContent::Empty
    ));
    core.snapshot().unwrap();
}

#[test]
fn margins_confine_line_insertion_deletion_and_scrolling() {
    let mut core = core(5, 4);
    for (row, text) in ["AAAA", "BBBB", "CCCC", "DDDD"].into_iter().enumerate() {
        core.reduce(ScreenAction::SetCursor {
            row: u16::try_from(row).unwrap(),
            column: 0,
        })
        .unwrap();
        print(&mut core, text);
    }
    let size = core.size();
    core.reduce(ScreenAction::SetMargins(
        Margins::new(1, 2, 1, 3, size).unwrap(),
    ))
    .unwrap();
    core.reduce(ScreenAction::ScrollUp(1)).unwrap();
    assert_eq!(row_text(&core, 0), "AAAA ");
    assert_eq!(row_text(&core, 1), "BCCC ");
    assert_eq!(row_text(&core, 2), "C    ");
    assert_eq!(row_text(&core, 3), "DDDD ");
}

#[test]
fn cell_edits_outside_horizontal_margins_use_the_full_screen_without_underflow() {
    let mut core = core(8, 3);
    let size = core.size();
    core.reduce(ScreenAction::SetMargins(
        Margins::new(1, 2, 2, 5, size).unwrap(),
    ))
    .unwrap();
    core.reduce(ScreenAction::SetCursor { row: 0, column: 7 })
        .unwrap();
    core.reduce(ScreenAction::InsertCells(3)).unwrap();
    core.reduce(ScreenAction::DeleteCells(3)).unwrap();
    core.reduce(ScreenAction::EraseCells(3)).unwrap();
    core.reduce(ScreenAction::Print(cluster("x", CellWidth::One)))
        .unwrap();
    core.snapshot().unwrap();
    assert!(matches!(
        core.state().cell(0, 7).unwrap().content,
        CellContent::Cluster(_)
    ));
}

#[test]
fn save_restore_modes_style_cursor_and_damage_are_closed() {
    let mut core = core(4, 2);
    core.reduce(ScreenAction::SetCursor { row: 1, column: 2 })
        .unwrap();
    core.reduce(ScreenAction::SetMode {
        mode: FoundationMode::Insert,
        enabled: true,
    })
    .unwrap();
    let mut style = CellStyle::default();
    style.attributes.set(CellAttribute::Bold, true);
    core.reduce(ScreenAction::SetStyle(style)).unwrap();
    core.reduce(ScreenAction::SaveCursor).unwrap();
    core.reduce(ScreenAction::SetCursor { row: 0, column: 0 })
        .unwrap();
    core.reduce(ScreenAction::SetMode {
        mode: FoundationMode::Insert,
        enabled: false,
    })
    .unwrap();
    let reduction = core.reduce(ScreenAction::RestoreCursor).unwrap();
    assert_eq!(
        core.state().cursor().position,
        CellPoint::new(1, 2, core.size()).unwrap()
    );
    assert!(core.state().modes().insert);
    assert!(
        core.state()
            .style()
            .attributes
            .contains(CellAttribute::Bold)
    );
    assert_eq!(
        reduction.damage().iter().collect::<Vec<_>>(),
        vec![Damage::Cursor]
    );
}

#[test]
fn seeded_edit_sequences_never_create_orphan_continuations() {
    let mut core = core(12, 6);
    let mut state = 0x5eed_d7c0_0000_0009_u64;
    for _ in 0..2_000 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let action = match state % 10 {
            0 => ScreenAction::Print(cluster("界", CellWidth::Two)),
            1 => ScreenAction::Print(cluster("x", CellWidth::One)),
            2 => ScreenAction::InsertCells(1 + (state % 4) as u16),
            3 => ScreenAction::DeleteCells(1 + (state % 4) as u16),
            4 => ScreenAction::EraseCells(1 + (state % 4) as u16),
            5 => ScreenAction::LineFeed,
            6 => ScreenAction::ReverseIndex,
            7 => ScreenAction::MoveCursor {
                rows: ((state >> 8) as i8) as i32,
                columns: ((state >> 16) as i8) as i32,
            },
            8 => ScreenAction::InsertLines(1 + (state % 3) as u16),
            _ => ScreenAction::DeleteLines(1 + (state % 3) as u16),
        };
        core.reduce(action).unwrap();
        core.snapshot().unwrap();
    }
}
