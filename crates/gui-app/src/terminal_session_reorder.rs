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
}
