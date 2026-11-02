//! Existing pane-header tool cluster, bound to the shared command dispatch.
use super::*;
use datum_gui_protocol::{EditorTool, PaneContent, WorkspaceTool};

pub(crate) const RIGHT_INSET_PX: f32 = 24.0;

const TOOLS: [(EditorTool, f32); 5] = [
    (EditorTool::Select, 42.0),
    (EditorTool::Move, 38.0),
    (EditorTool::Wire, 48.0),
    (EditorTool::Symbol, 48.0),
    (EditorTool::Power, 44.0),
];
pub(crate) fn reserve(header: RectPx) -> f32 {
    230.0_f32.min((header.width - RIGHT_INSET_PX).max(0.0))
}

pub(super) fn render(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    if state.project_unopened {
        return;
    }
    let panes = layout.viewport_panes(&state.ui.layout);
    for pane in &panes.panes {
        if pane.content != PaneContent::Schematic {
            continue;
        }
        let header = pane.rect.header;
        let starts = (quads.len(), text.len(), hits.len());
        let mut x = header.x + header.width - RIGHT_INSET_PX - reserve(header);
        let focused = pane.id == panes.focused;
        for (tool, width) in TOOLS {
            let rect = RectPx {
                x,
                y: header.y + 3.0,
                width,
                height: 25.0,
            };
            let active = focused && EditorTool::active(state) == tool;
            let available = !focused
                || match tool {
                    EditorTool::Select => true,
                    EditorTool::Move => {
                        datum_gui_protocol::gui_menu_model::native_project_edit_available(state)
                    }
                    _ => tool.menu_entry().is_some_and(|(menu, label)| {
                        menu_model()
                            .ok()
                            .and_then(|model| model.menubar.iter().find(|group| group.menu == menu))
                            .and_then(|group| group.items.iter().find(|item| item.label == label))
                            .is_some_and(|item| item.unavailable_reason(state).is_none())
                    }),
                };
            let color = if active {
                TEXT_ACCENT
            } else if focused && available {
                TEXT_PRIMARY
            } else {
                TEXT_MUTED
            };
            quads.push(Quad::from_rect(
                rect,
                if active {
                    REVIEW_ROW_ACTIVE_BG
                } else {
                    design_tokens::chrome::SURFACE_02
                },
            ));
            push_rect_border(
                quads,
                rect,
                if active {
                    TEXT_ACCENT
                } else {
                    PANEL_CARD_BORDER
                },
                1.0,
            );
            let icon = icon(tool, rect.x + 4.0, rect.y + 4.5, color, quads);
            let label = match tool {
                EditorTool::Select => "Sel",
                EditorTool::Symbol => "Sym",
                _ => tool.label(),
            };
            draw_text(
                label,
                rect.x + if icon { 22.0 } else { 5.0 },
                rect.y + 7.0,
                design_tokens::typography::CAPTION_SIZE,
                color,
                if active {
                    TextFace::UiMedium
                } else {
                    TextFace::Ui
                },
                text,
            );
            let target = match tool {
                EditorTool::Select => HitTarget::SetWorkspaceTool(WorkspaceTool::Select),
                EditorTool::Move => HitTarget::SetWorkspaceTool(WorkspaceTool::Move),
                _ => {
                    let (menu, label) = tool.menu_entry().unwrap();
                    HitTarget::MenuItem {
                        menu: menu.into(),
                        label: label.into(),
                    }
                }
            };
            hits.push(HitRegion { rect, target });
            x += width + 2.0;
        }
        crate::hit_clipping::clip_content(quads, text, hits, starts.0, starts.1, starts.2, header);
    }
}

/// Exact first-party paths from schematic-editor.html, viewBox 0 0 24 24.
/// Move/Power remain labeled: their catalog entries have no shipped glyph asset.
fn icon(tool: EditorTool, x: f32, y: f32, color: [f32; 3], quads: &mut Vec<Quad>) -> bool {
    let paths: &[&[(f32, f32)]] = match tool {
        EditorTool::Select => &[&[(5., 3.), (12., 20.), (14.2, 13.2), (21., 11.), (5., 3.)]],
        EditorTool::Wire => &[&[(3., 6.), (11., 6.), (11., 18.), (21., 18.)]],
        EditorTool::Symbol => &[
            &[(8., 7.), (16., 7.), (16., 17.), (8., 17.), (8., 7.)],
            &[(4., 10.), (8., 10.)],
            &[(4., 14.), (8., 14.)],
            &[(16., 10.), (20., 10.)],
            &[(16., 14.), (20., 14.)],
        ],
        _ => return false,
    };
    let scale = 16.0 / 24.0;
    for path in paths {
        for pair in path.windows(2) {
            let a = (x + pair[0].0 * scale, y + pair[0].1 * scale);
            let b = (x + pair[1].0 * scale, y + pair[1].1 * scale);
            let length = (b.0 - a.0).hypot(b.1 - a.1);
            let normal = (
                -(b.1 - a.1) / length * 1.7 * scale * 0.5,
                (b.0 - a.0) / length * 1.7 * scale * 0.5,
            );
            quads.push(Quad {
                points: [
                    (a.0 + normal.0, a.1 + normal.1),
                    (b.0 + normal.0, b.1 + normal.1),
                    (b.0 - normal.0, b.1 - normal.1),
                    (a.0 - normal.0, a.1 - normal.1),
                ],
                color,
            });
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_split_header_exposes_real_actions_and_live_wire_state() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.layout = datum_gui_protocol::WorkspaceLayout::board_schematic();
        state.ui.layout.focused = datum_gui_protocol::PaneId(1);
        state.schematic_authoring.wire = true;
        let mut layout = ShellLayout::for_window(1280, 800, None);
        layout.viewport.width = 666.0;
        let pane = layout
            .viewport_panes(&state.ui.layout)
            .panes
            .into_iter()
            .find(|pane| pane.content == PaneContent::Schematic)
            .unwrap();
        assert!(pane.rect.header.width < 420.0);
        let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
        render(&state, &layout, &mut quads, &mut text, &mut hits);
        assert_eq!(
            hits.len(),
            5,
            "all tools remain available in ordinary split panes"
        );
        for hit in &hits {
            assert!(pane.rect.header.contains(hit.rect.x, hit.rect.y));
            assert!(hit.rect.x + hit.rect.width <= pane.rect.header.x + pane.rect.header.width);
        }
        assert!(
            hits.iter()
                .any(|hit| hit.target == HitTarget::SetWorkspaceTool(WorkspaceTool::Select))
        );
        for label in ["Wire", "Symbol", "Power Port"] {
            assert!(hits.iter().any(|hit| matches!(&hit.target,
                HitTarget::MenuItem { menu, label: found } if menu == "Place" && found == label)));
        }
        assert_eq!(
            text.iter().find(|run| run.text == "Wire").unwrap().color,
            TEXT_ACCENT
        );
        assert_ne!(
            text.iter().find(|run| run.text == "Sel").unwrap().color,
            TEXT_ACCENT
        );
        state.schematic_authoring.cancel();
        text.clear();
        render(&state, &layout, &mut quads, &mut text, &mut hits);
        assert_eq!(
            text.iter().find(|run| run.text == "Sel").unwrap().color,
            TEXT_ACCENT
        );
    }
}
