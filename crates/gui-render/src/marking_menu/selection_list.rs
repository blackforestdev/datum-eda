//! Finite read-only Select leaves in the existing linear-overflow presentation.
use super::*;
pub(super) fn entry(
    preview: &MarkingMenuState,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    quads.push(Quad::from_rect(rect, PANEL_BG));
    push_rect_border(quads, rect, PANEL_CARD_BORDER, 1.0);
    draw_text(
        "Select…  S",
        rect.x + 8.0,
        rect.y + 8.0,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_PRIMARY,
        TextFace::Ui,
        text,
    );
    hits.push(HitRegion {
        rect,
        target: HitTarget::MarkingMenuItem {
            menu_key: preview.menu_key.clone(),
            slot: "SELECT".into(),
            label: "Select".into(),
        },
    });
}
pub(super) fn render(
    preview: &MarkingMenuState,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let menu = preview.selection.as_ref().unwrap();
    let rows: Vec<_> = menu
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| super::linear_list::Row {
            label: item.label.clone(),
            slot: format!("SELECT:{index}"),
            unavailable: item.unavailable.clone(),
        })
        .collect();
    super::linear_list::render(
        preview,
        layout,
        &format!("Select — {} entries · ↑↓ Enter Esc", menu.items.len()),
        &rows,
        menu.focus,
        menu.scroll,
        quads,
        text,
        hits,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focused_last_select_entry_is_visible_in_narrow_clipped_list() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let mut layout = ShellLayout::for_window(1280, 800, None);
        layout.viewport = RectPx {
            x: 40.0,
            y: 50.0,
            width: 200.0,
            height: 130.0,
        };
        state.ui.marking_menu = Some(MarkingMenuState {
            menu_key: "schematic.selection".into(),
            target_object_id: None,
            anchor_x_px: -100,
            anchor_y_px: 1000,
            preview_slot: None,
            gesture_dx_px: 0,
            gesture_dy_px: 0,
            selection: Some(datum_gui_protocol::selection_menu::SelectionMenu {
                pickup_world: datum_gui_protocol::PointNm { x: 0, y: 0 },
                project: datum_gui_protocol::SelectionProjectId::nil(),
                revision: "mechanism".into(),
                source_scene: "mechanism".into(),
                schematic: true,
                filters: state.ui.filters.clone(),
                inspection_subject: None,
                items: (0..303)
                    .map(|n| datum_gui_protocol::selection_menu::Item {
                        label: n.to_string(),
                        command: datum_gui_protocol::selection_menu::Command::All,
                        unavailable: (n == 301).then(|| "No source".into()),
                    })
                    .collect(),
                open: true,
                focus: 302,
                preview: None,
                preview_keys: Default::default(),
                scroll: 295,
            }),
            submenu_path: Vec::new(),
        });
        let mut q = Vec::new();
        let mut t = Vec::new();
        let mut h = Vec::new();
        render_marking_menu(&state, &layout, &mut q, &mut t, &mut h);
        assert!(h.iter().any(
            |hit| matches!(&hit.target,HitTarget::MarkingMenuItem {slot,..} if slot=="SELECT:302")
        ));
        // 3 visible rows + 1 panel dismiss-surface hit. The panel hit was added by
        // a5df9383 (dismiss on outside right release); this bound was left at 3 and
        // is corrected to 4 here — pre-existing, attributed to a5df9383, not MM-C03.
        assert!(h.len() <= 4);
        for hit in h {
            assert!(layout.viewport.contains(hit.rect.x, hit.rect.y));
            assert!(
                layout
                    .viewport
                    .contains(hit.rect.x + hit.rect.width, hit.rect.y + hit.rect.height)
            );
        }
        assert!(t.iter().any(|run| run.text.contains("Unavailable")));
        // Keyboard focus and pointer preview share this captured focus index.
        // The exact disabled cause is visible before any activation, including
        // when the list must reveal a late entry in a clipped viewport.
        let before = state.clone();
        state
            .ui
            .marking_menu
            .as_mut()
            .unwrap()
            .selection
            .as_mut()
            .unwrap()
            .focus = 301;
        for width in [200.0, 360.0] {
            layout.viewport.width = width;
            let mut text = Vec::new();
            let mut hits = Vec::new();
            render_marking_menu(&state, &layout, &mut Vec::new(), &mut text, &mut hits);
            let reason = text
                .iter()
                .find(|run| run.text == "Unavailable — No source")
                .expect("focused disabled cause is presented before activation");
            let hit = hits
                .iter()
                .find(|hit| {
                    matches!(&hit.target,
                HitTarget::MarkingMenuItem {slot,..} if slot == "SELECT:301")
                })
                .unwrap();
            assert_eq!(reason.clip_bounds, Some(hit.rect));
            assert!(reason.y >= hit.rect.y && reason.y < hit.rect.y + hit.rect.height);
        }
        assert_eq!(
            state
                .ui
                .marking_menu
                .as_ref()
                .unwrap()
                .selection
                .as_ref()
                .unwrap()
                .items,
            before
                .ui
                .marking_menu
                .as_ref()
                .unwrap()
                .selection
                .as_ref()
                .unwrap()
                .items
        );
        let captured = "Engine(UnavailableAssignment { origin: ElectricalOccurrence { class: \"wires\", source_id: 71000000-0000-4000-8000-000000000081, instance_path: [] } })";
        state
            .ui
            .marking_menu
            .as_mut()
            .unwrap()
            .selection
            .as_mut()
            .unwrap()
            .items[301]
            .unavailable = Some(captured.into());
        let mut text = Vec::new();
        render_marking_menu(&state, &layout, &mut Vec::new(), &mut text, &mut Vec::new());
        assert!(
            text.iter()
                .any(|run| run.text == "Unavailable — Electrical assignment unavailable")
        );
        assert_eq!(
            state
                .ui
                .marking_menu
                .as_ref()
                .unwrap()
                .selection
                .as_ref()
                .unwrap()
                .items[301]
                .unavailable
                .as_deref(),
            Some(captured)
        );
        for quad in q {
            assert!(
                quad.points
                    .iter()
                    .all(|&(x, y)| layout.viewport.contains(x, y))
            );
        }
    }
}
