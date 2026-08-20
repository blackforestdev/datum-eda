use super::*;

fn limits(snapshot_cells: usize, pending_damage: usize) -> CoreLimits {
    CoreLimits::try_from(CoreLimitValues {
        parameter_count: 64,
        parameter_digits: 32,
        parameter_value: usize::MAX,
        subparameter_count: 64,
        intermediate_bytes: 64,
        control_string_bytes: 1 << 20,
        cluster_bytes: 4_096,
        title_bytes: 4_096,
        working_directory_bytes: 4_096,
        clipboard_bytes: 1 << 20,
        hyperlink_bytes: 1 << 20,
        input_bytes: 1 << 20,
        keyboard_stack: 64,
        notification_bytes: 4_096,
        reply_bytes: 4_096,
        pending_events: 1_024,
        pending_damage,
        history_lines: 64,
        history_bytes: 1 << 20,
        graphic_objects: 128,
        graphic_pixels: 1 << 16,
        graphic_decoded_bytes: 1 << 18,
        graphic_frames: 128,
        compression_ratio: 1_024,
        parser_work: 1 << 20,
        search_work: 1 << 20,
        reflow_work: 1 << 20,
        screen_cells: 1 << 20,
        snapshot_cells,
    })
    .unwrap()
}

fn core(snapshot_cells: usize, pending_damage: usize) -> TerminalCore {
    TerminalCore::new(
        limits(snapshot_cells, pending_damage),
        TerminalSize::new(4, 2, 40, 20).unwrap(),
    )
    .unwrap()
}

fn print(terminal: &mut TerminalCore, text: &str) {
    for character in text.chars() {
        terminal.apply(Action::Print(character)).unwrap();
    }
}

fn osc(bytes: &[u8]) -> Action {
    Action::ControlString(ControlString {
        kind: ControlStringKind::Osc,
        bytes: bytes.to_vec(),
        terminator: StringTerminator::StringTerminator,
    })
}

fn apc(command: &str) -> Action {
    Action::ControlString(ControlString {
        kind: ControlStringKind::Apc,
        bytes: command.as_bytes().to_vec(),
        terminator: StringTerminator::StringTerminator,
    })
}

fn row_text(row: &RenderRow) -> String {
    row.cells()
        .iter()
        .filter_map(|cell| match &cell.content {
            CellContent::Cluster(cluster) => Some(cluster.text()),
            CellContent::Empty | CellContent::Continuation { .. } => None,
        })
        .collect()
}

#[test]
fn render_snapshot_is_stable_versioned_and_orders_history_before_screen() {
    let mut terminal = core(64, 16);
    print(&mut terminal, "abcdefghij");
    terminal
        .apply(osc(b"4;3;rgb:12/34/56"))
        .expect("palette update is valid");
    let snapshot = terminal.render_snapshot().unwrap();

    assert_eq!(snapshot.schema_version(), RENDER_SNAPSHOT_SCHEMA_VERSION);
    assert_eq!(snapshot.history_row_count(), 1);
    assert_eq!(snapshot.rows().len(), 3);
    assert_eq!(
        snapshot.rows().map(RenderRow::source).collect::<Vec<_>>(),
        vec![
            RenderRowSource::History { index: 0 },
            RenderRowSource::Screen { row: 0 },
            RenderRowSource::Screen { row: 1 },
        ]
    );
    assert_eq!(
        snapshot.palette().color(3),
        Color::Rgb(Rgb {
            red: 0x12,
            green: 0x34,
            blue: 0x56,
        })
    );
    let before = snapshot.rows().map(row_text).collect::<Vec<_>>();
    print(&mut terminal, "ZZ");
    assert_eq!(snapshot.rows().map(row_text).collect::<Vec<_>>(), before);
}

#[test]
fn alternate_render_snapshot_excludes_primary_history_and_preserves_selection() {
    let mut terminal = core(64, 16);
    print(&mut terminal, "abcdefghij");
    terminal
        .reduce(ScreenAction::SwitchBuffer {
            buffer: ScreenBuffer::Alternate,
            clear: true,
            home: true,
        })
        .unwrap();
    print(&mut terminal, "alt");
    let anchor = terminal.state().logical_point_at(0, 0).unwrap();
    terminal
        .set_selection(Selection::new(anchor, anchor, SelectionScope::Grapheme))
        .unwrap();

    let snapshot = terminal.render_snapshot().unwrap();
    assert_eq!(snapshot.active_buffer(), ScreenBuffer::Alternate);
    assert_eq!(snapshot.history_row_count(), 0);
    assert_eq!(snapshot.rows().len(), 2);
    assert_eq!(snapshot.selection().unwrap().anchor(), anchor);
    assert!(
        snapshot
            .rows()
            .all(|row| matches!(row.source(), RenderRowSource::Screen { .. }))
    );
}

#[test]
fn render_snapshot_accounts_history_cells_before_cloning() {
    let mut terminal = core(8, 16);
    print(&mut terminal, "abcdefghij");
    assert_eq!(terminal.snapshot().unwrap().rows().len(), 2);
    assert!(matches!(
        terminal.render_snapshot(),
        Err(SnapshotError::CellLimit(LimitError::Exceeded {
            kind: LimitKind::SnapshotCells,
            requested: 12,
            maximum: 8,
        }))
    ));
}

#[test]
fn render_graphics_are_resolved_and_sorted_by_z_then_identity() {
    let mut terminal = core(64, 16);
    terminal
        .apply(apc("Ga=T,f=32,s=1,v=1,i=7,p=1,z=8;/wAA/w=="))
        .unwrap();
    terminal
        .apply(apc("Ga=T,f=32,s=1,v=1,i=8,p=1,z=-2;AP8A/w=="))
        .unwrap();
    let snapshot = terminal.render_snapshot().unwrap();
    let graphics = snapshot.graphics().collect::<Vec<_>>();
    assert_eq!(graphics.len(), 2);
    assert_eq!(graphics[0].placement().z_index(), -2);
    assert_eq!(graphics[1].placement().z_index(), 8);
    assert!(
        graphics
            .iter()
            .all(|graphic| matches!(graphic.resolution(), GraphicAnchorResolution::Screen { .. }))
    );
}

#[test]
fn damage_reports_cell_rows_scroll_cursor_history_palette_and_graphics() {
    let mut terminal = core(64, 16);
    let printed = terminal.apply(Action::Print('x')).unwrap();
    assert_eq!(
        printed.damage().iter().collect::<Vec<_>>(),
        vec![
            Damage::Cell(CellPoint::new(0, 0, terminal.size()).unwrap()),
            Damage::Cursor,
        ]
    );
    let erased_row = terminal
        .reduce(ScreenAction::EraseLine {
            mode: EraseLine::All,
            selective: false,
        })
        .unwrap();
    assert_eq!(
        erased_row.damage().iter().collect::<Vec<_>>(),
        vec![Damage::Row(Row::new(0, terminal.size().rows).unwrap())]
    );
    let erased_rows = terminal
        .reduce(ScreenAction::EraseDisplay {
            mode: EraseDisplay::Below,
            selective: false,
        })
        .unwrap();
    assert!(matches!(
        erased_rows.damage().iter().next(),
        Some(Damage::Rows { .. })
    ));

    terminal
        .reduce(ScreenAction::SetCursor { row: 1, column: 0 })
        .unwrap();
    let line_feed = terminal
        .apply(Action::Execute(ControlCode::new(b'\n').unwrap()))
        .unwrap();
    assert!(line_feed.damage().iter().any(|damage| matches!(
        damage,
        Damage::Scroll {
            direction: ScrollDirection::Up,
            ..
        }
    )));
    assert!(
        line_feed
            .damage()
            .iter()
            .any(|damage| damage == Damage::History)
    );
    assert!(
        line_feed
            .damage()
            .iter()
            .any(|damage| damage == Damage::Cursor)
    );

    let palette = terminal.apply(osc(b"4;2;rgb:01/02/03")).unwrap();
    assert_eq!(
        palette.damage().iter().collect::<Vec<_>>(),
        vec![Damage::Palette(PaletteIndex::new(2))]
    );
    let graphic = terminal
        .apply(apc("Ga=T,f=32,s=1,v=1,i=9,p=1;/wAA/w=="))
        .unwrap();
    assert!(
        graphic
            .damage()
            .iter()
            .any(|damage| damage == Damage::Graphics)
    );
}

#[test]
fn wide_cell_prints_dirty_every_cell_they_create_or_clear() {
    let mut terminal = core(64, 16);
    let wide = Cluster::new("界", CellWidth::Two, ClusterBytesLimit::new(16).unwrap()).unwrap();
    let inserted = terminal.reduce(ScreenAction::Print(wide)).unwrap();
    assert_eq!(
        inserted.damage().iter().collect::<Vec<_>>(),
        vec![
            Damage::Row(Row::new(0, terminal.size().rows).unwrap()),
            Damage::Cursor
        ]
    );

    terminal
        .reduce(ScreenAction::SetCursor { row: 0, column: 0 })
        .unwrap();
    let replacement = terminal.apply(Action::Print('x')).unwrap();
    assert_eq!(
        replacement.damage().iter().collect::<Vec<_>>(),
        vec![
            Damage::Row(Row::new(0, terminal.size().rows).unwrap()),
            Damage::Cursor
        ]
    );
}

#[test]
fn damage_overflow_degrades_deterministically_to_full() {
    let mut terminal = core(64, 1);
    let update = terminal.apply(Action::Print('x')).unwrap();
    assert_eq!(
        update.damage().iter().collect::<Vec<_>>(),
        vec![Damage::Full]
    );
}

#[test]
fn full_visible_damage_retains_independent_history_invalidation() {
    let mut terminal = core(64, 16);
    terminal
        .reduce(ScreenAction::SetCursor { row: 1, column: 3 })
        .unwrap();
    terminal.apply(Action::Print('x')).unwrap();
    let update = terminal.apply(Action::Print('y')).unwrap();
    assert_eq!(
        update.damage().iter().collect::<Vec<_>>(),
        vec![Damage::Full, Damage::History]
    );
}
