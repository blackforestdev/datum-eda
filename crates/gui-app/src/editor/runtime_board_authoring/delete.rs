//! Board configures the existing shared Delete entry and native edit queue.
use super::*;
use datum_gui_protocol::native_edit_client::board_delete;

impl Runtime {
    pub(crate) fn delete_selected_board_track(&mut self) -> bool {
        match board_delete::selected_track(self.workspace()) {
            Ok(track) => self.queue_native_edit(NativeEditAction::DeleteBoardTrack { track }),
            Err(reason) => self.board_authoring_error(reason),
        }
    }

    /// Every native Board result is consumed here, including refusal. Unsupported
    /// targets must never escape into the legacy foreign-shell handoff.
    pub(crate) fn native_board_pointer_delete(&mut self, target: Option<&str>) -> Option<bool> {
        if self.workspace().ui.layout.focused_content() != PaneContent::Board
            || self.workspace().backing.as_ref().is_some_and(|backing| {
                backing.request.board_file.is_some() || backing.request.kicad_board_source.is_some()
            })
        {
            return None;
        }
        let result = (|| -> Result<eda_engine::substrate::ObjectId, String> {
            let target = target.ok_or("Pick one current native PCB Track")?;
            if !self
                .ensure_selection_authority()
                .map_err(|error| format!("{error:#}"))?
            {
                return Err("Native source is refreshing; select the current Track again".into());
            }
            let authority = &self
                .native_selection
                .as_ref()
                .ok_or("Native selection source unavailable")?
                .authority;
            let bindings = datum_gui_protocol::selection_scene::SceneSelectionBindings::capture(
                &self.workspace().scene,
                authority,
            );
            let id = &bindings
                .get(target)
                .ok_or("Track source binding is unavailable")?
                .projected;
            if id.class != datum_gui_protocol::AuthoredSelectionClass::Track
                || !id.instance_path.is_empty()
            {
                return Err("This Delete action supports one current native PCB Track".into());
            }
            Ok(id.id)
        })();
        Some(match result {
            Ok(track) => self.queue_native_edit(NativeEditAction::DeleteBoardTrack { track }),
            Err(reason) => self.board_authoring_error(reason),
        })
    }
}
