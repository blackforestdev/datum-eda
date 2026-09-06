//! Opt-in observation of production state/dispatch for the bounded PM041 pilot.
//! No input injection, project writer, availability override or acceptance flag.

use crate::Runtime;
use datum_gui_protocol::gui_menu_model::action_registry::{self, ACTION_CONSUMERS};
use datum_gui_protocol::{ReviewWorkspaceState, camera_scene_for_pane};
use serde_json::{Value, json};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Copy)]
pub(crate) enum EntrySurface {
    MenuPointer,
    MenuKeyboard,
    Shortcut,
}

impl EntrySurface {
    fn suffix(self) -> &'static str {
        match self {
            Self::MenuPointer => "menu-pointer",
            Self::MenuKeyboard => "menu-keyboard",
            Self::Shortcut => "shortcut",
        }
    }
}

fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("DATUM_ACTION_EVIDENCE").as_deref() == Ok("1"))
}

fn context(state: &ReviewWorkspaceState) -> String {
    let pane = state.ui.layout.focused;
    format!(
        "pane-{}-{}",
        pane.0,
        if camera_scene_for_pane(state, pane).is_some() {
            "resolved"
        } else {
            "unresolved"
        }
    )
}

fn handler(entry: &action_registry::ActionConsumer) -> Value {
    entry
        .handler_ref()
        .map(|(path, symbol)| json!({"path": path, "symbol": symbol}))
        .unwrap_or(Value::Null)
}

fn registry(state: &ReviewWorkspaceState) -> Value {
    let context = context(state);
    Value::Array(
        ACTION_CONSUMERS
            .iter()
            .map(|entry| {
                let admission = entry.admit(state);
                json!({
                    "dispatch_key": entry.key,
                    "handler_ref": handler(entry),
                    "entry_surfaces": entry.entry_surfaces,
                    "contexts": {context.clone(): {
                        "enabled": admission.is_ok(),
                        "reason": admission.err().unwrap_or("Available")
                    }}
                })
            })
            .collect(),
    )
}

fn attempt(
    state: &ReviewWorkspaceState,
    key: &str,
    surface: EntrySurface,
    invoked: bool,
) -> Option<Value> {
    let entry = action_registry::consumer(key)?;
    let suffix = surface.suffix();
    let surface = entry
        .entry_surfaces
        .iter()
        .find(|surface| surface.ends_with(suffix))?;
    let admission = entry.admit(state);
    Some(json!({
        "dispatch_key": key, "entry_surface": surface, "context": context(state),
        "eligible": admission.is_ok(), "enabled": admission.is_ok(),
        "invoked": invoked, "handler_ref": handler(entry),
        "unavailable_reason": admission.err().unwrap_or("Available")
    }))
}

#[derive(Default)]
struct Stream {
    sequence: u64,
    previous_state: Option<Value>,
}

fn emit(event: &str, value: Value) {
    static STREAM: Mutex<Stream> = Mutex::new(Stream {
        sequence: 0,
        previous_state: None,
    });
    let Ok(mut stream) = STREAM.lock() else {
        return;
    };
    if event == "state" {
        if stream.previous_state.as_ref() == Some(&value) {
            return;
        }
        stream.previous_state = Some(value.clone());
    }
    stream.sequence += 1;
    let record = json!({"kind": "datum.action-evidence/v1", "sequence": stream.sequence,
        "event": event, "value": value});
    // A closed diagnostic pipe must not panic or change product behavior.
    let _ = writeln!(std::io::stderr().lock(), "DATUM_ACTION_EVIDENCE {record}");
}

impl Runtime {
    pub(crate) fn trace_action_state(&self) {
        if !enabled() {
            return;
        }
        let state = self.workspace();
        let camera = self.camera;
        let panes = self.current_layout().viewport_panes(&state.ui.layout);
        let cameras: Vec<_> = panes
            .panes
            .iter()
            .map(|pane| {
                let camera = if panes.scene_leaf_id() == Some(pane.id) {
                    Some(self.camera)
                } else {
                    self.pane_cameras.camera(pane.id, pane.content)
                };
                json!({"pane": pane.id.0, "content": format!("{:?}", pane.content),
                "camera": camera.map(|camera| json!({"center_x_nm": camera.center_x_nm,
                    "center_y_nm": camera.center_y_nm, "zoom": camera.zoom}))})
            })
            .collect();
        emit(
            "state",
            json!({
                "context": context(state), "registry": registry(state),
                "focused_pane": state.ui.layout.focused.0,
                "focused_content": format!("{:?}", state.ui.layout.content_for(state.ui.layout.focused)),
                "focus": format!("{:?}", state.ui.focus),
                "selection": format!("{:?}", state.selection),
                "layout": format!("{:?}", state.ui.layout), "pane_cameras": cameras,
                "scene_bounds": state.scene.bounds,
                "layer_filters": format!("{:?}", state.ui.filters),
                "console_latest": state.ui.console.records().next_back().map(|record| &record.message),
                "active_review_target_id": state.active_review_target_id,
                "menu": state.ui.active_menu, "submenu": state.ui.active_submenu,
                "menu_focus_index": state.ui.menu_focus_index,
                "board_camera": {"center_x_nm": camera.center_x_nm, "center_y_nm": camera.center_y_nm, "zoom": camera.zoom},
                "preferences_open": state.ui.global_preferences.open,
                "dock": format!("{:?}", state.ui.active_dock_tab),
                "window_size": [self.config.width, self.config.height], "scale": self.scale_factor
            }),
        );
    }

    pub(crate) fn trace_action_attempt(&self, key: &str, surface: EntrySurface, invoked: bool) {
        if enabled()
            && let Some(record) = attempt(self.workspace(), key, surface, invoked)
        {
            emit("dispatch", record);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_observes_actual_registry_and_context_without_mutating_workspace() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let before = state.clone();
        let entries = registry(&state);
        assert_eq!(entries.as_array().unwrap().len(), ACTION_CONSUMERS.len());
        for (entry, exported) in ACTION_CONSUMERS.iter().zip(entries.as_array().unwrap()) {
            assert_eq!(exported["dispatch_key"], entry.key);
            assert_eq!(exported["handler_ref"], handler(entry));
            assert_eq!(
                exported["contexts"][context(&state)]["enabled"],
                entry.admit(&state).is_ok()
            );
        }
        assert_eq!(state, before);
        state.ui.layout.focused = datum_gui_protocol::PaneId(1);
        state.schematic_scene = None;
        let refusal = attempt(&state, "view.fit", EntrySurface::Shortcut, false).unwrap();
        assert_eq!(refusal["enabled"], false);
        assert_eq!(refusal["invoked"], false);
        assert!(attempt(&state, "not-a-production-key", EntrySurface::Shortcut, true).is_none());
    }

    #[test]
    fn invocation_and_entry_origin_are_observed_not_inferred_from_availability() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        for (surface, suffix) in [
            (EntrySurface::MenuPointer, "menu-pointer"),
            (EntrySurface::MenuKeyboard, "menu-keyboard"),
            (EntrySurface::Shortcut, "shortcut"),
        ] {
            let event = attempt(&state, "view.fit", surface, false).unwrap();
            assert!(event["entry_surface"].as_str().unwrap().ends_with(suffix));
            assert_eq!(event["enabled"], true);
            assert_eq!(event["invoked"], false);
        }
        assert!(attempt(&state, "help.about", EntrySurface::Shortcut, false).is_none());
    }
}
