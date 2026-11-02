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
            if label.port_direction.is_some() {
                "Click to place · Tab: edit name/direction · Esc: cancel"
            } else {
                "Click to place · Tab: edit name/kind · Esc: cancel"
            },
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
        if label.port_direction.is_some() {
            if e.named_edit.is_some() {
                "Edit Port"
            } else {
                "Place Port"
            }
        } else if e.named_edit.is_some() {
            "Edit Label"
        } else if label.kind == eda_engine::schematic::LabelKind::Power {
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
        if label.port_direction.is_some() {
            "Name"
        } else {
            "Net name"
        },
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
        if label.port_direction.is_some() {
            "Port name…"
        } else {
            "GND, VCC, signal…"
        },
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
        if label.port_direction.is_some() {
            "Direction"
        } else {
            "Kind"
        },
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
            e.refusal.as_deref().unwrap_or(if e.named_edit.is_some() {
                "Live preview · Enter: apply · Escape: cancel"
            } else {
                "Enter: preview · click canvas to place · Escape: cancel"
            }),
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
            if e.named_edit.is_some() {
                "Apply"
            } else {
                "Preview"
            },
            if e.named_edit.is_some() {
                HitTarget::SchematicEditApply
            } else {
                HitTarget::SchematicLabelPreview
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_label_form_offers_one_apply_and_cancel_with_live_preview() {
        use datum_gui_protocol::{
            AuthoredSelectionClass, AuthoredSelectionIdentity, AuthoringViewBasis, PaneContent,
            PaneId,
            schematic_authoring::{LabelDraft, selected::LabelEditDraft},
        };
        use eda_engine::schematic::{LabelKind, NetLabel};
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let id = uuid::Uuid::nil();
        state.schematic_authoring.label = Some(LabelDraft::new(LabelKind::Power));
        state.schematic_authoring.picker = true;
        state.schematic_authoring.named_edit = Some(
            LabelEditDraft {
                target: AuthoredSelectionIdentity {
                    class: AuthoredSelectionClass::Label,
                    id,
                    instance_path: vec![],
                },
                basis: AuthoringViewBasis {
                    project: id,
                    revision: "source".into(),
                    pane: PaneId(1),
                    content: PaneContent::Schematic,
                    source: id,
                    document: None,
                },
                tip: None,
                original: NetLabel {
                    uuid: id,
                    kind: LabelKind::Power,
                    name: "GND".into(),
                    position: eda_engine::ir::geometry::Point::zero(),
                },
            }
            .into(),
        );
        let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
        render(
            &state,
            &ShellLayout::for_window(1280, 800, None),
            &mut quads,
            &mut text,
            &mut hits,
        )
        .unwrap();
        assert!(text.iter().any(|r| r.text == "Edit Label"));
        assert!(text.iter().any(|r| r.text == "Apply"));
        assert!(
            !text
                .iter()
                .any(|r| r.text == "Preview" || r.text.contains("Click to place"))
        );
        assert_eq!(
            hits.iter()
                .filter(|h| h.target == HitTarget::SchematicEditApply)
                .count(),
            1
        );
        assert_eq!(
            hits.iter()
                .filter(|h| h.target == HitTarget::SchematicCancel)
                .count(),
            1
        );
        assert!(
            hits.iter()
                .any(|h| h.target == HitTarget::SchematicLabelName)
        );
        assert!(
            hits.iter()
                .any(|h| h.target == HitTarget::SchematicLabelKind)
        );
    }
    #[test]
    fn interface_port_form_configures_existing_name_choice_and_cancel_controls() {
        use datum_gui_protocol::schematic_authoring::LabelDraft;
        use eda_engine::schematic::PortDirection;
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.schematic_authoring.label = Some(LabelDraft::port(PortDirection::Bidirectional));
        state.schematic_authoring.picker = true;
        let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
        render(
            &state,
            &ShellLayout::for_window(1280, 800, None),
            &mut quads,
            &mut text,
            &mut hits,
        )
        .unwrap();
        for expected in [
            "Place Port",
            "Name",
            "Direction",
            "Bidirectional",
            "Preview",
            "Cancel",
        ] {
            assert!(
                text.iter().any(|run| run.text == expected),
                "missing {expected}"
            );
        }
        for target in [
            HitTarget::SchematicLabelName,
            HitTarget::SchematicLabelKind,
            HitTarget::SchematicLabelPreview,
            HitTarget::SchematicCancel,
        ] {
            assert_eq!(hits.iter().filter(|hit| hit.target == target).count(), 1);
        }
        assert!(
            !text
                .iter()
                .any(|run| run.text == "Kind" || run.text == "Net name")
        );
    }
}
