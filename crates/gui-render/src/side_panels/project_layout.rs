//! Retain Project panel geometry without retaining workspace or label content.
use super::*;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, PartialEq)]
struct LayoutKey {
    left: RectPx,
    attention_rows: usize,
    has_net: bool,
    has_import_notice: bool,
    has_status: bool,
}

impl LayoutKey {
    fn from_state(state: &ReviewWorkspaceState, left: RectPx) -> Self {
        Self {
            left,
            attention_rows: state.source_shards.attention.len().min(2),
            has_net: state.selected_review_action().is_some(),
            has_import_notice: state
                .backing
                .as_ref()
                .is_some_and(|backing| backing.request.board_file.is_some()),
            has_status: state.last_command_status.is_some(),
        }
    }
}

type LayoutCache = crate::shell_layout::cache::LayoutCache<LayoutKey, ProjectPanelLayout>;

thread_local! {
    static CACHE: RefCell<LayoutCache> = RefCell::new(LayoutCache::default());
}

pub(super) fn solve_project_panel_layout_with_taffy(
    state: &ReviewWorkspaceState,
    left: RectPx,
) -> Option<ProjectPanelLayout> {
    let key = LayoutKey::from_state(state, left);
    CACHE.with(|cache| cache.borrow_mut().try_resolve(key, || solve(key)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectPanelNode {
    ProjectName,
    BoardName,
    Net,
    SourceLabel,
    SourceRows,
    FitRow,
    ToolLabel,
    ToolGrid,
    ImportNotice,
    LastStatus,
    Spacer,
}

fn solve(key: LayoutKey) -> Option<ProjectPanelLayout> {
    record_solve();
    let left = key.left;
    let card_x = left.x + UI_CARD_MARGIN;
    let card_y = left.y + UI_CARD_MARGIN;
    let card_width = left.width - UI_CARD_MARGIN * 2.0;
    let content_x = card_x + UI_CARD_PADDING_X;
    let content_y = card_y + UI_CARD_CONTENT_TOP;
    let content_width = (card_width - UI_CARD_PADDING_X * 2.0).max(1.0);
    let source_attention_rows = key.attention_rows;
    let source_rows_height = if source_attention_rows == 0 {
        4.0
    } else {
        source_attention_rows as f32 * UI_ROW_SOURCE_ATTENTION + 4.0
    };

    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let mut nodes = Vec::new();
    let mut add_node = |kind: ProjectPanelNode, height: f32| -> Option<()> {
        let node = taffy
            .new_leaf(Style {
                size: Size {
                    width: length(content_width),
                    height: length(height),
                },
                ..Default::default()
            })
            .ok()?;
        nodes.push((kind, node));
        Some(())
    };

    add_node(ProjectPanelNode::ProjectName, UI_ROW_PROJECT_TITLE)?;
    add_node(ProjectPanelNode::BoardName, UI_ROW_BOARD_SUBTITLE)?;
    if key.has_net {
        add_node(ProjectPanelNode::Net, UI_ROW_NET)?;
    }
    add_node(ProjectPanelNode::Spacer, UI_STACK_GAP_SMALL)?;
    add_node(ProjectPanelNode::SourceLabel, UI_ROW_SOURCE_LABEL)?;
    add_node(ProjectPanelNode::SourceRows, source_rows_height)?;
    add_node(ProjectPanelNode::Spacer, UI_STACK_GAP_SMALL)?;
    add_node(ProjectPanelNode::FitRow, UI_ROW_BUTTON)?;
    add_node(ProjectPanelNode::Spacer, UI_STACK_GAP_MEDIUM)?;
    add_node(ProjectPanelNode::ToolLabel, UI_ROW_TOOL_LABEL)?;
    add_node(ProjectPanelNode::ToolGrid, UI_ROW_TOOL_GRID)?;
    if key.has_import_notice {
        add_node(ProjectPanelNode::Spacer, UI_STACK_GAP_SMALL)?;
        add_node(ProjectPanelNode::ImportNotice, UI_ROW_NOTICE)?;
    }
    if key.has_status {
        add_node(ProjectPanelNode::Spacer, UI_STACK_GAP_SMALL)?;
        add_node(ProjectPanelNode::LastStatus, UI_ROW_NOTICE)?;
    }

    let children = nodes.iter().map(|(_, node)| *node).collect::<Vec<_>>();
    let root = taffy
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: length(content_width),
                    height: Dimension::AUTO,
                },
                ..Default::default()
            },
            &children,
        )
        .ok()?;
    taffy.compute_layout(root, Size::MAX_CONTENT).ok()?;
    let root_layout = taffy.layout(root).ok()?;
    // Q1-A-amended makes the complete Design/Publish/Revision taxonomy
    // permanently visible. Reserve its real height so Revision rows never draw
    // through the sibling LAYERS panel.
    let project_height =
        (UI_CARD_CONTENT_TOP + root_layout.size.height + UI_CARD_CONTENT_BOTTOM).max(330.0);
    let filters_y = card_y + project_height + UI_CARD_MARGIN;

    let rect_for = |kind: ProjectPanelNode| -> Option<RectPx> {
        let node = nodes.iter().find(|(node_kind, _)| *node_kind == kind)?.1;
        let layout = taffy.layout(node).ok()?;
        Some(RectPx {
            x: content_x + layout.location.x,
            y: content_y + layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        })
    };

    Some(ProjectPanelLayout {
        project_rect: RectPx {
            x: card_x,
            y: card_y,
            width: card_width,
            height: project_height,
        },
        filters_rect: RectPx {
            x: left.x + UI_CARD_MARGIN,
            y: filters_y,
            width: left.width - UI_CARD_MARGIN * 2.0,
            height: (left.y + left.height - filters_y - UI_CARD_MARGIN).max(100.0),
        },
        project_name: rect_for(ProjectPanelNode::ProjectName)?,
        board_name: rect_for(ProjectPanelNode::BoardName)?,
        net: rect_for(ProjectPanelNode::Net),
        source_label: rect_for(ProjectPanelNode::SourceLabel)?,
        source_rows: rect_for(ProjectPanelNode::SourceRows)?,
        fit_row: rect_for(ProjectPanelNode::FitRow)?,
        tool_label: rect_for(ProjectPanelNode::ToolLabel)?,
        tool_grid: rect_for(ProjectPanelNode::ToolGrid)?,
        import_notice: rect_for(ProjectPanelNode::ImportNotice),
        last_status: rect_for(ProjectPanelNode::LastStatus),
    })
}

// No production counter or timing overhead.
#[inline]
fn record_solve() {
    #[cfg(test)]
    tests::SOLVES.with(|count| count.set(count.get() + 1));
}

#[cfg(test)]
mod tests {
    use super::*;
    thread_local! { pub(super) static SOLVES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

    #[test]
    fn geometry_keys_match_fresh_solver_and_two_entries_stay_bounded() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        for left in [
            RectPx {
                x: 0.0,
                y: 29.0,
                width: 224.0,
                height: 700.0,
            },
            RectPx {
                x: 0.5,
                y: 43.5,
                width: 336.0,
                height: 850.5,
            },
            RectPx {
                x: 0.0,
                y: 58.0,
                width: 448.0,
                height: 600.0,
            },
            RectPx {
                x: 0.0,
                y: 29.0,
                width: 1.0,
                height: 1.0,
            },
        ] {
            let mut cache = LayoutCache::default();
            let base = LayoutKey::from_state(&state, left);
            for attention_rows in 0..=2 {
                for flags in 0..8 {
                    let key = LayoutKey {
                        attention_rows,
                        has_net: flags & 1 != 0,
                        has_import_notice: flags & 2 != 0,
                        has_status: flags & 4 != 0,
                        ..base
                    };
                    let expected = solve(key).unwrap();
                    for _ in 0..2 {
                        assert_eq!(cache.try_resolve(key, || solve(key)).unwrap(), expected);
                    }
                    assert!(cache.len() <= 2);
                }
            }
        }
        let left = RectPx {
            x: 0.0,
            y: 29.0,
            width: 224.0,
            height: 700.0,
        };
        let base = LayoutKey::from_state(&state, left);
        let other = LayoutKey {
            has_status: !base.has_status,
            ..base
        };
        let third = LayoutKey {
            left: RectPx {
                width: 200.0,
                ..left
            },
            ..base
        };
        let mut cache = LayoutCache::default();
        SOLVES.with(|count| count.set(0));
        for key in [base, other]
            .into_iter()
            .cycle()
            .take(100)
            .chain([third, base])
        {
            cache.try_resolve(key, || solve(key)).unwrap();
        }
        assert_eq!(SOLVES.with(|count| count.get()), 4);
    }

    #[test]
    fn actual_frame_preparation_reuses_project_geometry() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        CACHE.with(|cache| *cache.borrow_mut() = LayoutCache::default());
        SOLVES.with(|count| count.set(0));
        let retained = crate::RetainedScene::from_workspace(&state, 1280, 800);
        let camera = crate::CameraState::fit_to_bounds(&state.scene.bounds);
        for _ in 0..3 {
            let _ =
                crate::PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
        }
        assert_eq!(
            SOLVES.with(|count| count.get()),
            1,
            "warm frames must reuse Project layout"
        );
        // Text changes are paint dependencies, not row-height dependencies.
        state.scene.project_name = "Renamed project".to_owned();
        let prepared =
            crate::PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
        assert!(
            prepared
                .text_runs
                .iter()
                .any(|run| run.text == "Renamed project")
        );
        assert_eq!(SOLVES.with(|count| count.get()), 1);
    }
}
