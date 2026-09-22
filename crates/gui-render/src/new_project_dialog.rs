//! Native New Project form for the eight-key Project Working Units genesis path.

use super::*;
use crate::global_preferences_primitives::{
    ControlMeshCache, ControlPainter, button, push_dashed_rect_border,
    push_rounded_rect_with_border,
};
use datum_gui_protocol::{NewProjectFocus, NewProjectUnitsChoice};

impl Renderer {
    /// Compose only the native form using the renderer's shared control owner.
    pub fn prepare_native_new_project(
        &mut self,
        dialog: &datum_gui_protocol::NewProjectDialogState,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> PreparedScene {
        let scale = scale_factor.max(0.01);
        let layout = ShellLayout::for_surface(width, height, scale, None);
        let mut quads = Vec::new();
        let mut text = Vec::new();
        let mut hits = Vec::new();
        render_new_project_dialog(
            dialog,
            &layout,
            true,
            &mut self.control_meshes,
            scale,
            &mut quads,
            &mut text,
            &mut hits,
        );
        PreparedScene::from_dialog_parts(layout, quads, text, hits, scale, (width, height))
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_new_project_dialog(
    dialog: &datum_gui_protocol::NewProjectDialogState,
    layout: &ShellLayout,
    native_window: bool,
    controls: &mut ControlMeshCache,
    scale: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    if !dialog.open || !native_window {
        return;
    }
    let mut painter = ControlPainter::new(quads, controls, scale);
    let quads = &mut painter;
    let window = RectPx {
        x: 0.0,
        y: 0.0,
        width: layout.top_menu_bar.width,
        height: layout.status_bar.y + layout.status_bar.height,
    };
    quads.push(Quad::from_rect(window, design_tokens::chrome::BG_BASE));
    hits.push(HitRegion {
        target: HitTarget::NewProjectModal,
        rect: window,
    });
    push_rect_border(quads, window, design_tokens::chrome::BORDER_STRONG, 1.0);
    let header = RectPx {
        x: 0.0,
        y: 0.0,
        width: window.width,
        height: 42.0,
    };
    quads.push(Quad::from_rect(header, design_tokens::chrome::SURFACE_01));
    push_rect_border(quads, header, design_tokens::chrome::BORDER_SUBTLE, 1.0);
    draw_text(
        "New Project",
        16.0,
        13.0,
        design_tokens::typography::BODY_SIZE,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    let doorway = "File > New Project";
    let doorway_width = measured_text_run_width_px(
        doorway,
        design_tokens::typography::MICRO_SIZE,
        TextFace::Mono,
    );
    draw_text(
        doorway,
        window.width - doorway_width - 16.0,
        15.0,
        design_tokens::typography::MICRO_SIZE,
        TEXT_MUTED,
        TextFace::Mono,
        text,
    );

    let inset = 18.0;
    let field_width = window.width - inset * 2.0;
    let mut y = header.height + 14.0;
    if let Some(refusal) = &dialog.refusal {
        let alert = RectPx {
            x: inset,
            y,
            width: field_width,
            height: 54.0,
        };
        quads.push(Quad::from_rect(alert, design_tokens::chrome::SURFACE_02));
        push_dashed_rect_border(quads, alert, design_tokens::chrome::STATUS_ERROR);
        draw_text(
            &format!("⊘ {}", truncate_text(refusal, 100)),
            alert.x + 12.0,
            alert.y + 18.0,
            design_tokens::typography::CAPTION_SIZE,
            design_tokens::chrome::STATUS_ERROR,
            TextFace::UiStrong,
            text,
        );
        y += alert.height + 10.0;
    }
    y = draw_text_field(
        "Project name",
        &dialog.project_name,
        "Enter a Project name",
        dialog.focus == NewProjectFocus::ProjectName,
        y,
        field_width,
        quads,
        text,
        hits,
        HitTarget::NewProjectName,
    );
    y = draw_text_field(
        "Location",
        &dialog.destination,
        "Enter the full destination path",
        dialog.focus == NewProjectFocus::Destination,
        y + 8.0,
        field_width,
        quads,
        text,
        hits,
        HitTarget::NewProjectDestination,
    );
    draw_text(
        "Working units for this new Project",
        inset,
        y + 13.0,
        design_tokens::typography::BODY_SIZE,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    y += 36.0;
    let choice_focus = dialog.focus == NewProjectFocus::UnitsChoice;
    for (choice, label) in [
        (
            NewProjectUnitsChoice::Global,
            "Use my Global Units defaults",
        ),
        (NewProjectUnitsChoice::Factory, "Use Datum factory Units"),
    ] {
        let rect = RectPx {
            x: inset,
            y,
            width: field_width,
            height: 30.0,
        };
        if choice_focus {
            push_rect_border(quads, rect, design_tokens::chrome::STATUS_INFO, 1.0);
        }
        quads.ellipse_fill(
            RectPx {
                x: rect.x + 8.0,
                y: rect.y + 8.0,
                width: 14.0,
                height: 14.0,
            },
            if dialog.units_choice == choice {
                TEXT_PRIMARY
            } else {
                design_tokens::chrome::BORDER_STRONG
            },
            16,
        );
        if dialog.units_choice != choice {
            quads.ellipse_fill(
                RectPx {
                    x: rect.x + 11.0,
                    y: rect.y + 11.0,
                    width: 8.0,
                    height: 8.0,
                },
                design_tokens::chrome::BG_BASE,
                16,
            );
        }
        draw_text(
            label,
            rect.x + 32.0,
            rect.y + 8.0,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_PRIMARY,
            TextFace::Ui,
            text,
        );
        hits.push(HitRegion {
            target: HitTarget::NewProjectUnitsChoice(choice),
            rect,
        });
        y += 30.0;
    }

    let summary_starts = (quads.len(), text.len(), hits.len());
    let summary_height = 34.0 + dialog.units_summary.len().max(1) as f32 * 23.0 + 48.0;
    let summary = RectPx {
        x: inset,
        y: y + 6.0,
        width: field_width,
        height: summary_height,
    };
    quads.push(Quad::from_rect(summary, design_tokens::chrome::SURFACE_01));
    push_rect_border(
        quads,
        summary,
        if dialog.focus == NewProjectFocus::UnitsSummary {
            design_tokens::chrome::STATUS_INFO
        } else {
            design_tokens::chrome::BORDER_STRONG
        },
        if dialog.focus == NewProjectFocus::UnitsSummary {
            2.0
        } else {
            1.0
        },
    );
    draw_text(
        "Working units this Project will be created with",
        summary.x + 12.0,
        summary.y + 10.0,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    let mut row_y = summary.y + 34.0;
    if dialog.units_summary.is_empty() {
        draw_text(
            "Not resolved — Global units could not be read",
            summary.x + 12.0,
            row_y + 4.0,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_MUTED,
            TextFace::Ui,
            text,
        );
        row_y += 23.0;
    } else {
        for row in &dialog.units_summary {
            draw_text(
                &row.label,
                summary.x + 12.0,
                row_y + 4.0,
                design_tokens::typography::CAPTION_SIZE,
                TEXT_SECONDARY,
                TextFace::Ui,
                text,
            );
            let value_width = measured_text_run_width_px(
                &row.value,
                design_tokens::typography::CAPTION_SIZE,
                TextFace::Mono,
            );
            draw_text(
                &row.value,
                (summary.x + summary.width - value_width - 12.0)
                    .max(summary.x + summary.width * 0.5),
                row_y + 4.0,
                design_tokens::typography::CAPTION_SIZE,
                TEXT_PRIMARY,
                TextFace::Mono,
                text,
            );
            row_y += 23.0;
        }
    }
    draw_text(
        &dialog.source_summary,
        summary.x + 12.0,
        row_y + 5.0,
        design_tokens::typography::MICRO_SIZE,
        TEXT_SECONDARY,
        TextFace::UiStrong,
        text,
    );
    draw_text(
        &truncate_text(&dialog.source_detail, 96),
        summary.x + 12.0,
        row_y + 24.0,
        design_tokens::typography::MICRO_SIZE,
        TEXT_MUTED,
        TextFace::Mono,
        text,
    );
    hits.push(HitRegion {
        target: HitTarget::NewProjectUnitsSummary,
        rect: summary,
    });
    crate::hit_clipping::clip_content(
        quads,
        text,
        hits,
        summary_starts.0,
        summary_starts.1,
        summary_starts.2,
        summary,
    );
    y = summary.y + summary.height + 12.0;

    if dialog.refusal.is_some() && dialog.units_choice == NewProjectUnitsChoice::Global {
        let retry = RectPx {
            x: inset,
            y,
            width: 164.0,
            height: 32.0,
        };
        button(
            "Re-read Global defaults",
            retry,
            dialog.focus == NewProjectFocus::RetryGlobal,
            true,
            quads,
            text,
        );
        hits.push(HitRegion {
            target: HitTarget::NewProjectRetryGlobal,
            rect: retry,
        });
    }
    let create = RectPx {
        x: window.width - inset - 88.0,
        y,
        width: 88.0,
        height: 32.0,
    };
    let cancel = RectPx {
        x: create.x - 98.0,
        y,
        width: 88.0,
        height: 32.0,
    };
    button(
        "Cancel",
        cancel,
        dialog.focus == NewProjectFocus::Cancel,
        true,
        quads,
        text,
    );
    button(
        "Create",
        create,
        dialog.focus == NewProjectFocus::Create,
        dialog.create_enabled(),
        quads,
        text,
    );
    hits.push(HitRegion {
        target: HitTarget::NewProjectCancel,
        rect: cancel,
    });
    hits.push(HitRegion {
        target: HitTarget::NewProjectCreate,
        rect: create,
    });
}

#[allow(clippy::too_many_arguments)]
fn draw_text_field(
    label: &str,
    value: &str,
    placeholder: &str,
    focused: bool,
    y: f32,
    width: f32,
    quads: &mut ControlPainter<'_>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
    target: HitTarget,
) -> f32 {
    let x = 18.0;
    draw_text(
        label,
        x,
        y + 10.0,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_SECONDARY,
        TextFace::UiStrong,
        text,
    );
    let field = RectPx {
        x: x + 150.0,
        y,
        width: width - 150.0,
        height: 34.0,
    };
    push_rounded_rect_with_border(
        quads,
        field,
        design_tokens::chrome::SURFACE_02,
        if focused {
            design_tokens::chrome::STATUS_INFO
        } else {
            design_tokens::chrome::BORDER_STRONG
        },
        if focused { 2.0 } else { 1.0 },
        design_tokens::radius::MD,
    );
    let shown = if value.is_empty() { placeholder } else { value };
    draw_text(
        &truncate_text(shown, 92),
        field.x + 10.0,
        field.y + 10.0,
        design_tokens::typography::BODY_SIZE,
        if value.is_empty() {
            TEXT_MUTED
        } else {
            TEXT_PRIMARY
        },
        TextFace::Ui,
        text,
    );
    if focused {
        let caret_x = (field.x
            + 10.0
            + measured_text_run_width_px(
                value,
                design_tokens::typography::BODY_SIZE,
                TextFace::Ui,
            ))
        .min(field.x + field.width - 10.0);
        quads.push(Quad::from_rect(
            RectPx {
                x: caret_x,
                y: field.y + 7.0,
                width: 1.5,
                height: 20.0,
            },
            TEXT_PRIMARY,
        ));
    }
    hits.push(HitRegion {
        target,
        rect: field,
    });
    y + 34.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::{NewProjectDialogState, NewProjectUnitsSummaryRow};

    fn prepared(complete: bool) -> PreparedScene {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.new_project = NewProjectDialogState::default();
        state.ui.new_project.open = true;
        state.ui.new_project.project_name = "sensor-node".to_owned();
        state.ui.new_project.destination = "/tmp/sensor-node".to_owned();
        if complete {
            state.ui.new_project.units_summary = (0..8)
                .map(|index| NewProjectUnitsSummaryRow {
                    key: format!("datum.units.key_{index}"),
                    label: format!("Units setting {index}"),
                    value: format!("Value {index}"),
                })
                .collect();
        } else {
            state.ui.new_project.refusal =
                Some("Your preference file could not be read. Nothing was created.".to_owned());
        }
        let retained = RetainedScene::from_workspace(&state, 760, 720);
        PreparedScene::from_workspace_with_terminal_renderer(
            &state,
            760,
            720,
            1.0,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
            &[],
            None,
            true,
        )
    }

    #[test]
    fn long_summary_text_uses_the_summary_group_ancestor() {
        let dialog = NewProjectDialogState {
            open: true,
            source_summary: "Source ".repeat(60),
            source_detail: "generation ".repeat(60),
            units_summary: vec![NewProjectUnitsSummaryRow {
                key: "datum.units.system".into(),
                label: "Measurement system".into(),
                value: "An intentionally long displayed value ".repeat(10),
            }],
            ..Default::default()
        };
        for width in [240, 760] {
            let layout = ShellLayout::for_surface(width, 720, 1.0, None);
            let mut quads = Vec::new();
            let mut text = Vec::new();
            let mut hits = Vec::new();
            render_new_project_dialog(
                &dialog,
                &layout,
                true,
                &mut ControlMeshCache::default(),
                1.0,
                &mut quads,
                &mut text,
                &mut hits,
            );
            let summary = hits
                .iter()
                .find(|hit| hit.target == HitTarget::NewProjectUnitsSummary)
                .expect("summary remains one target")
                .rect;
            for run in text.iter().filter(|run| {
                run.text == dialog.source_summary
                    || run.text == dialog.units_summary[0].value
                    || run.text == dialog.units_summary[0].label
            }) {
                assert_eq!(run.clip_bounds, Some(summary));
            }
            assert!(text.iter().any(|run| run.text == dialog.source_summary));
            assert_eq!(
                hits.iter()
                    .filter(|hit| hit.target == HitTarget::NewProjectCreate)
                    .count(),
                1
            );
        }
    }

    #[test]
    fn complete_form_has_exact_two_sources_and_one_create_action() {
        let scene = prepared(true);
        let targets: Vec<_> = scene
            .hit_regions
            .iter()
            .map(|region| &region.target)
            .collect();
        assert_eq!(
            targets
                .iter()
                .filter(|target| matches!(target, HitTarget::NewProjectUnitsChoice(_)))
                .count(),
            2
        );
        assert_eq!(
            targets
                .iter()
                .filter(|target| matches!(target, HitTarget::NewProjectCreate))
                .count(),
            1
        );
        let labels: Vec<_> = scene
            .menu_overlay_text_runs
            .iter()
            .map(|run| run.text.as_str())
            .collect();
        assert!(labels.contains(&"Working units for this new Project"));
        assert!(labels.contains(&"Use my Global Units defaults"));
        assert!(labels.contains(&"Use Datum factory Units"));
    }

    #[test]
    fn refusal_keeps_factory_available_and_exposes_retry() {
        let scene = prepared(false);
        assert!(scene.hit_regions.iter().any(|region| {
            region.target == HitTarget::NewProjectUnitsChoice(NewProjectUnitsChoice::Factory)
        }));
        assert!(
            scene
                .hit_regions
                .iter()
                .any(|region| region.target == HitTarget::NewProjectRetryGlobal)
        );
    }
}
