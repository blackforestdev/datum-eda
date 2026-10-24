//! Shared section rows and one clipped, virtual Inspector row painter.
use super::*;
use datum_gui_protocol::inspector_context::{
    InspectorContext, InspectorNavigation, InspectorSectionId, InspectorValue,
};

pub(super) struct Row<'a> {
    pub label: String,
    pub native_field: Option<&'a str>,
    pub target: Option<HitTarget>,
    pub active: bool,
}

pub(super) fn offsets(rows: &[Row<'_>]) -> Vec<(InspectorSectionId, usize)> {
    rows.iter()
        .enumerate()
        .filter_map(|(index, row)| match &row.target {
            Some(HitTarget::InspectorSectionToggle { section, .. }) => Some((*section, index)),
            _ => None,
        })
        .collect()
}

pub(super) fn rows<'a>(
    context: &InspectorContext<'a>,
    navigation: &InspectorNavigation,
    width: f32,
) -> anyhow::Result<Vec<Row<'a>>> {
    let mut rows = Vec::new();
    for section in &context.sections {
        let collapsed = navigation.collapsed.contains(&section.id);
        rows.push(Row {
            label: format!(
                "{} {}{}",
                if collapsed { "▸" } else { "▾" },
                section.id.label(),
                if section.id == InspectorSectionId::Checks && context.rerun_available {
                    " (R: rerun)"
                } else {
                    ""
                }
            ),
            native_field: None,
            target: Some(HitTarget::InspectorSectionToggle {
                section: section.id,
                basis: context.action_basis.clone(),
            }),
            active: navigation.focused == Some(section.id),
        });
        if collapsed {
            continue;
        }
        for field in &section.fields {
            if let InspectorValue::SelectionField(name) = field.value {
                rows.push(Row {
                    label: field.label.to_string(),
                    native_field: Some(name),
                    target: None,
                    active: false,
                });
            } else {
                let label = format!("{}: {}", field.label, context.field_text(field, usize::MAX));
                for line in crate::control_text::wrap_text_sized(
                    &label,
                    (width - 24.0).max(0.0),
                    11.0,
                    TextFace::Ui,
                )? {
                    rows.push(Row {
                        label: line,
                        native_field: None,
                        target: None,
                        active: false,
                    });
                }
            }
        }
        if section.id == InspectorSectionId::Checks {
            rows.push(Row {
                label: if context.rerun_available {
                    "Rerun ERC (R)"
                } else if context.action_basis.is_some() {
                    "Rerun ERC unavailable — wait for the job or finish the editor preview"
                } else {
                    "Rerun ERC unavailable — open a current native Project"
                }
                .into(),
                native_field: None,
                target: context
                    .action_basis
                    .clone()
                    .filter(|_| context.rerun_available)
                    .map(HitTarget::InspectorRunErc),
                active: false,
            });
        }
    }
    Ok(rows)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_row(
    rect: RectPx,
    body: RectPx,
    index: usize,
    start: usize,
    visible: usize,
    label: &str,
    target: Option<HitTarget>,
    active: bool,
    complete: bool,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    if index < start || index >= start + visible {
        return;
    }
    let y = body.y + (index - start) as f32 * 22.0;
    let hit = RectPx {
        x: rect.x + 10.0,
        y,
        width: (rect.width - 20.0).max(0.0),
        height: 22.0,
    };
    if active {
        push_rect_border(quads, hit, TEXT_ACCENT, 1.0);
    }
    // Disclosure is geometry, so it remains visible with the governed UI font
    // even when that font has no Unicode triangle glyphs.
    let disclosure = matches!(
        &target,
        Some(
            HitTarget::InspectorSectionToggle { .. }
                | HitTarget::SelectionInventoryToggle
                | HitTarget::SelectionFiltersToggle
        )
    );
    let label = if disclosure {
        let x = rect.x + 14.0;
        let top = y + 7.0;
        let points = if label.starts_with('▸') {
            [
                (x, top),
                (x + 5.0, top + 4.0),
                (x, top + 8.0),
                (x, top + 8.0),
            ]
        } else {
            [
                (x, top + 1.0),
                (x + 8.0, top + 1.0),
                (x + 4.0, top + 6.0),
                (x + 4.0, top + 6.0),
            ]
        };
        quads.push(Quad {
            points,
            color: if active { TEXT_ACCENT } else { TEXT_PRIMARY },
        });
        label.split_once(' ').map_or(label, |(_, text)| text)
    } else {
        label
    };
    let indent = if disclosure { 12.0 } else { 0.0 };
    if let Some(target) = target {
        hits.push(HitRegion { rect: hit, target });
    }
    let columns = ((rect.width - 24.0 - indent).max(0.0) / (11.0 * 0.58)).floor() as usize;
    let label = if complete {
        std::borrow::Cow::Borrowed(label)
    } else {
        std::borrow::Cow::Owned(truncate_text(label, columns))
    };
    draw_text(
        &label,
        rect.x + 12.0 + indent,
        y + 3.0,
        11.0,
        if active { TEXT_ACCENT } else { TEXT_PRIMARY },
        TextFace::Ui,
        text,
    );
    let run = text.last_mut().expect("visible Inspector row");
    run.clip_bounds = Some(hit);
    run.layout_size = Some(((hit.width - 2.0 - indent).max(0.0), 18.0));
}

pub(super) fn render_document(
    state: &ReviewWorkspaceState,
    context: &InspectorContext<'_>,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    super::inspector_chrome::push_inspector_title_band(
        rect,
        &context.title,
        "Read-only inspection",
        false,
        quads,
        text,
    )?;
    let body = RectPx {
        y: rect.y + 76.0,
        height: (rect.height - 76.0).max(0.0),
        ..rect
    };
    let visible = (body.height / 22.0).floor() as usize;
    if visible == 0 {
        return Ok(());
    }
    let rows = rows(context, &state.inspector_context, rect.width)?;
    let start = state
        .ui
        .selection_inspector
        .scroll_offset
        .min(rows.len().saturating_sub(visible));
    hits.push(HitRegion {
        rect: body,
        target: HitTarget::InspectorContextScroll {
            total: rows.len(),
            visible,
            basis: context.action_basis.clone(),
        },
    });
    hits.push(HitRegion {
        rect: body,
        target: HitTarget::InspectorSectionNavigation {
            basis: datum_gui_protocol::inspector_context::InspectorNavigationBasis::for_context(
                state, context, None,
            ),
            offsets: offsets(&rows),
            total: rows.len(),
            visible,
        },
    });
    for (index, row) in rows.into_iter().enumerate().skip(start).take(visible) {
        paint_row(
            rect, body, index, start, visible, &row.label, row.target, row.active, true, quads,
            text, hits,
        );
    }
    Ok(())
}
