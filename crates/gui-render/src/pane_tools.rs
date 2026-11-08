//! Existing pane-header tool cluster, bound to the shared command dispatch.
use super::*;
use datum_gui_protocol::{EditorTool, PaneContent, WorkspaceTool};

pub(crate) const RIGHT_INSET_PX: f32 = 24.0;

/// Shared title clip used by paint and breadcrumb hit projection.
pub(crate) fn title_bounds(header: RectPx, tools_reserved: f32, focused: bool) -> RectPx {
    let x = header.x + design_tokens::spacing::SP_04 + 11.0 + design_tokens::spacing::SP_03;
    let right = header.x + header.width
        - tools_reserved
        - if tools_reserved > 0.0 {
            RIGHT_INSET_PX
        } else if focused {
            design_tokens::spacing::SP_04 + 7.0 + design_tokens::spacing::SP_02
        } else {
            design_tokens::spacing::SP_02
        };
    RectPx {
        x,
        y: header.y,
        width: (right - x).max(0.0),
        height: header.height,
    }
}

fn context_text_width(text: &str) -> f32 {
    crate::control_text::measure_text_width(text, 10.0, TextFace::Mono)
        .unwrap_or_else(|_| estimated_text_run_width_px(text, 10.0, TextFace::Mono))
}

/// Allocate the existing inline controls only after the pane-local identity fits.
/// All actions remain available through their shared menu/command surfaces.
pub(crate) fn reserve_for(
    state: &ReviewWorkspaceState,
    pane: datum_gui_protocol::PaneId,
    header: RectPx,
    focused: bool,
) -> f32 {
    let reserved = reserve(header);
    let documents = &state.schematic_documents;
    if documents.project.is_none() {
        return reserved;
    }
    let context = documents.panes.get(&pane).and_then(|key| {
        documents.row(key).map(|row| {
            (
                format!(
                    "Occurrence: {}",
                    documents
                        .breadcrumbs(key)
                        .iter()
                        .map(|row| row.name.as_str())
                        .collect::<Vec<_>>()
                        .join(" / ")
                ),
                format!(
                    "Source: {}",
                    if datum_gui_protocol::camera_scene_for_pane(state, pane).is_some() {
                        row.source_name.as_str()
                    } else {
                        "unavailable"
                    }
                ),
            )
        })
    });
    let (occurrence, source) =
        context.unwrap_or_else(|| ("Occurrence unavailable".into(), "Source unavailable".into()));
    let width = context_text_width;
    let required = width(&occurrence).max(width(&source));
    if title_bounds(header, reserved, focused).width >= required {
        reserved
    } else {
        0.0
    }
}

fn breadcrumb_hits(
    state: &ReviewWorkspaceState,
    pane: datum_gui_protocol::PaneId,
    header: RectPx,
    focused: bool,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let documents = &state.schematic_documents;
    let Some(key) = documents.panes.get(&pane) else {
        return;
    };
    if documents.row(key).is_none()
        || (documents.project.is_some()
            && datum_gui_protocol::camera_scene_for_pane(state, pane).is_none())
    {
        return;
    }
    let Some(revision) = &documents.revision else {
        return;
    };
    let clip = title_bounds(header, reserve_for(state, pane, header, focused), focused);
    let width = context_text_width;
    let prefix = "Occurrence: ";
    let paint = |value: &str, x: f32, color, text: &mut Vec<TextRun>| {
        let remaining = (clip.x + clip.width - x).max(0.0);
        if remaining > 0.0 {
            draw_text_clipped(
                value,
                x,
                header.y + 2.0,
                10.0,
                color,
                TextFace::Mono,
                RectPx {
                    x,
                    width: remaining,
                    ..clip
                },
                text,
            );
            if let Some(run) = text.last_mut() {
                run.layout_size = Some((width(value), header.height));
            }
        }
    };
    paint(
        prefix,
        clip.x,
        if focused { TEXT_PRIMARY } else { TEXT_MUTED },
        text,
    );
    let mut x = clip.x + width(prefix);
    for ancestor in documents.breadcrumbs(key) {
        let w = width(&ancestor.name);
        paint(
            &ancestor.name,
            x,
            if ancestor.key == *key {
                TEXT_PRIMARY
            } else {
                TEXT_ACCENT
            },
            text,
        );
        let visible = (clip.x + clip.width - x).min(w).max(0.0);
        if ancestor.key != *key && visible > 0.0 {
            let rect = RectPx {
                x,
                y: header.y + 1.0,
                width: visible,
                height: 13.0,
            };
            hits.push(HitRegion {
                rect,
                target: HitTarget::SchematicBreadcrumb {
                    pane,
                    from: key.clone(),
                    document: ancestor.key.clone(),
                    revision: revision.0.clone(),
                },
            });
            quads.push(Quad::from_rect(
                RectPx {
                    y: rect.y + rect.height - 1.0,
                    height: 1.0,
                    ..rect
                },
                TEXT_ACCENT,
            ));
        }
        x += w;
        if ancestor.key != *key {
            paint(" / ", x, TEXT_MUTED, text);
        }
        x += width(" / ");
    }
}

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
        let focused = pane.id == panes.focused;
        let reserved = reserve_for(state, pane.id, header, focused);
        let mut x = header.x + header.width - RIGHT_INSET_PX - reserved;
        breadcrumb_hits(state, pane.id, header, focused, quads, text, hits);
        for (tool, width) in TOOLS
            .into_iter()
            .take(if reserved > 0.0 { TOOLS.len() } else { 0 })
        {
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
    fn breadcrumb_hits_keep_qualified_ancestors_and_clip_with_their_text() {
        use datum_gui_protocol::schematic_documents::{SchematicDocumentKey, SchematicDocumentRow};
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let pane = datum_gui_protocol::PaneId(1);
        let root = SchematicDocumentKey {
            project: uuid::Uuid::from_u128(1),
            schematic: uuid::Uuid::from_u128(2),
            source_sheet: uuid::Uuid::from_u128(3),
            instance_path: vec![],
        };
        let child = SchematicDocumentKey {
            source_sheet: uuid::Uuid::from_u128(4),
            instance_path: vec![uuid::Uuid::from_u128(5)],
            ..root.clone()
        };
        state.schematic_documents.rows = vec![
            SchematicDocumentRow {
                key: root.clone(),
                parent: None,
                name: "Root".into(),
                source_name: "Root source".into(),
                source_sheet_name: "Root".into(),
                definition: None,
                breadcrumb: vec!["Root".into()],
            },
            SchematicDocumentRow {
                key: child.clone(),
                parent: Some(root.clone()),
                name: "Child".into(),
                source_name: "Repeated source".into(),
                source_sheet_name: "Child source".into(),
                definition: Some(uuid::Uuid::from_u128(6)),
                breadcrumb: vec!["Root".into(), "Child".into()],
            },
        ];
        state.schematic_documents.panes.insert(pane, child.clone());
        state.schematic_documents.revision =
            Some(datum_gui_protocol::selection_resolution::SelectionModelRevision("basis".into()));
        for width in [800.0, 360.0, 80.0] {
            let header = RectPx {
                x: 30.0,
                y: 50.0,
                width,
                height: 32.0,
            };
            let clip = title_bounds(header, reserve(header), false);
            let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
            breadcrumb_hits(
                &state, pane, header, false, &mut quads, &mut text, &mut hits,
            );
            for hit in &hits {
                assert_eq!(
                    hit.target,
                    HitTarget::SchematicBreadcrumb {
                        pane,
                        from: child.clone(),
                        document: root.clone(),
                        revision: "basis".into()
                    }
                );
                assert!(hit.rect.x >= clip.x && hit.rect.x + hit.rect.width <= clip.x + clip.width);
                assert!(text.iter().any(|run| run.text == "Root"));
            }
            if width == 800.0 {
                assert_eq!(hits.len(), 1);
                assert!(text.iter().any(|run| run.text == "Child"));
            }
            if clip.width == 0.0 {
                assert!(hits.is_empty());
            }
        }
    }

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
