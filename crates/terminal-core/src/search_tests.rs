use crate::{
    Action, ControlCode, CoreLimitValues, CoreLimits, LimitError, LimitKind, ScreenAction,
    ScreenBuffer, SearchCase, SearchCursor, SearchError, SearchMatchState, SearchQuery,
    TerminalCore, TerminalSize,
};

#[test]
fn literal_search_is_incremental_case_aware_and_bidirectional() {
    let mut core = core(24, 3, 128, 65_536);
    print(&mut core, "Alpha beta ALPHA beta");
    let insensitive = SearchQuery::literal("alpha", SearchCase::Insensitive);
    let first = core
        .search(&insensitive, SearchCursor::forward(None))
        .unwrap()
        .matched()
        .unwrap();
    assert_eq!(first.start(), point(&core, 0, 0));
    let second = core
        .search(&insensitive, SearchCursor::forward(Some(first.end())))
        .unwrap()
        .matched()
        .unwrap();
    assert_eq!(second.start(), point(&core, 0, 11));
    let prior = core
        .search(&insensitive, SearchCursor::backward(Some(second.start())))
        .unwrap()
        .matched()
        .unwrap();
    assert_eq!(prior, first);
    assert!(
        core.search(
            &SearchQuery::literal("alpha", SearchCase::Sensitive),
            SearchCursor::forward(None)
        )
        .unwrap()
        .matched()
        .is_none()
    );
}

#[test]
fn thompson_regex_supports_groups_alternation_classes_quantifiers_and_anchors() {
    let mut core = core(32, 3, 128, 65_536);
    print(&mut core, "cat color colouur dog42 ] -");
    for pattern in [
        "c(at|olou?r)",
        "colou+r",
        "dog[0-9]+",
        "^cat",
        "[\\]]",
        "[\\-]$",
    ] {
        assert!(
            core.search(
                &SearchQuery::regex(pattern, SearchCase::Sensitive),
                SearchCursor::forward(None)
            )
            .unwrap()
            .matched()
            .is_some(),
            "pattern {pattern}"
        );
    }
    assert!(
        core.search(
            &SearchQuery::regex("^dog", SearchCase::Sensitive),
            SearchCursor::forward(None)
        )
        .unwrap()
        .matched()
        .is_none()
    );
}

#[test]
fn regex_literals_follow_unicode_grapheme_boundaries_and_quantifiers_are_greedy() {
    let mut core = core(16, 2, 128, 65_536);
    print(&mut core, "e\u{301}👩‍💻 aaa");
    for pattern in ["e\u{301}", "👩‍💻", "a+"] {
        let found = core
            .search(
                &SearchQuery::regex(pattern, SearchCase::Sensitive),
                SearchCursor::forward(None),
            )
            .unwrap()
            .matched()
            .unwrap();
        if pattern == "a+" {
            assert_eq!(found.start(), point(&core, 0, 4));
            assert_eq!(found.end(), point(&core, 0, 6));
        }
    }
}

#[test]
fn search_distinguishes_soft_wrap_from_hard_newline() {
    let mut core = core(4, 4, 128, 65_536);
    print(&mut core, "abcdef");
    assert!(
        core.search(
            &SearchQuery::literal("cdef", SearchCase::Sensitive),
            SearchCursor::forward(None)
        )
        .unwrap()
        .matched()
        .is_some()
    );
    hard_line(&mut core);
    print(&mut core, "xy");
    assert!(
        core.search(
            &SearchQuery::regex("ef$", SearchCase::Sensitive),
            SearchCursor::forward(None)
        )
        .unwrap()
        .matched()
        .is_some()
    );
    assert!(
        core.search(
            &SearchQuery::literal("f\nx", SearchCase::Sensitive),
            SearchCursor::forward(None)
        )
        .unwrap()
        .matched()
        .is_some()
    );
}

#[test]
fn matches_keep_logical_identity_across_output_and_reflow() {
    let mut core = core(8, 2, 128, 65_536);
    print(&mut core, "prefix needle suffix");
    let found = core
        .search(
            &SearchQuery::literal("needle", SearchCase::Sensitive),
            SearchCursor::forward(None),
        )
        .unwrap()
        .matched()
        .unwrap();
    hard_line(&mut core);
    print(&mut core, "new output");
    assert_eq!(core.search_match_state(found), SearchMatchState::Active);
    core.resize(TerminalSize::new(5, 4, 0, 0).unwrap()).unwrap();
    assert!(core.state().contains_logical_point(found.start()));
    assert!(core.state().contains_logical_point(found.end()));
    let found_after = core
        .search(
            &SearchQuery::literal("needle", SearchCase::Sensitive),
            SearchCursor::forward(None),
        )
        .unwrap()
        .matched()
        .unwrap();
    assert_eq!(found_after, found);
}

#[test]
fn retained_match_reports_trimmed_after_whole_line_eviction() {
    let mut core = core(8, 2, 1, 65_536);
    print(&mut core, "needle");
    let found = core
        .search(
            &SearchQuery::literal("needle", SearchCase::Sensitive),
            SearchCursor::forward(None),
        )
        .unwrap()
        .matched()
        .unwrap();
    for text in ["one", "two", "three", "four"] {
        hard_line(&mut core);
        print(&mut core, text);
    }
    assert_eq!(core.search_match_state(found), SearchMatchState::Trimmed);
}

#[test]
fn trimmed_and_unknown_navigation_cursors_fail_explicitly() {
    let mut core = core(4, 2, 1, 65_536);
    print(&mut core, "old");
    let old = point(&core, 0, 0);
    for text in ["one", "two", "three"] {
        hard_line(&mut core);
        print(&mut core, text);
    }
    assert_eq!(
        core.search(
            &SearchQuery::literal("two", SearchCase::Sensitive),
            SearchCursor::forward(Some(old))
        ),
        Err(SearchError::TrimmedCursor)
    );
}

#[test]
fn alternate_search_never_observes_primary_history() {
    let mut core = core(12, 2, 128, 65_536);
    print(&mut core, "primary");
    core.reduce(ScreenAction::SwitchBuffer {
        buffer: ScreenBuffer::Alternate,
        clear: true,
        home: true,
    })
    .unwrap();
    print(&mut core, "alternate");
    assert!(
        core.search(
            &SearchQuery::literal("primary", SearchCase::Sensitive),
            SearchCursor::forward(None)
        )
        .unwrap()
        .matched()
        .is_none()
    );
    assert!(
        core.search(
            &SearchQuery::literal("alternate", SearchCase::Sensitive),
            SearchCursor::forward(None)
        )
        .unwrap()
        .matched()
        .is_some()
    );
}

#[test]
fn hostile_regex_exhausts_search_work_without_backtracking() {
    let mut core = core(32, 2, 128, 80);
    print(&mut core, "aaaaaaaaaaaaaaaaaaaaaaaa");
    let error = core
        .search(
            &SearchQuery::regex("(a|aa)*b", SearchCase::Sensitive),
            SearchCursor::forward(None),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        SearchError::Limit(LimitError::Exceeded {
            kind: LimitKind::SearchWork,
            ..
        })
    ));
}

#[test]
fn invalid_and_empty_patterns_fail_closed() {
    let core = core(8, 2, 128, 65_536);
    assert_eq!(
        core.search(
            &SearchQuery::literal("", SearchCase::Sensitive),
            SearchCursor::forward(None)
        ),
        Err(SearchError::EmptyPattern)
    );
    for pattern in ["(", "a|", "[z-a]", "*a", "a\\"] {
        assert_eq!(
            core.search(
                &SearchQuery::regex(pattern, SearchCase::Sensitive),
                SearchCursor::forward(None)
            ),
            Err(SearchError::InvalidPattern)
        );
    }
}

fn core(columns: u16, rows: u16, history_lines: usize, search_work: usize) -> TerminalCore {
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
        clipboard_bytes: value,
        input_bytes: value,
        keyboard_stack: value,
        notification_bytes: value,
        reply_bytes: value,
        pending_events: value,
        pending_damage: value,
        history_lines,
        history_bytes: value,
        graphic_objects: value,
        graphic_pixels: value,
        graphic_decoded_bytes: value,
        graphic_frames: value,
        compression_ratio: value,
        parser_work: value,
        search_work,
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
fn point(core: &TerminalCore, row: u16, column: u16) -> crate::LogicalPoint {
    core.state().logical_point_at(row, column).unwrap()
}
