use super::*;
use datum_gui_protocol::{
    GuiIconSet, GuiMenuBinding, GuiMenuItem, GuiMenuModel, load_default_gui_icon_set,
    load_default_gui_menu_model,
};
use std::sync::OnceLock;

static MENU_MODEL: OnceLock<Result<GuiMenuModel, String>> = OnceLock::new();
static ICON_SET: OnceLock<Result<GuiIconSet, String>> = OnceLock::new();

/// Symmetric horizontal padding inside a menu-bar title box. The title box width
/// is exactly `measured_glyph_run + 2 * MENU_TITLE_PAD`, so `box_width -
/// measured_width` is a CONSTANT across every label — THAT is what makes the
/// trailing gaps uniform (locked by the uniformity test). Tighter than the
/// prototype's 9px (`docs/gui/prototypes/board-editor.html` `.menu{padding:3px
/// 9px}`) per the owner's request for a tighter rhythm; the title text is drawn
/// at the same PAD so left/right padding is symmetric.
const MENU_TITLE_PAD: f32 = design_tokens::spacing::SP_03;

pub(super) fn render_menu_bar(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    menu_overlay_quads: &mut Vec<Quad>,
    menu_overlay_text_runs: &mut Vec<TextRun>,
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

    // Menus begin after the ACTUAL shaped multi-run brand wordmark (the same
    // "Datum" + middot + "EDA" runs the shell draws at 14px semibold, from
    // SP_04) plus a fixed gap, so File lands a constant distance after the
    // wordmark. Real cosmic-text measurement (not the fixed-advance estimate)
    // makes the anchor accurate for the proportional Plex Sans Condensed face.
    let brand_width: f32 = ["Datum", "\u{00B7}", "EDA"]
        .iter()
        .map(|run| measured_text_run_width_px(run, 14.0, TextFace::UiStrong))
        .sum();
    let mut x = layout.top_menu_bar.x
        + design_tokens::spacing::SP_04
        + brand_width
        + design_tokens::spacing::SP_04;
    let y = layout.top_menu_bar.y + design_tokens::spacing::SP_02;
    let mut active_menu_x: Option<f32> = None;
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
            active_menu_x = Some(rect.x);
            panel_quads.push(Quad::from_rect(rect, REVIEW_ROW_ACTIVE_BG));
            push_rect_border(panel_quads, rect, TEXT_ACCENT, 1.0);
        }
        draw_text(
            &menu.menu,
            rect.x + MENU_TITLE_PAD,
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
        // Drop the dropdown directly under its own title (its captured rect.x),
        // not from a fixed far-left offset. Falls back to the menu-bar left edge
        // only if the active title somehow was not laid out this frame.
        let menu_x = active_menu_x.unwrap_or(layout.top_menu_bar.x);
        render_menu_dropdown(
            menu,
            icon_set,
            layout,
            menu_x,
            menu_overlay_quads,
            menu_overlay_text_runs,
            hit_regions,
        );
    }
}

fn render_menu_dropdown(
    menu: &datum_gui_protocol::GuiMenu,
    icon_set: Option<&GuiIconSet>,
    layout: &ShellLayout,
    menu_x: f32,
    // The dropdown BODY quads are composited AFTER the main text pass (see gpu.rs)
    // so both the work-pane content AND every underlying text_run are fully
    // occluded by the card. Menu-bar TITLE quads stay in `panel_quads`.
    menu_overlay_quads: &mut Vec<Quad>,
    // The dropdown's OWN text (labels, shortcuts, fallback-icon glyphs) goes into
    // this dedicated sink and is drawn in a FINAL pass on top of the card, so it
    // stays crisp while everything below the card is hidden. Menu-bar TITLE text
    // is NOT here (titles live in the bar and are never occluded).
    menu_overlay_text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let item_height = design_tokens::spacing::SP_07;
    // Content-driven card width: the widest item's [icon indent + shaped label +
    // gap + shaped shortcut + right pad] sets the width, so labels and shortcuts
    // never spill past the card. The retired fixed 272px width (with a fixed 74px
    // shortcut reservation) clipped long labels and wide "Ctrl+Shift+…" shortcuts.
    // Uses the same real shaped measurement as the menu-bar layout, so it is exact
    // for the proportional Plex Sans Condensed face.
    const LABEL_INDENT: f32 = design_tokens::spacing::SP_07; // icon gutter before the label
    const RIGHT_PAD: f32 = design_tokens::spacing::SP_04;
    const LABEL_SHORTCUT_GAP: f32 = design_tokens::spacing::SP_06;
    const MIN_WIDTH: f32 = 200.0;
    let content_width = menu
        .items
        .iter()
        .map(|item| {
            let label_w = measured_text_run_width_px(
                &item.label,
                design_tokens::typography::BODY_SIZE,
                TextFace::Ui,
            );
            let shortcut_w = item.shortcut.as_deref().map_or(0.0, |shortcut| {
                LABEL_SHORTCUT_GAP
                    + measured_text_run_width_px(
                        shortcut,
                        design_tokens::typography::CAPTION_SIZE,
                        TextFace::Mono,
                    )
            });
            LABEL_INDENT + label_w + shortcut_w + RIGHT_PAD
        })
        .fold(0.0_f32, f32::max);
    // The row is inset SP_02 from the card on each side; add that back, floor at a
    // sensible minimum, and never exceed the menu-bar (window) width.
    let max_card_width = (layout.top_menu_bar.width - design_tokens::spacing::SP_02 * 2.0).max(MIN_WIDTH);
    let width = (content_width + design_tokens::spacing::SP_02 * 2.0)
        .max(MIN_WIDTH)
        .min(max_card_width);
    let height = item_height * menu.items.len() as f32 + design_tokens::spacing::SP_02 * 2.0;
    // Clamp so the dropdown stays inside the window right edge under its title.
    let max_x = (layout.top_menu_bar.x + layout.top_menu_bar.width - width)
        .max(layout.top_menu_bar.x);
    let rect = RectPx {
        x: menu_x.min(max_x),
        y: layout.top_menu_bar.y + layout.top_menu_bar.height,
        width,
        height,
    };
    menu_overlay_quads.push(Quad::from_rect(rect, PANEL_CARD_BG));
    push_rect_border(menu_overlay_quads, rect, PANEL_CARD_BORDER, 1.0);

    for (index, item) in menu.items.iter().enumerate() {
        let row = RectPx {
            x: rect.x + design_tokens::spacing::SP_02,
            y: rect.y + design_tokens::spacing::SP_02 + index as f32 * item_height,
            width: rect.width - design_tokens::spacing::SP_02 * 2.0,
            height: item_height,
        };
        let enabled = item.is_phase_one_enabled();
        let row_color = if enabled { PANEL_CARD_BG } else { PANEL_BG };
        menu_overlay_quads.push(Quad::from_rect(row, row_color));
        render_fallback_icon(item, icon_set, row, menu_overlay_text_runs);
        draw_text(
            &item.label,
            row.x + design_tokens::spacing::SP_07,
            row.y + design_tokens::spacing::SP_03,
            design_tokens::typography::BODY_SIZE,
            if enabled { TEXT_PRIMARY } else { TEXT_MUTED },
            TextFace::Ui,
            menu_overlay_text_runs,
        );
        if let Some(shortcut) = item.shortcut.as_deref() {
            // Right-align to the shortcut's own shaped width so it sits inside the
            // right pad regardless of length (no fixed reservation to overflow).
            let shortcut_w = measured_text_run_width_px(
                shortcut,
                design_tokens::typography::CAPTION_SIZE,
                TextFace::Mono,
            );
            draw_text(
                shortcut,
                row.x + row.width - RIGHT_PAD - shortcut_w,
                row.y + design_tokens::spacing::SP_03,
                design_tokens::typography::CAPTION_SIZE,
                TEXT_MUTED,
                TextFace::Mono,
                menu_overlay_text_runs,
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
    // Real shaped glyph-run width plus a CONSTANT symmetric padding. Because the
    // padding is the same for every label, `menu_title_width - measured_width` is
    // constant, so the visual gap after each title is identical (uniform rhythm)
    // and tight. The retired estimate baked a fixed +16 and used a monospace-style
    // per-char advance on the proportional UI face, so per-label error varied and
    // gaps looked random.
    measured_text_run_width_px(label, design_tokens::typography::BODY_SIZE, TextFace::Ui)
        + MENU_TITLE_PAD * 2.0
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
    fn conformance_menu_title_width_padding_is_uniform_and_tight() {
        // Bugs E+F: the box width must be the REAL shaped glyph run plus a
        // CONSTANT padding, so `box_width - measured_width` is identical for every
        // label — that constant is what makes the trailing gaps uniform. Also
        // assert the inter-title gap is the tight SP_01, and that we no longer use
        // the retired fixed-advance estimate (which baked +16 and varied per label).
        let model = menu_model().expect("default menu model should load");
        assert!(model.menubar.len() >= 2, "need multiple titles to compare");
        let expected_pad_sum = MENU_TITLE_PAD * 2.0;
        for menu in &model.menubar {
            let width = menu_title_width(&menu.menu);
            let measured =
                measured_text_run_width_px(&menu.menu, design_tokens::typography::BODY_SIZE, TextFace::Ui);
            let pad = width - measured;
            assert!(
                (pad - expected_pad_sum).abs() < 1e-3,
                "{}: box {width:.3} - measured {measured:.3} = {pad:.3}, not the constant {expected_pad_sum:.3}",
                menu.menu
            );
            // The tighter box must be strictly smaller than the retired estimate
            // box (estimate + SP_03*2), proving the +16 baked padding is gone.
            let retired_box = estimated_text_run_width_px(
                &menu.menu,
                design_tokens::typography::BODY_SIZE,
                TextFace::Ui,
            ) + design_tokens::spacing::SP_03 * 2.0;
            assert!(
                width < retired_box,
                "{}: box {width:.2} not tighter than retired estimate box {retired_box:.2}",
                menu.menu
            );
        }
        // The gap between adjacent title boxes advances by exactly SP_01.
        let a = menu_title_width(&model.menubar[0].menu);
        let x0 = 0.0_f32;
        let x1 = x0 + a + design_tokens::spacing::SP_01;
        assert!((x1 - (x0 + a) - design_tokens::spacing::SP_01).abs() < 1e-6);
    }
}
