use super::*;
use datum_gui_protocol::library_browser::acquisition::{
    LibraryAcquisition, LibraryAcquisitionPreview, NativePartAcquisitionPreview,
};
fn raw(source: &str) -> Value {
    source.parse().unwrap()
}

fn selected() -> ReviewWorkspaceState {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let mut a = LibraryAcquisition::begin("revision".into(), None);
    a.use_native_part_source();
    a.component_picker = true;
    a.loaded = true;
    let id = uuid::Uuid::new_v4();
    a.selected = Some(id.to_string());
    a.selected_source = Some("/chosen/project".into());
    a.preview = Some(LibraryAcquisitionPreview {
        name: "Declared component".into(),
        source_digest: "pinned".into(),
        source_path: "/chosen/project".into(),
        unit: raw("{}"),
        symbol: raw("{}"),
        footprint: None,
        notices: vec![],
        native_part: Some(NativePartAcquisitionPreview {
            part: raw(&format!(r#"{{"uuid":"{id}","value":"10k"}}"#)),
            entity: raw("{}"),
            package: raw(r#"{"name":"Declared package"}"#),
            footprint: raw(r#"{"name":"Declared footprint"}"#),
            pin_pad_map: raw("{}"),
            gate_count: 1,
            object_count: 0,
            padstacks: vec![],
            gate_symbols: vec![(
                "A".into(),
                raw(r#"{"pins":{}}"#),
                raw(r#"{"drawings":[],"pin_anchors":[]}"#),
            )],
        }),
    });
    state.library.acquisition = Some(a);
    state
}

#[test]
fn narrow_picker_keeps_cancel_and_explicit_adoption_inside_the_window_and_loading_cannot_commit() {
    let mut state = selected();
    // A source refusal can be longer than the window. It must remain scrollable
    // without pushing Cancel or the separate adoption action offscreen.
    state.library.acquisition.as_mut().unwrap().refusal =
        Some("Source changed; refresh before placing. ".repeat(60));
    for (width, height) in [(1280, 900), (560, 720)] {
        let layout = ShellLayout::for_window(width, height, None);
        let mut quads = vec![];
        let mut text = vec![];
        let mut hits = vec![];
        render(&state, &layout, &mut quads, &mut text, &mut hits);
        for control in [C::Cancel, C::AcquireImport] {
            let hit = hits
                .iter()
                .find(|h| h.target == HitTarget::Library(control.clone()))
                .unwrap();
            assert!(hit.rect.y >= 0. && hit.rect.y + hit.rect.height <= height as f32);
            assert!(hit.rect.x >= 0. && hit.rect.x + hit.rect.width <= width as f32);
        }
        assert!(
            hits.iter()
                .any(|h| matches!(h.target, HitTarget::Library(C::Scroll { .. })))
        );
        assert!(
            text.iter()
                .any(|run| run.text.starts_with("Adopt into project library"))
        );
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
