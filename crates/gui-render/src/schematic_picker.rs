//! Native symbol selection and gesture controls using existing chrome tokens.
use crate::global_preferences_primitives::{ControlMeshCache, ControlPainter};
use crate::*;
#[path = "schematic_creation.rs"]
mod creation;
#[path = "schematic_label.rs"]
mod label;
use datum_gui_protocol::{ReviewWorkspaceState, schematic_authoring::PlacementField};
pub(super) fn render(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    if state.schematic_authoring.creation.is_some() {
        return creation::render(state, layout, quads, text, hits);
    }
    if state.schematic_authoring.label.is_some() {
        return label::render(state, layout, quads, text, hits);
    }
    let authoring = &state.schematic_authoring;
    if authoring.picker {
        let editing = authoring.editing.is_some();
        let width = 480f32.min(layout.top_menu_bar.width - 32.0);
        let panel = RectPx {
            x: (layout.top_menu_bar.width - width) / 2.,
            y: 76.,
            width,
            height: if editing { 235. } else { 350. },
        };
        quads.push(Quad::from_rect(panel, design_tokens::chrome::SURFACE_01));
        push_rect_border(quads, panel, design_tokens::chrome::BORDER_STRONG, 1.);
        hits.push(HitRegion {
            target: HitTarget::SchematicPicker,
            rect: panel,
        });
        draw_text(
            if editing {
                "Edit Symbol"
            } else {
                "Place Symbol"
            },
            panel.x + 16.,
            panel.y + 18.,
            16.,
            TEXT_PRIMARY,
            TextFace::UiStrong,
            text,
        );
        let mut y = panel.y + 50.;
        if !editing {
            button(
                "Browse libraries…",
                HitTarget::SchematicBrowseLibraries,
                RectPx {
                    x: panel.x + width - 192.,
                    y: panel.y + 10.,
                    width: 176.,
                    height: 29.,
                },
                quads,
                text,
                hits,
            );
            if authoring.choices.is_empty() {
                draw_text(
                    "No project symbols yet. Browse libraries to choose one.",
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
        }
        y = panel.y + if editing { 60. } else { 215. };
        let mut controls = ControlMeshCache::default();
        for (label, value, editor, field, target) in [
            (
                "Reference",
                &authoring.reference,
                authoring.reference_editor,
                PlacementField::Reference,
                HitTarget::SchematicReference,
            ),
            (
                "Value",
                &authoring.value,
                authoring.value_editor,
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
            crate::single_line_control::paint(
                value,
                "",
                editor,
                authoring.field == field
                    && state.ui.active_menu.is_none()
                    && state.ui.focus
                        == datum_gui_protocol::ApplicationFocus::Editor(state.ui.layout.focused),
                true,
                row,
                &mut ControlPainter::new(quads, &mut controls, 1.),
                text,
                hits,
                target,
            )?;
            y += 35.;
        }
        if let Some(refusal) = &authoring.refusal {
            draw_text(
                &truncate_text(refusal, 65),
                panel.x + 16.,
                panel.y + if editing { 140. } else { 203. },
                11.,
                design_tokens::chrome::STATUS_ERROR,
                TextFace::Ui,
                text,
            );
        }
        if !editing
            && authoring.refusal.is_none()
            && let Some(choice) = authoring.chosen.and_then(|i| authoring.choices.get(i))
        {
            draw_text(
                choice.binding_label(),
                panel.x + 16.,
                panel.y + if editing { 140. } else { 203. },
                11.,
                TEXT_MUTED,
                TextFace::Ui,
                text,
            );
        }
        button(
            "Cancel",
            HitTarget::SchematicCancel,
            RectPx {
                x: panel.x + 16.,
                y: panel.y + if editing { 190. } else { 305. },
                width: 90.,
                height: 29.,
            },
            quads,
            text,
            hits,
        );
        draw_text(
            if editing {
                "Enter: apply changes · Escape: cancel"
            } else {
                "Enter: resume placement"
            },
            panel.x + 16.,
            panel.y + if editing { 166. } else { 286. },
            11.,
            TEXT_MUTED,
            TextFace::Ui,
            text,
        );
        button(
            if editing { "Apply" } else { "Resume" },
            if editing {
                HitTarget::SchematicEditApply
            } else {
                HitTarget::SchematicPlace
            },
            RectPx {
                x: panel.x + width - 106.,
                y: panel.y + if editing { 190. } else { 305. },
                width: 90.,
                height: 29.,
            },
            quads,
            text,
            hits,
        );
    } else if authoring.active() {
        let panel = RectPx {
            x: layout.top_menu_bar.width / 2. - 200.,
            y: 76.,
            width: 400.,
            height: if authoring.wire { 65. } else { 86. },
        };
        quads.push(Quad::from_rect(panel, design_tokens::chrome::SURFACE_01));
        draw_text(
            if authoring.wire {
                "Place Wire · click vertices"
            } else {
                "Click: place · R: rotate · X/Y: flip horiz/vert"
            },
            panel.x + 10.,
            panel.y + 10.,
            12.,
            TEXT_PRIMARY,
            TextFace::UiStrong,
            text,
        );
        if !authoring.wire {
            draw_text(
                "Tab: reference/value · Esc: cancel",
                panel.x + 10.,
                panel.y + 31.,
                11.,
                TEXT_MUTED,
                TextFace::Ui,
                text,
            );
        }
        button(
            "Cancel",
            HitTarget::SchematicCancel,
            RectPx {
                x: panel.x + 10.,
                y: panel.y + if authoring.wire { 30. } else { 53. },
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
    } else if state.authoring.gesture.tool == datum_gui_protocol::WorkspaceTool::Move
        && state
            .authoring
            .gesture
            .target_object_id
            .as_deref()
            .is_some_and(|id| id.starts_with("schematic-symbol:"))
    {
        let panel = RectPx {
            x: layout.top_menu_bar.width / 2. - 210.,
            y: 76.,
            width: 420.,
            height: 60.,
        };
        quads.push(Quad::from_rect(panel, design_tokens::chrome::SURFACE_01));
        draw_text(
            "Move symbol · R: rotate · X/Y: flip horiz/vert",
            panel.x + 10.,
            panel.y + 10.,
            12.,
            TEXT_PRIMARY,
            TextFace::UiStrong,
            text,
        );
        draw_text(
            "Click: apply · Esc: cancel",
            panel.x + 10.,
            panel.y + 32.,
            11.,
            TEXT_MUTED,
            TextFace::Ui,
            text,
        );
    }
    Ok(())
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
