//! Fixed alignment presets configure the existing native Symbol preview.
use super::*;
use datum_gui_protocol::{
    AuthoringViewBasis, SelectionTarget,
    footprint_move_preview::native::group::{SymbolGroupPreview, symbol_members},
    schematic_alignment::MenuActionPreset,
};

impl Runtime {
    pub(crate) fn begin_schematic_alignment(&mut self, preset: MenuActionPreset) -> bool {
        let prepared = (|| -> anyhow::Result<_> {
            self.selected_schematic_idle().map_err(anyhow::Error::msg)?;
            let SelectionTarget::Native(view) = &self.workspace().selection else {
                anyhow::bail!("Select a complete native Symbol group before aligning");
            };
            anyhow::ensure!(
                !view.stale,
                "Selection changed; select the complete group again"
            );
            let members = symbol_members(&view.subject)?;
            let view_basis = AuthoringViewBasis::capture_qualified_schematic(
                self.workspace(),
                self.workspace().ui.layout.focused,
            )
            .map_err(anyhow::Error::msg)?;
            let base = datum_gui_protocol::camera_scene_for_pane(self.workspace(), view_basis.pane)
                .context("Schematic source unavailable")?;
            let native = self
                .native_selection
                .as_ref()
                .context("Native alignment authority unavailable")?;
            // The shared Move readout needs the authored Project units profile.
            // Refuse an unavailable presentation basis before arming its preview.
            eda_engine::api::native_write::project::project_display_units(&native.model)?;
            anyhow::ensure!(
                native.model.model_revision.0 == base.source_revision
                    && native.model.project.project_id == view.project
                    && view.revision.0 == base.source_revision,
                "Alignment source changed; select the complete group again"
            );
            let basis =
                eda_engine::api::native_write::schematic_symbol_move::SymbolMoveSetBasis::capture(
                    native.authority.snapshot(),
                    &members.iter().map(|id| id.id).collect::<Vec<_>>(),
                    &members[0].instance_path,
                )?;
            let root = self
                .workspace()
                .backing
                .as_ref()
                .context("Open a persistent Project before aligning")?
                .request
                .project_root
                .clone();
            let reference = datum_gui_protocol::PointNm {
                x: basis.symbols[0].position.x,
                y: basis.symbols[0].position.y,
            };
            let preview = SymbolGroupPreview::begin(
                base,
                &self.workspace().schematic_authoring,
                &view.subject,
                basis,
                reference,
                datum_gui_protocol::AuthoringSnapState {
                    enabled: false,
                    ..self.workspace().authoring.snap
                },
                root,
            )?
            .with_alignment(&native.model, preset, &view.subject)?;
            Ok((NativeMovePreview::Symbols(preview), view_basis))
        })();
        let (preview, basis) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                self.log_console_refusal(ConsoleFeedbackSource::Menu, error.to_string());
                return true;
            }
        };
        let reference = preview.reference_position();
        let previous_tool = self.workspace().tool;
        self.dispatch_session_command(SessionCommand::SetTool(WorkspaceTool::Move));
        self.native_edit.previous_context_tool = Some(previous_tool);
        self.dispatch_session_command(SessionCommand::BeginAuthoringGesture {
            world: reference,
            target_object_id: Some("schematic-symbol-alignment".into()),
        });
        self.session.workspace_mut().authoring.gesture.preset = Some(preset);
        self.session
            .workspace_mut()
            .schematic_authoring
            .circuit_basis = Some(basis.clone());
        self.session.workspace_mut().authoring.gesture.view_basis = Some(basis);
        self.native_edit.preview_tip = self
            .workspace()
            .supervision
            .journal
            .accepted_transaction_tip
            .clone();
        self.native_edit.exact_position = None;
        self.native_edit.exact_cursor = None;
        self.native_edit.current_position = Some(reference);
        self.native_edit.pending_position = Some(reference);
        self.native_edit.preview = Some(preview);
        self.log_console_echo(ConsoleFeedbackSource::Menu,
            "Align Symbols preview: click to accept, Escape to cancel; rotate and coordinate editing unavailable");
        self.invalidate_frame();
        true
    }
}
