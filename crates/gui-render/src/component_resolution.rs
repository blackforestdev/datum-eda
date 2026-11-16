//! Small retained-target recovery surface over existing Library operations.
use super::*;
use datum_gui_protocol::library_browser::component_resolution::ResolutionAction as A;
#[allow(clippy::too_many_arguments)]
pub(super) fn render(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    controls: &mut ControlMeshCache,
    scale: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> bool {
    let Some(context) = state.library.resolution.as_ref().filter(|c| !c.in_library) else {
        return false;
    };
    let area = layout.viewport.scale_by(1.0 / scale);
    let card = RectPx {
        x: area.x + 12.,
        y: area.y + 12.,
        width: (area.width - 24.).max(1.),
        height: (area.height - 24.).max(1.),
    };
    let mut painter = ControlPainter::new(quads, controls, scale);
    painter.push(Quad::from_rect(card, PANEL_BG));
    hits.push(HitRegion {
        rect: card,
        target: HitTarget::BoardTransfer(Action::Surface),
    });
    let x = card.x + 12.;
    let mut y = card.y + 12.;
    for message in [
        format!("Resolve {} · physical definition", context.reference()),
        "Exact saved Unit / Symbol retained".into(),
        format!(
            "{} complete compatible Part choices",
            context.evidence.options.choices.len()
        ),
        "Library commits are separate saved changes".into(),
        "Choose a package and map every pin explicitly".into(),
    ] {
        draw_text_clipped(&message, x, y, 16., TEXT_PRIMARY, TextFace::Ui, card, text);
        y += 27.;
    }
    let actions = context.actions(&state.library);
    for (label, action) in [
        ("Author Entity from this Unit / Symbol…", A::Entity),
        ("Choose local Package / Footprint…", A::Footprint),
        ("Author named Part…", A::Part),
        ("Map pads to gate-qualified pins…", A::Map),
        ("Review Part assignment…", A::Assign),
    ] {
        let rect = RectPx {
            x,
            y,
            width: (card.width - 24.).max(1.),
            height: 32.,
        };
        let enabled = actions.contains(&action);
        button(
            label,
            rect,
            context.focus == Some(action),
            enabled,
            &mut painter,
            text,
        );
        if enabled {
            hits.push(HitRegion {
                rect,
                target: HitTarget::BoardTransfer(Action::Resolution(action)),
            });
        }
        y += 39.;
    }
    if let Some(reason) = &context.refusal {
        draw_text_clipped(reason, x, y, 16., TEXT_PRIMARY, TextFace::Ui, card, text);
    }
    let rect = RectPx {
        x,
        y: card.y + card.height - 44.,
        width: (card.width - 24.).max(1.),
        height: 32.,
    };
    button(
        if context.transfer {
            "Back to transfer · no assignment"
        } else {
            "Cancel · return to schematic"
        },
        rect,
        context.focus == Some(A::Cancel),
        true,
        &mut painter,
        text,
    );
    hits.push(HitRegion {
        rect,
        target: HitTarget::BoardTransfer(Action::Resolution(A::Cancel)),
    });
    true
}
