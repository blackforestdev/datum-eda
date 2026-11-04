//! Declared long-tail actions configure the same linear list as native Select.
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
        "More... M",
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
            slot: "MORE".into(),
            label: "More".into(),
        },
    });
}
pub(super) fn render(
    state: &ReviewWorkspaceState,
    preview: &MarkingMenuState,
    menu: &GuiMarkingMenu,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let rows: Vec<_> = preview
        .overflow_slots(menu)
        .into_iter()
        .map(|slot| {
            if slot == "SELECT" {
                return linear_list::Row {
                    label: "Select...  S >".into(),
                    slot,
                    unavailable: None,
                };
            }
            let item = preview.menu_item(menu, &slot).unwrap();
            let parent = matches!(
                item.binding(),
                datum_gui_protocol::GuiMenuBinding::Submenu(_)
            );
            linear_list::Row {
                label: if parent {
                    format!("{} >", item.label)
                } else {
                    item.label.clone()
                },
                unavailable: if parent {
                    None
                } else {
                    preview.schematic_action_unavailable_reason(item, state)
                },
                slot,
            }
        })
        .collect();
    let focus = rows
        .iter()
        .position(|row| Some(&row.slot) == preview.preview_slot.as_ref())
        .unwrap_or(0);
    linear_list::render(
        preview,
        layout,
        "More · Up/Down Enter · Esc back",
        &rows,
        focus,
        focus.saturating_sub(7),
        quads,
        text,
        hits,
    );
}
