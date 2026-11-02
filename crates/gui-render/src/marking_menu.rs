use super::*;
use datum_gui_protocol::{
    GuiIconSet, GuiMarkingMenu, GuiMenuItem, MarkingMenuState, load_default_gui_icon_set,
    load_default_gui_menu_model,
};
use std::sync::OnceLock;

static MARKING_MENU_MODEL: OnceLock<Result<datum_gui_protocol::GuiMenuModel, String>> =
    OnceLock::new();
static MARKING_ICON_SET: OnceLock<Result<GuiIconSet, String>> = OnceLock::new();

mod selection_list;
mod sub_wheel;

const MARKING_RADIUS: f32 = 92.0;
const MARKING_LABEL_WIDTH: f32 = 96.0;
const MARKING_LABEL_HEIGHT: f32 = 26.0;
const MARKING_CENTER_SIZE: f32 = 22.0;
// Radial-wedge form (ratified §6 acceptance surface): the shell is a ring of
// pie-sector wedges between the hub (MARKING_INNER) and the rim (MARKING_RADIUS),
// with each item's icon+label at MARKING_LABEL_R along its slot angle. Wedges are
// separated by a small angular gap (MARKING_SECTOR_SPAN < the slot step) rather than
// borders; the active wedge is filled with the accent background.
const MARKING_INNER: f32 = 34.0;
const MARKING_LABEL_R: f32 = 63.0;
const MARKING_SECTOR_SPAN: f32 = 40.0;

/// Compass slot -> wheel angle in degrees (0 = +x / east, -90 = up / north).
pub(super) fn slot_angle(slot: &str) -> Option<f32> {
    Some(match slot {
        "N" => -90.0,
        "NE" => -45.0,
        "E" => 0.0,
        "SE" => 45.0,
        "S" => 90.0,
        "SW" => 135.0,
        "W" => 180.0,
        "NW" => -135.0,
        _ => return None,
    })
}

/// Emit a filled pie-sector (ring segment) as a strip of quads approximating the
/// arc between `ri`..`ro` over [center-span/2, center+span/2] degrees.
pub(super) fn push_sector(
    panel_quads: &mut Vec<Quad>,
    anchor: (f32, f32),
    center_deg: f32,
    span_deg: f32,
    ri: f32,
    ro: f32,
    color: [f32; 3],
) {
    let steps = 10usize;
    let start = (center_deg - span_deg * 0.5).to_radians();
    let step = span_deg.to_radians() / steps as f32;
    for i in 0..steps {
        let t0 = start + step * i as f32;
        let t1 = start + step * (i + 1) as f32;
        let (c0, s0) = (t0.cos(), t0.sin());
        let (c1, s1) = (t1.cos(), t1.sin());
        panel_quads.push(Quad {
            points: [
                (anchor.0 + ri * c0, anchor.1 + ri * s0),
                (anchor.0 + ro * c0, anchor.1 + ro * s0),
                (anchor.0 + ro * c1, anchor.1 + ro * s1),
                (anchor.0 + ri * c1, anchor.1 + ri * s1),
            ],
            color,
        });
    }
}

/// The icon+label rect for a wedge, centered at the label radius along its angle.
fn slot_label_rect(anchor: (f32, f32), angle_deg: f32, bounds: RectPx) -> RectPx {
    let rad = angle_deg.to_radians();
    let cx = anchor.0 + MARKING_LABEL_R * rad.cos();
    let cy = anchor.1 + MARKING_LABEL_R * rad.sin();
    let x = (cx - MARKING_LABEL_WIDTH * 0.5).clamp(
        bounds.x + 4.0,
        (bounds.x + bounds.width - MARKING_LABEL_WIDTH - 4.0).max(bounds.x + 4.0),
    );
    let y = (cy - MARKING_LABEL_HEIGHT * 0.5).clamp(
        bounds.y + 4.0,
        (bounds.y + bounds.height - MARKING_LABEL_HEIGHT - 4.0).max(bounds.y + 4.0),
    );
    RectPx {
        x,
        y,
        width: MARKING_LABEL_WIDTH,
        height: MARKING_LABEL_HEIGHT,
    }
}

pub(super) fn render_marking_menu(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    overlay_quads: &mut Vec<Quad>,
    overlay_text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let Some(preview) = state.ui.marking_menu.as_ref() else {
        return;
    };
    // The marking menu composites into the menu-overlay streams (painted above the
    // PCB), so these sinks are the overlay quad/text buffers, not panel buffers.
    let starts = (
        overlay_quads.len(),
        overlay_text_runs.len(),
        hit_regions.len(),
    );
    render_preview(
        state,
        preview,
        layout,
        overlay_quads,
        overlay_text_runs,
        hit_regions,
    );
    crate::hit_clipping::clip_content(
        overlay_quads,
        overlay_text_runs,
        hit_regions,
        starts.0,
        starts.1,
        starts.2,
        layout.viewport,
    );
}

fn render_preview(
    state: &ReviewWorkspaceState,
    preview: &MarkingMenuState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let model = match marking_menu_model() {
        Ok(model) => model,
        Err(err) => {
            draw_marking_error(preview, layout, panel_quads, text_runs, err);
            return;
        }
    };
    if preview.selection.as_ref().is_some_and(|m| m.open) {
        selection_list::render(preview, layout, panel_quads, text_runs, hit_regions);
        return;
    }
    let schematic = GuiMarkingMenu::default();
    let Some(menu) = model
        .marking_menus
        .get(&preview.menu_key)
        .or_else(|| (preview.menu_key == "schematic.selection").then_some(&schematic))
    else {
        draw_marking_error(
            preview,
            layout,
            panel_quads,
            text_runs,
            &format!("missing marking menu {}", preview.menu_key),
        );
        return;
    };
    let icon_set = marking_icon_set().ok();
    let anchor = clamp_anchor(preview, layout.viewport);
    draw_marking_trail(preview, anchor, panel_quads);
    // Nested sub-wheel: when a submenu is open, render that submenu's items as a
    // content-sized radial instead of the top-level wheel. Navigation-only; the
    // activation handler pushes/pops `submenu_path`.
    if let Some(sub_key) = preview.submenu_path.last() {
        match menu.submenus.get(sub_key) {
            Some(items) => sub_wheel::render(
                preview,
                items,
                icon_set,
                anchor,
                layout.viewport,
                panel_quads,
                text_runs,
                hit_regions,
            ),
            None => {
                draw_marking_error(
                    preview,
                    layout,
                    panel_quads,
                    text_runs,
                    &format!("missing sub-wheel {sub_key}"),
                );
                return;
            }
        }
    } else {
        draw_marking_center(preview, anchor, panel_quads, text_runs, hit_regions);
        render_slots(
            state,
            preview,
            menu,
            icon_set,
            anchor,
            layout.viewport,
            panel_quads,
            text_runs,
            hit_regions,
        );
    }
    // Truthful disabled-leaf reason (MM-C03): the focused leaf's reason is shown in
    // the ordinary shell so the designer sees WHY it is unavailable — not only in
    // the accessibility model. Submenu parents navigate and carry no reason.
    draw_marking_reason(
        state,
        menu,
        preview,
        anchor,
        layout.viewport,
        panel_quads,
        text_runs,
    );
}

// The same admission used by dispatch explains unavailable schematic leaves.
// Existing board leaves retain the inert shell; submenu parents only navigate.
fn draw_marking_reason(
    state: &ReviewWorkspaceState,
    menu: &GuiMarkingMenu,
    preview: &MarkingMenuState,
    anchor: (f32, f32),
    bounds: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let Some(slot) = preview.preview_slot.as_deref() else {
        return;
    };
    let item = if let Some(sub_key) = preview.submenu_path.last() {
        let Some(index) = slot
            .strip_prefix("SUB:")
            .and_then(|s| s.parse::<usize>().ok())
        else {
            return;
        };
        match menu
            .submenus
            .get(sub_key)
            .and_then(|items| items.get(index))
        {
            Some(item) => item,
            None => return,
        }
    } else {
        match menu.cardinal.get(slot).or_else(|| menu.secondary.get(slot)) {
            Some(item) => item,
            None => return,
        }
    };
    if matches!(
        item.binding(),
        datum_gui_protocol::GuiMenuBinding::Submenu(_)
    ) {
        return;
    }
    let reason = if preview.menu_key.starts_with("schematic.") {
        let Some(reason) = preview.schematic_action_unavailable_reason(item, state) else {
            return;
        };
        reason
    } else {
        item.unavailable_reason(state)
            .unwrap_or("Inert shell — this action is not invoked in this build.")
            .to_owned()
    };
    let width = 280.0_f32.min((bounds.width - 8.0).max(0.0));
    let rect = RectPx {
        x: (anchor.0 - width * 0.5).clamp(
            bounds.x + 4.0,
            (bounds.x + bounds.width - width - 4.0).max(bounds.x + 4.0),
        ),
        y: anchor.1 + MARKING_RADIUS + 54.0,
        width,
        height: 20.0,
    };
    panel_quads.push(Quad::from_rect(rect, PANEL_CARD_BG));
    push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);
    draw_text(
        &truncate_text(&reason, 44),
        rect.x + design_tokens::spacing::SP_03,
        rect.y + design_tokens::spacing::SP_03,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_MUTED,
        TextFace::Ui,
        text_runs,
    );
}

// Render helper threads many quad/text-run/hit-region sinks.
#[allow(clippy::too_many_arguments)]
fn render_slots(
    state: &ReviewWorkspaceState,
    preview: &MarkingMenuState,
    menu: &GuiMarkingMenu,
    icon_set: Option<&GuiIconSet>,
    anchor: (f32, f32),
    bounds: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    for slot in ["N", "E", "S", "W", "NE", "SE", "SW", "NW"] {
        let item = menu.cardinal.get(slot).or_else(|| menu.secondary.get(slot));
        let Some(item) = item else {
            continue;
        };
        let Some(angle) = slot_angle(slot) else {
            continue;
        };
        let active = preview.preview_slot.as_deref() == Some(slot);
        // Existing typed schematic leaves share admission with invocation.
        let navigable = matches!(
            item.binding(),
            datum_gui_protocol::GuiMenuBinding::Submenu(_)
        );
        let enabled = navigable
            || preview
                .schematic_action_unavailable_reason(item, state)
                .is_none();
        // Radial-wedge form (ratified §6 surface): a pie sector per slot (active =
        // accent fill), the icon+label at the label radius, and a label-area hit rect
        // for clicks (the flick gesture resolves by angle separately).
        let fill = if active {
            REVIEW_ROW_ACTIVE_BG
        } else {
            PANEL_CARD_BG
        };
        push_sector(
            panel_quads,
            anchor,
            angle,
            MARKING_SECTOR_SPAN,
            MARKING_INNER,
            MARKING_RADIUS,
            fill,
        );
        let rect = slot_label_rect(anchor, angle, bounds);
        render_marking_icon(item, icon_set, rect, enabled && active, text_runs);
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
            if enabled { TEXT_PRIMARY } else { TEXT_MUTED },
            TextFace::Ui,
            text_runs,
        );
        hit_regions.push(HitRegion {
            target: HitTarget::MarkingMenuItem {
                menu_key: preview.menu_key.clone(),
                slot: slot.to_string(),
                label: item.label.clone(),
            },
            rect,
        });
    }
    if preview.selection.is_some() {
        selection_list::entry(
            preview,
            slot_rect(anchor, "MORE", bounds),
            panel_quads,
            text_runs,
            hit_regions,
        );
    } else if !menu.overflow.is_empty() {
        let rect = slot_rect(anchor, "MORE", bounds);
        panel_quads.push(Quad::from_rect(rect, PANEL_BG));
        push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);
        draw_text(
            "MORE...",
            rect.x + design_tokens::spacing::SP_03,
            rect.y + design_tokens::spacing::SP_03,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_MUTED,
            TextFace::Ui,
            text_runs,
        );
    }
}

fn draw_marking_trail(preview: &MarkingMenuState, anchor: (f32, f32), panel_quads: &mut Vec<Quad>) {
    if preview.gesture_dx_px == 0 && preview.gesture_dy_px == 0 {
        return;
    }
    let dx = preview.gesture_dx_px as f32;
    let dy = preview.gesture_dy_px as f32;
    let length = (dx * dx + dy * dy).sqrt().max(1.0);
    let ux = dx / length;
    let uy = dy / length;
    let normal_x = -uy * 2.0;
    let normal_y = ux * 2.0;
    let end = (
        anchor.0 + dx.clamp(-MARKING_RADIUS, MARKING_RADIUS),
        anchor.1 + dy.clamp(-MARKING_RADIUS, MARKING_RADIUS),
    );
    panel_quads.push(Quad {
        points: [
            (anchor.0 + normal_x, anchor.1 + normal_y),
            (end.0 + normal_x, end.1 + normal_y),
            (end.0 - normal_x, end.1 - normal_y),
            (anchor.0 - normal_x, anchor.1 - normal_y),
        ],
        color: TEXT_ACCENT,
    });
}

fn draw_marking_center(
    preview: &MarkingMenuState,
    anchor: (f32, f32),
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let rect = RectPx {
        x: anchor.0 - MARKING_CENTER_SIZE * 0.5,
        y: anchor.1 - MARKING_CENTER_SIZE * 0.5,
        width: MARKING_CENTER_SIZE,
        height: MARKING_CENTER_SIZE,
    };
    if preview.selection.is_some() {
        hit_regions.push(HitRegion {
            rect,
            target: HitTarget::MarkingMenuSurface {
                menu_key: preview.menu_key.clone(),
                anchor_x_px: preview.anchor_x_px,
                anchor_y_px: preview.anchor_y_px,
            },
        });
    }
    panel_quads.push(Quad::from_rect(rect, PANEL_CARD_BG));
    push_rect_border(panel_quads, rect, TEXT_ACCENT, 1.0);
    draw_text(
        "MM",
        rect.x + 3.0,
        rect.y + 7.0,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    draw_text(
        if preview.menu_key == "schematic.selection" {
            "Schematic"
        } else {
            &preview.menu_key
        },
        anchor.0 - 54.0,
        anchor.1 + MARKING_RADIUS + 46.0,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
}

fn draw_marking_error(
    preview: &MarkingMenuState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    error: &str,
) {
    let anchor = clamp_anchor(preview, layout.viewport);
    let rect = RectPx {
        x: anchor.0 - 120.0,
        y: anchor.1 - 16.0,
        width: 240.0,
        height: 32.0,
    };
    panel_quads.push(Quad::from_rect(rect, PANEL_CARD_BG));
    push_rect_border(panel_quads, rect, TEXT_ACCENT, 1.0);
    draw_text(
        &truncate_text(error, 28),
        rect.x + design_tokens::spacing::SP_03,
        rect.y + design_tokens::spacing::SP_03,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
}

fn render_marking_icon(
    item: &GuiMenuItem,
    icon_set: Option<&GuiIconSet>,
    rect: RectPx,
    active: bool,
    text_runs: &mut Vec<TextRun>,
) {
    let glyph = item
        .icon
        .as_deref()
        .and_then(|icon_id| {
            icon_set
                .and_then(|set| set.icons.get(icon_id))
                .and_then(|icon| icon.glyph.as_deref().or(icon.asset.as_deref()))
                .or(Some(icon_id))
        })
        .and_then(|label| label.chars().next())
        .unwrap_or('?')
        .to_ascii_uppercase()
        .to_string();
    draw_text(
        &glyph,
        rect.x + design_tokens::spacing::SP_03,
        rect.y + design_tokens::spacing::SP_03,
        design_tokens::typography::CAPTION_SIZE,
        if active { TEXT_ACCENT } else { TEXT_MUTED },
        TextFace::Mono,
        text_runs,
    );
}

fn slot_rect(anchor: (f32, f32), slot: &str, bounds: RectPx) -> RectPx {
    let (dx, dy) = match slot {
        "N" => (0.0, -MARKING_RADIUS),
        "NE" => (MARKING_RADIUS * 0.75, -MARKING_RADIUS * 0.75),
        "E" => (MARKING_RADIUS, 0.0),
        "SE" => (MARKING_RADIUS * 0.75, MARKING_RADIUS * 0.75),
        "S" => (0.0, MARKING_RADIUS),
        "SW" => (-MARKING_RADIUS * 0.75, MARKING_RADIUS * 0.75),
        "W" => (-MARKING_RADIUS, 0.0),
        "NW" => (-MARKING_RADIUS * 0.75, -MARKING_RADIUS * 0.75),
        "MORE" => (0.0, MARKING_RADIUS + MARKING_LABEL_HEIGHT + 8.0),
        _ => (0.0, 0.0),
    };
    let x = (anchor.0 + dx - MARKING_LABEL_WIDTH * 0.5).clamp(
        bounds.x + 4.0,
        (bounds.x + bounds.width - MARKING_LABEL_WIDTH - 4.0).max(bounds.x + 4.0),
    );
    let y = (anchor.1 + dy - MARKING_LABEL_HEIGHT * 0.5).clamp(
        bounds.y + 4.0,
        (bounds.y + bounds.height - MARKING_LABEL_HEIGHT - 4.0).max(bounds.y + 4.0),
    );
    RectPx {
        x,
        y,
        width: MARKING_LABEL_WIDTH,
        height: MARKING_LABEL_HEIGHT,
    }
}

fn clamp_anchor(preview: &MarkingMenuState, bounds: RectPx) -> (f32, f32) {
    let margin_x = MARKING_RADIUS.min(bounds.width.max(0.0) * 0.5);
    let margin_y = MARKING_RADIUS.min(bounds.height.max(0.0) * 0.5);
    (
        (preview.anchor_x_px as f32).clamp(
            bounds.x + margin_x,
            bounds.x + bounds.width.max(0.0) - margin_x,
        ),
        (preview.anchor_y_px as f32).clamp(
            bounds.y + margin_y,
            bounds.y + bounds.height.max(0.0) - margin_y,
        ),
    )
}

fn marking_menu_model() -> Result<&'static datum_gui_protocol::GuiMenuModel, &'static str> {
    MARKING_MENU_MODEL
        .get_or_init(|| load_default_gui_menu_model().map_err(|err| err.to_string()))
        .as_ref()
        .map_err(String::as_str)
}

fn marking_icon_set() -> Result<&'static GuiIconSet, &'static str> {
    MARKING_ICON_SET
        .get_or_init(|| load_default_gui_icon_set().map_err(|err| err.to_string()))
        .as_ref()
        .map_err(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focused_disabled_leaf_shows_truthful_reason_but_submenu_parent_does_not() {
        let base = datum_gui_protocol::load_fixture_workspace_state();
        let mut layout = ShellLayout::for_window(1280, 800, None);
        layout.viewport = RectPx {
            x: 0.0,
            y: 0.0,
            width: 1280.0,
            height: 800.0,
        };
        let render = |slot: &str| -> Vec<TextRun> {
            let mut state = base.clone();
            state.ui.marking_menu = Some(MarkingMenuState {
                menu_key: "pcb.multiselect".into(),
                target_object_id: None,
                anchor_x_px: 640,
                anchor_y_px: 360,
                preview_slot: Some(slot.into()),
                gesture_dx_px: 0,
                gesture_dy_px: 0,
                selection: None,
                submenu_path: Vec::new(),
            });
            let mut q = Vec::new();
            let mut t = Vec::new();
            let mut h = Vec::new();
            render_marking_menu(&state, &layout, &mut q, &mut t, &mut h);
            t
        };
        let has_reason = |runs: &[TextRun]| {
            runs.iter().any(|run| {
                let lower = run.text.to_lowercase();
                lower.contains("unavailable") || lower.contains("inert")
            })
        };
        // N = Rotate (a disabled verb leaf) → the designer sees a truthful reason in
        // the ordinary shell, not only in the accessibility model.
        assert!(
            has_reason(&render("N")),
            "a focused disabled leaf must show a truthful reason"
        );
        // W = Align ▸ (a submenu parent) navigates → no disabled-reason caption.
        assert!(
            !has_reason(&render("W")),
            "a submenu parent navigates and shows no disabled reason"
        );
    }

    #[test]
    fn constrained_marking_preview_and_error_share_visible_bounds() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let mut layout = ShellLayout::for_window(1280, 800, None);
        for (width, height) in [(100.0, 70.0), (20.0, 20.0), (0.0, 0.0)] {
            layout.viewport = RectPx {
                x: 40.0,
                y: 50.0,
                width,
                height,
            };
            for key in ["pcb.component", "missing-preview"] {
                state.ui.marking_menu = Some(MarkingMenuState {
                    menu_key: key.into(),
                    target_object_id: None,
                    anchor_x_px: -500,
                    anchor_y_px: 1000,
                    preview_slot: Some("N".into()),
                    gesture_dx_px: 90,
                    gesture_dy_px: -72,
                    selection: None,
                    submenu_path: Vec::new(),
                });
                let mut quads = Vec::new();
                let mut text = Vec::new();
                let mut hits = Vec::new();
                render_marking_menu(&state, &layout, &mut quads, &mut text, &mut hits);
                for quad in quads {
                    assert!(
                        quad.points
                            .iter()
                            .all(|&(x, y)| layout.viewport.contains(x, y))
                    );
                }
                for run in text {
                    let clip = run.clip_bounds.unwrap();
                    assert!(layout.viewport.contains(clip.x, clip.y));
                    assert!(
                        layout
                            .viewport
                            .contains(clip.x + clip.width, clip.y + clip.height)
                    );
                }
                if key == "pcb.component" && width > 0.0 {
                    assert!(!hits.is_empty());
                }
                if width == 0.0 {
                    assert!(hits.is_empty());
                }
                for hit in hits {
                    assert!(layout.viewport.contains(hit.rect.x, hit.rect.y));
                    assert!(
                        layout
                            .viewport
                            .contains(hit.rect.x + hit.rect.width, hit.rect.y + hit.rect.height)
                    );
                }
            }
        }
    }
}
