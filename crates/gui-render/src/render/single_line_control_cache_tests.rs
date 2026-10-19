//! A real shaped-cache regression for horizontally revealed single-line values.
use super::*;
use crate::global_preferences_primitives::{ControlMeshCache, ControlPainter};
use datum_gui_protocol::single_line_editor::SingleLineEditor;

#[test]
fn terminal_context_short_reported_home_retains_its_glyph_in_real_shaped_cache() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.terminal.status = "running".into();
    state.ui.terminal.session_context = Some(datum_gui_protocol::TerminalSessionContext {
        binding: datum_gui_protocol::TerminalSessionBinding::Independent,
        default_tab_label: "Home".into(),
        directory: std::env::var("HOME").expect("test runner Home"),
        directory_reported: true,
    });
    state.ui.terminal.tabs = vec![datum_gui_protocol::TerminalTabState {
        session_id: "real-shaped-home".into(),
        previous_session_id: None,
        label: "Home".into(),
        event_log_path: String::new(),
        activity_event_count: 0,
        activity_summary: vec![],
        active: true,
        attached: true,
        status: "running".into(),
        restart_count: 0,
        unread_output: false,
        unread_bell_count: 0,
    }];
    let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
    crate::terminal_tab_strip::render_terminal_tab_strip(
        &state,
        RectPx {
            x: 0.0,
            y: 560.0,
            width: 1344.0,
            height: 44.0,
        },
        1.0,
        &mut quads,
        &mut text,
        &mut hits,
    );
    let directory = text
        .iter()
        .position(|t| t.text == "~")
        .expect("short reported Home must not be truncated to an empty string");
    let mut fonts = load_datum_fonts();
    let mut cache = TextBufferCache::default();
    cache.begin_frame(Profile::Workspace);
    let (indices, _) = cache.indices(&mut fonts, &text, 1344, 806);
    let rows = cache.entries()[indices[directory]]
        .buffer
        .layout_runs()
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].glyphs.last().unwrap().end, 1);
}

#[test]
fn single_line_reveal_retains_visible_tail_through_real_surface_bounded_cache() {
    let value = format!(
        "/home/designer/{}/TAIL-END",
        "long-exact-parent-segment/".repeat(18)
    );
    let mut fonts = load_datum_fonts();
    for mode in ["end", "home", "selection"] {
        let mut editor = SingleLineEditor::at_end(&value);
        match mode {
            "home" => editor.home(&value, false),
            "selection" => editor.select_all(&value),
            _ => {}
        }
        let rect = RectPx {
            x: 180.0,
            y: 142.0,
            width: 334.0,
            height: 34.0,
        };
        let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
        let mut controls = ControlMeshCache::default();
        let mut painter = ControlPainter::new(&mut quads, &mut controls, 1.0);
        crate::single_line_control::paint(
            &value,
            "",
            editor,
            true,
            true,
            rect,
            &mut painter,
            &mut text,
            &mut hits,
            HitTarget::FolderBrowser(datum_gui_protocol::FolderBrowserFocus::Path),
        )
        .unwrap();
        let mut cache = TextBufferCache::default();
        cache.begin_frame(Profile::Overlay);
        let (indices, _) = cache.indices(&mut fonts, &text, 620, 620);
        for (run, index) in text.iter().zip(indices) {
            let rows = cache.entries()[index]
                .buffer
                .layout_runs()
                .collect::<Vec<_>>();
            assert_eq!(
                rows.len(),
                1,
                "single-line input must stay on one visible row ({mode})"
            );
            let last = rows[0].glyphs.last().expect("visible path glyphs");
            assert_eq!(
                last.end,
                run.text.len(),
                "surface-width clamp must not wrap away the painted tail ({mode})"
            );
            let clip = run.clip_bounds.unwrap();
            let visible = rows[0]
                .glyphs
                .iter()
                .filter_map(|glyph| {
                    let left = (run.x + glyph.x).max(clip.x);
                    let right = (run.x + glyph.x + glyph.w).min(clip.x + clip.width);
                    (right > left).then_some((left, right))
                })
                .collect::<Vec<_>>();
            let left = visible.iter().map(|v| v.0).fold(f32::INFINITY, f32::min);
            let right = visible
                .iter()
                .map(|v| v.1)
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(
                right - left > clip.width - 24.0,
                "revealed text must fill the field, not leave a tiny first-row fragment ({mode})"
            );
            assert!(
                text_buffer_extent(run, 620, 620).0 <= 620,
                "global layout bound stays intact"
            );
        }
        if mode == "home" {
            assert_eq!(
                crate::single_line_control::single_line_caret_at(&value, editor, rect.width, 9.0)
                    .unwrap(),
                0
            );
        } else {
            assert!(text[0].text.ends_with("TAIL-END"));
            assert_eq!(
                crate::single_line_control::single_line_caret_at(
                    &value,
                    editor,
                    rect.width,
                    rect.width - 1.0
                )
                .unwrap(),
                value.len()
            );
        }
    }
}
