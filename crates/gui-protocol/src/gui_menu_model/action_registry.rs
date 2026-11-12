//! Production GUI-local dispatch identity and bounded pilot admission (PM041).
//! Other families retain their existing owners and contextual behavior.

use crate::{ReviewWorkspaceState, camera_scene_for_pane};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionHandler {
    FitCamera,
    FocusNavigator,
    OpenLibrary,
}

/// All existing GUI-local dispatch targets. Partial pilot consumers keep their
/// contextual admission owner; other families retain their runtime behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiLocalHandler {
    Registered(ActionHandler),
    SaveProject,
    NewProject,
    OpenProject,
    OpenRecentProject(usize),
    PowerPort,
    SymbolFields,
    BoardTransfer,
    SymbolFieldsTable,
    AnnotateSheet,
    AnnotateProject,
    PlaceAnotherGate,
    PlaceChildSheet,
    DeleteSchematicSelection,
    CopyDesign,
    CutDesign,
    PasteDesign,
    DuplicateDesign,
    WireEdit(super::wire_commands::WireCommand),
    GlobalPreferences,
    ProjectPreferences,
    ZoomIn,
    ZoomOut,
    ToggleTerminal,
    SplitVertical,
    SplitHorizontal,
    ClosePane,
    FocusNext,
    FocusPrevious,
    MaximizePane,
    PresetSingle,
    PresetBoardSchematic,
    FillBoard,
    FillSchematic,
    CursorFull,
    CursorSmall,
    CursorNone,
    ConsoleHistory,
}

pub const UNAVAILABLE_GUI_LOCAL_REASON: &str = "This action is unavailable in this build.";

/// Shared by menu readiness and exhaustive runtime dispatch. A missing key never
/// acquires an executable fallback, including keys added to the menu manifest.
pub fn gui_local_handler(key: &str) -> Option<GuiLocalHandler> {
    if let Some(command) = super::wire_commands::WireCommand::from_action(key) {
        return Some(GuiLocalHandler::WireEdit(command));
    }
    if let Some(index) = key.strip_prefix("project.open_recent.") {
        let parsed = index.parse::<usize>().ok()?;
        return (index == parsed.to_string()).then_some(GuiLocalHandler::OpenRecentProject(parsed));
    }
    if let Some(entry) = consumer(key) {
        return entry.handler.map(GuiLocalHandler::Registered);
    }
    Some(match key {
        "edit.copy" => GuiLocalHandler::CopyDesign,
        "edit.cut" => GuiLocalHandler::CutDesign,
        "edit.paste" => GuiLocalHandler::PasteDesign,
        "edit.duplicate" => GuiLocalHandler::DuplicateDesign,
        "project.save" => GuiLocalHandler::SaveProject,
        "project.new.open" => GuiLocalHandler::NewProject,
        "project.open" => GuiLocalHandler::OpenProject,
        "schematic.power_port.open" => GuiLocalHandler::PowerPort,
        "schematic.symbol_fields.open" => GuiLocalHandler::SymbolFields,
        "schematic.board_transfer.open" => GuiLocalHandler::BoardTransfer,
        "schematic.symbol_fields_table.open" => GuiLocalHandler::SymbolFieldsTable,
        "schematic.annotation.review" => GuiLocalHandler::AnnotateSheet,
        "schematic.annotation.project" => GuiLocalHandler::AnnotateProject,
        "schematic.child_sheet.place" => GuiLocalHandler::PlaceChildSheet,
        "schematic.gate.place" => GuiLocalHandler::PlaceAnotherGate,
        "schematic.selection.delete" => GuiLocalHandler::DeleteSchematicSelection,
        "preferences.global.open" => GuiLocalHandler::GlobalPreferences,
        "preferences.project.open" => GuiLocalHandler::ProjectPreferences,
        "view.zoom_in" => GuiLocalHandler::ZoomIn,
        "view.zoom_out" => GuiLocalHandler::ZoomOut,
        "terminal.toggle" => GuiLocalHandler::ToggleTerminal,
        "view.split_vertical" => GuiLocalHandler::SplitVertical,
        "view.split_horizontal" => GuiLocalHandler::SplitHorizontal,
        "view.close_pane" => GuiLocalHandler::ClosePane,
        "view.focus_next" => GuiLocalHandler::FocusNext,
        "view.focus_prev" => GuiLocalHandler::FocusPrevious,
        "view.maximize_pane" => GuiLocalHandler::MaximizePane,
        "view.preset_single" => GuiLocalHandler::PresetSingle,
        "view.preset_board_schematic" => GuiLocalHandler::PresetBoardSchematic,
        "view.fill_board" => GuiLocalHandler::FillBoard,
        "view.fill_schematic" => GuiLocalHandler::FillSchematic,
        "view.cursor.full" => GuiLocalHandler::CursorFull,
        "view.cursor.small" => GuiLocalHandler::CursorSmall,
        "view.cursor.none" => GuiLocalHandler::CursorNone,
        "view.console_history" => GuiLocalHandler::ConsoleHistory,
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy)]
pub struct ActionConsumer {
    pub key: &'static str,
    pub handler: Option<ActionHandler>,
    pub entry_surfaces: &'static [&'static str],
    pub unavailable_reason: &'static str,
}

pub const ACTION_CONSUMERS: &[ActionConsumer] = &[
    ActionConsumer {
        key: "library.open",
        handler: Some(ActionHandler::OpenLibrary),
        entry_surfaces: &["library-menu-keyboard", "library-menu-pointer"],
        unavailable_reason: "Library Browser requires an open native Datum Project.",
    },
    ActionConsumer {
        key: "view.fit",
        handler: Some(ActionHandler::FitCamera),
        entry_surfaces: &[
            "view-fit-menu-keyboard",
            "view-fit-menu-pointer",
            "view-fit-shortcut",
        ],
        unavailable_reason: "Fit is unavailable because the focused pane has no resolved camera view.",
    },
    ActionConsumer {
        key: "view.layers",
        handler: None,
        entry_surfaces: &["view-layers-menu-keyboard", "view-layers-menu-pointer"],
        unavailable_reason: "Layer Visibility menu action is unavailable in this build.",
    },
    ActionConsumer {
        key: "window.documents",
        handler: Some(ActionHandler::FocusNavigator),
        entry_surfaces: &[
            "window-documents-menu-keyboard",
            "window-documents-menu-pointer",
        ],
        unavailable_reason: "Documents menu action is unavailable in this build.",
    },
    ActionConsumer {
        key: "help.about",
        handler: None,
        entry_surfaces: &["help-about-menu-keyboard", "help-about-menu-pointer"],
        unavailable_reason: "About Datum is unavailable in this build.",
    },
];

pub fn consumer(key: &str) -> Option<&'static ActionConsumer> {
    ACTION_CONSUMERS.iter().find(|entry| entry.key == key)
}

impl ActionConsumer {
    pub fn admit(&self, state: &ReviewWorkspaceState) -> Result<ActionHandler, &'static str> {
        match self.handler {
            Some(ActionHandler::OpenLibrary) if state.library.basis.is_some() => {
                Ok(ActionHandler::OpenLibrary)
            }
            Some(ActionHandler::FocusNavigator) if state.schematic_documents.project.is_some() => {
                Ok(ActionHandler::FocusNavigator)
            }
            Some(ActionHandler::FitCamera)
                if camera_scene_for_pane(state, state.ui.layout.focused).is_some() =>
            {
                Ok(ActionHandler::FitCamera)
            }
            _ => Err(self.unavailable_reason),
        }
    }

    /// Export identity comes from the typed dispatch target, not a parallel list
    /// of purported handlers. Native proof must still establish its actual effect.
    pub fn handler_ref(&self) -> Option<(&'static str, &'static str)> {
        match self.handler? {
            ActionHandler::OpenLibrary => Some((
                "crates/gui-app/src/editor/runtime_library.rs",
                "datum_gui::Runtime::open_library",
            )),
            ActionHandler::FocusNavigator => Some((
                "crates/gui-app/src/editor/runtime_schematic_navigation.rs",
                "datum_gui::Runtime::focus_schematic_navigator",
            )),
            ActionHandler::FitCamera => Some((
                "crates/gui-app/src/viewport/runtime_camera_pane.rs",
                "datum_gui::Runtime::fit_camera",
            )),
        }
    }
}

pub fn admit(key: &str, state: &ReviewWorkspaceState) -> Result<ActionHandler, &'static str> {
    consumer(key)
        .ok_or("No production consumer is registered for this action.")?
        .admit(state)
}

#[cfg(test)]
mod tests;

/// Clipboard readiness is a source/selection decision, never a platform read
/// during painting. The editor/text/terminal input owners dispatch first.
pub fn design_clipboard_unavailable_reason(
    key: &str,
    state: &ReviewWorkspaceState,
) -> Option<Option<&'static str>> {
    if !matches!(
        key,
        "edit.copy" | "edit.cut" | "edit.paste" | "edit.duplicate"
    ) {
        return None;
    }
    let reason = (|| {
        if state.ui.layout.focused_content() != crate::PaneContent::Schematic {
            return Some("Focus a schematic source pane");
        }
        if let Some(reason) =
            super::native_project_verb_unavailable_reason(state, "datum.schematic.draw_wire")
        {
            return Some(reason);
        }
        if state.schematic_authoring.active()
            || state.schematic_authoring.picker
            || state.schematic_authoring.delete_review.is_some()
        {
            return Some("Finish or cancel the current schematic gesture first");
        }
        if key != "edit.paste" {
            let crate::SelectionTarget::Native(view) = &state.selection else {
                return Some("Select Symbols, wires or junctions first");
            };
            if key == "edit.cut"
                && view
                    .members
                    .iter()
                    .any(|m| m.class == crate::AuthoredSelectionClass::Symbol)
            {
                return Some("Symbol Cut is not supported; use Copy or Duplicate");
            }
            if view.stale
                || view.members.is_empty()
                || view.members.iter().any(|id| {
                    !matches!(
                        id.class,
                        crate::AuthoredSelectionClass::Wire
                            | crate::AuthoredSelectionClass::Junction
                            | crate::AuthoredSelectionClass::Symbol
                    )
                })
            {
                return Some("Select current Symbols, wires or junctions");
            }
        }
        None
    })();
    Some(reason)
}
