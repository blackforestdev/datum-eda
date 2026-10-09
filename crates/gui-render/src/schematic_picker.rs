//! Native symbol selection and gesture controls using existing chrome tokens.
use crate::*;
use datum_gui_protocol::{ReviewWorkspaceState, schematic_authoring::PlacementField};
pub(super) fn render(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let authoring = &state.schematic_authoring;
    if authoring.picker {
        let width = 480f32.min(layout.top_menu_bar.width - 32.0);
        let panel = RectPx {
            x: (layout.top_menu_bar.width - width) / 2.,
            y: 76.,
            width,
            height: 350.,
        };
        quads.push(Quad::from_rect(panel, design_tokens::chrome::SURFACE_01));
        push_rect_border(quads, panel, design_tokens::chrome::BORDER_STRONG, 1.);
        hits.push(HitRegion {
            target: HitTarget::SchematicPicker,
            rect: panel,
        });
        draw_text(
            "Place Symbol",
            panel.x + 16.,
            panel.y + 18.,
            16.,
            TEXT_PRIMARY,
            TextFace::UiStrong,
            text,
        );
        let mut y = panel.y + 50.;
        if authoring.choices.is_empty() {
            draw_text(
                "No native symbols in this Project's library",
                panel.x + 16.,
                y,
                12.,
                TEXT_MUTED,
                TextFace::Ui,
                text,
            );
        }
        for (index, choice) in authoring
            .choices
            .iter()
            .enumerate()
            .skip(authoring.choice_offset)
            .take(4)
        {
            let row = RectPx {
                x: panel.x + 12.,
                y,
                width: width - 24.,
                height: 29.,
            };
            quads.push(Quad::from_rect(
                row,
                if authoring.chosen == Some(index) {
                    design_tokens::chrome::SURFACE_02
                } else {
                    design_tokens::chrome::SURFACE_01
                },
            ));
            let label = format!(
                "{} · {} pins{}",
                choice.name,
                choice.pins.len(),
                if choice.refusal.is_some() {
                    " · unavailable"
                } else {
                    ""
                }
            );
            draw_text(
                &truncate_text(&label, 58),
                row.x + 6.,
                y + 7.,
                12.,
                TEXT_PRIMARY,
                TextFace::Ui,
                text,
            );
            hits.push(HitRegion {
                target: HitTarget::SchematicLibraryChoice(index),
                rect: row,
            });
            y += 31.;
        }
        if authoring.choices.len() > 4 {
            button(
                "Previous",
                HitTarget::SchematicLibraryPage(false),
                RectPx {
                    x: panel.x + 16.,
                    y: panel.y + 177.,
                    width: 90.,
                    height: 25.,
                },
                quads,
                text,
                hits,
            );
            button(
                "Next",
                HitTarget::SchematicLibraryPage(true),
                RectPx {
                    x: panel.x + width - 106.,
                    y: panel.y + 177.,
                    width: 90.,
                    height: 25.,
                },
                quads,
                text,
                hits,
            );
        }
        y = panel.y + 215.;
        for (label, value, field, target) in [
            (
                "Reference",
                &authoring.reference,
                PlacementField::Reference,
                HitTarget::SchematicReference,
            ),
            (
                "Value",
                &authoring.value,
                PlacementField::Value,
                HitTarget::SchematicValue,
            ),
        ] {
            draw_text(
                label,
                panel.x + 16.,
                y + 7.,
                12.,
                TEXT_MUTED,
                TextFace::Ui,
                text,
            );
            let row = RectPx {
                x: panel.x + 110.,
                y,
                width: width - 126.,
                height: 27.,
            };
            quads.push(Quad::from_rect(row, design_tokens::chrome::SURFACE_02));
            push_rect_border(
                quads,
                row,
                if authoring.field == field {
                    TEXT_ACCENT
                } else {
                    design_tokens::chrome::BORDER_SUBTLE
                },
                1.,
            );
            draw_text(
                &truncate_text(value, 46),
                row.x + 7.,
                y + 7.,
                12.,
                TEXT_PRIMARY,
                TextFace::Ui,
                text,
            );
            hits.push(HitRegion { target, rect: row });
            y += 35.;
        }
        if let Some(refusal) = &authoring.refusal {
            draw_text(
                &truncate_text(refusal, 65),
                panel.x + 16.,
                panel.y + 203.,
                11.,
                design_tokens::chrome::STATUS_ERROR,
                TextFace::Ui,
                text,
            );
        }
        button(
            "Cancel",
            HitTarget::SchematicCancel,
            RectPx {
                x: panel.x + 16.,
                y: panel.y + 305.,
                width: 90.,
                height: 29.,
            },
            quads,
            text,
            hits,
        );
        button(
            "Place",
            HitTarget::SchematicPlace,
            RectPx {
                x: panel.x + width - 106.,
                y: panel.y + 305.,
                width: 90.,
                height: 29.,
            },
            quads,
            text,
            hits,
        );
    } else if authoring.active() {
        let panel = RectPx {
            x: layout.top_menu_bar.width / 2. - 160.,
            y: 76.,
            width: 320.,
            height: 65.,
        };
        quads.push(Quad::from_rect(panel, design_tokens::chrome::SURFACE_01));
        draw_text(
            if authoring.wire {
                "Place Wire · click vertices"
            } else {
                "Place Symbol · click destination"
            },
            panel.x + 10.,
            panel.y + 10.,
            12.,
            TEXT_PRIMARY,
            TextFace::UiStrong,
            text,
        );
        button(
            "Cancel",
            HitTarget::SchematicCancel,
            RectPx {
                x: panel.x + 10.,
                y: panel.y + 30.,
                width: 90.,
                height: 26.,
            },
            quads,
            text,
            hits,
        );
        if authoring.wire {
            button(
                "Finish wire",
                HitTarget::SchematicFinishWire,
                RectPx {
                    x: panel.x + 115.,
                    y: panel.y + 30.,
                    width: 110.,
                    height: 26.,
                },
                quads,
                text,
                hits,
            );
        }
    }
}
fn button(
    label: &str,
    target: HitTarget,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    quads.push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_02));
    push_rect_border(quads, rect, design_tokens::chrome::BORDER_STRONG, 1.);
    draw_text(
        label,
        rect.x + 10.,
        rect.y + 7.,
        12.,
        TEXT_PRIMARY,
        TextFace::Ui,
        text,
    );
    hits.push(HitRegion { target, rect });
}
