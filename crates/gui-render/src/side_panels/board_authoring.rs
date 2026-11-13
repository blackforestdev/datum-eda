//! PCB Route/Zone forms configure the shared Inspector scroll and field services.
use super::*;
use crate::global_preferences_primitives::{ControlMeshCache, ControlPainter, button};
use datum_gui_protocol::board_authoring_controls::{
    BoardAuthoringControls, BoardControl as Control, BoardField as Field, BoardToolKind,
};
const ROW: f32 = 22.;
enum FormRow {
    Text(String),
    Field(Field, &'static str),
    Action(String, Control, bool),
}
fn paragraph(rows: &mut Vec<FormRow>, value: &str, width: f32) -> anyhow::Result<()> {
    rows.extend(
        crate::control_text::wrap_text_words_sized(
            value,
            (width - 24.).max(1.),
            11.5,
            TextFace::Ui,
        )?
        .into_iter()
        .map(FormRow::Text),
    );
    Ok(())
}
pub(super) fn render(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<bool> {
    if state.ui.layout.focused_content() != datum_gui_protocol::PaneContent::Board
        || state.inspector_context.owner
            == datum_gui_protocol::inspector_context::InspectorOwner::Navigator
    {
        return Ok(false);
    }
    let Some(setup) = &state.authoring.board_controls else {
        return render_zone(state, rect, quads, text, hits);
    };
    if !setup.owns_pane(state) {
        return Ok(false);
    }
    let current = setup.current(state);
    let title = match setup.kind {
        BoardToolKind::Route => "Route Track",
        BoardToolKind::Zone => "Zone",
    };
    inspector_chrome::push_inspector_title_band(
        rect,
        title,
        &setup.layer_name,
        false,
        quads,
        text,
    )?;
    let mut rows = Vec::new();
    match setup.kind {
        BoardToolKind::Route => rows.push(FormRow::Field(Field::Width, "Width")),
        BoardToolKind::Zone => {
            rows.extend([
                FormRow::Field(Field::Net, "Net"),
                FormRow::Field(Field::Priority, "Priority"),
                FormRow::Field(Field::ThermalGap, "Spoke gap"),
                FormRow::Field(Field::SpokeWidth, "Spoke width"),
            ]);
            let label = match setup.thermal {
                None => "Choose thermal intent",
                Some(false) => "Solid",
                Some(true) => "Thermal relief (fill unsupported)",
            };
            rows.push(FormRow::Action(label.into(), Control::Thermal, current));
        }
    }
    paragraph(
        &mut rows,
        &format!(
            "Dimensions: {:?} · explicit unit suffix accepted",
            setup.units.board.unit
        ),
        rect.width,
    )?;
    if let Some(route) = &state.authoring.board_route {
        paragraph(
            &mut rows,
            &format!(
                "Net {} · {} placed vertices",
                route.basis().net,
                route.vertices().len()
            ),
            rect.width,
        )?;
        if let Some(policy) = &route.basis().clearance {
            paragraph(
                &mut rows,
                &format!(
                    "Clearance {} nm · {} · {:?}",
                    policy.minimum_nm, policy.name, policy.kind
                ),
                rect.width,
            )?;
        }
        rows.push(FormRow::Action(
            format!("Posture: {:?}", route.posture().routing),
            Control::Posture,
            current,
        ));
    } else if setup.kind == BoardToolKind::Route {
        paragraph(
            &mut rows,
            "Pick a connected Pad. The start Pad supplies the Net.",
            rect.width,
        )?;
    }
    let needs_rule = match setup.kind {
        BoardToolKind::Route => state
            .authoring
            .board_route
            .as_ref()
            .is_some_and(|r| r.basis().clearance.is_none()),
        BoardToolKind::Zone => setup.needs_clearance_rule,
    };
    if needs_rule {
        paragraph(
            &mut rows,
            "No clearance applies. Save an explicit Net rule to continue.",
            rect.width,
        )?;
        rows.extend([
            FormRow::Field(Field::Clearance, "Clearance"),
            FormRow::Field(Field::RulePriority, "Rule priority"),
        ]);
        rows.push(FormRow::Action(
            "Apply rule".into(),
            Control::ApplyRule,
            current
                && setup.length(Field::Clearance).is_ok_and(|v| v > 0)
                && setup.rule_priority().is_ok(),
        ));
    }
    if setup.kind == BoardToolKind::Zone {
        if current {
            policy_rows(
                &mut rows,
                &setup.zone_clearance,
                setup.zone_clearance_label(),
                rect.width,
            )?;
        } else {
            paragraph(
                &mut rows,
                "Clearance unavailable — source changed",
                rect.width,
            )?;
        }
    }
    if let Some(zone) = &state.authoring.board_zone {
        paragraph(
            &mut rows,
            &format!("Boundary: {} vertices · unfilled", zone.vertices().len()),
            rect.width,
        )?;
    }
    if state.scene.outline.is_empty() {
        paragraph(
            &mut rows,
            "No board outline. Board-edge checking is unavailable.",
            rect.width,
        )?;
    }
    if !current {
        paragraph(
            &mut rows,
            if setup.pending {
                "Applying edit…"
            } else {
                "Source, pane or layer changed. Cancel and restart."
            },
            rect.width,
        )?;
    }
    if let Some(reason) = &setup.refusal {
        paragraph(&mut rows, reason, rect.width)?;
    }
    let can_finish = current
        && match setup.kind {
            BoardToolKind::Route => {
                setup.refusal.is_none()
                    && state.authoring.board_route.as_ref().is_some_and(|r| {
                        r.vertices().len() >= 2 && setup.length(Field::Width).is_ok_and(|v| v > 0)
                    })
            }
            BoardToolKind::Zone => {
                setup.zone_settings().is_ok()
                    && state.authoring.board_zone.as_ref().is_some_and(|z| {
                        z.vertices().len() >= 3
                            && setup.resolved_zone_net.as_ref().ok().copied() == Some(z.basis().net)
                    })
            }
        };
    rows.push(FormRow::Action(
        if setup.kind == BoardToolKind::Route {
            "Finish"
        } else {
            "Finish Boundary"
        }
        .into(),
        Control::Finish,
        can_finish,
    ));
    rows.push(FormRow::Action(
        "Cancel".into(),
        Control::Cancel,
        !setup.pending,
    ));
    paragraph(
        &mut rows,
        "Click: vertex · Backspace: remove · Esc: cancel",
        rect.width,
    )?;
    paint_rows(state, Some(setup), rect, rows, quads, text, hits)?;
    Ok(true)
}
fn span(row: &FormRow, width: f32) -> usize {
    match row {
        FormRow::Text(_) => 1,
        FormRow::Field(..) if width < 240. => 3,
        _ => 2,
    }
}
fn control(row: &FormRow, current: bool) -> Option<Control> {
    match row {
        FormRow::Field(field, _) if current => Some(Control::Field(*field)),
        FormRow::Action(_, control, true) => Some(*control),
        _ => None,
    }
}
fn paint_rows(
    state: &ReviewWorkspaceState,
    setup: Option<&BoardAuthoringControls>,
    panel: RectPx,
    rows: Vec<FormRow>,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    let body = RectPx {
        y: panel.y + 68.,
        height: (panel.height - 68.).max(0.),
        ..panel
    };
    let visible = (body.height / ROW).floor() as usize;
    if visible == 0 {
        return Ok(());
    }
    let total = rows.iter().map(|r| span(r, panel.width)).sum::<usize>();
    let start = state
        .ui
        .selection_inspector
        .scroll_offset
        .min(total.saturating_sub(visible));
    let first = (quads.len(), text.len(), hits.len());
    hits.push(HitRegion {
        rect: body,
        target: HitTarget::InspectorContextScroll {
            total,
            visible,
            basis: datum_gui_protocol::inspector_context::InspectorActionBasis::capture(state),
        },
    });
    if let Some(setup) = setup {
        let mut at = 0;
        let offsets = rows
            .iter()
            .filter_map(|row| {
                let position = at;
                at += span(row, panel.width);
                control(row, setup.current(state))
                    .map(|control| (control, position, span(row, panel.width)))
            })
            .collect();
        hits.push(HitRegion {
            rect: body,
            target: HitTarget::BoardAuthoringNavigation {
                basis: setup.basis.clone(),
                kind: setup.kind,
                offsets,
                total,
                visible,
            },
        });
    }
    let mut cache = ControlMeshCache::default();
    let mut painter = ControlPainter::new(quads, &mut cache, 1.);
    let mut at = 0;
    for row in rows {
        let size = span(&row, panel.width);
        let position = at;
        at += size;
        if at <= start || position >= start + visible {
            continue;
        }
        let y = body.y + (position as f32 - start as f32) * ROW;
        match row {
            FormRow::Text(value) => draw_text_clipped(
                &value,
                body.x + 12.,
                y + 3.,
                11.5,
                TEXT_MUTED,
                TextFace::Ui,
                body,
                text,
            ),
            FormRow::Field(field, label) => {
                let Some(setup) = setup else {
                    continue;
                };
                let stacked = panel.width < 240.;
                draw_text_clipped(
                    label,
                    panel.x + 12.,
                    y + if stacked { 2. } else { 9. },
                    11.,
                    TEXT_MUTED,
                    TextFace::Ui,
                    body,
                    text,
                );
                let rect = RectPx {
                    x: panel.x + if stacked { 12. } else { 94. },
                    y: y + if stacked { 22. } else { 0. },
                    width: (panel.width - if stacked { 24. } else { 106. }).max(0.),
                    height: 32.,
                };
                let value = setup.field(field);
                crate::single_line_control::paint(
                    &value.text,
                    "Required",
                    value.editor,
                    setup.focus == Some(Control::Field(field)),
                    setup.current(state),
                    rect,
                    &mut painter,
                    text,
                    hits,
                    HitTarget::BoardAuthoring(Control::Field(field)),
                )?;
            }
            FormRow::Action(label, control, enabled) => {
                let rect = RectPx {
                    x: panel.x + 12.,
                    y,
                    width: (panel.width - 24.).max(0.),
                    height: 29.,
                };
                button(
                    &label,
                    rect,
                    setup.is_some_and(|s| s.focus == Some(control)),
                    enabled,
                    &mut painter,
                    text,
                );
                if enabled {
                    hits.push(HitRegion {
                        rect,
                        target: HitTarget::BoardAuthoring(control),
                    });
                }
            }
        }
    }
    crate::hit_clipping::clip_content(quads, text, hits, first.0, first.1, first.2, body);
    Ok(())
}
fn render_zone(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<bool> {
    use datum_gui_protocol::{AuthoredSelectionClass, SelectionSubject, SelectionTarget};
    let SelectionTarget::Native(view) = &state.selection else {
        return Ok(false);
    };
    let SelectionSubject::Object(identity) = &view.subject else {
        return Ok(false);
    };
    if identity.class != AuthoredSelectionClass::Zone || view.stale {
        return Ok(false);
    }
    let id = identity.id;
    let Some(model) = &state.library.basis else {
        return Ok(false);
    };
    if view.project != model.project.project_id
        || view.revision.0 != model.model_revision.0
        || view.revision.0 != state.scene.source_revision
    {
        return Ok(false);
    }
    let Some(values) = view.values.get(identity) else {
        return Ok(false);
    };
    inspector_chrome::push_inspector_title_band(
        rect,
        "Zone",
        "Boundary and explicit fill",
        true,
        quads,
        text,
    )?;
    let status = match model.zone_fills.get(&id) {
        None => "Unfilled — boundary authored".into(),
        Some(fill) if fill.model_revision != model.model_revision => {
            "Stale — refill explicitly".into()
        }
        Some(fill)
            if fill.state == eda_engine::substrate::ZoneFillState::Filled
                && fill.islands.is_empty() =>
        {
            "Filled empty — verified, no copper".into()
        }
        Some(fill) => format!("{:?} · {} copper islands", fill.state, fill.islands.len()),
    };
    let mut rows = Vec::new();
    paragraph(&mut rows, &format!("Zone {id}"), rect.width)?;
    paragraph(
        &mut rows,
        &format!("Net: {}", view.field_display_preview("board_net_id", 100)),
        rect.width,
    )?;
    paragraph(
        &mut rows,
        &format!("Layer: {}", view.field_display_preview("board_layer", 100)),
        rect.width,
    )?;
    let vertices = values
        .get("zone_boundary")
        .and_then(|v| v.get("vertices"))
        .and_then(|v| v.as_array())
        .map(|v| v.len().to_string())
        .unwrap_or_else(|| "Unavailable".into());
    paragraph(
        &mut rows,
        &format!(
            "Vertices: {vertices} · Priority: {}",
            view.field_display_preview("priority", 100)
        ),
        rect.width,
    )?;
    for (label, key) in [
        ("Thermal relief", "thermal_relief"),
        ("Spoke gap", "thermal_gap_nm"),
        ("Spoke width", "thermal_spoke_width_nm"),
    ] {
        paragraph(
            &mut rows,
            &format!("{label}: {}", view.field_display_preview(key, 100)),
            rect.width,
        )?;
    }
    let readback = datum_gui_protocol::board_zones::captured_zone_clearance(view);
    policy_rows(
        &mut rows,
        &readback,
        Ok(view.field_display_preview("zone_effective_clearance_nm", 100)),
        rect.width,
    )?;
    paragraph(&mut rows, &status, rect.width)?;
    paragraph(
        &mut rows,
        "Fill never runs automatically. Unsupported results keep the boundary unchanged.",
        rect.width,
    )?;
    let fill_available = datum_gui_protocol::gui_menu_model::native_project_edit_available(state)
        && state
            .backing
            .as_ref()
            .is_some_and(|b| b.native_write_verbs.contains("datum.check.fill_zones"));
    rows.push(FormRow::Action(
        "Fill".into(),
        Control::Fill,
        fill_available,
    ));
    paint_rows(state, None, rect, rows, quads, text, hits)?;
    Ok(true)
}
fn policy_rows(
    rows: &mut Vec<FormRow>,
    readback: &Result<
        eda_engine::api::native_write::board_routing::zone::ZoneClearanceReadback,
        String,
    >,
    label: Result<String, String>,
    width: f32,
) -> anyhow::Result<()> {
    match (readback, label) {
        (Ok(readback), Ok(label)) => {
            paragraph(rows, &format!("Conservative clearance: {label}"), width)?;
            paragraph(
                rows,
                "Largest saved minimum across current Board Nets",
                width,
            )?;
            for source in &readback.sources {
                paragraph(
                    rows,
                    &format!("{:?}: {} · {}", source.kind, source.name, source.source),
                    width,
                )?;
            }
        }
        (Err(reason), _) => paragraph(rows, &format!("Clearance unavailable: {reason}"), width)?,
        (_, Err(reason)) => paragraph(rows, &format!("Clearance unavailable: {reason}"), width)?,
    }
    Ok(())
}

#[cfg(test)]
#[path = "board_authoring_tests.rs"]
mod tests;
