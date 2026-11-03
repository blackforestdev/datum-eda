//! Qualified findings configure the existing read-only Inspector chrome and rows.
use super::*;
use datum_gui_protocol::inspector_context::finding::FindingInspection;
use datum_gui_protocol::selection_view::inspector_state::InspectorPresentationBasis;

pub(super) fn render_check_finding_inspector(
    state: &ReviewWorkspaceState,
    finding: Result<FindingInspection<'_>, &'static str>,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    inspector_chrome::push_inspector_title_band(
        rect,
        "Check finding",
        "Read-only inspection",
        true,
        quads,
        text,
    )?;
    let body = RectPx {
        x: rect.x,
        y: rect.y + 76.,
        width: rect.width,
        height: (rect.height - 76.).max(0.),
    };
    let visible = (body.height / 22.).floor() as usize;
    if visible == 0 {
        return Ok(());
    }
    let mut rows: Vec<(String, Option<HitTarget>)> = Vec::new();
    let mut paragraph = |label: &str, value: &str| -> anyhow::Result<()> {
        if value.is_empty() {
            return Ok(());
        }
        let value = if label.is_empty() {
            value.to_owned()
        } else {
            format!("{label}: {value}")
        };
        rows.extend(
            crate::control_text::wrap_text_words_sized(
                &value,
                (body.width - 24.).max(1.),
                11.,
                TextFace::Ui,
            )?
            .into_iter()
            .map(|line| (line, None)),
        );
        Ok(())
    };
    match finding {
        Err(reason) => {
            paragraph("Unavailable", reason)?;
        }
        Ok(view) => {
            let f = view.finding;
            paragraph(
                "",
                &format!(
                    "{} · {}",
                    match f.severity.as_str() {
                        "error" => "Error",
                        "warning" => "Warning",
                        "info" => "Info",
                        _ => "Severity unavailable",
                    },
                    f.rule_id.replace('_', " ")
                ),
            )?;
            paragraph("", view.currentness)?;
            if let Some(revision) = view.report.model_revision.as_deref() {
                paragraph(
                    "Describes revision",
                    &revision.chars().take(12).collect::<String>(),
                )?;
            }
            if let Some(target) = &view.target {
                paragraph("Affected", target)?;
            }
            if let Some(occurrence) = &view.occurrence {
                paragraph("Occurrence", occurrence)?;
            }
            paragraph("", &f.message)?;
            if f.explanation != f.message {
                paragraph("", &f.explanation)?;
            }
            paragraph("Status", &f.status.replace('_', " "))?;
            for detail in view.disposition_details() {
                paragraph(&detail.kind, &detail.rationale)?;
            }
            if (!f.waiver_refs.is_empty() || !f.deviation_refs.is_empty())
                && view.disposition_details().next().is_none()
            {
                paragraph("Rationale", "Unavailable for this producing report")?;
            }
            if let Some(basis) = f.standards_basis_label() {
                paragraph("Standards basis", &basis)?;
            }
            if let Some(next) = &f.suggested_next_action {
                paragraph("Next", next)?;
            }
            if f.domain == "erc"
                && f.status == "active"
                && view.currentness == "Current persisted result"
            {
                paragraph("", "Checks → Waive / Accept Deviation…")?;
            }
            // Reuse the existing expansion state and hit; IDs are details, not the headline.
            let details = state.ui.selection_inspector.inventory_expanded;
            paragraph(
                "",
                if details {
                    "▾ Technical details"
                } else {
                    "▸ Technical details"
                },
            )?;
            let detail_row = rows.len().saturating_sub(1);
            rows[detail_row].1 = Some(HitTarget::SelectionInventoryToggle);
            if details {
                // The paragraph closure borrows rows, so detailed rows use the same wrapper below.
                for (label, value) in [
                    (
                        "CheckRun",
                        view.report
                            .check_run_id
                            .as_deref()
                            .unwrap_or("Unavailable")
                            .to_owned(),
                    ),
                    (
                        "Producing revision",
                        view.report
                            .model_revision
                            .as_deref()
                            .unwrap_or("Unavailable")
                            .to_owned(),
                    ),
                    ("Fingerprint", f.fingerprint.clone()),
                    (
                        "Finding",
                        f.finding_id.clone().unwrap_or_else(|| "Unavailable".into()),
                    ),
                    ("Target identity", f.primary_target.to_string()),
                    ("Waiver references", f.waiver_refs.join(", ")),
                    ("Deviation references", f.deviation_refs.join(", ")),
                ] {
                    if !value.is_empty() {
                        rows.extend(
                            crate::control_text::wrap_text_words_sized(
                                &format!("{label}: {value}"),
                                (body.width - 24.).max(1.),
                                11.,
                                TextFace::Ui,
                            )?
                            .into_iter()
                            .map(|line| (line, None)),
                        );
                    }
                }
            }
        }
    }
    let total = rows.len();
    let start = state
        .ui
        .selection_inspector
        .scroll_offset
        .min(total.saturating_sub(visible));
    let target = if let SelectionTarget::Native(view) = &state.selection {
        HitTarget::SelectionInspectorScroll {
            total,
            visible,
            sidebar: [0; 4],
            visible_members: Vec::new(),
            basis: InspectorPresentationBasis::capture(view, &state.ui.selection_inspector),
        }
    } else {
        HitTarget::InspectorContextScroll {
            total,
            visible,
            basis: datum_gui_protocol::inspector_context::InspectorActionBasis::capture(state),
        }
    };
    hits.push(HitRegion { rect: body, target });
    for (i, (line, action)) in rows.iter().skip(start).take(visible).enumerate() {
        if let Some(target) = action {
            // The shared painter draws disclosure geometry with the governed
            // UI font; a Unicode triangle here would render as a missing glyph.
            inspector_sections::paint_row(
                rect,
                body,
                i,
                0,
                visible,
                line,
                None,
                Some(target.clone()),
                false,
                true,
                quads,
                text,
                hits,
            );
            continue;
        }
        let row = RectPx {
            x: body.x + 12.,
            y: body.y + i as f32 * 22.,
            width: (body.width - 24.).max(0.),
            height: 22.,
        };
        draw_text(
            line,
            row.x,
            row.y + 4.,
            11.,
            TEXT_SECONDARY,
            TextFace::Ui,
            text,
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "inspector_check_finding_tests.rs"]
mod tests;
