//! Actual Inspector paint/scroll regression for complete refusal disclosure.
use super::*;
const REASON: &str = "Cannot remove members from a run selection. Membership follows connectivity. Select an object without modifiers to start an explicit selection.";

fn total(hits: &[HitRegion]) -> usize {
    hits.iter()
        .find_map(|hit| match hit.target {
            HitTarget::SelectionInspectorScroll { total, .. } => Some(total),
            _ => None,
        })
        .expect("Inspector scroll owner")
}
fn notice_sequence(rows: &[String]) -> &[String] {
    for start in 0..rows.len() {
        let mut complete = String::new();
        for end in start..rows.len() {
            complete.push_str(&rows[end]);
            if complete == REASON {
                return &rows[start..=end];
            }
            if !REASON.starts_with(&complete) {
                break;
            }
        }
    }
    panic!("full refusal reason/remedy absent from actual paint rows: {rows:?}");
}
#[test]
fn refusal_notice_is_complete_measured_and_scrollable_at_narrow_widths() {
    let (fixture, mut state) = selected_state();
    let bytes = fixture.source_bytes();
    let SelectionTarget::Native(view) = &mut state.selection else {
        panic!("native selection")
    };
    view.notice = Some(REASON.into());
    let selected = state.selection.clone();
    for width in [160.0, 296.0, 480.0] {
        let rect = RectPx {
            x: 400.,
            y: 80.,
            width,
            height: 1300.,
        };
        state.ui.selection_inspector.scroll_offset = 0;
        let (_, text, hits) = frame(&state, rect);
        let with_notice = total(&hits);
        let rows: Vec<_> = text
            .iter()
            .filter(|run| {
                run.clip_bounds
                    .is_some_and(|clip| clip.y >= rect.y + 76. && clip.height == 22.)
            })
            .map(|run| run.text.clone())
            .collect();
        let lines = notice_sequence(&rows);
        assert!(lines.len() > 1);
        for line in lines {
            assert!(!line.contains('…'));
            assert!(
                crate::measured_text_run_width_px(line, 11., TextFace::Ui).unwrap() <= width - 24.
            );
        }
        // A one-row viewport must reach every notice fragment and the final
        // navigation row; bookkeeping cannot count a multi-line notice once.
        let short = RectPx {
            height: 98.,
            ..rect
        };
        let mut pages = Vec::new();
        for offset in 0..with_notice {
            state.ui.selection_inspector.scroll_offset = offset;
            let (_, text, hits) = frame(&state, short);
            assert_eq!(total(&hits), with_notice);
            pages.extend(
                text.iter()
                    .filter(|run| {
                        run.clip_bounds
                            .is_some_and(|clip| clip.y == short.y + 76. && clip.height == 22.)
                    })
                    .map(|run| run.text.clone()),
            );
        }
        assert_eq!(notice_sequence(&pages), lines);
        state.ui.selection_inspector.scroll_offset = usize::MAX;
        let (_, _, hits) = frame(&state, short);
        assert!(
            hits.iter()
                .any(|hit| matches!(hit.target, HitTarget::SelectionFiltersToggle))
        );
        assert_eq!(
            state.selection, selected,
            "paint/scroll must preserve complete selection"
        );
        let SelectionTarget::Native(view) = &mut state.selection else {
            unreachable!()
        };
        view.notice = None;
        state.ui.selection_inspector.scroll_offset = 0;
        let (_, _, without) = frame(&state, rect);
        assert_eq!(with_notice - total(&without), lines.len());
        let SelectionTarget::Native(view) = &mut state.selection else {
            unreachable!()
        };
        view.notice = Some(REASON.into());
    }
    assert_eq!(state.selection, selected);
    for (path, before) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
}
