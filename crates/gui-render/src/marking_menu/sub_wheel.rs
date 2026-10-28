//! Nested sub-wheel radial (MM-C03). Renders one submenu's items as an N-wide,
//! content-sized wheel (ruling G6 — not a fixed 4/8 slot count) around the same
//! anchor, with a center back ("◀") affordance that pops one level. Inert shell:
//! every leaf is disabled and fires nothing; a child that is itself a submenu
//! shows the ▸ navigable affordance. Navigation state (which submenu is open)
//! lives in `MarkingMenuState::submenu_path`; this module only draws it.
use super::*;
use datum_gui_protocol::{GuiIconSet, GuiMenuItem};

// Full sub-wheel draw parameter set (anchor geometry, items, style, hit sink); a
// parameter struct would be a single-use aggregate with no clarity gain here.
#[allow(clippy::too_many_arguments)]
pub(super) fn render(
    preview: &MarkingMenuState,
    items: &[GuiMenuItem],
    icon_set: Option<&GuiIconSet>,
    anchor: (f32, f32),
    bounds: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    // Center = back affordance. Reuses MarkingMenuSurface as the pop target; the
    // activation handler pops one `submenu_path` level when it is hit.
    let back = RectPx {
        x: anchor.0 - MARKING_CENTER_SIZE * 0.5,
        y: anchor.1 - MARKING_CENTER_SIZE * 0.5,
        width: MARKING_CENTER_SIZE,
        height: MARKING_CENTER_SIZE,
    };
    hit_regions.push(HitRegion {
        rect: back,
        target: HitTarget::MarkingMenuSurface {
            menu_key: preview.menu_key.clone(),
            anchor_x_px: preview.anchor_x_px,
            anchor_y_px: preview.anchor_y_px,
        },
    });
    panel_quads.push(Quad::from_rect(back, PANEL_CARD_BG));
    push_rect_border(panel_quads, back, TEXT_ACCENT, 1.0);
    draw_text(
        "<",
        back.x + 6.0,
        back.y + 6.0,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_MUTED,
        TextFace::Ui,
        text_runs,
    );

    // Content-sized radial (ruling G6): N items evenly spaced, each a pie sector
    // proportional to the count (fewer items -> wider wedges), with the icon+label at
    // the label radius and a label-area hit rect for clicks.
    let count = items.len().max(1);
    let step = 360.0 / count as f32;
    let span = (step * 0.72).min(100.0);
    for (index, item) in items.iter().enumerate() {
        let angle = -90.0 + index as f32 * step;
        let slot_id = format!("SUB:{index}");
        let active = preview.preview_slot.as_deref() == Some(slot_id.as_str());
        // Inert-shell rule: a submenu child navigates; every other leaf is disabled
        // and fires nothing (independent of registry `is_enabled`).
        let navigable = matches!(
            item.binding(),
            datum_gui_protocol::GuiMenuBinding::Submenu(_)
        );
        let fill = if active {
            REVIEW_ROW_ACTIVE_BG
        } else {
            PANEL_CARD_BG
        };
        push_sector(
            panel_quads,
            anchor,
            angle,
            span,
            MARKING_INNER,
            MARKING_RADIUS,
            fill,
        );
        let rect = slot_label_rect(anchor, angle, bounds);
        render_marking_icon(item, icon_set, rect, navigable && active, text_runs);
        let label = if navigable {
            format!("{} >", truncate_text(&item.label, 11))
        } else {
            truncate_text(&item.label, 13)
        };
        draw_text(
            &label,
            rect.x + 16.0,
            rect.y + design_tokens::spacing::SP_03,
            design_tokens::typography::CAPTION_SIZE,
            if navigable && active {
                TEXT_PRIMARY
            } else {
                TEXT_MUTED
            },
            TextFace::Ui,
            text_runs,
        );
        hit_regions.push(HitRegion {
            rect,
            target: HitTarget::MarkingMenuItem {
                menu_key: preview.menu_key.clone(),
                slot: slot_id,
                label: item.label.clone(),
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_submenu_renders_content_sized_inert_subwheel_with_back() {
        // pcb.multiselect → "Align" submenu = Horizontal / Vertical / Distribute
        // (3 content-sized leaves, G6), plus a center back affordance. The shell is
        // inert: hits carry no verb/command, only slot identity for navigation.
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let mut layout = ShellLayout::for_window(1280, 800, None);
        layout.viewport = RectPx {
            x: 0.0,
            y: 0.0,
            width: 1280.0,
            height: 800.0,
        };
        state.ui.marking_menu = Some(MarkingMenuState {
            menu_key: "pcb.multiselect".into(),
            target_object_id: None,
            anchor_x_px: 640,
            anchor_y_px: 400,
            preview_slot: None,
            gesture_dx_px: 0,
            gesture_dy_px: 0,
            selection: None,
            submenu_path: vec!["Align".into()],
        });
        let mut quads = Vec::new();
        let mut text = Vec::new();
        let mut hits = Vec::new();
        render_marking_menu(&state, &layout, &mut quads, &mut text, &mut hits);

        let sub_hits = hits
            .iter()
            .filter(|hit| {
                matches!(&hit.target, HitTarget::MarkingMenuItem { slot, .. } if slot.starts_with("SUB:"))
            })
            .count();
        assert_eq!(
            sub_hits, 3,
            "Align sub-wheel should render its 3 content-sized leaves"
        );
        assert!(
            hits.iter()
                .any(|hit| matches!(&hit.target, HitTarget::MarkingMenuSurface { .. })),
            "sub-wheel needs a center back affordance"
        );
        for hit in &hits {
            assert!(layout.viewport.contains(hit.rect.x, hit.rect.y));
        }
    }
}
