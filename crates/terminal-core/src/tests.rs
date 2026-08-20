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
        hyperlink_bytes: value,
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

fn limits() -> CoreLimits {
    CoreLimits::try_from(raw_limits(64)).expect("positive fixture limits are valid")
}

#[test]
fn all_resource_families_reject_zero_and_expose_no_default_policy() {
    macro_rules! assert_zero {
        ($type:ty, $kind:ident) => {
            assert_eq!(
                <$type>::new(0),
                Err(LimitError::Zero {
                    kind: LimitKind::$kind
                })
            );
        };
    }

    assert_zero!(ParameterCountLimit, ParameterCount);
    assert_zero!(ParameterDigitsLimit, ParameterDigits);
    assert_zero!(ParameterValueLimit, ParameterValue);
    assert_zero!(SubparameterCountLimit, SubparameterCount);
    assert_zero!(IntermediateBytesLimit, IntermediateBytes);
    assert_zero!(ControlStringBytesLimit, ControlStringBytes);
    assert_zero!(ClusterBytesLimit, ClusterBytes);
    assert_zero!(TitleBytesLimit, TitleBytes);
    assert_zero!(WorkingDirectoryBytesLimit, WorkingDirectoryBytes);
    assert_zero!(ClipboardBytesLimit, ClipboardBytes);
    assert_zero!(HyperlinkBytesLimit, HyperlinkBytes);
    assert_zero!(InputBytesLimit, InputBytes);
    assert_zero!(KeyboardStackLimit, KeyboardStack);
    assert_zero!(NotificationBytesLimit, NotificationBytes);
    assert_zero!(ReplyBytesLimit, ReplyBytes);
    assert_zero!(PendingEventsLimit, PendingEvents);
    assert_zero!(PendingDamageLimit, PendingDamage);
    assert_zero!(HistoryLinesLimit, HistoryLines);
    assert_zero!(HistoryBytesLimit, HistoryBytes);
    assert_zero!(GraphicObjectsLimit, GraphicObjects);
    assert_zero!(GraphicPixelsLimit, GraphicPixels);
    assert_zero!(GraphicDecodedBytesLimit, GraphicDecodedBytes);
    assert_zero!(GraphicFramesLimit, GraphicFrames);
    assert_zero!(CompressionRatioLimit, CompressionRatio);
    assert_zero!(ParserWorkLimit, ParserWork);
    assert_zero!(SearchWorkLimit, SearchWork);
    assert_zero!(ReflowWorkLimit, ReflowWork);
    assert_zero!(ScreenCellsLimit, ScreenCells);
    assert_zero!(SnapshotCellsLimit, SnapshotCells);

    let error = CoreLimits::try_from(raw_limits(0)).expect_err("zero policy is invalid");
    assert_eq!(
        error,
        LimitError::Zero {
            kind: LimitKind::ParameterCount
        }
    );
}

#[test]
fn checked_limits_reject_excess_and_accounting_overflow() {
    let limit = ReplyBytesLimit::new(8).unwrap();
    assert_eq!(limit.check(8), Ok(()));
    assert_eq!(
        limit.check(9),
        Err(LimitError::Exceeded {
            kind: LimitKind::ReplyBytes,
            requested: 9,
            maximum: 8,
        })
    );
    assert_eq!(
        limit.checked_total(usize::MAX, 1),
        Err(LimitError::ArithmeticOverflow {
            kind: LimitKind::ReplyBytes
        })
    );
}

#[test]
fn dimensions_and_logical_coordinates_are_checked() {
    assert_eq!(
        TerminalSize::new(0, 24, 0, 0),
        Err(CoordinateError::ZeroColumns)
    );
    assert_eq!(
        TerminalSize::new(80, 0, 0, 0),
        Err(CoordinateError::ZeroRows)
    );

    let size = TerminalSize::new(80, 24, 800, 480).unwrap();
    assert_eq!(size.cell_count(), Some(1_920));
    assert!(CellPoint::new(23, 79, size).is_ok());
    assert!(matches!(
        CellPoint::new(24, 79, size),
        Err(CoordinateError::RowOutOfBounds { .. })
    ));
    assert_eq!(LogicalLineId::new(42).get(), 42);
    assert_eq!(Percent::new(100).map(Percent::get), Some(100));
    assert_eq!(Percent::new(101), None);
}

#[test]
fn clusters_replies_events_and_damage_are_bounded_at_construction() {
    let limits = limits();
    let cluster = Cluster::new("e\u{301}", CellWidth::One, limits.cluster_bytes).unwrap();
    assert_eq!(cluster.text(), "e\u{301}");
    assert!(matches!(
        Cluster::new(
            "too large",
            CellWidth::One,
            ClusterBytesLimit::new(2).unwrap()
        ),
        Err(ClusterError::TooLarge(_))
    ));

    let reply = TerminalReply::new(
        ReplyKind::CursorPosition,
        b"\x1b[1;1R".to_vec(),
        limits.reply_bytes,
    )
    .unwrap();
    assert_eq!(reply.bytes(), b"\x1b[1;1R");

    let title = TitleText::new("Datum", limits.title_bytes).unwrap();
    assert_eq!(
        CoreEvent::TitleChanged(title.clone()),
        CoreEvent::TitleChanged(title)
    );

    let size = TerminalSize::new(4, 2, 0, 0).unwrap();
    let mut damage = DamageSet::new(PendingDamageLimit::new(1).unwrap());
    damage
        .push(Damage::Cell(CellPoint::new(0, 0, size).unwrap()))
        .unwrap();
    assert!(matches!(
        damage.push(Damage::Cursor),
        Err(LimitError::Exceeded { .. })
    ));
}

#[test]
fn terminal_core_starts_with_closed_renderer_independent_state() {
    let size = TerminalSize::new(120, 40, 1_200, 800).unwrap();
    let core = TerminalCore::new(CoreLimits::try_from(raw_limits(10_000)).unwrap(), size).unwrap();
    assert_eq!(core.size(), size);
    assert_eq!(core.state().active_buffer(), ScreenBuffer::Primary);
    assert_eq!(
        core.state().cursor().position,
        CellPoint::new(0, 0, size).unwrap()
    );
    assert_eq!(core.state().margins(), Margins::full(size));
    assert_eq!(
        core.state().modes(),
        ModeState {
            auto_wrap: true,
            ..ModeState::default()
        }
    );
    assert_eq!(
        core.state()
            .tab_stops()
            .iter()
            .map(Column::get)
            .collect::<Vec<_>>(),
        (8..120).step_by(8).collect::<Vec<_>>()
    );
    assert!(core.state().saved_cursor().is_none());
}

#[test]
fn snapshots_reject_orphan_continuations_and_expose_immutable_rows() {
    let limits = limits();
    let size = TerminalSize::new(2, 1, 0, 0).unwrap();
    let lead = Cluster::new("界", CellWidth::Two, limits.cluster_bytes).unwrap();
    let row = SnapshotRow::new(
        vec![
            Cell {
                content: CellContent::Cluster(lead),
                ..Cell::default()
            },
            Cell {
                content: CellContent::Continuation {
                    lead: Column::new(0, size.columns).unwrap(),
                },
                ..Cell::default()
            },
        ],
        size.columns,
        false,
    )
    .unwrap();
    let snapshot = TerminalSnapshot::new(
        size,
        vec![row],
        CursorState::home(size),
        ModeState::default(),
        ScreenBuffer::Primary,
        limits.snapshot_cells,
    )
    .unwrap();
    assert_eq!(snapshot.rows().next().unwrap().cells().len(), 2);

    let orphan = SnapshotRow::new(
        vec![
            Cell::default(),
            Cell {
                content: CellContent::Continuation {
                    lead: Column::new(0, size.columns).unwrap(),
                },
                ..Cell::default()
            },
        ],
        size.columns,
        false,
    )
    .unwrap();
    assert!(matches!(
        TerminalSnapshot::new(
            size,
            vec![orphan],
            CursorState::home(size),
            ModeState::default(),
            ScreenBuffer::Primary,
            limits.snapshot_cells,
        ),
        Err(SnapshotError::ContinuationWithoutWideLead { .. })
    ));
}
