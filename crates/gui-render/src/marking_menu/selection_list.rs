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
    let bounds = layout.viewport;
    let width = 360.0_f32.min(bounds.width.max(0.0));
    let row = 30.0;
    let visible = ((bounds.height - 40.0).max(0.0) / row).floor() as usize;
    let mut start = menu.scroll.min(menu.items.len().saturating_sub(1));
    if visible > 0 {
        if menu.focus < start {
            start = menu.focus;
        } else if menu.focus >= start + visible {
            start = menu.focus + 1 - visible;
        }
    }
    let height = (40.0 + row * visible.min(menu.items.len().saturating_sub(start)) as f32)
        .min(bounds.height.max(0.0));
    let x = (preview.anchor_x_px as f32)
        .clamp(bounds.x, (bounds.x + bounds.width - width).max(bounds.x));
    let y = (preview.anchor_y_px as f32)
        .clamp(bounds.y, (bounds.y + bounds.height - height).max(bounds.y));
    let panel = RectPx {
        x,
        y,
        width,
        height,
    };
    quads.push(Quad::from_rect(panel, PANEL_BG));
    push_rect_border(quads, panel, PANEL_CARD_BORDER, 1.0);
    draw_text(
        &format!("Select — {} entries · ↑↓ Enter Esc", menu.items.len()),
        x + 8.0,
        y + 10.0,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_PRIMARY,
        TextFace::Ui,
        text,
    );
    for (index, item) in menu.items.iter().enumerate().skip(start).take(visible) {
        let rect = RectPx {
            x: x + 4.0,
            y: y + 36.0 + row * (index - start) as f32,
            width: (width - 8.0).max(0.0),
            height: row,
        };
        let focused = menu.focus == index;
        quads.push(Quad::from_rect(
            rect,
            if focused {
                REVIEW_ROW_ACTIVE_BG
            } else {
                PANEL_BG
            },
        ));
        if focused {
            push_rect_border(quads, rect, TEXT_ACCENT, 1.0);
        }
        let label = if item.unavailable.is_some() {
            format!("Unavailable: {}", item.label)
        } else {
            item.label.clone()
        };
        draw_text(
            &truncate_text(&label, 48),
            rect.x + 6.0,
            rect.y + 8.0,
            design_tokens::typography::CAPTION_SIZE,
            if item.unavailable.is_some() {
                TEXT_MUTED
            } else {
                TEXT_PRIMARY
            },
            TextFace::Ui,
            text,
        );
        hits.push(HitRegion {
            rect,
            target: HitTarget::MarkingMenuItem {
                menu_key: preview.menu_key.clone(),
                slot: format!("SELECT:{index}"),
                label: item.label.clone(),
            },
        });
    }
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
                        unavailable: (n == 301).then(|| "Unavailable authority".into()),
                    })
                    .collect(),
                open: true,
                focus: 302,
                preview: None,
                preview_keys: Default::default(),
                scroll: 295,
            }),
        });
        let mut q = Vec::new();
        let mut t = Vec::new();
        let mut h = Vec::new();
        render_marking_menu(&state, &layout, &mut q, &mut t, &mut h);
        assert!(h.iter().any(
            |hit| matches!(&hit.target,HitTarget::MarkingMenuItem {slot,..} if slot=="SELECT:302")
        ));
        assert!(h.len() <= 3);
        for hit in h {
            assert!(layout.viewport.contains(hit.rect.x, hit.rect.y));
            assert!(
                layout
                    .viewport
                    .contains(hit.rect.x + hit.rect.width, hit.rect.y + hit.rect.height)
            );
        }
        assert!(t.iter().any(|run| run.text.contains("Unavailable")));
        for quad in q {
            assert!(
                quad.points
                    .iter()
                    .all(|&(x, y)| layout.viewport.contains(x, y))
            );
        }
    }
}
