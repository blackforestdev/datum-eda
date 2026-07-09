use super::*;
use datum_gui_protocol::{
    GuiIconSet, GuiMarkingMenu, GuiMenuItem, MarkingMenuState, load_default_gui_icon_set,
    load_default_gui_menu_model,
};
use std::sync::OnceLock;

static MARKING_MENU_MODEL: OnceLock<Result<datum_gui_protocol::GuiMenuModel, String>> =
    OnceLock::new();
static MARKING_ICON_SET: OnceLock<Result<GuiIconSet, String>> = OnceLock::new();

const MARKING_RADIUS: f32 = 86.0;
const MARKING_LABEL_WIDTH: f32 = 118.0;
const MARKING_LABEL_HEIGHT: f32 = 30.0;
const MARKING_CENTER_SIZE: f32 = 22.0;

pub(super) fn render_marking_menu(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let Some(preview) = state.ui.marking_menu.as_ref() else {
        return;
    };
    let model = match marking_menu_model() {
        Ok(model) => model,
        Err(err) => {
            draw_marking_error(preview, layout, panel_quads, text_runs, err);
            return;
        }
    };
    let Some(menu) = model.marking_menus.get(&preview.menu_key) else {
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
    draw_marking_center(preview, anchor, panel_quads, text_runs);
    render_slots(
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

fn render_slots(
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
        let rect = slot_rect(anchor, slot, bounds);
        let active = preview.preview_slot.as_deref() == Some(slot);
        let fill = if active {
            REVIEW_ROW_ACTIVE_BG
        } else {
            PANEL_CARD_BG
        };
        panel_quads.push(Quad::from_rect(rect, fill));
        push_rect_border(
            panel_quads,
            rect,
            if active {
                TEXT_ACCENT
            } else {
                PANEL_CARD_BORDER
            },
            1.0,
        );
        render_marking_icon(item, icon_set, rect, active, text_runs);
        draw_text(
            &truncate_text(&item.label, 18),
            rect.x + design_tokens::spacing::SP_07,
            rect.y + design_tokens::spacing::SP_03,
            design_tokens::typography::CAPTION_SIZE,
            if active { TEXT_PRIMARY } else { TEXT_MUTED },
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
    if !menu.overflow.is_empty() {
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
) {
    let rect = RectPx {
        x: anchor.0 - MARKING_CENTER_SIZE * 0.5,
        y: anchor.1 - MARKING_CENTER_SIZE * 0.5,
        width: MARKING_CENTER_SIZE,
        height: MARKING_CENTER_SIZE,
    };
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
        &preview.menu_key,
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
        bounds.x + bounds.width - MARKING_LABEL_WIDTH - 4.0,
    );
    let y = (anchor.1 + dy - MARKING_LABEL_HEIGHT * 0.5).clamp(
        bounds.y + 4.0,
        bounds.y + bounds.height - MARKING_LABEL_HEIGHT - 4.0,
    );
    RectPx {
        x,
        y,
        width: MARKING_LABEL_WIDTH,
        height: MARKING_LABEL_HEIGHT,
    }
}

fn clamp_anchor(preview: &MarkingMenuState, bounds: RectPx) -> (f32, f32) {
    (
        (preview.anchor_x_px as f32).clamp(
            bounds.x + MARKING_RADIUS,
            bounds.x + bounds.width - MARKING_RADIUS,
        ),
        (preview.anchor_y_px as f32).clamp(
            bounds.y + MARKING_RADIUS,
            bounds.y + bounds.height - MARKING_RADIUS,
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
