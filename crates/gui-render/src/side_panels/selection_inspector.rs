//! Shared complete selection Inspector; navigation never alters membership.
use super::inspector_chrome::push_inspector_title_band;
use super::*;
use datum_gui_protocol::selection_view::inspector_state::{
    ACQUISITION_CLASSES as CLASSES, class_label,
};
use datum_gui_protocol::{
    AuthoredSelectionClass as Class,
    selection_view::{SelectionView, inspector_state::SelectionInspectorState},
};
use std::collections::BTreeSet;

#[allow(clippy::too_many_arguments)]
pub(super) fn render(
    view: &SelectionView,
    navigation: &SelectionInspectorState,
    excluded: &BTreeSet<Class>,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<Option<RectPx>> {
    let title = if view.subject.kind() == "object" {
        view.members
            .iter()
            .next()
            .map(|id| view.member_display_label(id, 1))
            .unwrap_or_else(|| "Selection".into())
    } else if view.subject.kind() == "compound" {
        format!("Compound Selection — {} objects", view.members.len())
    } else {
        format!(
            "{} — {} objects",
            view.subject.display_name(),
            view.members.len()
        )
    };
    push_inspector_title_band(rect, &title, "Read-only inspection", true, quads, text)?;
    let body = RectPx {
        x: rect.x,
        y: rect.y + 76.0,
        width: rect.width,
        height: (rect.height - 76.0).max(0.0),
    };
    let visible = (body.height / 22.0).floor() as usize;
    if visible == 0 {
        return Ok(None);
    }
    let mut counts = std::collections::BTreeMap::new();
    for id in &view.members {
        *counts.entry(id.class).or_insert(0usize) += 1;
    }
    let fields = view.fields();
    let total = 2
        + usize::from(view.stale)
        + usize::from(view.focus().is_some())
        + usize::from(view.notice.is_some())
        + usize::from(!view.related.is_empty())
        + 1
        + 1
        + counts.len()
        + fields.len()
        + 1
        + if navigation.inventory_expanded {
            view.scoped_members().count()
        } else {
            0
        }
        + 1
        + if navigation.filters_expanded {
            CLASSES.len() + 1
        } else {
            0
        };
    let start = navigation.scroll_offset.min(total.saturating_sub(visible));
    hits.push(HitRegion {
        rect: body,
        target: HitTarget::SelectionInspectorScroll { total, visible },
    });
    let columns = ((rect.width - 24.0).max(0.0) / (11.0 * 0.58)).floor() as usize;
    let row_index = std::cell::Cell::new(0usize);
    let visual_row = std::cell::Cell::new(None);
    let mut row = |label: &str, target: Option<HitTarget>, active: bool| {
        let index = row_index.get();
        row_index.set(index + 1);
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
        // Row one is reserved for the final camera-qualified presentation
        // summary. Return typed chrome geometry, never identify rows by text.
        if index == 1 {
            visual_row.set(Some(hit));
            return;
        }
        if active {
            push_rect_border(quads, hit, TEXT_ACCENT, 1.0);
        }
        if let Some(target) = target {
            hits.push(HitRegion { rect: hit, target });
        }
        let label = truncate_text(label, columns);
        draw_text(
            &label,
            rect.x + 12.0,
            y + 3.0,
            11.0,
            if active { TEXT_ACCENT } else { TEXT_PRIMARY },
            TextFace::Ui,
            text,
        );
        let run = text.last_mut().expect("visible Inspector row");
        run.clip_bounds = Some(hit);
        run.layout_size = Some(((hit.width - 2.0).max(0.0), 18.0));
    };
    row(
        &format!(
            "{} hidden · {} locked · {} unprojected",
            view.hidden.len(),
            view.locked.len(),
            view.unprojected.len()
        ),
        None,
        false,
    );
    row("", None, false);
    if view.stale {
        row(
            "STALE: previous basis; current geometry unavailable",
            None,
            false,
        );
    }
    if let Some(focus) = view.focus() {
        row(
            &format!(
                "Focus: {}",
                view.member_display_label(
                    focus,
                    view.members
                        .iter()
                        .position(|id| id == focus)
                        .map_or(0, |index| index + 1)
                )
            ),
            None,
            false,
        );
    }
    if let Some(notice) = &view.notice {
        row(notice, None, false);
    }
    if !view.related.is_empty() {
        row(
            &format!("Related {} · normal appearance", view.related.len()),
            None,
            false,
        );
    }
    row(
        "Move / Rotate / Mirror / Lock / Group: unavailable",
        None,
        false,
    );
    row(
        &format!("All {}", view.members.len()),
        Some(HitTarget::SelectionScope(None)),
        view.scope.is_none(),
    );
    for (class, n) in counts {
        row(
            &format!("{} {n}", class_label(class)),
            Some(HitTarget::SelectionScope(Some(class))),
            view.scope == Some(class),
        );
    }
    for field in fields {
        if row_index.get() < start || row_index.get() >= start + visible {
            row("", None, false);
            continue;
        }
        let name = truncate_text(
            &datum_gui_protocol::selection_view::field_label(field),
            columns / 2,
        );
        let value =
            view.field_display_preview(field, columns.saturating_sub(name.chars().count() + 2));
        row(&format!("{name}: {value}"), None, false);
    }
    row(
        &format!(
            "{} Members {} (complete scope)",
            if navigation.inventory_expanded {
                "▾"
            } else {
                "▸"
            },
            view.scoped_members().count()
        ),
        Some(HitTarget::SelectionInventoryToggle),
        false,
    );
    if navigation.inventory_expanded {
        let inventory_start = row_index.get();
        let count = view.scoped_members().count();
        // Do not format or allocate labels for clipped inventory rows.
        let skip = start.saturating_sub(inventory_start).min(count);
        row_index.set(row_index.get() + skip);
        let end = (start + visible).saturating_sub(inventory_start).min(count);
        for (index, id) in view
            .scoped_members()
            .skip(skip)
            .take(end.saturating_sub(skip))
            .enumerate()
        {
            let mut status = String::new();
            if view.hidden.contains(id) {
                status.push_str(" · hidden");
            }
            if view.locked.contains(id) {
                status.push_str(" · locked");
            }
            if view.unprojected.contains(id) {
                status.push_str(" · unprojected");
            }
            let unavailable = view.unavailable_fields(id);
            if !unavailable.is_empty() {
                status.push_str(&format!(" · unavailable: {}", unavailable.join(", ")));
            }
            let occurrence = if id.instance_path.is_empty() {
                String::new()
            } else {
                format!(
                    " · sheet path {}",
                    id.instance_path
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" / ")
                )
            };
            row(
                &format!(
                    "{} · {}{occurrence}{status}",
                    view.member_display_label(id, skip + index + 1),
                    id.id
                ),
                None,
                false,
            );
        }
        row_index.set(inventory_start + count);
    }
    row(
        &format!(
            "{} Acquisition filters",
            if navigation.filters_expanded {
                "▾"
            } else {
                "▸"
            }
        ),
        Some(HitTarget::SelectionFiltersToggle),
        false,
    );
    if navigation.filters_expanded {
        row(
            "Future acquisition only; Ctrl+A bypasses filters",
            None,
            false,
        );
        for class in CLASSES {
            row(
                &format!(
                    "{} {}",
                    if excluded.contains(&class) {
                        "[ ]"
                    } else {
                        "[x]"
                    },
                    class_label(class)
                ),
                Some(HitTarget::SelectionClassToggle(class)),
                false,
            );
        }
    }
    if total > visible {
        draw_text(
            &format!("{}–{} / {total}", start + 1, (start + visible).min(total)),
            rect.x + rect.width - 96.0,
            rect.y + 56.0,
            9.0,
            TEXT_MUTED,
            TextFace::Mono,
            text,
        );
    }
    Ok(visual_row.get())
}

#[cfg(test)]
#[path = "selection_inspector/tests.rs"]
mod tests;
