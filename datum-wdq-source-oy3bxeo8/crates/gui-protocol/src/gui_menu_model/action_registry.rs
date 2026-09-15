//! Bounded production action admission (PM041). Other families retain their
//! existing owners; a missing key in this registry is never an executable action.

use crate::{ReviewWorkspaceState, camera_scene_for_pane};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionHandler {
    FitCamera,
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
        handler: None,
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
            ActionHandler::FitCamera => Some((
                "crates/gui-app/src/runtime_camera_pane.rs",
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
