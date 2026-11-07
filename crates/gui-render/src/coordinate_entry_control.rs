//! Common exact coordinate form, painted with the existing single-line controls.
use crate::global_preferences_primitives::{ControlMeshCache, ControlPainter, button};
use crate::*;
use datum_gui_protocol::{CoordinateEntryControl as C, CoordinateEntryMode, ReviewWorkspaceState};
use eda_engine::ir::units::LengthQuantity;

pub(crate) fn render(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    let Some(form) = &state.authoring.coordinates else {
        return Ok(());
    };
    let panes = layout.viewport_panes(&state.ui.layout);
    let Some(pane) = panes.panes.iter().find(|pane| pane.id == form.basis.pane) else {
        return Ok(());
    };
    let viewport = pane.rect.scene;
    let width = 420f32.min((viewport.width - 16.).max(0.));
    if width < 80. || viewport.height <= 16. {
        return Ok(());
    }
    let reason = form.refusal.as_deref().unwrap_or("");
    let reason_height = if reason.is_empty() {
        0.
    } else {
        text_metrics::measured_text_run_height_px(reason, width - 24., 11., TextFace::Ui)? + 8.
    };
    let height = (260. + reason_height).min(viewport.height - 16.);
    let panel = RectPx {
        x: viewport.x + (viewport.width - width) / 2.,
        y: viewport.y + 8.,
        width,
        height,
    };
    let starts = (quads.len(), text.len(), hits.len());
    quads.push(Quad::from_rect(panel, design_tokens::chrome::SURFACE_01));
    push_rect_border(quads, panel, design_tokens::chrome::BORDER_STRONG, 1.);
    let owner_available = state.ui.active_menu.is_none()
        && !state.ui.new_project.open
        && !state.ui.project_open.open
        && state.library.acquisition.is_none();
    // A passive body prevents backdrop clicks from applying Move, but only
    // while this form's owner can receive input ahead of canvas authoring.
    if owner_available {
        hits.push(HitRegion {
            target: HitTarget::SchematicPicker,
            rect: panel,
        });
    }
    let active = owner_available
        && state.ui.layout.focused == form.basis.pane
        && state.ui.focus == datum_gui_protocol::ApplicationFocus::Editor(form.basis.pane);
    let enabled = owner_available
        && !form.pending
        && !form.stale
        && state.ui.layout.focused == form.basis.pane;
    let content = RectPx {
        x: panel.x + 12.,
        y: panel.y + 8.,
        width: width - 24.,
        height: height - 16.,
    };
    draw_text_clipped(
        &format!("{} — exact position", form.label),
        content.x,
        panel.y + 12.,
        14.,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        content,
        text,
    );
    let mut cache = ControlMeshCache::default();
    let mut painter = ControlPainter::new(quads, &mut cache, 1.);
    let button_width = 92f32.min((width - 32.) / 2.);
    for (control, label, x) in [
        (C::Absolute, "Absolute", panel.x + 12.),
        (C::Delta, "Delta", panel.x + 24. + button_width),
    ] {
        let selected = matches!(
            (form.entry.mode(), control),
            (CoordinateEntryMode::Absolute, C::Absolute) | (CoordinateEntryMode::Delta, C::Delta)
        );
        let rect = RectPx {
            x,
            y: panel.y + 40.,
            width: button_width,
            height: 26.,
        };
        button(
            label,
            rect,
            active && (selected || form.focus == control),
            enabled,
            &mut painter,
            text,
        );
        if enabled {
            hits.push(HitRegion {
                target: HitTarget::CoordinateEntry(control),
                rect,
            });
        }
    }
    for (control, label, value, editor, y) in [
        (
            C::X,
            if form.entry.mode() == CoordinateEntryMode::Delta {
                "ΔX"
            } else {
                "X"
            },
            form.entry.x.as_str(),
            form.entry.x_editor,
            panel.y + 78.,
        ),
        (
            C::Y,
            if form.entry.mode() == CoordinateEntryMode::Delta {
                "ΔY"
            } else {
                "Y"
            },
            form.entry.y.as_str(),
            form.entry.y_editor,
            panel.y + 116.,
        ),
    ] {
        draw_text_clipped(
            label,
            panel.x + 14.,
            y + 8.,
            12.,
            TEXT_MUTED,
            TextFace::Ui,
            content,
            text,
        );
        let first = hits.len();
        single_line_control::paint(
            value,
            "",
            editor,
            active && form.focus == control,
            enabled,
            RectPx {
                x: panel.x + 54.,
                y,
                width: (width - 68.).max(1.),
                height: 29.,
            },
            &mut painter,
            text,
            hits,
            HitTarget::CoordinateEntry(control),
        )?;
        if !enabled {
            hits.truncate(first);
        }
    }
    let exact = match form.entry.destination() {
        Ok(point) if form.entry.mode() == CoordinateEntryMode::Delta => format!(
            "Exact Δ: {:+} · {:+} nm",
            i128::from(point.x) - i128::from(form.entry.reference().x),
            i128::from(point.y) - i128::from(form.entry.reference().y)
        ),
        Ok(point) => format!("Exact: {} · {} nm", point.x, point.y),
        Err(_) => "Exact: enter valid coordinates".into(),
    };
    draw_text_clipped(
        &exact,
        panel.x + 12.,
        panel.y + 156.,
        11.,
        TEXT_MUTED,
        TextFace::Mono,
        content,
        text,
    );
    let units = form.entry.units();
    let unit = match form.entry.quantity() {
        LengthQuantity::BoardLayout => units.board.unit,
        LengthQuantity::DrillHole => units.drill.unit,
        LengthQuantity::SchematicGeometry => units.schematic.unit,
    };
    draw_text_clipped(
        &format!("Units: {} · captured at edit begin", unit.suffix()),
        panel.x + 12.,
        panel.y + 176.,
        11.,
        TEXT_MUTED,
        TextFace::Ui,
        content,
        text,
    );
    let button_y = (panel.y + 204. + reason_height).min(panel.y + height - 54.);
    if !reason.is_empty() {
        let rect = RectPx {
            x: panel.x + 12.,
            y: panel.y + 196.,
            width: width - 24.,
            height: (button_y - panel.y - 204.).max(0.),
        };
        draw_text_clipped(
            reason,
            rect.x,
            rect.y,
            11.,
            design_tokens::chrome::STATUS_ERROR,
            TextFace::Ui,
            rect,
            text,
        );
        text.last_mut().expect("full refusal").layout_size = Some((rect.width, reason_height));
    }
    // Buttons remain at the bottom of the captured pane even when a long
    // refusal needs the existing Console to expose its remaining lines.
    let y = button_y;
    for (control, label, x) in [
        (C::Apply, "Apply", panel.x + 12.),
        (C::Cancel, "Cancel", panel.x + 24. + button_width),
    ] {
        let rect = RectPx {
            x,
            y,
            width: button_width,
            height: 29.,
        };
        button(
            label,
            rect,
            active && form.focus == control,
            enabled,
            &mut painter,
            text,
        );
        if enabled {
            hits.push(HitRegion {
                target: HitTarget::CoordinateEntry(control),
                rect,
            });
        }
    }
    draw_text_clipped(
        if reason_height + 260. > height {
            "Full refusal: Console · Esc: close form"
        } else {
            "Esc: close form · keep Move preview"
        },
        panel.x + 12.,
        panel.y + height - 24.,
        10.,
        TEXT_MUTED,
        TextFace::Ui,
        content,
        text,
    );
    drop(painter);
    hit_clipping::clip_content(quads, text, hits, starts.0, starts.1, starts.2, viewport);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::{AuthoringViewBasis, CoordinateEntry, CoordinateEntryForm, PointNm};
    use eda_engine::ir::units::UnitsProfile;

    fn fixture() -> (ReviewWorkspaceState, ShellLayout, RectPx) {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let layout = ShellLayout::for_window(1344, 840, None);
        let panes = layout.viewport_panes(&state.ui.layout);
        let pane = panes
            .panes
            .iter()
            .find(|pane| pane.content == datum_gui_protocol::PaneContent::Schematic)
            .unwrap();
        let viewport = pane.rect.scene;
        state.ui.layout.focused = pane.id;
        state.ui.focus = datum_gui_protocol::ApplicationFocus::Editor(pane.id);
        let entry = CoordinateEntry::new(
            uuid::Uuid::nil(),
            LengthQuantity::SchematicGeometry,
            UnitsProfile::default().resolve().unwrap(),
            PointNm { x: 1, y: 2 },
            PointNm {
                x: 24_000_001,
                y: 18_600_017,
            },
            CoordinateEntryMode::Absolute,
        )
        .unwrap();
        state.authoring.coordinates = Some(CoordinateEntryForm {
            entry,
            focus: C::X,
            label: "Move wire end".into(),
            refusal: None,
            restore_position: PointNm {
                x: 24_000_001,
                y: 18_600_017,
            },
            basis: AuthoringViewBasis {
                project: uuid::Uuid::nil(),
                revision: "render-test".into(),
                pane: pane.id,
                content: pane.content,
                source: uuid::Uuid::nil(),
                document: None,
            },
            pending: false,
            stale: false,
        });
        (state, layout, viewport)
    }
    #[test]
    fn coordinate_form_keeps_exact_fields_and_all_hit_geometry_in_captured_pane() {
        let (mut state, layout, viewport) = fixture();
        let refusal = "Apply refused: this destination would lose the existing fixed junction contact. Nothing was written; correct the entered coordinate or cancel the form.";
        state.authoring.coordinates.as_mut().unwrap().refusal = Some(refusal.into());
        let before = state.authoring.coordinates.clone();
        let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
        render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
        assert_eq!(
            state.authoring.coordinates, before,
            "painting cannot change focus, values or caret"
        );
        assert!(text.iter().any(|run| run.text == "24.000001mm"));
        assert!(text.iter().any(|run| run.text == "18.600017mm"));
        assert!(text.iter().any(|run| run.text == refusal));
        for control in [C::Absolute, C::Delta, C::X, C::Y, C::Apply, C::Cancel] {
            assert!(
                hits.iter()
                    .any(|hit| hit.target == HitTarget::CoordinateEntry(control))
            );
        }
        for hit in hits {
            assert_eq!(hit.rect.intersect(viewport), Some(hit.rect));
        }
        for quad in quads {
            for (x, y) in quad.points {
                assert!(x >= viewport.x && x <= viewport.x + viewport.width);
                assert!(y >= viewport.y && y <= viewport.y + viewport.height);
            }
        }
        assert!(text.iter().all(|run| {
            run.clip_bounds
                .is_some_and(|clip| clip.intersect(viewport) == Some(clip))
        }));
    }
    #[test]
    fn pending_stale_and_unresolved_capture_do_not_publish_form_actions() {
        let (mut state, layout, _) = fixture();
        for (pending, stale) in [(true, false), (false, true)] {
            let form = state.authoring.coordinates.as_mut().unwrap();
            form.pending = pending;
            form.stale = stale;
            let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
            render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
            assert!(
                !hits
                    .iter()
                    .any(|hit| matches!(hit.target, HitTarget::CoordinateEntry(_)))
            );
            assert!(text.iter().any(|run| run.text == "Apply"));
        }
        state.authoring.coordinates.as_mut().unwrap().basis.pane =
            datum_gui_protocol::PaneId(u32::MAX);
        let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
        render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
        assert!(quads.is_empty() && text.is_empty() && hits.is_empty());
    }
    #[test]
    fn menu_and_modal_owners_keep_coordinate_values_painted_without_action_hits() {
        let (mut state, layout, _) = fixture();
        let before = state.authoring.coordinates.clone();
        for owner in 0..4 {
            state.ui.active_menu = (owner == 0).then(|| "View".into());
            state.ui.new_project.open = owner == 1;
            state.ui.project_open.open = owner == 2;
            state.library.acquisition = (owner == 3).then(|| {
                datum_gui_protocol::library_browser::acquisition::LibraryAcquisition::begin(
                    "render-test".into(),
                    None,
                )
            });
            let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
            render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
            assert!(
                hits.is_empty(),
                "menu/modal owner {owner} retains action and passive-body mouse authority"
            );
            assert!(text.iter().any(|run| run.text == "24.000001mm"));
            assert!(text.iter().any(|run| run.text == "18.600017mm"));
            assert_eq!(state.authoring.coordinates, before);
        }
    }
}
