//! Small label form using the same field, focus, and gesture controls as symbols.
use super::*;

pub(super) fn render(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    let e = &state.schematic_authoring;
    let Some(label) = &e.label else { return Ok(()) };
    let width = 440f32.min(layout.top_menu_bar.width - 32.);
    let panel = RectPx {
        x: (layout.top_menu_bar.width - width) / 2.,
        y: 76.,
        width,
        height: if e.picker { 214. } else { 66. },
    };
    quads.push(Quad::from_rect(panel, design_tokens::chrome::SURFACE_01));
    push_rect_border(quads, panel, design_tokens::chrome::BORDER_STRONG, 1.);
    hits.push(HitRegion {
        target: HitTarget::SchematicPicker,
        rect: panel,
    });
    if !e.picker {
        draw_text(
            "Click to place · Tab: edit name/kind · Esc: cancel",
            panel.x + 12.,
            panel.y + 12.,
            12.,
            TEXT_PRIMARY,
            TextFace::UiStrong,
            text,
        );
        button(
            "Cancel",
            HitTarget::SchematicCancel,
            RectPx {
                x: panel.x + 12.,
                y: panel.y + 34.,
                width: 90.,
                height: 25.,
            },
            quads,
            text,
            hits,
        );
        return Ok(());
    }
    draw_text(
        if label.kind == eda_engine::schematic::LabelKind::Power {
            "Place Power Port"
        } else {
            "Place Net Label"
        },
        panel.x + 16.,
        panel.y + 16.,
        16.,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    draw_text(
        "Net name",
        panel.x + 16.,
        panel.y + 61.,
        12.,
        TEXT_MUTED,
        TextFace::Ui,
        text,
    );
    let focused = state.ui.active_menu.is_none()
        && state.ui.focus == datum_gui_protocol::ApplicationFocus::Editor(state.ui.layout.focused);
    let mut controls = ControlMeshCache::default();
    crate::single_line_control::paint(
        &label.name,
        "GND, VCC, signal…",
        label.editor,
        focused && label.focus == 0,
        true,
        RectPx {
            x: panel.x + 90.,
            y: panel.y + 50.,
            width: width - 106.,
            height: 34.,
        },
        &mut ControlPainter::new(quads, &mut controls, 1.),
        text,
        hits,
        HitTarget::SchematicLabelName,
    )?;
    draw_text(
        "Kind",
        panel.x + 16.,
        panel.y + 103.,
        12.,
        TEXT_MUTED,
        TextFace::Ui,
        text,
    );
    let kind = RectPx {
        x: panel.x + 90.,
        y: panel.y + 94.,
        width: width - 106.,
        height: 29.,
    };
    button(
        label.kind_name(),
        HitTarget::SchematicLabelKind,
        kind,
        quads,
        text,
        hits,
    );
    if focused && label.focus == 1 {
        push_rect_border(quads, kind, TEXT_ACCENT, 2.);
    }
    draw_text(
        &truncate_text(
            e.refusal
                .as_deref()
                .unwrap_or("Enter: preview · click canvas to place · Escape: cancel"),
            66,
        ),
        panel.x + 16.,
        panel.y + 137.,
        11.,
        if e.refusal.is_some() {
            design_tokens::chrome::STATUS_ERROR
        } else {
            TEXT_MUTED
        },
        TextFace::Ui,
        text,
    );
    for (name, target, x, focus) in [
        ("Cancel", HitTarget::SchematicCancel, panel.x + 16., 2),
        (
            "Preview",
            HitTarget::SchematicLabelPreview,
            panel.x + width - 106.,
            3,
        ),
    ] {
        let rect = RectPx {
            x,
            y: panel.y + 169.,
            width: 90.,
            height: 29.,
        };
        button(name, target, rect, quads, text, hits);
        if focused && label.focus == focus {
            push_rect_border(quads, rect, TEXT_ACCENT, 2.);
        }
    }
    Ok(())
}
