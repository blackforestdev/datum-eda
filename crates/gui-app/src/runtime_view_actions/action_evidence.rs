//! Opt-in observation of production state/dispatch for the bounded PM041 pilot.
//! No input injection, project writer, availability override or acceptance flag.

use crate::Runtime;
use datum_gui_protocol::gui_menu_model::action_registry::{self, ACTION_CONSUMERS};
use datum_gui_protocol::{ReviewWorkspaceState, camera_scene_for_pane};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex, OnceLock};
mod output;

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
    previous_state: Option<Arc<Value>>,
    output: Option<output::Output>,
}

fn stream() -> &'static Mutex<Stream> {
    static STREAM: OnceLock<Mutex<Stream>> = OnceLock::new();
    STREAM.get_or_init(|| Mutex::new(Stream::default()))
}

/// Drain only after the native event loop has stopped dispatching input.
pub(crate) fn finish() {
    let output = stream()
        .lock()
        .ok()
        .and_then(|mut stream| stream.output.take());
    if let Some(output) = output {
        output.finish();
    }
}

fn emit(event: &'static str, value: Value) {
    let Ok(mut stream) = stream().lock() else {
        return;
    };
    let value = Arc::new(value);
    if event == "state" {
        if stream.previous_state.as_deref() == Some(value.as_ref()) {
            return;
        }
        stream.previous_state = Some(value.clone());
    }
    stream.sequence += 1;
    let record = output::Record {
        event,
        sequence: stream.sequence,
        value,
    };
    if stream.output.is_none() {
        stream.output = output::Output::start_stderr().ok();
    }
    if let Some(output) = &stream.output {
        output.submit(record);
    }
}

fn capture_state_value(
    state: &ReviewWorkspaceState,
    camera: datum_gui_viewport::CameraState,
    cameras: Vec<Value>,
    presented: Option<Value>,
    window_size: [u32; 2],
    scale_factor: f32,
) -> Value {
    let mut value = json!({
        "context": context(state), "registry": registry(state),
        "focused_pane": state.ui.layout.focused.0,
        "focused_content": format!("{:?}", state.ui.layout.content_for(state.ui.layout.focused)),
        "focus": format!("{:?}", state.ui.focus),
        "selection_inspector": {
            "expanded": state.ui.selection_inspector.inventory_expanded,
            "filters_expanded": state.ui.selection_inspector.filters_expanded,
            "scroll_offset": state.ui.selection_inspector.scroll_offset,
            "presented": presented
        },
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
        "window_size": window_size, "scale": scale_factor
    });
    crate::append_gui_verbose_diagnostic_line(|| "action evidence selection format begin");
    let selection = format!("{:?}", state.selection);
    crate::append_gui_verbose_diagnostic_line(|| {
        format!(
            "action evidence selection format end bytes={}",
            selection.len()
        )
    });
    // json! serializes expressions by reference. Move this complete String
    // directly into the value so the large Debug payload is never cloned here.
    value["selection"] = Value::String(selection);
    crate::append_gui_verbose_diagnostic_line(|| "action evidence state value ready");
    value
}

impl Runtime {
    pub(crate) fn trace_action_state(&self) {
        if !enabled() {
            return;
        }
        crate::append_gui_verbose_diagnostic_line(|| "action evidence capture begin");
        let _probe = crate::gui_runtime_support::phase_probe::Probe::start("evidence_capture");
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
        let presented = self.presented_hits.regions().iter().find_map(|region| {
            if let datum_gui_render::HitTarget::SelectionInspectorScroll {total,visible,basis,visible_members,..} = &region.target {
                Some(json!({"total":total,"visible":visible,"basis_scope":format!("{:?}",basis.scope),"basis_expanded":basis.navigation.inventory_expanded,"basis_offset":basis.navigation.scroll_offset,"visible_members":visible_members}))
            } else { None }
        });
        let value = capture_state_value(
            state,
            camera,
            cameras,
            presented,
            [self.config.width, self.config.height],
            self.scale_factor,
        );
        emit("state", value);
        crate::append_gui_verbose_diagnostic_line(|| "action evidence emit returned");
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
    fn complete_compound_state_wire_matches_prior_capture_with_unicode_and_escapes() {
        use crate::terminal_accessibility_platform::selection_fixture as fixture;
        use datum_gui_protocol::{
            AuthoredSelectionClass, CompoundSelection, SelectionSubject, SelectionTarget,
            selection_resolution::NativeSelectionResolution, selection_view::build_selection_view,
        };
        use eda_engine::ir::geometry::Point;

        let mut f = fixture::Fixture::new("complete-state-wire");
        let tracks = [
            f.line(fixture::id(1), Point::new(0, 0), Point::new(10, 0)),
            f.line(fixture::id(2), Point::new(20, 0), Point::new(30, 0)),
        ];
        let files = f.source_bytes();
        let authority = NativeSelectionResolution::capture(&f.model).unwrap();
        let members: Vec<_> = authority
            .authored_identities()
            .iter()
            .filter(|id| id.class == AuthoredSelectionClass::Track)
            .cloned()
            .collect();
        assert_eq!(members.len(), 2);
        let subject = SelectionSubject::Compound(
            CompoundSelection::new(members.clone(), Some(members[0].clone())).unwrap(),
        );
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.scene.project_uuid = f.model.project.project_id.to_string();
        state.scene.board_uuid = f.board.to_string();
        state.scene.source_revision = f.model.model_revision.0.clone();
        let template = state.scene.tracks[0].clone();
        state.scene.tracks = tracks
            .iter()
            .map(|track| {
                let mut rendered = template.clone();
                rendered.object_id = track.uuid.to_string();
                rendered.source_object_uuid = track.uuid.to_string();
                rendered.track_uuid = track.uuid.to_string();
                rendered.net_uuid = Some(f.net.to_string());
                rendered
            })
            .collect();
        let mut view =
            build_selection_view(&f.model, &authority, subject, &state.scene, None, |_| true)
                .unwrap();
        assert_eq!(view.members.len(), 2);
        view.notice = Some("λ雪\n\t\"quoted\"\\path\u{0000}".into());
        state.selection = SelectionTarget::Native(Box::new(view));
        let before = state.clone();
        let camera = datum_gui_viewport::CameraState {
            center_x_nm: 12.5,
            center_y_nm: -4.0,
            zoom: 1.25,
        };
        let cameras = vec![json!({"pane": 0, "content": "Board", "camera": null})];
        let presented = Some(json!({"total": 2, "visible": 2, "visible_members": members}));
        // Prior complete state schema and json!(format!) encoding are the wire oracle.
        let prior = json!({
            "context": context(&state), "registry": registry(&state),
            "focused_pane": state.ui.layout.focused.0,
            "focused_content": format!("{:?}", state.ui.layout.content_for(state.ui.layout.focused)),
            "focus": format!("{:?}", state.ui.focus), "selection": format!("{:?}", state.selection),
            "selection_inspector": {"expanded": state.ui.selection_inspector.inventory_expanded,
                "filters_expanded": state.ui.selection_inspector.filters_expanded,
                "scroll_offset": state.ui.selection_inspector.scroll_offset, "presented": presented},
            "layout": format!("{:?}", state.ui.layout), "pane_cameras": cameras,
            "scene_bounds": state.scene.bounds, "layer_filters": format!("{:?}", state.ui.filters),
            "console_latest": state.ui.console.records().next_back().map(|record| &record.message),
            "active_review_target_id": state.active_review_target_id,
            "menu": state.ui.active_menu, "submenu": state.ui.active_submenu,
            "menu_focus_index": state.ui.menu_focus_index,
            "board_camera": {"center_x_nm": camera.center_x_nm, "center_y_nm": camera.center_y_nm, "zoom": camera.zoom},
            "preferences_open": state.ui.global_preferences.open, "dock": format!("{:?}", state.ui.active_dock_tab),
            "window_size": [1920, 1051], "scale": 1.25_f32
        });
        let actual = capture_state_value(&state, camera, cameras, presented, [1920, 1051], 1.25);
        assert_eq!(
            serde_json::to_vec(&actual).unwrap(),
            serde_json::to_vec(&prior).unwrap()
        );
        assert!(actual["selection"].as_str().unwrap().contains("Compound"));
        assert!(actual["selection"].as_str().unwrap().contains("λ雪"));
        assert_eq!(state, before);
        fixture::Fixture::assert_bytes(&files);
    }

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
