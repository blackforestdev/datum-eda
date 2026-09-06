//! Native menu projection from the production inventory and contextual admission.

use super::{GuiMenuBinding, GuiMenuItem, GuiMenuModel};
use crate::{ApplicationFocus, ReviewWorkspaceState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAccessibleRole {
    Menu,
    MenuItem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuAccessibleNode {
    pub id: String,
    pub parent: Option<String>,
    pub name: String,
    pub role: MenuAccessibleRole,
    pub key: Option<String>,
    pub description: String,
    pub available: bool,
    pub focused: bool,
}

pub fn menu_accessibility_nodes(
    model: &GuiMenuModel,
    state: &ReviewWorkspaceState,
) -> Vec<MenuAccessibleNode> {
    let Some(active) = state.ui.active_menu.as_deref() else {
        return Vec::new();
    };
    let Some(group) = model.menubar.iter().find(|group| group.menu == active) else {
        return Vec::new();
    };
    let root = format!("menu:{active}");
    let mut nodes = vec![menu_node(root.clone(), None, active)];
    let focused = state.ui.focus == ApplicationFocus::Overlay;
    append_rows(
        &mut nodes,
        &root,
        &group.items,
        state,
        (focused && state.ui.active_submenu.is_none()).then_some(state.ui.menu_focus_index),
    );
    if let Some(submenu) = state.ui.active_submenu.as_deref()
        && let Some(items) = group.submenus.get(submenu)
        && let Some(parent) = group
            .items
            .iter()
            .find(|item| item.submenu.as_deref() == Some(submenu))
    {
        let id = format!("menu:{submenu}");
        nodes.push(menu_node(
            id.clone(),
            Some(row_id(&root, parent)),
            &parent.label,
        ));
        append_rows(
            &mut nodes,
            &id,
            items,
            state,
            focused.then_some(state.ui.menu_focus_index),
        );
    }
    nodes
}

fn menu_node(id: String, parent: Option<String>, name: &str) -> MenuAccessibleNode {
    MenuAccessibleNode {
        id,
        parent,
        name: name.to_owned(),
        role: MenuAccessibleRole::Menu,
        key: None,
        description: "Menu navigation".to_owned(),
        available: true,
        focused: false,
    }
}

fn row_key(item: &GuiMenuItem) -> &str {
    match item.binding() {
        GuiMenuBinding::GuiLocal(key)
        | GuiMenuBinding::Verb(key)
        | GuiMenuBinding::Submenu(key) => key,
        _ => &item.label,
    }
}

fn row_id(parent: &str, item: &GuiMenuItem) -> String {
    format!("{parent}/{}", row_key(item))
}

fn append_rows(
    nodes: &mut Vec<MenuAccessibleNode>,
    parent: &str,
    items: &[GuiMenuItem],
    state: &ReviewWorkspaceState,
    focused: Option<usize>,
) {
    for (index, item) in items.iter().enumerate() {
        let reason = item.unavailable_reason(state);
        nodes.push(MenuAccessibleNode {
            id: row_id(parent, item),
            parent: Some(parent.to_owned()),
            name: item.label.clone(),
            role: MenuAccessibleRole::MenuItem,
            key: Some(row_key(item).to_owned()),
            description: reason.unwrap_or("Available").to_owned(),
            available: reason.is_none(),
            focused: focused == Some(index),
        });
    }
}
