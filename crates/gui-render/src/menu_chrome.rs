use super::*;
use datum_gui_protocol::{
    GuiIconSet, GuiMenuBinding, GuiMenuItem, GuiMenuModel, load_default_gui_icon_set,
    load_default_gui_menu_model,
};
use std::sync::OnceLock;

static MENU_MODEL: OnceLock<Result<GuiMenuModel, String>> = OnceLock::new();
static ICON_SET: OnceLock<Result<GuiIconSet, String>> = OnceLock::new();

pub(super) fn render_menu_bar(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let model = match menu_model() {
        Ok(model) => model,
        Err(err) => {
            draw_text(
                &format!("Menu unavailable: {}", truncate_text(err, 48)),
                layout.top_menu_bar.x + 112.0,
                layout.top_menu_bar.y + design_tokens::spacing::SP_03,
                design_tokens::typography::DATA_SIZE,
                TEXT_MUTED,
                TextFace::Mono,
                text_runs,
            );
            return;
        }
    };
    let icon_set = icon_set().ok();

    // Menus begin after the measured brand wordmark (drawn at SP_04) plus a
    // gap, so the wordmark and first menu title never overlap regardless of
    // brand text or font metrics.
    let brand_width = estimated_text_run_width_px(
        "Datum EDA",
        design_tokens::typography::BODY_SIZE,
        TextFace::Ui,
    );
    let mut x = layout.top_menu_bar.x
        + design_tokens::spacing::SP_04
        + brand_width
        + design_tokens::spacing::SP_03;
    let y = layout.top_menu_bar.y + design_tokens::spacing::SP_02;
    for menu in &model.menubar {
        let width = menu_title_width(&menu.menu);
        let rect = RectPx {
            x,
            y,
            width,
            height: layout.top_menu_bar.height - design_tokens::spacing::SP_02 * 2.0,
        };
        let active = state.ui.active_menu.as_deref() == Some(menu.menu.as_str());
        if active {
            panel_quads.push(Quad::from_rect(rect, REVIEW_ROW_ACTIVE_BG));
            push_rect_border(panel_quads, rect, TEXT_ACCENT, 1.0);
        }
        draw_text(
            &menu.menu,
            rect.x + design_tokens::spacing::SP_03,
            rect.y + design_tokens::spacing::SP_02 + 1.0,
            design_tokens::typography::BODY_SIZE,
            if active { TEXT_PRIMARY } else { TEXT_SECONDARY },
            TextFace::Ui,
            text_runs,
        );
        hit_regions.push(HitRegion {
            target: HitTarget::MenuTitle(menu.menu.clone()),
            rect,
        });
        x += width + design_tokens::spacing::SP_01;
    }

    if let Some(active_menu) = state.ui.active_menu.as_deref()
        && let Some(menu) = model.menubar.iter().find(|menu| menu.menu == active_menu)
    {
        render_menu_dropdown(menu, icon_set, layout, panel_quads, text_runs, hit_regions);
    }
}

fn render_menu_dropdown(
    menu: &datum_gui_protocol::GuiMenu,
    icon_set: Option<&GuiIconSet>,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let item_height = design_tokens::spacing::SP_07;
    let width = 272.0;
    let height = item_height * menu.items.len() as f32 + design_tokens::spacing::SP_02 * 2.0;
    let rect = RectPx {
        x: layout.top_menu_bar.x + design_tokens::spacing::SP_09,
        y: layout.top_menu_bar.y + layout.top_menu_bar.height,
        width,
        height,
    };
    panel_quads.push(Quad::from_rect(rect, PANEL_CARD_BG));
    push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);

    for (index, item) in menu.items.iter().enumerate() {
        let row = RectPx {
            x: rect.x + design_tokens::spacing::SP_02,
            y: rect.y + design_tokens::spacing::SP_02 + index as f32 * item_height,
            width: rect.width - design_tokens::spacing::SP_02 * 2.0,
            height: item_height,
        };
        let enabled = item.is_phase_one_enabled();
        let row_color = if enabled { PANEL_CARD_BG } else { PANEL_BG };
        panel_quads.push(Quad::from_rect(row, row_color));
        render_fallback_icon(item, icon_set, row, text_runs);
        draw_text(
            &item.label,
            row.x + design_tokens::spacing::SP_07,
            row.y + design_tokens::spacing::SP_03,
            design_tokens::typography::BODY_SIZE,
            if enabled { TEXT_PRIMARY } else { TEXT_MUTED },
            TextFace::Ui,
            text_runs,
        );
        if let Some(shortcut) = item.shortcut.as_deref() {
            draw_text(
                shortcut,
                row.x + row.width - 74.0,
                row.y + design_tokens::spacing::SP_03,
                design_tokens::typography::CAPTION_SIZE,
                TEXT_MUTED,
                TextFace::Mono,
                text_runs,
            );
        }
        hit_regions.push(HitRegion {
            target: HitTarget::MenuItem {
                menu: menu.menu.clone(),
                label: item.label.clone(),
            },
            rect: row,
        });
    }
}

fn render_fallback_icon(
    item: &GuiMenuItem,
    icon_set: Option<&GuiIconSet>,
    row: RectPx,
    text_runs: &mut Vec<TextRun>,
) {
    let icon_rect = RectPx {
        x: row.x + design_tokens::spacing::SP_03,
        y: row.y + design_tokens::spacing::SP_03,
        width: design_tokens::spacing::SP_05,
        height: design_tokens::spacing::SP_05,
    };
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
    let color = match item.binding() {
        GuiMenuBinding::GuiLocal(_) => TEXT_SECONDARY,
        _ => TEXT_MUTED,
    };
    draw_text(
        &glyph,
        icon_rect.x,
        icon_rect.y,
        design_tokens::typography::CAPTION_SIZE,
        color,
        TextFace::Mono,
        text_runs,
    );
}

#[cfg(test)]
fn find_menu_item(menu_name: &str, label: &str) -> Option<GuiMenuItem> {
    menu_model()
        .ok()
        .and_then(|model| model.menubar.iter().find(|menu| menu.menu == menu_name))
        .and_then(|menu| menu.items.iter().find(|item| item.label == label))
        .cloned()
}

fn menu_title_width(label: &str) -> f32 {
    // Match the actual rendered advance (estimated_text_run_width_px already
    // includes end padding) plus symmetric item padding, so adjacent titles
    // never overlap for long labels like "Manufacturing".
    estimated_text_run_width_px(label, design_tokens::typography::BODY_SIZE, TextFace::Ui)
        + design_tokens::spacing::SP_02 * 2.0
}

fn menu_model() -> Result<&'static GuiMenuModel, &'static str> {
    MENU_MODEL
        .get_or_init(|| load_default_gui_menu_model().map_err(|err| err.to_string()))
        .as_ref()
        .map_err(String::as_str)
}

fn icon_set() -> Result<&'static GuiIconSet, &'static str> {
    ICON_SET
        .get_or_init(|| load_default_gui_icon_set().map_err(|err| err.to_string()))
        .as_ref()
        .map_err(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_model_is_available_to_renderer() {
        let model = menu_model().expect("default menu model should load");
        assert!(model.menubar.len() >= 3);
        assert!(find_menu_item("View", "Fit to Board").is_some());
    }

    #[test]
    fn conformance_menu_title_width_uses_condensed_measured_advance() {
        let model = menu_model().expect("default menu model should load");
        let mut x = 0.0_f32;
        for menu in &model.menubar {
            let width = menu_title_width(&menu.menu);
            let measured_run = estimated_text_run_width_px(
                &menu.menu,
                design_tokens::typography::BODY_SIZE,
                TextFace::Ui,
            );
            let upper_bound = measured_run + design_tokens::spacing::SP_02 * 2.0 + 0.5;
            let flat_advance_078 =
                menu.menu.chars().count() as f32 * design_tokens::typography::BODY_SIZE * 0.78
                    + 16.0
                    + design_tokens::spacing::SP_02 * 2.0;

            assert!(
                width <= upper_bound,
                "{} title box {width:.2} exceeds measured run bound {upper_bound:.2}",
                menu.menu
            );
            assert!(
                width + 0.5 < flat_advance_078,
                "{} title box still matches the retired 0.78 flat advance",
                menu.menu
            );
            let next_x = x + width + design_tokens::spacing::SP_01;
            assert!(next_x > x, "{} title has non-positive advance", menu.menu);
            x = next_x;
        }
    }
}
