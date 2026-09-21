//! Output invalidation for the always-visible terminal tab strip when the dock
//! is closed. Row/history/activity data still update; they do not paint there.
use datum_gui_protocol::TerminalLaneState;

pub(super) struct ClosedStrip {
    title: Option<String>,
    status: String,
    shutdown_blocked: Option<String>,
    tabs: Vec<Tab>,
}
struct Tab {
    session: String,
    label: String,
    active: bool,
    unread: bool,
    bell: bool,
}
impl ClosedStrip {
    fn eligible(dock_open: bool, lane: &TerminalLaneState) -> bool {
        // These controls expose changing text/state beyond the routine strip.
        !dock_open
            && !lane.search.active
            && lane.link_confirmation.is_none()
            && lane.clipboard_confirmation.is_none()
    }

    pub(super) fn capture(dock_open: bool, lane: &TerminalLaneState) -> Option<Self> {
        Self::eligible(dock_open, lane).then(|| Self {
            title: lane.title.clone(),
            status: lane.status.clone(),
            shutdown_blocked: lane.application_shutdown_blocked.clone(),
            tabs: lane
                .tabs
                .iter()
                .map(|tab| Tab {
                    session: tab.session_id.clone(),
                    label: tab.label.clone(),
                    active: tab.active,
                    unread: tab.unread_output,
                    bell: tab.unread_bell_count > 0,
                })
                .collect(),
        })
    }

    pub(super) fn unchanged(&self, dock_open: bool, lane: &TerminalLaneState) -> bool {
        Self::eligible(dock_open, lane)
            && self.title == lane.title
            && self.status == lane.status
            && self.shutdown_blocked == lane.application_shutdown_blocked
            && self.tabs.len() == lane.tabs.len()
            && self.tabs.iter().zip(&lane.tabs).all(|(old, tab)| {
                old.session == tab.session_id
                    && old.label == tab.label
                    && old.active == tab.active
                    && old.unread == tab.unread_output
                    && old.bell == (tab.unread_bell_count > 0)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::TerminalTabState;

    fn lane() -> TerminalLaneState {
        TerminalLaneState {
            tabs: vec![TerminalTabState {
                session_id: "session".into(),
                previous_session_id: None,
                label: "shell".into(),
                event_log_path: "log".into(),
                activity_event_count: 0,
                activity_summary: Vec::new(),
                active: true,
                attached: true,
                status: "running".into(),
                restart_count: 0,
                unread_output: false,
                unread_bell_count: 0,
            }],
            ..Default::default()
        }
    }

    #[test]
    fn closed_strip_ignores_history_metadata_but_preserves_visible_changes() {
        let mut lane = lane();
        let before = ClosedStrip::capture(false, &lane).unwrap();
        lane.tabs[0].activity_event_count = 1024;
        lane.tabs[0].activity_summary.push("output".into());
        lane.activity_summary.push("more output".into());
        assert!(before.unchanged(false, &lane));
        assert!(!before.unchanged(true, &lane));
        lane.tabs[0].unread_output = true;
        assert!(!before.unchanged(false, &lane));
        lane.tabs[0].unread_output = false;
        lane.tabs[0].unread_bell_count = 1;
        assert!(!before.unchanged(false, &lane));
        let bell = ClosedStrip::capture(false, &lane).unwrap();
        lane.tabs[0].unread_bell_count = 2;
        assert!(bell.unchanged(false, &lane)); // Same visible attention marker.
        lane.tabs[0].label = "build".into();
        assert!(!bell.unchanged(false, &lane));
    }

    #[test]
    fn status_tab_identity_and_search_cannot_be_suppressed() {
        let original = lane();
        let before = ClosedStrip::capture(false, &original).unwrap();
        for mutate in [
            |lane: &mut TerminalLaneState| lane.status = "exited 3".into(),
            |lane: &mut TerminalLaneState| lane.application_shutdown_blocked = Some("retry".into()),
            |lane: &mut TerminalLaneState| lane.tabs[0].session_id = "replacement".into(),
            |lane: &mut TerminalLaneState| lane.tabs[0].active = false,
            |lane: &mut TerminalLaneState| lane.title = Some("title".into()),
            |lane: &mut TerminalLaneState| lane.search.active = true,
        ] {
            let mut changed = original.clone();
            mutate(&mut changed);
            assert!(!before.unchanged(false, &changed));
        }
        assert!(ClosedStrip::capture(true, &original).is_none());
        let mut search = original;
        search.search.active = true;
        assert!(ClosedStrip::capture(false, &search).is_none());
    }
}
