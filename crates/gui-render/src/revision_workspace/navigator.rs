use super::*;
use datum_gui_protocol::{RevisionNavContextAction, RevisionNavEntry};

const NAV_ROWS: [(&str, RevisionNavEntry); 5] = [
    ("Changes  1 draft", RevisionNavEntry::Changes),
    ("Baselines  BL-…-24-01", RevisionNavEntry::Baselines),
    ("Releases  RLS-0007", RevisionNavEntry::Releases),
    (
        "Controlled Documents  1",
        RevisionNavEntry::ControlledDocuments,
    ),
    ("Evidence  12 records", RevisionNavEntry::Evidence),
];

const CONTEXT_ACTIONS: [(&str, RevisionNavContextAction); 5] = [
    ("Open Beside", RevisionNavContextAction::OpenBeside),
    ("Properties", RevisionNavContextAction::Properties),
    ("Show Impact", RevisionNavContextAction::ShowImpact),
    (
        "Prepare Revision",
        RevisionNavContextAction::PrepareRevision,
    ),
    (
        "Compare to Baseline",
        RevisionNavContextAction::CompareToBaseline,
    ),
];

pub(crate) fn render_navigator_inspector(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    text: &mut Vec<TextRun>,
) -> bool {
    let Some(entry) = state.ui.revision.selected_entry else {
        return false;
    };
    draw_text(
        entry.label(),
        rect.x + 12.0,
        rect.y + 46.0,
        16.0,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    draw_text(
        "Navigator selection",
        rect.x + 12.0,
        rect.y + 69.0,
        10.0,
        TEXT_MUTED,
        TextFace::Mono,
        text,
    );
    for (index, line) in [
        "Single click selects only",
        "Enter or double-click opens beside",
        "Right-click shows local actions",
    ]
    .into_iter()
    .enumerate()
    {
        draw_text_clipped(
            line,
            rect.x + 12.0,
            rect.y + 101.0 + index as f32 * 24.0,
            11.0,
            TEXT_SECONDARY,
            TextFace::Ui,
            RectPx {
                x: rect.x + 8.0,
                y: rect.y + 88.0,
                width: (rect.width - 16.0).max(1.0),
                height: 96.0,
            },
            text,
        );
    }
    true
}

pub(crate) fn render_navigator(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let x = rect.x + 12.0;
    let mut y = rect.y + 82.0;
    for (label, strong) in [
        ("DESIGN", true),
        ("  Schematic · sensor-node", false),
        ("  Board · main", false),
        ("PUBLISH", true),
        ("  All Sheets  5", false),
        ("REVISION", true),
    ] {
        draw_text(
            label,
            x,
            y,
            if strong { 10.0 } else { 11.0 },
            if strong { TEXT_MUTED } else { TEXT_SECONDARY },
            if strong {
                TextFace::UiStrong
            } else {
                TextFace::Ui
            },
            text,
        );
        y += if strong { 19.0 } else { 20.0 };
    }
    for (label, entry) in NAV_ROWS {
        let row = RectPx {
            x,
            y: y - 5.0,
            width: (rect.width - 24.0).max(1.0),
            height: 21.0,
        };
        if state.ui.revision.selected_entry == Some(entry) {
            quads.push(Quad::from_rect(row, REVIEW_ROW_BADGE));
            quads.push(Quad::from_rect(
                RectPx {
                    x: row.x,
                    y: row.y,
                    width: 2.0,
                    height: row.height,
                },
                TEXT_ACCENT,
            ));
        }
        draw_text(label, x + 8.0, y, 10.5, TEXT_SECONDARY, TextFace::Ui, text);
        if matches!(
            entry,
            RevisionNavEntry::Baselines | RevisionNavEntry::Releases
        ) {
            draw_lock_glyph(
                row.x + row.width - 13.0,
                row.y + 5.0,
                7.0,
                TEXT_SECONDARY,
                quads,
            );
        }
        hits.push(HitRegion {
            target: HitTarget::SelectRevisionNavEntry(entry),
            rect: row,
        });
        y += 24.0;
    }
}

pub(crate) fn render_navigator_context_menu(
    state: &ReviewWorkspaceState,
    window: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let Some(menu) = state.ui.revision.context_menu else {
        return;
    };
    let width = 190.0;
    let height = 28.0 + CONTEXT_ACTIONS.len() as f32 * 28.0;
    let x = (menu.anchor_x_px as f32).clamp(window.x, window.x + window.width - width);
    let y = (menu.anchor_y_px as f32).clamp(window.y, window.y + window.height - height);
    let card = RectPx {
        x,
        y,
        width,
        height,
    };
    quads.push(Quad::from_rect(card, PANEL_CARD_BG));
    push_rect_border(quads, card, PANEL_CARD_BORDER, 1.0);
    draw_text_clipped(
        menu.entry.label(),
        x + 10.0,
        y + 7.0,
        10.0,
        TEXT_MUTED,
        TextFace::UiStrong,
        card,
        text,
    );
    for (index, (label, action)) in CONTEXT_ACTIONS.into_iter().enumerate() {
        let row = RectPx {
            x: x + 4.0,
            y: y + 28.0 + index as f32 * 28.0,
            width: width - 8.0,
            height: 27.0,
        };
        draw_text_clipped(
            label,
            row.x + 8.0,
            row.y + 6.0,
            11.0,
            TEXT_PRIMARY,
            TextFace::Ui,
            row,
            text,
        );
        hits.push(HitRegion {
            target: HitTarget::RevisionNavContextAction(action),
            rect: row,
        });
    }
}
