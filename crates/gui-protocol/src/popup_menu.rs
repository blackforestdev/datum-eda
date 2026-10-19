//! Shared popup interaction; clients supply typed commands and their admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopupItem<A> {
    pub label: String,
    pub shortcut: Option<String>,
    pub action: A,
    pub unavailable: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PopupMenu<A> {
    pub anchor: (f32, f32),
    pub items: Vec<PopupItem<A>>,
    pub focused: Option<usize>,
}

impl<A: Clone> PopupMenu<A> {
    pub fn new(anchor: (f32, f32), items: Vec<PopupItem<A>>) -> Self {
        let focused = items.iter().position(|item| item.unavailable.is_none());
        Self {
            anchor,
            items,
            focused,
        }
    }

    pub fn edge(&mut self, last: bool) {
        self.focused = if last {
            self.items.iter().rposition(|i| i.unavailable.is_none())
        } else {
            self.items.iter().position(|i| i.unavailable.is_none())
        };
    }

    pub fn step(&mut self, reverse: bool) {
        if self.items.is_empty() {
            self.focused = None;
            return;
        }
        let count = self.items.len();
        let start = self.focused.unwrap_or(if reverse { 0 } else { count - 1 });
        self.focused = (1..=count)
            .map(|offset| {
                if reverse {
                    (start + count - offset) % count
                } else {
                    (start + offset) % count
                }
            })
            .find(|&index| self.items[index].unavailable.is_none());
    }

    pub fn activate(&self, index: usize) -> Option<A> {
        self.items
            .get(index)
            .filter(|item| item.unavailable.is_none())
            .map(|item| item.action.clone())
    }

    pub fn activate_focused(&self) -> Option<A> {
        self.focused.and_then(|i| self.activate(i))
    }

    pub fn accessibility_nodes(
        &self,
        prefix: &str,
        title: &str,
        enabled: bool,
    ) -> Vec<crate::GlobalPreferencesAccessibleNode> {
        use crate::{
            GlobalPreferencesAccessibleNode as Node, GlobalPreferencesAccessibleRole as Role,
        };
        let mut nodes = vec![Node {
            id: prefix.into(),
            name: title.into(),
            role: Role::Menu,
            value: None,
            available: enabled,
            focused: false,
            description:
                "Up/Down chooses an available action; Enter invokes it; Escape dismisses this menu."
                    .into(),
        }];
        nodes.extend(self.items.iter().enumerate().map(|(index, item)| {
            Node {
                id: format!("{prefix}-item-{index}"),
                name: item.label.clone(),
                role: Role::MenuItem,
                value: item.shortcut.clone(),
                available: enabled && item.unavailable.is_none(),
                focused: self.focused == Some(index),
                description: item
                    .unavailable
                    .clone()
                    .unwrap_or_else(|| item.shortcut.clone().unwrap_or_default()),
            }
        }));
        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_popup_disabled_commands_never_invoke_and_keyboard_wraps() {
        let item = |action, unavailable| PopupItem {
            label: format!("{action}"),
            shortcut: None,
            action,
            unavailable,
        };
        let mut menu = PopupMenu::new(
            (4.0, 5.0),
            vec![
                item(0, None),
                item(1, Some("protected".into())),
                item(2, None),
            ],
        );
        assert_eq!(menu.activate_focused(), Some(0));
        assert_eq!(menu.activate(1), None);
        menu.step(false);
        assert_eq!(menu.activate_focused(), Some(2));
        menu.step(false);
        assert_eq!(menu.activate_focused(), Some(0));
        menu.step(true);
        assert_eq!(menu.activate_focused(), Some(2));
        menu.edge(false);
        assert_eq!(menu.activate_focused(), Some(0));
        menu.items[0].unavailable = Some("changed".into());
        assert_eq!(menu.activate_focused(), None);
    }
}
