//! Shared immutable menu inventory, schema and compiled source loading.
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};
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
    #[serde(default)]
    pub submenus: BTreeMap<String, Vec<GuiMenuItem>>,
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
    #[serde(default)]
    pub requires_project: bool,
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
    default_gui_menu_model().cloned()
}

/// Borrow the immutable, compiled-in inventory without parsing or cloning it on
/// each frame. Action availability still comes from the caller's live workspace.
pub fn default_gui_menu_model() -> Result<&'static GuiMenuModel> {
    static MODEL: OnceLock<std::result::Result<GuiMenuModel, serde_json::Error>> = OnceLock::new();
    MODEL
        .get_or_init(|| serde_json::from_str(include_str!("../../../../docs/gui/menu_model.json")))
        .as_ref()
        .map_err(|error| anyhow::anyhow!("{error}"))
        .context("parse docs/gui/menu_model.json")
}

pub fn load_default_gui_icon_set() -> Result<GuiIconSet> {
    serde_json::from_str(include_str!("../../../../docs/gui/icon_set.json"))
        .context("parse docs/gui/icon_set.json")
}
