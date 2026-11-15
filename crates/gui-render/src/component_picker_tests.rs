//! Renderer projection checks using the shipped resistor, not a fabricated Part.
//! Ordinary entry, canonical placement and history require the real GUI workflow.
use super::*;
use datum_gui_protocol::library_browser::acquisition::{
    AcquisitionSource, ComponentPickerResultKind, LibraryAcquisition, LibraryAcquisitionPreview,
    catalog,
};

fn selected_shipped_resistor() -> ReviewWorkspaceState {
    let mut state = ReviewWorkspaceState::unopened();
    let mut a = LibraryAcquisition::begin(String::new(), None);
    a.source = AcquisitionSource::BundledSymbols;
    a.component_picker = true;
    a.loaded = true;
    a.sources = catalog::bundled().unwrap().symbols;
    a.names = a
        .sources
        .iter()
        .map(|source| source.label.clone())
        .collect();
    a.search = "resistor".into();
    a.search_editor = datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&a.search);
    let source = &a.sources[a.results()[0]];
    assert_eq!(source.label, "Device:R");
    assert_eq!(
        a.picker_result_kind(source),
        ComponentPickerResultKind::Symbol
    );
    let real = eda_engine::import::library_bundle::preview(&source.path, &source.name).unwrap();
    assert_eq!(
        real.symbol.uuid.to_string(),
        "24b67160-a450-51f1-971e-a1499e1fca69"
    );
    assert_eq!(
        real.unit.uuid.to_string(),
        "0920a831-9ab8-5249-b627-47f55cae198b"
    );
    a.selected = Some(source.name.clone());
    a.selected_source = Some(source.path.clone());
    a.preview = Some(LibraryAcquisitionPreview::from_converted_symbol(real).unwrap());
    assert_eq!(a.request().unwrap().source_format, "datum_bundle");
    state.library.focus = Some(C::AcquireSearch);
    state.library.acquisition = Some(a);
    state
}

#[test]
fn real_symbol_preview_discloses_kind_and_keeps_search_and_footer_reachable() {
    let mut state = selected_shipped_resistor();
    for (width, height) in [(1344, 806), (560, 720)] {
        state.library.acquisition.as_mut().unwrap().loading = false;
        let layout = ShellLayout::for_window(width, height, None);
        let mut quads = vec![];
        let mut text = vec![];
        let mut hits = vec![];
        render(&state, &layout, &mut quads, &mut text, &mut hits);
        for control in [C::Cancel, C::AcquireImport, C::AcquireSearch] {
            let hit = hits
                .iter()
                .find(|h| h.target == HitTarget::Library(control.clone()))
                .unwrap();
            assert!(hit.rect.y >= 0. && hit.rect.y + hit.rect.height <= height as f32);
            assert!(hit.rect.x >= 0. && hit.rect.x + hit.rect.width <= width as f32);
        }
        assert!(!hits.iter().any(|h| matches!(
            h.target,
            HitTarget::Library(C::AcquirePath | C::AcquireLoad | C::AcquirePage(_))
        )));
        assert!(
            text.iter()
                .any(|run| run.text == "Place R — schematic only" && run.size == 16.)
        );
        assert!(text.iter().any(|run| (run.text.contains("SCHEMATIC ONLY")
            || run.text.contains("Symbol · schematic only"))
            && run.size == 16.));
        let kind = text
            .iter()
            .find(|run| run.text == "Symbol — SCHEMATIC ONLY")
            .unwrap();
        let visible = kind.clip_bounds.unwrap();
        assert!(kind.y >= visible.y && kind.y + kind.size <= visible.y + visible.height);
        let list = hits
            .iter()
            .find(|h| matches!(h.target, HitTarget::Library(C::Scroll { .. })))
            .unwrap()
            .rect;
        let row_runs: Vec<_> = text
            .iter()
            .filter(|run| {
                run.x >= list.x
                    && run.x < list.x + list.width
                    && run.y >= list.y
                    && run.y < list.y + list.height
            })
            .collect();
        assert!(row_runs.iter().any(|run| run.text.ends_with('…')));
        for run in row_runs {
            let available = run.layout_size.unwrap().0;
            assert!(
                measured_text_run_width_px(&run.text, run.size, run.face).unwrap() <= available
            );
        }
        if width == 1344 {
            assert!(kind.x >= list.x + list.width);
            assert!(
                hits.iter()
                    .filter(|h| matches!(h.target, HitTarget::Library(C::AcquireResult(_))))
                    .count()
                    > 4
            );
        } else {
            assert!(kind.y >= list.y + list.height);
        }
        assert!(
            !text
                .iter()
                .any(|run| run.text.contains("Adopt into project library"))
        );
        if width == 560 {
            assert!(
                hits.iter()
                    .any(|h| matches!(h.target, HitTarget::Library(C::Scroll { content, height, .. }) if content > height))
            );
        }
        state.library.acquisition.as_mut().unwrap().loading = true;
        hits.clear();
        quads.clear();
        text.clear();
        render(&state, &layout, &mut quads, &mut text, &mut hits);
        assert!(
            !hits
                .iter()
                .any(|h| h.target == HitTarget::Library(C::AcquireImport))
        );
        assert!(
            hits.iter()
                .any(|h| h.target == HitTarget::Library(C::Cancel))
        );
        state.library.acquisition.as_mut().unwrap().loading = false;
    }
}

#[test]
fn real_filtered_list_reaches_its_last_row_without_moving_selected_preview() {
    let mut state = selected_shipped_resistor();
    let last = *state
        .library
        .acquisition
        .as_ref()
        .unwrap()
        .results()
        .last()
        .unwrap();
    state.library.scroll_offset = usize::MAX;
    state.library.reveal_focus = false;
    let layout = ShellLayout::for_window(1344, 806, None);
    let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
    render(&state, &layout, &mut quads, &mut text, &mut hits);
    assert!(
        hits.iter()
            .any(|h| h.target == HitTarget::Library(C::AcquireResult(last)))
    );
    assert!(text.iter().any(|t| t.text == "Symbol — SCHEMATIC ONLY"));
    assert!(
        hits.iter()
            .any(|h| h.target == HitTarget::Library(C::AcquireImport))
    );
}

#[test]
fn real_symbol_details_scroll_independently_of_list_and_fixed_identity() {
    let mut state = selected_shipped_resistor();
    let first = state.library.acquisition.as_ref().unwrap().results()[0];
    let layout = ShellLayout::for_window(560, 720, None);
    let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
    render(&state, &layout, &mut quads, &mut text, &mut hits);
    let identity = text.iter().rfind(|t| t.text == "Device:R").unwrap();
    let fixed = (identity.x, identity.y);
    let kind_y = text
        .iter()
        .find(|t| t.text == "Symbol — SCHEMATIC ONLY")
        .unwrap()
        .y;
    state
        .library
        .acquisition
        .as_mut()
        .unwrap()
        .preview_scroll_offset = usize::MAX;
    quads.clear();
    text.clear();
    hits.clear();
    render(&state, &layout, &mut quads, &mut text, &mut hits);
    let identity = text.iter().rfind(|t| t.text == "Device:R").unwrap();
    assert_eq!(fixed, (identity.x, identity.y));
    assert_eq!(
        kind_y,
        text.iter()
            .find(|t| t.text == "Symbol — SCHEMATIC ONLY")
            .unwrap()
            .y
    );
    assert!(
        hits.iter()
            .any(|h| h.target == HitTarget::Library(C::AcquireResult(first)))
    );
    assert!(hits.iter().any(|h| matches!(h.target,
        HitTarget::Library(C::AcquirePreviewScroll { offset, .. }) if offset > 0)));
    assert!(
        hits.iter()
            .any(|h| matches!(h.target, HitTarget::Library(C::Scroll { offset: 0, .. })))
    );
}
