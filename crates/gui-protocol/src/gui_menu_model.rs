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
}
