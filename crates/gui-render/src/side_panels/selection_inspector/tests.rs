use super::*;
use crate::native_selection_overlay::tests::selected_state;
fn frame(state: &ReviewWorkspaceState, rect: RectPx) -> (Vec<Quad>, Vec<TextRun>, Vec<HitRegion>) {
    let mut quads = Vec::new();
    let mut text = Vec::new();
    let mut hits = Vec::new();
    super::super::inspector_dispatch::render_active_inspector(
        state, rect, &mut quads, &mut text, &mut hits,
    )
    .unwrap();
    (quads, text, hits)
}
#[test]
fn inventory_and_filters_are_scrollable_with_complete_identity_and_no_reselection() {
    let (fixture, mut state) = selected_state();
    let bytes = fixture.source_bytes();
    let selection = state.selection.clone();
    let rect = RectPx {
        x: 400.0,
        y: 80.0,
        width: 300.0,
        height: 164.0,
    };
    let (_, _, hits) = frame(&state, rect);
    let (total, visible) = hits
        .iter()
        .find_map(|r| match r.target {
            HitTarget::SelectionInspectorScroll { total, visible } => Some((total, visible)),
            _ => None,
        })
        .unwrap();
    assert_eq!(visible, 4);
    assert!(total > visible);
    state.ui.selection_inspector.inventory_expanded = true;
    state.ui.selection_inspector.filters_expanded = true;
    let (_, _, hits) = frame(&state, rect);
    let total = hits
        .iter()
        .find_map(|r| match r.target {
            HitTarget::SelectionInspectorScroll { total, .. } => Some(total),
            _ => None,
        })
        .unwrap();
    let mut found_members = false;
    let mut classes = BTreeSet::new();
    let SelectionTarget::Native(view) = &state.selection else {
        panic!("native fixture");
    };
    let member = view.members.first().unwrap().id.to_string();
    for offset in 0..total {
        state.ui.selection_inspector.scroll_offset = offset;
        let (quads, text, hits) = frame(&state, rect);
        found_members |= text
            .iter()
            .any(|run| run.text.starts_with("Tracks ") && run.text.contains(&member[..8]));
        for hit in hits {
            assert_eq!(hit.rect.intersect(rect), Some(hit.rect));
            if let HitTarget::SelectionClassToggle(class) = hit.target {
                classes.insert(class);
            }
        }
        assert!(
            quads
                .iter()
                .all(|q| q.points.iter().all(|&(x, y)| rect.contains(x, y)))
        );
        assert!(text.iter().all(|run| {
            run.clip_bounds
                .is_some_and(|clip| clip.intersect(rect) == Some(clip))
        }));
        assert_eq!(state.selection, selection);
    }
    assert!(
        found_members,
        "offscreen inventory can be reached by scrolling"
    );
    assert_eq!(classes, CLASSES.into_iter().collect());
    for (path, bytes) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}

#[test]
fn long_values_and_reasons_are_explicit_previews_clipped_to_one_scroll_row() {
    let (fixture, mut state) = selected_state();
    let source = fixture.source_bytes();
    if let SelectionTarget::Native(view) = &mut state.selection {
        view.inspection
            .iter_mut()
            .find(|s| s.scope == view.scope)
            .unwrap()
            .fields
            .insert(
                "zz_large_value".into(),
                datum_gui_protocol::selection_view::InspectionValue::Common(
                    "long value ".repeat(100_000).into(),
                ),
            );
    }
    let selection = state.selection.clone();
    let rect = RectPx {
        x: 400.0,
        y: 80.0,
        width: 300.0,
        height: 200.0,
    };
    let (_, _, hits) = frame(&state, rect);
    let total = hits
        .iter()
        .find_map(|h| match h.target {
            HitTarget::SelectionInspectorScroll { total, .. } => Some(total),
            _ => None,
        })
        .unwrap();
    let mut saw_preview = false;
    for offset in 0..total {
        state.ui.selection_inspector.scroll_offset = offset;
        let (_, text, _) = frame(&state, rect);
        for run in text
            .iter()
            .filter(|r| r.layout_size.is_some_and(|(_, h)| h == 18.0))
        {
            let clip = run.clip_bounds.expect("row-owned clip");
            assert_eq!(clip.height, 22.0);
            assert!(run.y >= clip.y && run.y < clip.y + clip.height);
            assert!(run.text.chars().count() <= 43);
            assert_eq!(clip.intersect(rect), Some(clip));
            if run.text.starts_with("zz large value:") {
                assert!(run.text.ends_with('…'));
                saw_preview = true;
            }
        }
    }
    assert!(saw_preview);
    assert_eq!(state.selection, selection);
    assert_eq!(fixture.source_bytes(), source);
}
