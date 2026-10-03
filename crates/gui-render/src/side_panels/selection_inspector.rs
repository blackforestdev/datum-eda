//! Shared complete selection Inspector; scope is independent of canvas membership.
use super::inspector_chrome::push_inspector_title_band;
use super::*;
use datum_gui_protocol::selection_view::{InspectionValue, SelectionView};
pub(super) fn render(
    view: &SelectionView,
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
    let mut y = rect.y + 76.0;
    let mut row = |label: &str, text: &mut Vec<TextRun>| {
        draw_text(
            label,
            rect.x + 12.0,
            y,
            11.5,
            TEXT_PRIMARY,
            TextFace::Ui,
            text,
        );
        y += 22.0;
    };
    row(
        &format!(
            "{} hidden · {} locked · {} unprojected",
            view.hidden.len(),
            view.locked.len(),
            view.unprojected.len()
        ),
        text,
    );
    if view.stale {
        row("STALE: previous basis; current geometry unavailable", text);
    }
    if let Some(focus) = view.focus() {
        row(&format!("Focus: {:?} {}", focus.class, focus.id), text);
    }
    if let Some(notice) = &view.notice {
        row(notice, text);
    }
    row(
        "Unavailable: Move / Rotate / Mirror / Lock / Group (S5A)",
        text,
    );
    let mut counts = std::collections::BTreeMap::new();
    for id in &view.members {
        *counts.entry(id.class).or_insert(0usize) += 1;
    }
    for (scope, label) in std::iter::once((None, format!("All {}", view.members.len()))).chain(
        counts
            .into_iter()
            .map(|(class, n)| (Some(class), format!("{class:?} {n}"))),
    ) {
        let current = view.scope == scope;
        let target = RectPx {
            x: rect.x + 10.0,
            y: y - 3.0,
            width: (rect.width - 20.0).max(0.0),
            height: 22.0,
        };
        if current {
            push_rect_border(quads, target, TEXT_ACCENT, 1.0);
        }
        hits.push(HitRegion {
            rect: target,
            target: HitTarget::SelectionScope(scope),
        });
        draw_text(
            &label,
            rect.x + 12.0,
            y,
            11.5,
            if current { TEXT_ACCENT } else { TEXT_PRIMARY },
            TextFace::Ui,
            text,
        );
        y += 24.0;
    }
    for field in view.fields() {
        let value = match view.field(field) {
            InspectionValue::Common(v) => v.to_string(),
            InspectionValue::Mixed => "Mixed".into(),
            InspectionValue::Unavailable(reason) => format!("Unavailable — {reason}"),
        };
        draw_text(
            &format!("{field}: {value}"),
            rect.x + 12.0,
            y,
            11.0,
            TEXT_PRIMARY,
            TextFace::Ui,
            text,
        );
        y += 22.0;
    }
    draw_text(
        &format!("Members {} (complete scope)", view.scoped_members().count()),
        rect.x + 12.0,
        y,
        11.5,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    y += 24.0;
    // Render only visible rows; complete inventory lives in the projection, not this viewport.
    for id in view.scoped_members() {
        if y + 16.0 > rect.y + rect.height {
            break;
        }
        let status = if view.hidden.contains(id) {
            " · hidden"
        } else if view.locked.contains(id) {
            " · locked"
        } else {
            ""
        };
        draw_text(
            &format!("{:?} {}{status}", id.class, id.id),
            rect.x + 12.0,
            y,
            11.0,
            TEXT_PRIMARY,
            TextFace::Mono,
            text,
        );
        y += 20.0;
    }
    Ok(())
}
