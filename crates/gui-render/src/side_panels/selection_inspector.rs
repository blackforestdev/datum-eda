//! Shared complete selection Inspector; navigation never alters membership.
use super::inspector_chrome::push_inspector_title_band;
use super::*;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class,
    selection_view::{InspectionValue, SelectionView, inspector_state::SelectionInspectorState},
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
) -> anyhow::Result<()> {
    let title = if view.members.len() == 1 {
        "Object Selection".to_owned()
    } else if view.subject.kind() == "compound" {
        format!("Compound Selection — {} objects", view.members.len())
    } else {
        format!("{} — {} objects", view.subject.kind(), view.members.len())
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
        return Ok(());
    }
    let mut counts = std::collections::BTreeMap::new();
    for id in &view.members {
        *counts.entry(id.class).or_insert(0usize) += 1;
    }
    let fields = view.fields();
    let total = 1
        + usize::from(view.stale)
        + usize::from(view.focus().is_some())
        + usize::from(view.notice.is_some())
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
    let row_index = std::cell::Cell::new(0usize);
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
        if active {
            push_rect_border(quads, hit, TEXT_ACCENT, 1.0);
        }
        if let Some(target) = target {
            hits.push(HitRegion { rect: hit, target });
        }
        draw_text(
            label,
            rect.x + 12.0,
            y + 3.0,
            11.0,
            if active { TEXT_ACCENT } else { TEXT_PRIMARY },
            TextFace::Ui,
            text,
        );
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
    if view.stale {
        row(
            "STALE: previous basis; current geometry unavailable",
            None,
            false,
        );
    }
    if let Some(focus) = view.focus() {
        row(
            &format!("Focus: {:?} {}", focus.class, focus.id),
            None,
            false,
        );
    }
    if let Some(notice) = &view.notice {
        row(notice, None, false);
    }
    row(
        "Unavailable: Move / Rotate / Mirror / Lock / Group (S5A)",
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
        let value = match view.field(field) {
            InspectionValue::Common(v) => v.to_string(),
            InspectionValue::Mixed => "Mixed".into(),
            InspectionValue::Unavailable(reason) => format!("Unavailable — {reason}"),
        };
        row(
            &format!("{}: {value}", field.replace('_', " ")),
            None,
            false,
        );
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
        for id in view
            .scoped_members()
            .skip(skip)
            .take(end.saturating_sub(skip))
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
                format!(" · occurrence {:?}", id.instance_path)
            };
            row(
                &format!("{} {}{occurrence}{status}", class_label(id.class), id.id),
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
    Ok(())
}
const CLASSES: [Class; 18] = [
    Class::Footprint,
    Class::Pad,
    Class::Track,
    Class::Via,
    Class::Zone,
    Class::BoardText,
    Class::BoardGraphic,
    Class::BoardOutline,
    Class::Symbol,
    Class::Pin,
    Class::Wire,
    Class::BusSection,
    Class::Label,
    Class::Port,
    Class::Junction,
    Class::NoConnect,
    Class::SchematicText,
    Class::SchematicGraphic,
];
fn class_label(class: Class) -> &'static str {
    match class {
        Class::Footprint => "Parts",
        Class::Pad => "Pads",
        Class::Track => "Tracks",
        Class::Via => "Vias",
        Class::Zone => "Zones",
        Class::BoardText => "Board text",
        Class::BoardGraphic => "Board graphics",
        Class::BoardOutline => "Board outline",
        Class::Symbol => "Symbols",
        Class::Pin => "Pins",
        Class::Wire => "Wires",
        Class::BusSection => "Bus sections",
        Class::Label => "Labels",
        Class::Port => "Ports",
        Class::Junction => "Junctions",
        Class::NoConnect => "No-connects",
        Class::SchematicText => "Schematic text",
        Class::SchematicGraphic => "Schematic graphics",
    }
}

#[cfg(test)]
#[path = "selection_inspector/tests.rs"]
mod tests;
