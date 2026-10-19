//! Compatibility projection entry; ordinary Open supplies the shared navigator.
use super::*;
impl Renderer {
    pub fn prepare_native_open_project(
        &mut self,
        dialog: &datum_gui_protocol::ProjectOpenDialogState,
        width: u32,
        height: u32,
        scale: f32,
    ) -> anyhow::Result<PreparedScene> {
        self.prepare_native_project_form(
            FormData::Open(dialog),
            None,
            crate::DialogView {
                width,
                height,
                scale,
            },
            &mut ScrollViewport::default(),
            &mut ScrollViewport::default(),
            &mut ScrollViewport::default(),
            false,
        )
    }
}
