//! Native symbol selection and gesture controls using existing chrome tokens.
use crate::global_preferences_primitives::{ControlMeshCache, ControlPainter};
use crate::*;
#[path = "schematic_creation.rs"]
mod creation;
#[path = "schematic_field_controls.rs"]
mod field_controls;
#[path = "schematic_label.rs"]
mod label;
use datum_gui_protocol::{
    ReviewWorkspaceState,
    schematic_authoring::{MarkerKind, PlacementField},
};
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
        let bus = authoring.bus;
        let compact = editing || bus;
        let entries = match &authoring.editing {
            Some(datum_gui_protocol::schematic_authoring::FieldsEditDraft::Bus(draft)) => {
                draft.entries.as_slice()
            }
            _ => &[],
        };
        let custom = matches!(
            authoring.editing,
            Some(datum_gui_protocol::schematic_authoring::FieldsEditDraft::Custom(_))
        );
        let extra = field_controls::extra(authoring).max(if entries.is_empty() { 0. } else { 34. });
        let width = 480f32.min(layout.top_menu_bar.width - 32.0);
        let panel = RectPx {
            x: (layout.top_menu_bar.width - width) / 2.,
            y: 76.,
            width,
            height: if compact { 235. + extra } else { 350. },
        };
        quads.push(Quad::from_rect(panel, design_tokens::chrome::SURFACE_01));
        push_rect_border(quads, panel, design_tokens::chrome::BORDER_STRONG, 1.);
        hits.push(HitRegion {
            target: HitTarget::SchematicPicker,
            rect: panel,
        });
        draw_text(
            if custom {
                "Custom metadata"
            } else if bus {
                if editing { "Edit Bus" } else { "Place Bus" }
            } else if editing {
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
        if !compact {
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
        y = panel.y + if compact { 60. } else { 215. };
        let mut controls = ControlMeshCache::default();
        for (label, value, editor, field, target) in [
            (
                if custom {
                    "Key"
                } else if bus {
                    "Name"
                } else {
                    "Reference"
                },
                &authoring.reference,
                authoring.reference_editor,
                PlacementField::Reference,
                HitTarget::SchematicReference,
            ),
            (
                if bus { "Members" } else { "Value" },
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
                authoring.editing.as_ref().is_none_or(|d| d.text_editable()),
                row,
                &mut ControlPainter::new(quads, &mut controls, 1.),
                text,
                hits,
                target,
            )?;
            y += 35.;
        }
        field_controls::render(state, panel, quads, text, hits);
        if let Some(entry) = entries.get(authoring.bus_entry_choice) {
            let label = format!(
                "Entry {}/{} · {:.3}, {:.3} mm",
                authoring.bus_entry_choice + 1,
                entries.len(),
                entry.position.x as f64 / 1e6,
                entry.position.y as f64 / 1e6
            );
            button(
                &label,
                HitTarget::SchematicBusEntryChoice,
                RectPx {
                    x: panel.x + 16.,
                    y: panel.y + 133.,
                    width: width - 142.,
                    height: 27.,
                },
                quads,
                text,
                hits,
            );
            button(
                "Delete entry",
                HitTarget::SchematicBusEntryDelete,
                RectPx {
                    x: panel.x + width - 118.,
                    y: panel.y + 133.,
                    width: 102.,
                    height: 27.,
                },
                quads,
                text,
                hits,
            );
        }
        if let Some(refusal) = &authoring.refusal {
            draw_text(
                &truncate_text(refusal, 65),
                panel.x + 16.,
                panel.y + if compact { 140. + extra } else { 203. },
                11.,
                design_tokens::chrome::STATUS_ERROR,
                TextFace::Ui,
                text,
            );
        }
        if bus && authoring.refusal.is_none() {
            draw_text(
                "Members: existing scalar names separated by commas",
                panel.x + 16.,
                panel.y + 140. + extra,
                11.,
                TEXT_MUTED,
                TextFace::Ui,
                text,
            );
        }
        if !compact
            && authoring.refusal.is_none()
            && let Some(choice) = authoring.chosen.and_then(|i| authoring.choices.get(i))
        {
            draw_text(
                choice.binding_label(),
                panel.x + 16.,
                panel.y + if compact { 140. + extra } else { 203. },
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
                y: panel.y + if compact { 190. + extra } else { 305. },
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
            } else if bus {
                "Enter: draw bus · Escape: cancel"
            } else {
                "Enter: resume placement"
            },
            panel.x + 16.,
            panel.y + if compact { 166. + extra } else { 286. },
            11.,
            TEXT_MUTED,
            TextFace::Ui,
            text,
        );
        button(
            if editing {
                "Apply"
            } else if bus {
                "Draw"
            } else {
                "Resume"
            },
            if editing {
                HitTarget::SchematicEditApply
            } else {
                HitTarget::SchematicPlace
            },
            RectPx {
                x: panel.x + width - 106.,
                y: panel.y + if compact { 190. + extra } else { 305. },
                width: 90.,
                height: 29.,
            },
            quads,
            text,
            hits,
        );
    } else if authoring.active() {
        let compact = authoring.wire || authoring.marker.is_some();
        let panel = RectPx {
            x: layout.top_menu_bar.width / 2. - 200.,
            y: 76.,
            width: 400.,
            height: if compact { 65. } else { 86. },
        };
        quads.push(Quad::from_rect(panel, design_tokens::chrome::SURFACE_01));
        draw_text(
            match authoring.marker {
                Some(MarkerKind::Junction) => "Place Junction · click a wire crossing",
                Some(MarkerKind::NoConnect) => "Place No-Connect · click an open pin",
                None if authoring.wire && authoring.bus => "Place Bus · click vertices",
                None if authoring.wire => "Place Wire · click vertices",
                None => "Click: place · R: rotate · X/Y: flip horiz/vert",
            },
            panel.x + 10.,
            panel.y + 10.,
            12.,
            TEXT_PRIMARY,
            TextFace::UiStrong,
            text,
        );
        if !compact {
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
                y: panel.y + if compact { 30. } else { 53. },
                width: 90.,
                height: 26.,
            },
            quads,
            text,
            hits,
        );
        if authoring.wire {
            button(
                if authoring.bus {
                    "Finish bus"
                } else {
                    "Finish wire"
                },
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
        } else if authoring.marker.is_some() {
            draw_text(
                "Esc: cancel",
                panel.x + 115.,
                panel.y + 37.,
                11.,
                TEXT_MUTED,
                TextFace::Ui,
                text,
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marker_placement_guidance_names_command_and_only_offers_relevant_controls() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let layout = ShellLayout::for_window(1280, 800, None);
        for marker in [MarkerKind::Junction, MarkerKind::NoConnect] {
            state.schematic_authoring.marker = Some(marker);
            let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
            render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
            assert!(
                text.iter()
                    .any(|run| run.text.starts_with(&format!("Place {}", marker.label())))
            );
            assert!(text.iter().any(|run| run.text == "Esc: cancel"));
            assert!(
                hits.iter()
                    .any(|hit| hit.target == HitTarget::SchematicCancel)
            );
            assert!(
                !hits
                    .iter()
                    .any(|hit| hit.target == HitTarget::SchematicFinishWire)
            );
            assert!(
                !text
                    .iter()
                    .any(|run| run.text.contains("rotate") || run.text.contains("reference/value"))
            );
            state.schematic_authoring.cancel();
            text.clear();
            hits.clear();
            render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
            assert!(
                text.is_empty() && hits.is_empty(),
                "cancel removes marker placement controls"
            );
        }
    }
    #[test]
    fn bus_form_and_polyline_configure_shared_fields_and_finish_control() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.schematic_authoring.bus = true;
        state.schematic_authoring.picker = true;
        let layout = ShellLayout::for_window(1280, 800, None);
        let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
        render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
        for expected in ["Place Bus", "Name", "Members", "Draw", "Cancel"] {
            assert!(
                text.iter().any(|r| r.text == expected),
                "missing {expected}"
            );
        }
        assert!(
            !hits
                .iter()
                .any(|h| h.target == HitTarget::SchematicBrowseLibraries)
        );
        state.schematic_authoring.picker = false;
        state.schematic_authoring.wire = true;
        text.clear();
        hits.clear();
        render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
        assert!(text.iter().any(|r| r.text == "Place Bus · click vertices"));
        assert!(text.iter().any(|r| r.text == "Finish bus"));
        assert_eq!(
            hits.iter()
                .filter(|h| h.target == HitTarget::SchematicFinishWire)
                .count(),
            1
        );
    }
}
