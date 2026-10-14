//! Fixed-size native semantic snapshot shared by input observation modes.
use crate::Runtime;
use serde_json::{Value, json};
pub(super) const HOVER_BYTES: usize = 128;

#[derive(Clone, Copy)]
pub(super) struct State {
    pub(super) cursor: Option<[f32; 2]>,
    pub(super) native_cursor: Option<(f32, f32)>,
    pub(super) center: [f32; 2],
    pub(super) zoom: f32,
    pub(super) pan: bool,
    pub(super) epoch: u64,
    pub(super) render_revision: u64,
    pub(super) render_activity: [u64; 2],
    pub(super) hover: [u8; HOVER_BYTES],
    pub(super) hover_len: usize,
    pub(super) hover_surface: Option<datum_gui_protocol::PaneContent>,
    pub(super) truncated: bool,
    pub(super) focused: bool,
    pub(super) extent: [u32; 2],
    pub(super) scale: f32,
}
impl State {
    pub(super) fn capture(runtime: &Runtime) -> Self {
        let ui = &runtime.workspace().ui;
        let mut value = Self {
            cursor: ui.cursor_pos.map(|p| [p.x, p.y]),
            native_cursor: runtime.last_cursor_pos,
            center: [runtime.camera.center_x_nm, runtime.camera.center_y_nm],
            zoom: runtime.camera.zoom,
            pan: runtime.pan_gesture.is_active(),
            epoch: runtime.measurements.epoch(),
            render_revision: runtime.renderer.render_session().content_revision(),
            render_activity: runtime.renderer.render_session().measurement_activity(),
            hover: [0; HOVER_BYTES],
            hover_len: 0,
            hover_surface: None,
            truncated: false,
            focused: runtime.window_focused,
            extent: [runtime.config.width, runtime.config.height],
            scale: runtime.scale_factor,
        };
        if let Some(hover) = &ui.hovered_object {
            value.hover_len = hover.object_id.len().min(HOVER_BYTES);
            value.hover[..value.hover_len]
                .copy_from_slice(&hover.object_id.as_bytes()[..value.hover_len]);
            value.hover_surface = Some(hover.surface);
            value.truncated = hover.object_id.len() > HOVER_BYTES;
        }
        value
    }
    pub(super) fn value(&self) -> Value {
        json!({"cursor":self.cursor,"native_cursor":self.native_cursor,
            "camera_center_nm":self.center,"camera_zoom":self.zoom,"pan_active":self.pan,
            "device_epoch":self.epoch,"render_revision":self.render_revision,"hover_utf8":std::str::from_utf8(&self.hover[..self.hover_len]).ok(),
            "hover_surface":self.hover_surface.map(|s|format!("{s:?}")),"truncated":self.truncated,"window_focused":self.focused,"extent":self.extent,"scale":self.scale,"render_activity":self.render_activity})
    }
}

impl State {
    pub(super) fn semantic_eq(&self, other: &Self) -> bool {
        self.cursor == other.cursor
            && self.native_cursor == other.native_cursor
            && self.center == other.center
            && self.zoom == other.zoom
            && self.pan == other.pan
            && self.epoch == other.epoch
            && self.hover == other.hover
            && self.hover_len == other.hover_len
            && self.hover_surface == other.hover_surface
            && self.truncated == other.truncated
            && self.focused == other.focused
            && self.extent == other.extent
            && self.scale == other.scale
    }
}
