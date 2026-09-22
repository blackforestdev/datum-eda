#[path = "inspector_layout.rs"]
mod inspector_layout;
use inspector_layout::solve_inspector_detail_layout_with_taffy;
pub(super) use inspector_layout::solve_right_panel_layout_with_taffy;

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

#[derive(Debug, Clone, PartialEq)]
struct InspectorDetailLayout {
    divider_y: Option<f32>,
    contract: Option<RectPx>,
    net: Option<RectPx>,
    segment: Option<RectPx>,
    layer: Option<RectPx>,
    last_status: Option<RectPx>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RightPanelLayout {
    pub(super) inspector_rect: RectPx,
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
