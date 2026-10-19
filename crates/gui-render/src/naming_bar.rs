//! Reusable inline naming bar over the shared editor; no new native window.
use super::*;
use crate::global_preferences_primitives::{
    ControlMeshCache, ControlPainter, push_dashed_rect_border, push_rounded_rect_with_border,
};
use datum_gui_protocol::{FolderBrowserFocus as Focus, NamingDraft};

pub(crate) struct Plan {
    pub bounds: RectPx,
    pub editor: RectPx,
    pub cancel: RectPx,
    pub submit: RectPx,
    title: Vec<String>,
    reason: Vec<String>,
}
impl Plan {
    pub fn new(draft: &NamingDraft, mut bounds: RectPx) -> anyhow::Result<Self> {
        let title = crate::control_text::wrap_text_sized(
            &draft.title(),
            bounds.width - 22.0,
            11.5,
            TextFace::Ui,
        )?;
        let reason = draft
            .issue()
            .map(|s| {
                crate::control_text::wrap_text_sized(
                    &format!("⊘ {s}"),
                    bounds.width - 22.0,
                    12.0,
                    TextFace::Ui,
                )
            })
            .transpose()?
            .unwrap_or_default();
        // .nbar border+padding18, header15/line+6, row34; refusal6+18/line.
        bounds.height = 18.0
            + title.len().max(1) as f32 * 15.0
            + 6.0
            + 34.0
            + if reason.is_empty() {
                0.0
            } else {
                6.0 + reason.len() as f32 * 18.0
            };
        let y = bounds.y + 9.0 + title.len().max(1) as f32 * 15.0 + 6.0;
        let submit_width =
            measured_text_run_width_px(draft.action_label(), 12.5, TextFace::Ui)? + 30.0;
        let cancel_width = measured_text_run_width_px("Cancel", 12.5, TextFace::Ui)? + 30.0;
        let submit = RectPx {
            x: bounds.x + bounds.width - 11.0 - submit_width,
            y,
            width: submit_width,
            height: 34.0,
        };
        let cancel = RectPx {
            x: submit.x - 8.0 - cancel_width,
            y,
            width: cancel_width,
            height: 34.0,
        };
        let editor = RectPx {
            x: bounds.x + 11.0,
            y,
            width: (cancel.x - 8.0 - bounds.x - 11.0).max(1.0),
            height: 34.0,
        };
        Ok(Self {
            bounds,
            editor,
            cancel,
            submit,
            title,
            reason,
        })
    }
}
fn shift(rect: RectPx, dy: f32) -> RectPx {
    RectPx {
        y: rect.y + dy,
        ..rect
    }
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    draft: &NamingDraft,
    plan: &Plan,
    dy: f32,
    focus: Option<Focus>,
    controls: &mut ControlMeshCache,
    scale: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    let mut painter = ControlPainter::new(quads, controls, scale);
    let bounds = shift(plan.bounds, dy);
    push_rounded_rect_with_border(
        &mut painter,
        bounds,
        design_tokens::chrome::SURFACE_01,
        design_tokens::chrome::BORDER_STRONG,
        1.0,
        6.0,
    );
    for (i, line) in plan.title.iter().enumerate() {
        draw_text(
            line,
            bounds.x + 11.0,
            bounds.y + 9.0 + i as f32 * 15.0,
            11.5,
            TEXT_SECONDARY,
            TextFace::Ui,
            text,
        );
    }
    crate::single_line_control::paint(
        &draft.value,
        draft.field_name(),
        draft.editor,
        focus == Some(Focus::NameEditor),
        !draft.busy,
        shift(plan.editor, dy),
        &mut painter,
        text,
        hits,
        HitTarget::FolderBrowser(Focus::NameEditor),
    )?;
    for (label, rect, control, available) in [
        ("Cancel", plan.cancel, Focus::NameCancel, !draft.busy),
        (
            draft.action_label(),
            plan.submit,
            Focus::NameSubmit,
            draft.enabled(),
        ),
    ] {
        let rect = shift(rect, dy);
        let border = if focus == Some(control) {
            design_tokens::chrome::STATUS_INFO
        } else {
            design_tokens::chrome::BORDER_STRONG
        };
        if available {
            push_rounded_rect_with_border(
                &mut painter,
                rect,
                design_tokens::chrome::SURFACE_02,
                border,
                if focus == Some(control) { 2.0 } else { 1.0 },
                5.0,
            );
        } else {
            painter.push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_02));
            push_dashed_rect_border(&mut painter, rect, border);
        }
        draw_text(
            label,
            rect.x + 15.0,
            rect.y + 10.0,
            12.5,
            if available { TEXT_PRIMARY } else { TEXT_MUTED },
            TextFace::Ui,
            text,
        );
        hits.push(HitRegion {
            target: HitTarget::FolderBrowser(control),
            rect,
        });
    }
    for (i, line) in plan.reason.iter().enumerate() {
        draw_text(
            line,
            bounds.x + 11.0,
            plan.editor.y + dy + 40.0 + i as f32 * 18.0,
            12.0,
            if i == 0 {
                design_tokens::chrome::STATUS_ERROR
            } else {
                TEXT_SECONDARY
            },
            TextFace::Ui,
            text,
        );
    }
    Ok(())
}
