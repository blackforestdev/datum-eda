//! Shared section rows and one clipped, virtual Inspector row painter.
use super::*;
use datum_gui_protocol::inspector_context::{
    InspectorContext, InspectorNavigation, InspectorSectionId,
};

pub(super) struct Row<'a> {
    pub label: String,
    pub native_field: Option<&'a str>,
    pub target: Option<HitTarget>,
    pub active: bool,
    /// Two-column key/value grid cell: the value-column text for this line plus
    /// the panel-wide key-column width, so key/value rows align and long values
    /// wrap within the value column instead of mid-word across the full width.
    pub value: Option<(String, f32)>,
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

/// Gap between the key and value columns in the Inspector key/value grid.
const KV_GAP: f32 = 8.0;

/// Width of the Inspector key column: the widest currently expanded key (min
/// 100px) plus padding, capped at 45% of the panel so a long key never starves
/// the value column. Every expanded field participates, including SelectionField
/// identity rows, so the whole section aligns on one grid.
fn key_column_width(
    context: &InspectorContext<'_>,
    navigation: &InspectorNavigation,
    width: f32,
) -> anyhow::Result<f32> {
    const KEY_MIN: f32 = 100.0;
    const KEY_PAD: f32 = 10.0;
    let mut key = KEY_MIN;
    for section in &context.sections {
        if navigation.collapsed.contains(&section.id) {
            continue;
        }
        for field in &section.fields {
            let measured =
                crate::control_text::measure_text_width(&field.label, 11.0, TextFace::Ui)?
                    + KEY_PAD;
            key = key.max(measured);
        }
    }
    Ok(key.min((width * 0.45).max(KEY_MIN)))
}

pub(super) fn rows<'a>(
    context: &InspectorContext<'a>,
    navigation: &InspectorNavigation,
    width: f32,
) -> anyhow::Result<Vec<Row<'a>>> {
    let mut rows = Vec::new();
    // Key/value grid geometry (prototype .kv: grid-template-columns:100px 1fr).
    // The key column is sized to the widest key across the currently expanded
    // fields — min 100px, capped at 45% of the panel — so native keys longer
    // than the prototype's 100px placeholder (e.g. "Source materialization")
    // still align without crowding out the value column.
    let key_col = key_column_width(context, navigation, width)?;
    let value_col = (width - 24.0 - key_col - KV_GAP).max(1.0);
    for section in &context.sections {
        let collapsed = navigation.collapsed.contains(&section.id);
        rows.push(Row {
            // DEV-C: section headers render UPPERCASE to match the prototype's
            // `.sect-hd { text-transform:uppercase }` and the already-uppercased
            // panel header; the protocol `label()` values stay unchanged.
            label: format!(
                "{} {}{}",
                if collapsed { "▸" } else { "▾" },
                section.id.label().to_uppercase(),
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
            value: None,
        });
        if collapsed {
            continue;
        }
        for field in &section.fields {
            // DEV-D: every value row — including the SelectionField identity
            // rows (library package/part, component bindings) — is an aligned
            // key/value grid cell. `context.field_text` resolves a SelectionField
            // through the live view at full width (usize::MAX = no elision), so no
            // identity UUID is truncated; the value wraps within the value column,
            // its first line carrying the key and continuation lines indenting
            // under the value column.
            let value = context.field_text(field, usize::MAX);
            let lines =
                crate::control_text::wrap_text_words_sized(&value, value_col, 11.0, TextFace::Ui)?;
            for (index, line) in lines.into_iter().enumerate() {
                rows.push(Row {
                    label: if index == 0 {
                        field.label.to_string()
                    } else {
                        String::new()
                    },
                    native_field: None,
                    target: None,
                    active: false,
                    value: Some((line, key_col)),
                });
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
                value: None,
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
    value: Option<(&str, f32)>,
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
    if let Some((value_text, key_col)) = value {
        // DEV-D two-column key/value grid: key in the left column (width-limited
        // to the key column), value in the right column. Each value line was
        // pre-wrapped to the value column, so continuation lines align under it.
        let color = if active { TEXT_ACCENT } else { TEXT_PRIMARY };
        let key_x = rect.x + 12.0;
        let value_x = key_x + key_col + KV_GAP;
        if !label.is_empty() {
            draw_text(label, key_x, y + 3.0, 11.0, color, TextFace::Ui, text);
            let run = text.last_mut().expect("visible Inspector key");
            run.clip_bounds = Some(hit);
            run.layout_size = Some((key_col.max(0.0), 18.0));
        }
        draw_text(
            value_text,
            value_x,
            y + 3.0,
            11.0,
            color,
            TextFace::Ui,
            text,
        );
        let run = text.last_mut().expect("visible Inspector value");
        run.clip_bounds = Some(hit);
        run.layout_size = Some(((rect.x + rect.width - 12.0 - value_x).max(0.0), 18.0));
        if let Some(target) = target {
            hits.push(HitRegion { rect: hit, target });
        }
        return;
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
            rect,
            body,
            index,
            start,
            visible,
            &row.label,
            row.value.as_ref().map(|(v, w)| (v.as_str(), *w)),
            row.target,
            row.active,
            true,
            quads,
            text,
            hits,
        );
    }
    Ok(())
}
