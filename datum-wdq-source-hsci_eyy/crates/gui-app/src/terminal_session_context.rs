use datum_gui_protocol::{DockTab, TerminalLaneState, WorkspaceTool};
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub(super) struct TerminalSessionContextSummary {
    pub(super) active_session_id: Option<String>,
    pub(super) active_label: Option<String>,
    pub(super) active_status: Option<String>,
    pub(super) active_attached: bool,
    pub(super) active_working_directory: Option<String>,
    pub(super) tab_count: usize,
    pub(super) columns: u16,
    pub(super) rows: u16,
    pub(super) activity_summary: Vec<String>,
    pub(super) tabs: Vec<TerminalSessionContextTab>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct TerminalSessionContextTab {
    pub(super) session_id: String,
    pub(super) previous_session_id: Option<String>,
    pub(super) label: String,
    pub(super) event_log_path: String,
    pub(super) activity_event_count: usize,
    pub(super) activity_summary: Vec<String>,
    pub(super) active: bool,
    pub(super) attached: bool,
    pub(super) status: String,
    pub(super) restart_count: usize,
}

impl TerminalSessionContextSummary {
    pub(super) fn from_lane_state(state: &TerminalLaneState) -> Self {
        let active_tab = state.tabs.iter().find(|tab| tab.active);
        Self {
            active_session_id: state.active_session_id.clone(),
            active_label: active_tab.map(|tab| tab.label.clone()),
            active_status: active_tab.map(|tab| tab.status.clone()),
            active_attached: active_tab.map(|tab| tab.attached).unwrap_or(false),
            active_working_directory: state.current_working_directory.clone(),
            tab_count: state.tabs.len(),
            columns: state.columns,
            rows: state.rows,
            activity_summary: state.activity_summary.clone(),
            tabs: state
                .tabs
                .iter()
                .map(|tab| TerminalSessionContextTab {
                    session_id: tab.session_id.clone(),
                    previous_session_id: tab.previous_session_id.clone(),
                    label: tab.label.clone(),
                    event_log_path: tab.event_log_path.clone(),
                    activity_event_count: tab.activity_event_count,
                    activity_summary: tab.activity_summary.clone(),
                    active: tab.active,
                    attached: tab.attached,
                    status: tab.status.clone(),
                    restart_count: tab.restart_count,
                })
                .collect(),
        }
    }
}

pub(super) fn workspace_tool_name(tool: WorkspaceTool) -> &'static str {
    tool.label()
}

pub(super) fn dock_tab_name(tab: DockTab) -> &'static str {
    match tab {
        DockTab::Terminal => "terminal",
    }
}
