//! UUID browsing and metadata state within the shared two-text-field picker.
use super::*;
use datum_gui_protocol::schematic_authoring::{
    CustomFieldControl as C, FieldsEditDraft, SchematicAuthoring,
};

pub(super) fn extra(e: &SchematicAuthoring) -> f32 {
    match e.editing {
        Some(FieldsEditDraft::Custom(_)) => 102.,
        Some(FieldsEditDraft::Symbol(_)) => 34.,
        _ => 0.,
    }
}
pub(super) fn render(
    state: &ReviewWorkspaceState,
    panel: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let e = &state.schematic_authoring;
    let available = datum_gui_protocol::gui_menu_model::native_edit_unavailable_reason(
        state,
        "datum.schematic.set_symbol_field",
    )
    .is_none();
    if matches!(e.editing, Some(FieldsEditDraft::Symbol(_))) {
        control(
            "Custom fields…",
            C::Open,
            available,
            RectPx {
                x: panel.x + 16.,
                y: panel.y + 133.,
                width: panel.width - 32.,
                height: 27.,
            },
            quads,
            text,
            hits,
        );
        return;
    }
    let Some(FieldsEditDraft::Custom(draft)) = &e.editing else {
        return;
    };
    control(
        "Symbol fields…",
        C::Symbol,
        available,
        RectPx {
            x: panel.x + panel.width - 156.,
            y: panel.y + 10.,
            width: 140.,
            height: 29.,
        },
        quads,
        text,
        hits,
    );
    let width = (panel.width - 50.) / 4.;
    for (index, (label, control_id, enabled)) in [
        ("Previous", C::Previous, !draft.original.fields.is_empty()),
        ("Next", C::Next, !draft.original.fields.is_empty()),
        ("New", C::New, true),
        (
            "Remove",
            C::Remove,
            draft.selected().is_some() && !draft.remove,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        control(
            label,
            control_id,
            available && enabled,
            RectPx {
                x: panel.x + 16. + index as f32 * (width + 6.),
                y: panel.y + 133.,
                width,
                height: 27.,
            },
            quads,
            text,
            hits,
        );
    }
    let field = draft.selected();
    let visible = field.is_some_and(|f| f.visible);
    let positioned = field.is_some_and(|f| f.position.is_some());
    let width = (panel.width - 38.) / 2.;
    for (index, (label, target)) in [
        (
            if visible { "Hide" } else { "Show" },
            if visible { C::Hide } else { C::Show },
        ),
        (
            if positioned {
                "Move text…"
            } else {
                "Place text…"
            },
            C::Place,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        control(
            label,
            target,
            available && field.is_some() && !draft.remove && draft.visibility.is_none(),
            RectPx {
                x: panel.x + 16. + index as f32 * (width + 6.),
                y: panel.y + 167.,
                width,
                height: 27.,
            },
            quads,
            text,
            hits,
        );
    }
    let (identity, metadata) = if let Some(field) = draft.selected() {
        let index = draft
            .original
            .fields
            .iter()
            .position(|f| f.uuid == field.uuid)
            .unwrap();
        let identity = format!("Field {}/{}", index + 1, draft.original.fields.len());
        (
            identity,
            if draft.remove {
                "Removal pending · Apply to remove; Cancel to keep".into()
            } else if let Some(visible) = draft.visibility {
                if visible {
                    "Show pending · Apply to show".into()
                } else {
                    "Hide pending · Apply to hide".into()
                }
            } else if field.visible && field.position.is_some() {
                "Visible · manually positioned".into()
            } else if field.visible {
                "Position required · Place text to choose a location".into()
            } else if field.position.is_some() {
                "Hidden · position preserved".into()
            } else {
                "Hidden / not drawn · no position".into()
            },
        )
    } else {
        (
            "New custom metadata".into(),
            "Hidden / not drawn · no position".into(),
        )
    };
    for (label, y) in [(identity, 204.), (metadata, 219.)] {
        draw_text(
            &truncate_text(&label, 66),
            panel.x + 16.,
            panel.y + y,
            11.,
            TEXT_MUTED,
            TextFace::Ui,
            text,
        );
    }
}

fn control(
    label: &str,
    target: C,
    enabled: bool,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    if enabled {
        button(
            label,
            HitTarget::SchematicCustomField(target),
            rect,
            quads,
            text,
            hits,
        );
    } else {
        quads.push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_02));
        push_rect_border(quads, rect, design_tokens::chrome::BORDER_SUBTLE, 1.);
        draw_text(
            label,
            rect.x + 8.,
            rect.y + 7.,
            12.,
            TEXT_MUTED,
            TextFace::Ui,
            text,
        );
    }
}

#[cfg(test)]
#[path = "schematic_field_controls_tests.rs"]
mod tests;
