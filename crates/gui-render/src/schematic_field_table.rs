//! Active-sheet rows use the existing field painter, focus and Proposal controls.
use super::*;
use datum_gui_protocol::schematic_authoring::field_table::FieldTableControl as C;

pub(super) fn render(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    let Some(review) = &state.schematic_authoring.delete_review else {
        return Ok(());
    };
    let Some(table) = &review.fields else {
        return Ok(());
    };
    let width = (layout.top_menu_bar.width - 24.).min(760.).max(160.);
    let y = layout.top_menu_bar.y + layout.top_menu_bar.height + 16.;
    let available = layout.status_bar.y - y - 12.;
    let rows = (((available - 194.) / 36.).floor() as usize)
        .clamp(1, 12)
        .min(table.rows.len());
    let desired_height = 194. + rows as f32 * 36.;
    let panel = RectPx {
        x: (layout.top_menu_bar.width - width) / 2.,
        y,
        width,
        height: desired_height.min(available.max(0.)),
    };
    quads.push(Quad::from_rect(panel, design_tokens::chrome::SURFACE_01));
    push_rect_border(quads, panel, design_tokens::chrome::BORDER_STRONG, 1.);
    hits.push(HitRegion {
        target: HitTarget::SchematicPicker,
        rect: panel,
    });
    draw_text(
        "Symbol Fields Table",
        panel.x + 16.,
        y + 15.,
        16.,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    draw_text(
        &truncate_text(
            &format!(
                "Active sheet: {} · {} Symbols",
                review.source.name,
                table.rows.len()
            ),
            ((width - 32.) / 7.).max(1.) as usize,
        ),
        panel.x + 16.,
        y + 40.,
        11.,
        TEXT_MUTED,
        TextFace::Ui,
        text,
    );
    if available < desired_height || width < 300. {
        draw_text(
            "Enlarge the window to view the table",
            panel.x + 16.,
            y + 65.,
            11.,
            TEXT_MUTED,
            TextFace::Ui,
            text,
        );
        button(
            "Cancel",
            HitTarget::SchematicFieldTable(C::Cancel),
            RectPx {
                x: panel.x + 16.,
                y: y + 92.,
                width: 90.,
                height: 28.,
            },
            quads,
            text,
            hits,
        );
        return Ok(());
    }
    let reference_width = (width - 40.) * 0.35;
    let value_x = panel.x + 24. + reference_width;
    for (label, x) in [
        (
            if table.editable() {
                "Reference"
            } else {
                "Reference · before → after"
            },
            panel.x + 16.,
        ),
        (
            if table.editable() {
                "Value"
            } else {
                "Value · before → after"
            },
            value_x,
        ),
    ] {
        draw_text(label, x, y + 65., 11., TEXT_MUTED, TextFace::Ui, text);
    }
    let start = table.offset.min(table.rows.len().saturating_sub(1));
    let end = (start + rows).min(table.rows.len());
    let mut controls = ControlMeshCache::default();
    for (visible, row) in (start..end).enumerate() {
        for (field, x, cell_width) in [
            (PlacementField::Reference, panel.x + 16., reference_width),
            (
                PlacementField::Value,
                value_x,
                width - reference_width - 40.,
            ),
        ] {
            let (value, editor) = table.cell(row, field).unwrap();
            crate::single_line_control::paint(
                &value,
                "",
                editor,
                table.focus_row == row
                    && table.field == field
                    && state.ui.active_menu.is_none()
                    && state.ui.focus
                        == datum_gui_protocol::ApplicationFocus::Editor(state.ui.layout.focused),
                table.editable() && review.pending.is_none() && review.current(state),
                RectPx {
                    x,
                    y: y + 84. + visible as f32 * 36.,
                    width: cell_width,
                    height: 29.,
                },
                &mut ControlPainter::new(quads, &mut controls, 1.),
                text,
                hits,
                HitTarget::SchematicFieldTable(C::Cell(row, field)),
            )?;
        }
    }
    let footer = y + 84. + rows as f32 * 36.;
    if start > 0 {
        button(
            "Previous",
            HitTarget::SchematicFieldTable(C::Page(start.saturating_sub(rows))),
            RectPx {
                x: panel.x + 16.,
                y: footer + 6.,
                width: 90.,
                height: 27.,
            },
            quads,
            text,
            hits,
        );
    }
    if end < table.rows.len() {
        button(
            "Next",
            HitTarget::SchematicFieldTable(C::Page(end)),
            RectPx {
                x: panel.x + 112.,
                y: footer + 6.,
                width: 70.,
                height: 27.,
            },
            quads,
            text,
            hits,
        );
    }
    draw_text(
        &format!("Rows {}–{} of {}", start + 1, end, table.rows.len()),
        panel.x + 198.,
        footer + 14.,
        11.,
        TEXT_MUTED,
        TextFace::Ui,
        text,
    );
    let status = review.refusal.clone().unwrap_or_else(|| {
        if review.pending.is_some() {
            "Waiting for Proposal acknowledgement…".into()
        } else if review.finished() {
            "Proposal completed · Close to return to the sheet".into()
        } else if table.editable() {
            "Tab: next cell · Page Up/Down: rows · Enter: review · Esc: cancel".into()
        } else {
            "Up/Down: rows · F6: column · Home/End: full text · Tab: accept · Esc: dismiss".into()
        }
    });
    draw_text(
        &truncate_text(&status, ((width - 32.) / 6.5).max(1.) as usize),
        panel.x + 16.,
        footer + 46.,
        11.,
        if review.refusal.is_some() {
            design_tokens::chrome::STATUS_ERROR
        } else {
            TEXT_MUTED
        },
        TextFace::Ui,
        text,
    );
    button(
        if review.finished() { "Close" } else { "Cancel" },
        HitTarget::SchematicFieldTable(C::Cancel),
        RectPx {
            x: panel.x + 16.,
            y: footer + 70.,
            width: 90.,
            height: 28.,
        },
        quads,
        text,
        hits,
    );
    let action = if table.editable() {
        Some(("Review changes", C::Review))
    } else if review.ready() && review.current(state) {
        Some(("Accept changes", C::Accept))
    } else {
        None
    };
    if let Some((label, control)) = action.filter(|_| review.pending.is_none()) {
        button(
            label,
            HitTarget::SchematicFieldTable(control),
            RectPx {
                x: panel.x + width - 166.,
                y: footer + 70.,
                width: 150.,
                height: 28.,
            },
            quads,
            text,
            hits,
        );
    }
    Ok(())
}
