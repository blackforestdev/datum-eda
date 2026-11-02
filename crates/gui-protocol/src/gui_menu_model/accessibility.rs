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
    if let Some(marking) = state.ui.marking_menu.as_ref() {
        // Radial leaves share native context admission; the Select list stays available.
        let mut nodes = radial_marking_nodes(model, state, marking);
        if let Some(menu) = marking.selection.as_ref() {
            let root = format!("menu:native-select:{}", menu.project);
            nodes.push(menu_node(root.clone(), None, "Select"));
            if menu.open {
                for (i, item) in menu.items.iter().enumerate() {
                    nodes.push(MenuAccessibleNode {
                        id: format!("{root}/{:?}", item.command),
                        parent: Some(root.clone()),
                        name: item.label.clone(),
                        role: MenuAccessibleRole::MenuItem,
                        key: Some("Arrow keys, Enter".into()),
                        description: item.unavailable.clone().unwrap_or_else(|| {
                            "Read-only selection; complete source membership".into()
                        }),
                        available: item.unavailable.is_none(),
                        focused: state.ui.focus == ApplicationFocus::Overlay && menu.focus == i,
                    });
                }
            } else {
                nodes.push(MenuAccessibleNode {
                    id: format!("{root}/open"),
                    parent: Some(root.clone()),
                    name: "Select".into(),
                    role: MenuAccessibleRole::MenuItem,
                    key: Some("S".into()),
                    description: "Open read-only Select menu".into(),
                    available: true,
                    focused: state.ui.focus == ApplicationFocus::Overlay,
                });
            }
        }
        return nodes;
    }
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

/// Publish the same typed actions and unavailable leaves as the visible wheel.
fn radial_marking_nodes(
    model: &GuiMenuModel,
    state: &ReviewWorkspaceState,
    marking: &crate::MarkingMenuState,
) -> Vec<MenuAccessibleNode> {
    let Some(menu) = model.marking_menus.get(&marking.menu_key) else {
        return Vec::new();
    };
    let root = format!("menu:marking:{}", marking.menu_key);
    let mut nodes = vec![menu_node(
        root.clone(),
        None,
        &format!("Marking menu — {}", marking.menu_key),
    )];
    let overlay = state.ui.focus == ApplicationFocus::Overlay;
    if let Some(sub_key) = marking.submenu_path.last() {
        nodes.push(MenuAccessibleNode {
            id: format!("{root}/back"),
            parent: Some(root.clone()),
            name: "Back".into(),
            role: MenuAccessibleRole::MenuItem,
            key: Some("Centre".into()),
            description: "Return to the previous wheel".into(),
            available: true,
            focused: false,
        });
        if let Some(items) = menu.submenus.get(sub_key) {
            for (index, item) in items.iter().enumerate() {
                let slot = format!("SUB:{index}");
                nodes.push(radial_item_node(
                    &root, item, &slot, marking, overlay, state,
                ));
            }
        }
    } else {
        for slot in ["N", "E", "S", "W", "NE", "SE", "SW", "NW"] {
            if let Some(item) = menu.cardinal.get(slot).or_else(|| menu.secondary.get(slot)) {
                nodes.push(radial_item_node(&root, item, slot, marking, overlay, state));
            }
        }
        for (index, item) in menu.overflow.iter().enumerate() {
            let slot = format!("MORE:{index}");
            nodes.push(radial_item_node(
                &root, item, &slot, marking, overlay, state,
            ));
        }
    }
    nodes
}

fn radial_item_node(
    root: &str,
    item: &GuiMenuItem,
    slot: &str,
    marking: &crate::MarkingMenuState,
    overlay: bool,
    state: &ReviewWorkspaceState,
) -> MenuAccessibleNode {
    let navigable = matches!(item.binding(), GuiMenuBinding::Submenu(_));
    let (available, description) = if navigable {
        (true, format!("Opens the {} actions", item.label))
    } else if marking.menu_key.starts_with("schematic.") {
        match marking.schematic_action_unavailable_reason(item, state) {
            Some(reason) => (false, reason),
            None => (true, format!("{} for this context", item.label)),
        }
    } else {
        (
            false,
            item.unavailable_reason(state)
                .unwrap_or("This action is not invoked in this build.")
                .to_owned(),
        )
    };
    MenuAccessibleNode {
        id: format!("{root}/{slot}"),
        parent: Some(root.to_owned()),
        name: item.label.clone(),
        role: MenuAccessibleRole::MenuItem,
        key: Some(format!("{slot} (flick or click)")),
        description,
        available,
        focused: overlay && marking.preview_slot.as_deref() == Some(slot),
    }
}

#[cfg(test)]
mod radial_marking_tests {
    use super::*;
    fn radial_state(
        menu_key: &str,
        submenu_path: Vec<String>,
        preview: Option<&str>,
    ) -> ReviewWorkspaceState {
        let mut state = crate::load_fixture_workspace_state();
        state.ui.marking_menu = Some(crate::MarkingMenuState {
            menu_key: menu_key.into(),
            target_object_id: None,
            anchor_x_px: 640,
            anchor_y_px: 400,
            preview_slot: preview.map(|s| s.to_owned()),
            gesture_dx_px: 0,
            gesture_dy_px: 0,
            selection: None,
            submenu_path,
        });
        state.ui.focus = ApplicationFocus::Overlay;
        state
    }

    #[test]
    fn radial_marking_menu_emits_truthful_inert_nodes() {
        let model = crate::load_default_gui_menu_model().unwrap();
        let state = radial_state("pcb.multiselect", Vec::new(), Some("N"));
        let nodes = menu_accessibility_nodes(&model, &state);
        // The radial wheel yields a root Menu node plus item nodes (previously 0).
        assert!(nodes.iter().any(|n| n.role == MenuAccessibleRole::Menu));
        let rotate = nodes
            .iter()
            .find(|n| n.name == "Rotate")
            .expect("Rotate leaf present");
        assert!(!rotate.available, "an inert leaf must be unavailable");
        assert!(
            !rotate.description.is_empty(),
            "an inert leaf must carry a truthful reason"
        );
        assert!(rotate.focused, "preview_slot N focuses the N (Rotate) leaf");
        let align = nodes
            .iter()
            .find(|n| n.name == "Align")
            .expect("Align submenu parent present");
        assert!(align.available, "a submenu parent navigates (available)");
        assert!(align.description.contains("Align"));
        // Nested level: a Back node plus the Align children.
        let sub = menu_accessibility_nodes(
            &model,
            &radial_state("pcb.multiselect", vec!["Align".into()], None),
        );
        assert!(sub.iter().any(|n| n.name == "Back"), "sub-wheel has Back");
        assert!(
            sub.iter().any(|n| n.name == "Horizontal"),
            "Align sub-wheel exposes its children"
        );
    }
}
