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
            HitTarget::Library(C::AcquirePath | C::AcquireLoad)
        )));
        assert!(
            text.iter()
                .any(|run| run.text == "Place R — schematic only" && run.size == 16.)
        );
        assert!(text.iter().any(|run| (run.text.contains("SCHEMATIC ONLY")
            || run.text.contains("Symbol · schematic only"))
            && run.size == 16.));
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
