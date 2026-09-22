#[path = "filters_layout.rs"]
mod filters_layout;
use filters_layout::solve_filters_panel_layout_with_taffy;

#[path = "project_layout.rs"]
mod project_layout;
use project_layout::solve_project_panel_layout_with_taffy;

#[derive(Debug, Clone, PartialEq)]
struct ProjectPanelLayout {
    project_rect: RectPx,
    filters_rect: RectPx,
    project_name: RectPx,
    board_name: RectPx,
    net: Option<RectPx>,
    source_label: RectPx,
    source_rows: RectPx,
    fit_row: RectPx,
    tool_label: RectPx,
    tool_grid: RectPx,
    import_notice: Option<RectPx>,
    last_status: Option<RectPx>,
}

#[derive(Debug, Clone, PartialEq)]
struct FiltersPanelLayout {
    authored: RectPx,
    proposed: RectPx,
    unrouted: RectPx,
    dim_unrelated: RectPx,
    layer_rows: Vec<RectPx>,
    active_summary: Option<RectPx>,
    layers_summary: RectPx,
    focus_summary: RectPx,
    outputs_summary: RectPx,
}

#[derive(Debug, Clone)]
struct InspectorDetailLayout {
    divider_y: Option<f32>,
    contract: Option<RectPx>,
    net: Option<RectPx>,
    segment: Option<RectPx>,
    layer: Option<RectPx>,
    last_status: Option<RectPx>,
}

#[derive(Debug, Clone)]
pub(super) struct RightPanelLayout {
    pub(super) inspector_rect: RectPx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RightPanelNode {
    Inspector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InspectorDetailNode {
    Contract,
    Net,
    Segment,
    Layer,
    LastStatus,
}

pub(super) fn solve_right_panel_layout_with_taffy(
    state: &ReviewWorkspaceState,
    right: RectPx,
) -> Option<RightPanelLayout> {
    let card_x = right.x + UI_CARD_MARGIN;
    let card_y = right.y + UI_CARD_MARGIN;
    let card_width = (right.width - UI_CARD_MARGIN * 2.0).max(1.0);
    let content_height = (right.height - UI_CARD_MARGIN * 2.0).max(1.0);
    let inspector_height =
        content_height.max(inspector_height_for_state(state).min(content_height));

    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let inspector = taffy
        .new_leaf(Style {
            size: Size {
                width: length(card_width),
                height: length(inspector_height),
            },
            ..Default::default()
        })
        .ok()?;
    let nodes = [(RightPanelNode::Inspector, inspector)];
    let children = nodes.iter().map(|(_, node)| *node).collect::<Vec<_>>();
    let root = taffy
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: length(card_width),
                    height: length(content_height),
                },
                ..Default::default()
            },
            &children,
        )
        .ok()?;
    taffy.compute_layout(root, Size::MAX_CONTENT).ok()?;
    let rect_for = |kind: RightPanelNode| -> Option<RectPx> {
        let node = nodes.iter().find(|(node_kind, _)| *node_kind == kind)?.1;
        let layout = taffy.layout(node).ok()?;
        Some(RectPx {
            x: card_x + layout.location.x,
            y: card_y + layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        })
    };
    Some(RightPanelLayout {
        inspector_rect: rect_for(RightPanelNode::Inspector)?,
    })
}

fn fallback_right_panel_layout(state: &ReviewWorkspaceState, right: RectPx) -> RightPanelLayout {
    let inspector_rect = RectPx {
        x: right.x + UI_CARD_MARGIN,
        y: right.y + UI_CARD_MARGIN,
        width: right.width - UI_CARD_MARGIN * 2.0,
        height: inspector_height_for_state(state),
    };
    RightPanelLayout { inspector_rect }
}

fn fallback_project_panel_layout(state: &ReviewWorkspaceState, left: RectPx) -> ProjectPanelLayout {
    let project_rect = RectPx {
        x: left.x + UI_CARD_MARGIN,
        y: left.y + UI_CARD_MARGIN,
        width: left.width - UI_CARD_MARGIN * 2.0,
        height: 330.0,
    };
    let filters_rect = RectPx {
        x: left.x + UI_CARD_MARGIN,
        y: left.y + 330.0,
        width: left.width - UI_CARD_MARGIN * 2.0,
        height: (left.height - 340.0).max(100.0),
    };
    let content_x = project_rect.x + UI_CARD_PADDING_X;
    let content_width = (project_rect.width - UI_CARD_PADDING_X * 2.0).max(1.0);
    ProjectPanelLayout {
        project_rect,
        filters_rect,
        project_name: RectPx {
            x: content_x,
            y: project_rect.y + UI_CARD_CONTENT_TOP,
            width: content_width,
            height: UI_ROW_PROJECT_TITLE,
        },
        board_name: RectPx {
            x: content_x,
            y: project_rect.y + UI_CARD_CONTENT_TOP + UI_ROW_PROJECT_TITLE + 2.0,
            width: content_width,
            height: UI_ROW_BOARD_SUBTITLE,
        },
        net: state.selected_review_action().map(|_| RectPx {
            x: content_x,
            y: project_rect.y + 74.0,
            width: content_width,
            height: UI_ROW_NET,
        }),
        source_label: RectPx {
            x: content_x,
            y: project_rect.y + 94.0,
            width: content_width,
            height: UI_ROW_SOURCE_LABEL,
        },
        source_rows: RectPx {
            x: content_x,
            y: project_rect.y + 110.0,
            width: content_width,
            height: 32.0,
        },
        fit_row: RectPx {
            x: content_x,
            y: project_rect.y + 144.0,
            width: content_width,
            height: UI_ROW_BUTTON,
        },
        tool_label: RectPx {
            x: content_x,
            y: project_rect.y + 178.0,
            width: content_width,
            height: UI_ROW_TOOL_LABEL,
        },
        tool_grid: RectPx {
            x: content_x,
            y: project_rect.y + 196.0,
            width: content_width,
            height: UI_ROW_TOOL_GRID,
        },
        import_notice: None,
        last_status: state.last_command_status.as_ref().map(|_| RectPx {
            x: content_x,
            y: project_rect.y + 264.0,
            width: content_width,
            height: UI_ROW_NOTICE,
        }),
    }
}

fn filter_hit_rect(row: RectPx) -> RectPx {
    RectPx {
        x: row.x - 8.0,
        y: row.y - 8.0,
        width: row.width + 8.0,
        height: 22.0,
    }
}

fn solve_inspector_detail_layout_with_taffy(
    state: &ReviewWorkspaceState,
    inspector_rect: RectPx,
) -> Option<InspectorDetailLayout> {
    let content_x = inspector_rect.x + UI_CARD_PADDING_X;
    // Detail rows sit below the title band (y+34..y+66) and the "ROUTE ACTION"
    // section-header strip (y+72..y+90).
    let content_y = inspector_rect.y + 96.0;
    let content_width = (inspector_rect.width - UI_CARD_PADDING_X * 2.0).max(1.0);
    let mut nodes = Vec::new();
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let row_height = key_value_row_height();
    let mut add_node = |kind: InspectorDetailNode| -> Option<()> {
        let node = taffy
            .new_leaf(Style {
                size: Size {
                    width: length(content_width),
                    height: length(row_height),
                },
                ..Default::default()
            })
            .ok()?;
        nodes.push((kind, node));
        Some(())
    };

    if state.selected_review_action().is_some() {
        add_node(InspectorDetailNode::Contract)?;
        add_node(InspectorDetailNode::Net)?;
        add_node(InspectorDetailNode::Segment)?;
    }
    if state.selected_segment_evidence().is_some() {
        add_node(InspectorDetailNode::Layer)?;
    }
    if state.last_command_status.is_some() {
        add_node(InspectorDetailNode::LastStatus)?;
    }

    if nodes.is_empty() {
        return Some(InspectorDetailLayout {
            divider_y: None,
            contract: None,
            net: None,
            segment: None,
            layer: None,
            last_status: None,
        });
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

    let rect_for = |kind: InspectorDetailNode| -> Option<RectPx> {
        let node = nodes.iter().find(|(node_kind, _)| *node_kind == kind)?.1;
        let layout = taffy.layout(node).ok()?;
        Some(RectPx {
            x: content_x + layout.location.x,
            y: content_y + layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        })
    };

    Some(InspectorDetailLayout {
        divider_y: Some(inspector_rect.y + 76.0),
        contract: rect_for(InspectorDetailNode::Contract),
        net: rect_for(InspectorDetailNode::Net),
        segment: rect_for(InspectorDetailNode::Segment),
        layer: rect_for(InspectorDetailNode::Layer),
        last_status: rect_for(InspectorDetailNode::LastStatus),
    })
}
