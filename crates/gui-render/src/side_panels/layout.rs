#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RightPanelNode {
    Inspector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FiltersPanelNode {
    Authored,
    Proposed,
    Unrouted,
    DimUnrelated,
    Layer(usize),
    ActiveSummary,
    LayersSummary,
    FocusSummary,
    OutputsSummary,
    Gap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InspectorDetailNode {
    Contract,
    Net,
    Segment,
    Layer,
    LastStatus,
}

fn solve_project_panel_layout_with_taffy(
    state: &ReviewWorkspaceState,
    left: RectPx,
) -> Option<ProjectPanelLayout> {
    let card_x = left.x + UI_CARD_MARGIN;
    let card_y = left.y + UI_CARD_MARGIN;
    let card_width = left.width - UI_CARD_MARGIN * 2.0;
    let content_x = card_x + UI_CARD_PADDING_X;
    let content_y = card_y + UI_CARD_CONTENT_TOP;
    let content_width = (card_width - UI_CARD_PADDING_X * 2.0).max(1.0);
    let source_attention_rows = state.source_shards.attention.len().min(2);
    let source_rows_height = if source_attention_rows == 0 {
        4.0
    } else {
        source_attention_rows as f32 * UI_ROW_SOURCE_ATTENTION + 4.0
    };
    let has_import_notice = state
        .backing
        .as_ref()
        .is_some_and(|backing| backing.request.board_file.is_some());
    let has_status = state.last_command_status.is_some();

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
    if state.selected_review_action().is_some() {
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
    if has_import_notice {
        add_node(ProjectPanelNode::Spacer, UI_STACK_GAP_SMALL)?;
        add_node(ProjectPanelNode::ImportNotice, UI_ROW_NOTICE)?;
    }
    if has_status {
        add_node(ProjectPanelNode::Spacer, UI_STACK_GAP_SMALL)?;
        add_node(ProjectPanelNode::LastStatus, UI_ROW_NOTICE)?;
    }
    drop(add_node);

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
    let project_height = UI_CARD_CONTENT_TOP + root_layout.size.height + UI_CARD_CONTENT_BOTTOM;
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

pub(super) fn solve_right_panel_layout_with_taffy(
    state: &ReviewWorkspaceState,
    right: RectPx,
) -> Option<RightPanelLayout> {
    let card_x = right.x + UI_CARD_MARGIN;
    let card_y = right.y + UI_CARD_MARGIN;
    let card_width = (right.width - UI_CARD_MARGIN * 2.0).max(1.0);
    let content_height = (right.height - UI_CARD_MARGIN * 2.0).max(1.0);
    let inspector_height = content_height.max(inspector_height_for_state(state).min(content_height));

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
    RightPanelLayout {
        inspector_rect,
    }
}

fn fallback_project_panel_layout(state: &ReviewWorkspaceState, left: RectPx) -> ProjectPanelLayout {
    let project_rect = RectPx {
        x: left.x + UI_CARD_MARGIN,
        y: left.y + UI_CARD_MARGIN,
        width: left.width - UI_CARD_MARGIN * 2.0,
        height: 300.0,
    };
    let filters_rect = RectPx {
        x: left.x + UI_CARD_MARGIN,
        y: left.y + 326.0,
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

fn solve_filters_panel_layout_with_taffy(
    state: &ReviewWorkspaceState,
    filters_rect: RectPx,
) -> Option<FiltersPanelLayout> {
    let content_x = filters_rect.x + UI_CARD_PADDING_X;
    let content_y = filters_rect.y + UI_CARD_CONTENT_TOP;
    let content_width = (filters_rect.width - UI_CARD_PADDING_X * 2.0).max(1.0);
    let summary_height = if state.selected_review_action().is_some() {
        62.0
    } else {
        44.0
    };
    let fixed_height = 4.0 * 20.0 + UI_STACK_GAP_MEDIUM + summary_height + UI_CARD_CONTENT_BOTTOM;
    let available_layer_height =
        (filters_rect.height - UI_CARD_CONTENT_TOP - fixed_height).max(20.0);
    let max_layer_rows = (available_layer_height / 20.0).floor().max(1.0) as usize;
    let layer_count = state.scene.layers.len().min(max_layer_rows);

    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let mut nodes = Vec::new();
    let mut add_node = |kind: FiltersPanelNode, height: f32| -> Option<()> {
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

    add_node(FiltersPanelNode::Authored, 20.0)?;
    add_node(FiltersPanelNode::Proposed, 20.0)?;
    add_node(FiltersPanelNode::Unrouted, 20.0)?;
    add_node(FiltersPanelNode::DimUnrelated, 20.0)?;
    for index in 0..layer_count {
        add_node(FiltersPanelNode::Layer(index), 20.0)?;
    }
    add_node(FiltersPanelNode::Gap, UI_STACK_GAP_MEDIUM)?;
    if state.selected_review_action().is_some() {
        add_node(FiltersPanelNode::ActiveSummary, 18.0)?;
    }
    add_node(FiltersPanelNode::LayersSummary, 16.0)?;
    add_node(FiltersPanelNode::FocusSummary, 16.0)?;
    add_node(FiltersPanelNode::OutputsSummary, 16.0)?;
    drop(add_node);

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

    let rect_for = |kind: FiltersPanelNode| -> Option<RectPx> {
        let node = nodes.iter().find(|(node_kind, _)| *node_kind == kind)?.1;
        let layout = taffy.layout(node).ok()?;
        Some(RectPx {
            x: content_x + layout.location.x,
            y: content_y + layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        })
    };
    let mut layer_rows = Vec::with_capacity(layer_count);
    for index in 0..layer_count {
        layer_rows.push(rect_for(FiltersPanelNode::Layer(index))?);
    }

    Some(FiltersPanelLayout {
        authored: rect_for(FiltersPanelNode::Authored)?,
        proposed: rect_for(FiltersPanelNode::Proposed)?,
        unrouted: rect_for(FiltersPanelNode::Unrouted)?,
        dim_unrelated: rect_for(FiltersPanelNode::DimUnrelated)?,
        layer_rows,
        active_summary: rect_for(FiltersPanelNode::ActiveSummary),
        layers_summary: rect_for(FiltersPanelNode::LayersSummary)?,
        focus_summary: rect_for(FiltersPanelNode::FocusSummary)?,
        outputs_summary: rect_for(FiltersPanelNode::OutputsSummary)?,
    })
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
    let content_y = inspector_rect.y + 84.0;
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
    drop(add_node);

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
