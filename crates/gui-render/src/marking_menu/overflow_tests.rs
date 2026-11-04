use super::*;

fn state() -> ReviewWorkspaceState {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.marking_menu = Some(MarkingMenuState {
        menu_key: "schematic.empty".into(),
        target_object_id: None,
        anchor_x_px: 900,
        anchor_y_px: 900,
        preview_slot: None,
        gesture_dx_px: 0,
        gesture_dy_px: 0,
        submenu_path: Vec::new(),
        selection: Some(datum_gui_protocol::selection_menu::SelectionMenu {
            pickup_world: datum_gui_protocol::PointNm { x: 0, y: 0 },
            project: datum_gui_protocol::SelectionProjectId::nil(),
            revision: "captured-source".into(),
            source_scene: "captured-scene".into(),
            schematic: true,
            filters: state.ui.filters.clone(),
            inspection_subject: None,
            items: vec![datum_gui_protocol::selection_menu::Item {
                label: "Select all on sheet".into(),
                command: datum_gui_protocol::selection_menu::Command::AllInDocument,
                unavailable: None,
            }],
            open: false,
            focus: 0,
            preview: None,
            preview_keys: Default::default(),
            scroll: 0,
        }),
    });
    state
}
fn draw(state: &ReviewWorkspaceState, height: f32) -> (Vec<TextRun>, Vec<HitRegion>) {
    let mut layout = ShellLayout::for_window(1280, 800, None);
    layout.viewport = RectPx {
        x: 40.,
        y: 50.,
        width: 333.,
        height,
    };
    let (mut text, mut hits) = (Vec::new(), Vec::new());
    render_marking_menu(state, &layout, &mut Vec::new(), &mut text, &mut hits);
    for hit in &hits {
        assert!(layout.viewport.contains(hit.rect.x, hit.rect.y));
        assert!(
            layout
                .viewport
                .contains(hit.rect.x + hit.rect.width, hit.rect.y + hit.rect.height)
        );
    }
    (text, hits)
}
fn has(hits: &[HitRegion], slot: &str) -> bool {
    hits.iter()
        .any(|hit| matches!(&hit.target,HitTarget::MarkingMenuItem{slot:s,..} if s==slot))
}
#[test]
fn marking_overflow_annotation_and_native_select_are_reachable_without_changing_source() {
    let mut state = state();
    let before = state.clone();
    let model = marking_menu_model().unwrap();
    let menu = &model.marking_menus["schematic.empty"];
    let (_, root) = draw(&state, 400.);
    assert!(
        has(&root, "MORE"),
        "real disclosure hit must exist alongside native selection capture"
    );
    assert!(
        !has(&root, "MORE:0"),
        "hidden commands are not root hit targets"
    );
    let marking = state.ui.marking_menu.as_mut().unwrap();
    assert!(marking.open_overflow(menu));
    assert!(!marking.open_overflow(menu));
    assert_eq!(
        marking
            .menu_item(menu, "MORE:0")
            .unwrap()
            .gui_local
            .as_deref(),
        Some("schematic.annotation.review")
    );
    assert!(marking.menu_item(menu, "MORE:1").is_none());
    let (text, hits) = draw(&state, 400.);
    assert!(has(&hits, "MORE:0") && has(&hits, "SELECT"));
    assert!(
        text.iter()
            .any(|run| run.text.contains("Annotate This Sheet"))
    );
    assert!(
        text.iter().any(|run| run.text.contains("Unavailable")),
        "unqualified fixture source stays visibly disabled"
    );
    assert_eq!(state.selection, before.selection);
    assert_eq!(
        state.schematic_authoring.source,
        before.schematic_authoring.source
    );
    assert_eq!(
        state.ui.marking_menu.as_ref().unwrap().selection,
        before.ui.marking_menu.as_ref().unwrap().selection
    );
    // Grouped Select retains its exact independent capture and ordinary row index.
    state
        .ui
        .marking_menu
        .as_mut()
        .unwrap()
        .selection
        .as_mut()
        .unwrap()
        .open = true;
    let (_, hits) = draw(&state, 400.);
    assert!(has(&hits, "SELECT:0"));
    assert!(!has(&hits, "MORE:0"));
}
#[test]
fn marking_overflow_focus_reveals_select_in_short_viewport_and_leaves_rows_qualified() {
    let mut state = state();
    let menu = &marking_menu_model().unwrap().marking_menus["schematic.empty"];
    let marking = state.ui.marking_menu.as_mut().unwrap();
    marking.open_overflow(menu);
    marking.preview_slot = Some("SELECT".into());
    let (_, hits) = draw(&state, 75.);
    assert!(has(&hits, "SELECT"));
    assert!(!has(&hits, "MORE:0"));
    let marking = state.ui.marking_menu.as_mut().unwrap();
    marking.submenu_path.pop();
    marking.preview_slot = None;
    let (_, hits) = draw(&state, 400.);
    assert!(has(&hits, "MORE"));
    assert!(!has(&hits, "MORE:0"));
}
