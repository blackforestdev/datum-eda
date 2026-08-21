//! Renderer-neutral ownership tree for terminal tabs and split panes.
//!
//! A tab owns one recursive tree. Every leaf names exactly one Datum terminal
//! session; split nodes contain layout only and never duplicate PTY or screen
//! state. The app owns mutation, while viewport/render consumers receive this
//! inert projection so all layers agree about pane identity.

/// Direction in which a terminal pane is divided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalSplitDirection {
    /// Children are placed left and right.
    SideBySide,
    /// Children are placed above and below.
    Stacked,
}

/// One edge in a root-to-split path. Paths keep divider identity stable even
/// when sibling leaves have unrelated process/session identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalSplitChild {
    First,
    Second,
}

/// A persistent terminal-pane ownership tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalSplitNode {
    Session {
        session_id: String,
    },
    Split {
        direction: TerminalSplitDirection,
        /// First-child share in thousandths. Consumers clamp malformed values
        /// to the safe 10%..90% layout interval; app mutations emit 500.
        ratio_millis: u16,
        first: Box<Self>,
        second: Box<Self>,
    },
}

impl TerminalSplitNode {
    pub fn session(session_id: impl Into<String>) -> Self {
        Self::Session {
            session_id: session_id.into(),
        }
    }

    pub fn contains_session(&self, sought: &str) -> bool {
        match self {
            Self::Session { session_id } => session_id == sought,
            Self::Split { first, second, .. } => {
                first.contains_session(sought) || second.contains_session(sought)
            }
        }
    }

    pub fn session_ids(&self) -> Vec<&str> {
        let mut ids = Vec::new();
        self.collect_session_ids(&mut ids);
        ids
    }

    fn collect_session_ids<'a>(&'a self, ids: &mut Vec<&'a str>) {
        match self {
            Self::Session { session_id } => ids.push(session_id),
            Self::Split { first, second, .. } => {
                first.collect_session_ids(ids);
                second.collect_session_ids(ids);
            }
        }
    }

    pub fn set_ratio_at_path(&mut self, path: &[TerminalSplitChild], ratio_millis: u16) -> bool {
        if path.is_empty() {
            let Self::Split {
                ratio_millis: ratio,
                ..
            } = self
            else {
                return false;
            };
            *ratio = ratio_millis.clamp(100, 900);
            return true;
        }
        let Self::Split { first, second, .. } = self else {
            return false;
        };
        match path[0] {
            TerminalSplitChild::First => first.set_ratio_at_path(&path[1..], ratio_millis),
            TerminalSplitChild::Second => second.set_ratio_at_path(&path[1..], ratio_millis),
        }
    }
}

/// One terminal tab. Its focused session is always one leaf in `root`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalTabLayout {
    pub tab_id: String,
    pub focused_session_id: String,
    pub root: TerminalSplitNode,
}

impl TerminalTabLayout {
    pub fn single(session_id: impl Into<String>) -> Self {
        let session_id = session_id.into();
        Self {
            tab_id: session_id.clone(),
            focused_session_id: session_id.clone(),
            root: TerminalSplitNode::session(session_id),
        }
    }

    pub fn is_consistent(&self) -> bool {
        self.root.contains_session(&self.focused_session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recursive_tree_preserves_leaf_order_and_focus_identity() {
        let tab = TerminalTabLayout {
            tab_id: "tab-a".to_string(),
            focused_session_id: "right-bottom".to_string(),
            root: TerminalSplitNode::Split {
                direction: TerminalSplitDirection::SideBySide,
                ratio_millis: 500,
                first: Box::new(TerminalSplitNode::session("left")),
                second: Box::new(TerminalSplitNode::Split {
                    direction: TerminalSplitDirection::Stacked,
                    ratio_millis: 500,
                    first: Box::new(TerminalSplitNode::session("right-top")),
                    second: Box::new(TerminalSplitNode::session("right-bottom")),
                }),
            },
        };

        assert_eq!(
            tab.root.session_ids(),
            ["left", "right-top", "right-bottom"]
        );
        assert!(tab.is_consistent());
        assert!(!tab.root.contains_session("another-tab"));
        let mut resized = tab.clone();
        assert!(
            resized
                .root
                .set_ratio_at_path(&[TerminalSplitChild::Second], 735,)
        );
        let TerminalSplitNode::Split { second, .. } = &resized.root else {
            panic!("root remains split");
        };
        assert!(matches!(
            second.as_ref(),
            TerminalSplitNode::Split {
                ratio_millis: 735,
                ..
            }
        ));
    }
}
