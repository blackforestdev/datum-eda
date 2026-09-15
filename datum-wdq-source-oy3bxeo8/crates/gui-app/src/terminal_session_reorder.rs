use super::TerminalSessionRegistry;
use anyhow::Result;

impl TerminalSessionRegistry {
    /// Move one realized terminal session to another realized tab's position.
    /// The whole slot moves, so its PTY, screen, lifecycle, and parked
    /// projection cannot separate. Index-based authorities are remapped to the
    /// same identities after the move.
    pub(crate) fn reorder_session(&mut self, session_id: &str, target_id: &str) -> Result<bool> {
        let from = self
            .sessions
            .iter()
            .position(|slot| slot.session.session_id() == session_id)
            .ok_or_else(|| anyhow::anyhow!("terminal session not found: {session_id}"))?;
        let to = self
            .sessions
            .iter()
            .position(|slot| slot.session.session_id() == target_id)
            .ok_or_else(|| anyhow::anyhow!("terminal session not found: {target_id}"))?;
        if from == to {
            return Ok(false);
        }

        let active = remap_index(self.active_index, from, to);
        let next_drain = remap_index(self.next_drain_index, from, to);
        move_item(&mut self.sessions, from, to);
        let tab_from = self
            .terminal_tabs
            .iter()
            .position(|tab| tab.root.contains_session(session_id));
        let tab_to = self
            .terminal_tabs
            .iter()
            .position(|tab| tab.root.contains_session(target_id));
        if let (Some(tab_from), Some(tab_to)) = (tab_from, tab_to) {
            move_item(&mut self.terminal_tabs, tab_from, tab_to);
        }
        self.active_index = active;
        self.next_drain_index = next_drain;
        Ok(true)
    }
}

fn move_item<T>(items: &mut Vec<T>, from: usize, to: usize) {
    let item = items.remove(from);
    items.insert(to, item);
}

fn remap_index(index: usize, from: usize, to: usize) -> usize {
    if index == from {
        to
    } else if from < to && (from + 1..=to).contains(&index) {
        index - 1
    } else if to < from && (to..from).contains(&index) {
        index + 1
    } else {
        index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_session::TerminalLaunchContext;
    use datum_gui_protocol::TerminalLaneState;
    use std::fs;

    #[test]
    fn moving_tabs_remaps_active_and_fair_drain_indices_by_identity() {
        let mut tabs = vec!["shell 1", "shell 2", "shell 3"];
        move_item(&mut tabs, 0, 2);
        assert_eq!(tabs, ["shell 2", "shell 3", "shell 1"]);
        assert_eq!(remap_index(0, 0, 2), 2);
        assert_eq!(remap_index(1, 0, 2), 0);
        assert_eq!(remap_index(2, 0, 2), 1);
        assert_eq!(remap_index(2, 2, 0), 0);
        assert_eq!(remap_index(0, 2, 0), 1);
        assert_eq!(remap_index(1, 2, 0), 2);
        assert_eq!(remap_index(3, 0, 2), 3);
    }

    #[test]
    fn dropped_real_session_keeps_its_creation_label_and_projection_identity() {
        let root =
            std::env::temp_dir().join(format!("datum-terminal-tab-reorder-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let context = TerminalLaunchContext::for_project_root(&root);
        let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
        let first_id = registry.active().session_id().to_string();
        let second_id = registry.spawn_and_activate(&context).unwrap().to_string();
        let mut lane = TerminalLaneState::default();

        assert!(registry.reorder_session(&first_id, &second_id).unwrap());
        registry.activate_with_lane(&first_id, &mut lane).unwrap();
        registry.sync_lane_tabs(&mut lane);

        assert_eq!(
            (&lane.tabs[0].session_id, lane.tabs[0].label.as_str()),
            (&second_id, "shell 2")
        );
        assert_eq!(
            (&lane.tabs[1].session_id, lane.tabs[1].label.as_str()),
            (&first_id, "shell 1")
        );
        assert_eq!(
            lane.tab_layouts
                .iter()
                .flat_map(|tab| tab.root.session_ids())
                .collect::<Vec<_>>(),
            [second_id.as_str(), first_id.as_str()]
        );
        assert!(lane.tabs[1].active);
        let _ = fs::remove_dir_all(root);
    }
}
