use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GuiMenuModel {
    pub schema: String,
    #[serde(default)]
    pub menubar: Vec<GuiMenu>,
    #[serde(default)]
    pub marking_menus: BTreeMap<String, GuiMarkingMenu>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GuiMenu {
    pub menu: String,
    #[serde(default)]
    pub active_editor: Option<String>,
    #[serde(default)]
    pub items: Vec<GuiMenuItem>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GuiMenuItem {
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub shortcut: Option<String>,
    #[serde(default)]
    pub destructive: bool,
    #[serde(default)]
    pub verb: Option<String>,
    #[serde(default)]
    pub gui_local: Option<String>,
    #[serde(default)]
    pub not_built: Option<String>,
    #[serde(default)]
    pub submenu: Option<String>,
}

impl GuiMenuItem {
    pub fn binding(&self) -> GuiMenuBinding<'_> {
        if let Some(reason) = self.not_built.as_deref() {
            GuiMenuBinding::NotBuilt(reason)
        } else if let Some(action) = self.gui_local.as_deref() {
            GuiMenuBinding::GuiLocal(action)
        } else if let Some(verb) = self.verb.as_deref() {
            GuiMenuBinding::Verb(verb)
        } else if let Some(submenu) = self.submenu.as_deref() {
            GuiMenuBinding::Submenu(submenu)
        } else {
            GuiMenuBinding::Empty
        }
    }

    pub fn is_phase_one_enabled(&self) -> bool {
        matches!(self.binding(), GuiMenuBinding::GuiLocal(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiMenuBinding<'a> {
    Verb(&'a str),
    GuiLocal(&'a str),
    NotBuilt(&'a str),
    Submenu(&'a str),
    Empty,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub struct GuiMarkingMenu {
    #[serde(default)]
    pub cardinal: BTreeMap<String, GuiMenuItem>,
    #[serde(default)]
    pub secondary: BTreeMap<String, GuiMenuItem>,
    #[serde(default)]
    pub overflow: Vec<GuiMenuItem>,
    #[serde(default)]
    pub submenus: BTreeMap<String, Vec<GuiMenuItem>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GuiIconSet {
    pub schema: String,
    #[serde(default)]
    pub icons: BTreeMap<String, GuiIconDef>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GuiIconDef {
    pub source: String,
    #[serde(default)]
    pub glyph: Option<String>,
    #[serde(default)]
    pub asset: Option<String>,
    pub status: String,
}

pub fn load_default_gui_menu_model() -> Result<GuiMenuModel> {
    serde_json::from_str(include_str!("../../../docs/gui/menu_model.json"))
        .context("parse docs/gui/menu_model.json")
}

pub fn load_default_gui_icon_set() -> Result<GuiIconSet> {
    serde_json::from_str(include_str!("../../../docs/gui/icon_set.json"))
        .context("parse docs/gui/icon_set.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_menu_and_icon_manifests_parse() {
        let menu = load_default_gui_menu_model().expect("menu model should parse");
        let icons = load_default_gui_icon_set().expect("icon set should parse");

        assert_eq!(menu.schema, "datum_menu_model_v1");
        assert_eq!(icons.schema, "datum_icon_set_v1");
        assert!(menu.menubar.iter().any(|menu| menu.menu == "File"));
        assert!(menu.menubar.iter().all(|menu| !menu.items.is_empty()));
    }

    #[test]
    fn phase_one_only_enables_gui_local_menu_items() {
        let menu = load_default_gui_menu_model().expect("menu model should parse");
        let fit = menu
            .menubar
            .iter()
            .find(|menu| menu.menu == "View")
            .and_then(|menu| menu.items.iter().find(|item| item.label == "Fit to Board"))
            .expect("View/Fit to Board should exist");
        let export = menu
            .menubar
            .iter()
            .find(|menu| menu.menu == "File")
            .and_then(|menu| {
                menu.items
                    .iter()
                    .find(|item| item.label.starts_with("Export"))
            })
            .expect("File/Export should exist");

        assert!(fit.is_phase_one_enabled());
        assert!(!export.is_phase_one_enabled());
    }

    #[test]
    fn every_menu_entry_has_declared_fallback_icon() {
        let menu = load_default_gui_menu_model().expect("menu model should parse");
        let icons = load_default_gui_icon_set().expect("icon set should parse");

        for menu_group in &menu.menubar {
            for item in &menu_group.items {
                let icon_id = item.icon.as_deref().unwrap_or_else(|| {
                    panic!("{}/{} is missing an icon", menu_group.menu, item.label)
                });
                let icon = icons.icons.get(icon_id).unwrap_or_else(|| {
                    panic!(
                        "{}/{} references undeclared icon {icon_id}",
                        menu_group.menu, item.label
                    )
                });
                assert!(
                    icon.glyph.is_some() || icon.asset.is_some() || icon.status == "mapped",
                    "icon {icon_id} needs a fallback glyph or asset declaration"
                );
            }
        }
    }

    #[test]
    fn view_menu_exposes_workspace_pane_ops() {
        // Decision 021: the View menu must emit the pane-op action ids the FEEL
        // dispatch already handles, each as a live gui_local item with a declared
        // icon. This locks the id↔dispatch wiring at the menu-manifest seam.
        let menu = load_default_gui_menu_model().expect("menu model should parse");
        let icons = load_default_gui_icon_set().expect("icon set should parse");
        let view = menu
            .menubar
            .iter()
            .find(|m| m.menu == "View")
            .expect("View menu should exist");
        let by_action: BTreeMap<&str, &GuiMenuItem> = view
            .items
            .iter()
            .filter_map(|it| it.gui_local.as_deref().map(|a| (a, it)))
            .collect();
        for action in [
            "view.split_vertical",
            "view.split_horizontal",
            "view.close_pane",
            "view.maximize_pane",
            "view.focus_next",
            "view.focus_prev",
            "view.fill_board",
            "view.fill_schematic",
            "view.preset_single",
            "view.preset_board_schematic",
        ] {
            let item = by_action
                .get(action)
                .unwrap_or_else(|| panic!("View menu missing gui_local action {action}"));
            assert!(
                item.is_phase_one_enabled(),
                "{action} must be a live gui_local item"
            );
            let icon = item
                .icon
                .as_deref()
                .unwrap_or_else(|| panic!("{action} missing an icon"));
            assert!(
                icons.icons.contains_key(icon),
                "{action} references undeclared icon {icon}"
            );
        }
        // Keyboard-mirrored shortcuts are present on the right items.
        assert_eq!(
            by_action.get("view.maximize_pane").unwrap().shortcut.as_deref(),
            Some("Z")
        );
        assert_eq!(
            by_action.get("view.focus_next").unwrap().shortcut.as_deref(),
            Some("Tab")
        );
        assert_eq!(
            by_action.get("view.focus_prev").unwrap().shortcut.as_deref(),
            Some("Shift+Tab")
        );
    }

    #[test]
    fn phase_one_marking_menus_cover_board_object_classes() {
        let menu = load_default_gui_menu_model().expect("menu model should parse");
        for key in [
            "pcb.component",
            "pcb.pad",
            "pcb.track",
            "pcb.via",
            "pcb.zone",
            "pcb.empty",
        ] {
            assert!(
                menu.marking_menus.contains_key(key),
                "missing marking menu {key}"
            );
        }
    }

    #[test]
    fn every_marking_menu_entry_has_declared_icon() {
        let menu = load_default_gui_menu_model().expect("menu model should parse");
        let icons = load_default_gui_icon_set().expect("icon set should parse");

        for (menu_key, marking_menu) in &menu.marking_menus {
            for (slot, item) in marking_menu
                .cardinal
                .iter()
                .chain(marking_menu.secondary.iter())
            {
                assert_marking_item_icon(menu_key, slot, item, &icons);
            }
            for (index, item) in marking_menu.overflow.iter().enumerate() {
                assert_marking_item_icon(menu_key, &format!("overflow[{index}]"), item, &icons);
            }
            for (submenu, items) in &marking_menu.submenus {
                for (index, item) in items.iter().enumerate() {
                    assert_marking_item_icon(
                        menu_key,
                        &format!("{submenu}[{index}]"),
                        item,
                        &icons,
                    );
                }
            }
        }
    }

    fn assert_marking_item_icon(
        menu_key: &str,
        slot: &str,
        item: &GuiMenuItem,
        icons: &GuiIconSet,
    ) {
        let icon_id = item
            .icon
            .as_deref()
            .unwrap_or_else(|| panic!("{menu_key}/{slot}/{} is missing an icon", item.label));
        assert!(
            icons.icons.contains_key(icon_id),
            "{menu_key}/{slot}/{} references undeclared icon {icon_id}",
            item.label
        );
    }
}
