//! Named schematic form configured from existing picker chrome and text control.
use super::*;
use crate::global_preferences_primitives::{ControlMeshCache, ControlPainter};
pub(super) fn render(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    let Some(draft) = &state.schematic_authoring.creation else {
        return Ok(());
    };
    let width = 480f32.min(layout.top_menu_bar.width - 32.0);
    let panel = RectPx {
        x: (layout.top_menu_bar.width - width) / 2.,
        y: 76.,
        width,
        height: 190.,
    };
    quads.push(Quad::from_rect(panel, design_tokens::chrome::SURFACE_01));
    push_rect_border(quads, panel, design_tokens::chrome::BORDER_STRONG, 1.);
    hits.push(HitRegion {
        target: HitTarget::SchematicPicker,
        rect: panel,
    });
    draw_text(
        "New Schematic",
        panel.x + 16.,
        panel.y + 18.,
        16.,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    draw_text(
        "Name",
        panel.x + 16.,
        panel.y + 62.,
        12.,
        TEXT_MUTED,
        TextFace::Ui,
        text,
    );
    let mut controls = ControlMeshCache::default();
    crate::single_line_control::paint(
        &draft.name,
        "Schematic name",
        draft.editor,
        draft.focus == 0,
        true,
        RectPx {
            x: panel.x + 82.,
            y: panel.y + 50.,
            width: width - 98.,
            height: 34.,
        },
        &mut ControlPainter::new(quads, &mut controls, 1.),
        text,
        hits,
        HitTarget::SchematicName,
    )?;
    if let Some(reason) = &draft.refusal {
        draw_text(
            &truncate_text(reason, 65),
            panel.x + 16.,
            panel.y + 100.,
            11.,
            design_tokens::chrome::STATUS_ERROR,
            TextFace::Ui,
            text,
        );
    }
    for (label, target, x, focus) in [
        ("Cancel", HitTarget::SchematicCancel, panel.x + 16., 1),
        (
            "Create",
            HitTarget::SchematicCreate,
            panel.x + width - 106.,
            2,
        ),
    ] {
        let rect = RectPx {
            x,
            y: panel.y + 144.,
            width: 90.,
            height: 29.,
        };
        button(label, target, rect, quads, text, hits);
        if draft.focus == focus {
            push_rect_border(quads, rect, TEXT_ACCENT, 2.);
        }
    }
    Ok(())
}
