//! Persistent terminal tab/split ownership.
//!
//! This module owns identity mutation only. PTYs and TerminalCore instances
//! remain in `TerminalSessionSlot`; each tree leaf refers to exactly one slot.

use super::{PendingTerminalSpawn, TerminalSessionRegistry};
use anyhow::{Result, anyhow};
use datum_gui_protocol::{
    TerminalLaneState, TerminalSplitDirection, TerminalSplitNode, TerminalTabLayout,
};

impl TerminalSessionRegistry {
    pub(super) fn add_standalone_terminal_tab(&mut self, session_id: impl Into<String>) {
        self.terminal_tabs
            .push(TerminalTabLayout::single(session_id));
    }

    pub(super) fn replace_pending_terminal_tab(
        &mut self,
        pending_id: &str,
        session_id: impl Into<String>,
    ) {
        let session_id = session_id.into();
        if let Some(tab) = self
            .terminal_tabs
            .iter_mut()
            .find(|tab| tab.tab_id == pending_id)
        {
            *tab = TerminalTabLayout::single(session_id);
            return;
        }
        for tab in &mut self.terminal_tabs {
            if replace_leaf_id(&mut tab.root, pending_id, &session_id) {
                tab.focused_session_id = session_id;
                return;
            }
        }
        self.add_standalone_terminal_tab(session_id);
    }

    pub(super) fn remove_terminal_tab_session(&mut self, session_id: &str) {
        self.terminal_tabs = self
            .terminal_tabs
            .drain(..)
            .filter_map(|mut tab| {
                tab.root = remove_leaf(tab.root, session_id)?;
                if tab.focused_session_id == session_id {
                    tab.focused_session_id = tab.root.session_ids()[0].to_string();
                }
                Some(tab)
            })
            .collect();
    }

    #[allow(dead_code)]
    pub(super) fn split_terminal_session(
        &mut self,
        existing_session_id: &str,
        new_session_id: impl Into<String>,
        direction: TerminalSplitDirection,
    ) -> Result<()> {
        let new_session_id = new_session_id.into();
        if self
            .terminal_tabs
            .iter()
            .any(|candidate| candidate.root.contains_session(&new_session_id))
        {
            return Err(anyhow!(
                "terminal split target already exists: {new_session_id}"
            ));
        }
        let tab = self
            .terminal_tabs
            .iter_mut()
            .find(|tab| tab.root.contains_session(existing_session_id))
            .ok_or_else(|| anyhow!("terminal split source not found: {existing_session_id}"))?;
        if !split_leaf(
            &mut tab.root,
            existing_session_id,
            &new_session_id,
            direction,
        ) {
            return Err(anyhow!(
                "terminal split source disappeared: {existing_session_id}"
            ));
        }
        tab.focused_session_id = new_session_id;
        Ok(())
    }

    pub(super) fn sync_terminal_tab_layouts(&self, state: &mut TerminalLaneState) {
        state.tab_layouts = self.terminal_tabs.clone();
        state.active_tab_id = if let Some(pending_id) = self.active_pending_id.as_ref() {
            Some(pending_id.clone())
        } else {
            self.active_tab_for_session(self.active().session_id())
                .map(|tab| tab.tab_id.clone())
        };
    }

    pub(super) fn focus_terminal_session_in_tab(&mut self, session_id: &str) {
        if let Some(tab) = self
            .terminal_tabs
            .iter_mut()
            .find(|tab| tab.root.contains_session(session_id))
        {
            tab.focused_session_id = session_id.to_string();
        }
    }

    pub(super) fn replace_terminal_session_identity(&mut self, old_id: &str, new_id: &str) {
        for tab in &mut self.terminal_tabs {
            if replace_leaf_id(&mut tab.root, old_id, new_id) {
                if tab.focused_session_id == old_id {
                    tab.focused_session_id = new_id.to_string();
                }
                return;
            }
        }
    }

    pub(crate) fn set_active_split_ratio(
        &mut self,
        path: &[datum_gui_protocol::TerminalSplitChild],
        ratio_millis: u16,
    ) -> Result<()> {
        let session_id = self.active().session_id().to_string();
        let tab = self
            .terminal_tabs
            .iter_mut()
            .find(|tab| tab.root.contains_session(&session_id))
            .ok_or_else(|| anyhow!("active terminal tab layout not found: {session_id}"))?;
        if !tab.root.set_ratio_at_path(path, ratio_millis) {
            return Err(anyhow!("terminal split divider path is stale"));
        }
        Ok(())
    }

    fn active_tab_for_session(&self, session_id: &str) -> Option<&TerminalTabLayout> {
        self.terminal_tabs
            .iter()
            .find(|tab| tab.root.contains_session(session_id))
    }

    pub(super) fn remove_pending_terminal_tab(&mut self, pending: &PendingTerminalSpawn) {
        self.remove_terminal_tab_session(&pending.pending_id);
    }
}

#[allow(dead_code)]
fn split_leaf(
    node: &mut TerminalSplitNode,
    existing_session_id: &str,
    new_session_id: &str,
    direction: TerminalSplitDirection,
) -> bool {
    match node {
        TerminalSplitNode::Session { session_id } if session_id == existing_session_id => {
            let existing = TerminalSplitNode::session(session_id.clone());
            *node = TerminalSplitNode::Split {
                direction,
                ratio_millis: 500,
                first: Box::new(existing),
                second: Box::new(TerminalSplitNode::session(new_session_id)),
            };
            true
        }
        TerminalSplitNode::Session { .. } => false,
        TerminalSplitNode::Split { first, second, .. } => {
            split_leaf(first, existing_session_id, new_session_id, direction)
                || split_leaf(second, existing_session_id, new_session_id, direction)
        }
    }
}

fn replace_leaf_id(node: &mut TerminalSplitNode, old_id: &str, new_id: &str) -> bool {
    match node {
        TerminalSplitNode::Session { session_id } if session_id == old_id => {
            *session_id = new_id.to_string();
            true
        }
        TerminalSplitNode::Session { .. } => false,
        TerminalSplitNode::Split { first, second, .. } => {
            replace_leaf_id(first, old_id, new_id) || replace_leaf_id(second, old_id, new_id)
        }
    }
}

fn remove_leaf(node: TerminalSplitNode, session_id: &str) -> Option<TerminalSplitNode> {
    match node {
        TerminalSplitNode::Session {
            session_id: current,
        } => (current != session_id).then(|| TerminalSplitNode::session(current)),
        TerminalSplitNode::Split {
            direction,
            ratio_millis,
            first,
            second,
        } => match (
            remove_leaf(*first, session_id),
            remove_leaf(*second, session_id),
        ) {
            (Some(first), Some(second)) => Some(TerminalSplitNode::Split {
                direction,
                ratio_millis,
                first: Box::new(first),
                second: Box::new(second),
            }),
            (Some(remaining), None) | (None, Some(remaining)) => Some(remaining),
            (None, None) => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_and_remove_preserve_session_identity_and_collapse_parent() {
        let mut root = TerminalSplitNode::session("shell-a");
        assert!(split_leaf(
            &mut root,
            "shell-a",
            "shell-b",
            TerminalSplitDirection::SideBySide,
        ));
        assert!(split_leaf(
            &mut root,
            "shell-b",
            "shell-c",
            TerminalSplitDirection::Stacked,
        ));
        assert_eq!(root.session_ids(), ["shell-a", "shell-b", "shell-c"]);

        let root = remove_leaf(root, "shell-b").expect("two leaves remain");
        assert_eq!(root.session_ids(), ["shell-a", "shell-c"]);
        let root = remove_leaf(root, "shell-c").expect("one leaf remains");
        assert_eq!(root, TerminalSplitNode::session("shell-a"));
        assert!(remove_leaf(root, "shell-a").is_none());
    }
}
