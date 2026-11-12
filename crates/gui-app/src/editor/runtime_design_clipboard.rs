//! Shared design clipboard commands configure the existing schematic placement.
use super::*;
use datum_gui_protocol::schematic_authoring::DesignClipboardPlacement;
use datum_gui_protocol::{AuthoredSelectionClass as Class, SelectionTarget};
use eda_engine::api::native_write::design_clipboard::{
    ClipboardClass, ClipboardTarget, DesignClipboardBundle,
};
use eda_engine::substrate::ObjectId as Uuid;

impl Runtime {
    fn capture_design_clipboard(&mut self) -> Result<DesignClipboardBundle> {
        anyhow::ensure!(
            !self.native_edit.pending()
                && self.native_edit.preview.is_none()
                && !self.check_work.pending(),
            "Finish the current edit first"
        );
        anyhow::ensure!(
            !self.workspace().schematic_authoring.active()
                && !self.workspace().schematic_authoring.picker
                && self.workspace().schematic_authoring.delete_review.is_none(),
            "Finish or cancel the current schematic gesture first"
        );
        let pane = self.workspace().ui.layout.focused;
        let basis = datum_gui_protocol::AuthoringViewBasis::capture_qualified_schematic(
            self.workspace(),
            pane,
        )
        .map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            self.ensure_selection_authority()?,
            "Open a native Project first"
        );
        let SelectionTarget::Native(view) = &self.workspace().selection else {
            anyhow::bail!("Select wires or junctions to copy")
        };
        let native = self
            .native_selection
            .as_ref()
            .context("Selection source unavailable")?;
        anyhow::ensure!(
            !view.stale
                && view.project.to_string() == native.model.project.project_id.to_string()
                && view.revision.0 == native.model.model_revision.0,
            "Select current source wiring to copy"
        );
        let path = &basis
            .document
            .as_ref()
            .context("Choose a schematic source occurrence")?
            .instance_path;
        let mut targets = Vec::new();
        for member in &view.members {
            anyhow::ensure!(
                &member.instance_path == path,
                "Copy requires one schematic occurrence"
            );
            let class = match member.class {
                Class::Symbol => ClipboardClass::Symbol,
                Class::Wire => ClipboardClass::Wire,
                Class::Junction => ClipboardClass::Junction,
                _ => anyhow::bail!("Select native Symbols, wires or junctions to copy"),
            };
            targets.push(ClipboardTarget {
                class,
                id: member.id,
            });
        }
        let source = self
            .workspace()
            .schematic_authoring
            .source
            .as_ref()
            .context("Schematic source unavailable")?;
        let sheet = source
            .sheets
            .get(&basis.source)
            .context("Schematic source unavailable")?;
        let reference = targets
            .iter()
            .filter_map(|t| sheet.symbols.get(&t.id))
            .min_by_key(|s| (s.position.y, s.position.x, s.uuid))
            .map(|s| s.position)
            .or_else(|| {
                targets
                    .iter()
                    .find_map(|t| sheet.wires.get(&t.id).map(|w| w.from))
            })
            .or_else(|| {
                targets
                    .iter()
                    .find_map(|t| sheet.junctions.get(&t.id).map(|j| j.position))
            })
            .context("Select Symbols, wires or junctions to copy")?;
        Ok(DesignClipboardBundle::capture(
            &native.model,
            basis.source,
            path,
            &targets,
            reference,
        )?)
    }
    pub(crate) fn copy_design_selection(&mut self, cut: bool) -> bool {
        if self.workspace().ui.layout.focused_content() != PaneContent::Schematic {
            return false;
        }
        let result = (|| -> Result<()> {
            let bundle = self.capture_design_clipboard()?;
            anyhow::ensure!(
                !cut || bundle.symbols.is_none(),
                "Symbol Cut is not supported; use Copy or Duplicate"
            );
            self.write_clipboard_text(&serde_json::to_string(&bundle)?)?;
            if cut {
                self.delete_selected_schematic();
            } else {
                self.log_console_echo(
                    ConsoleFeedbackSource::Tool,
                    "Copied selected design objects",
                );
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
        }
        true
    }
    pub(crate) fn paste_design_clipboard(&mut self) -> bool {
        if self.workspace().ui.layout.focused_content() != PaneContent::Schematic {
            return false;
        }
        let result = (|| -> Result<()> {
            let text = self.read_clipboard_text()?;
            let bundle: DesignClipboardBundle = serde_json::from_str(&text)
                .context("Clipboard does not contain a Datum design bundle")?;
            self.begin_design_placement(bundle)
        })();
        if let Err(error) = result {
            self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
        }
        true
    }
    pub(crate) fn duplicate_design_selection(&mut self) -> bool {
        if self.workspace().ui.layout.focused_content() != PaneContent::Schematic {
            return false;
        }
        let result = self
            .capture_design_clipboard()
            .and_then(|bundle| self.begin_design_placement(bundle));
        if let Err(error) = result {
            self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
        }
        true
    }
    fn begin_design_placement(&mut self, mut bundle: DesignClipboardBundle) -> Result<()> {
        anyhow::ensure!(
            !self.native_edit.pending()
                && self.native_edit.preview.is_none()
                && !self.check_work.pending()
                && !self.workspace().schematic_authoring.active()
                && !self.workspace().schematic_authoring.picker,
            "Finish or cancel the current gesture first"
        );
        if bundle.symbols.is_some() {
            anyhow::ensure!(
                self.ensure_selection_authority()?,
                "Open a native Project first"
            );
            bundle.prepare_placement(
                &self
                    .native_selection
                    .as_ref()
                    .context("Clipboard source unavailable")?
                    .model,
            )?;
        }
        let placement = DesignClipboardPlacement::capture(bundle, Uuid::new_v4())
            .map_err(anyhow::Error::msg)?;
        let cursor = self.last_cursor_pos.and_then(|screen| {
            if self.pane_at_screen(screen.0, screen.1) != Some(self.workspace().ui.layout.focused) {
                return None;
            }
            self.prepared_scene()?
                .world_point_at_screen(screen.0, screen.1)
                .filter(|(_, surface)| *surface == SceneSurface::Schematic)
                .map(|(world, _)| (world, screen))
        });
        crate::append_gui_verbose_diagnostic_line(|| format!("clipboard begin cursor={cursor:?}"));
        anyhow::ensure!(
            self.prepare_native_circuit(false),
            "Choose a writable schematic source occurrence"
        );
        let editor = &mut self.session.workspace_mut().schematic_authoring;
        editor.chosen = None;
        editor.picker = false;
        editor.design_placement = Some(placement);
        editor.rotation_deg = 0;
        editor.mirrored = false;
        editor.position = None;
        editor.preview_dirty = true;
        if let Some((world, screen)) = cursor {
            let point = self.schematic_command_point(world, screen);
            self.session.workspace_mut().schematic_authoring.position = point;
        }
        self.log_console_echo(
            ConsoleFeedbackSource::Tool,
            "Place copied objects: click to place; R rotates, X/Y mirror; Escape cancels",
        );
        self.invalidate_frame();
        Ok(())
    }
    pub(crate) fn refuse_design_placement_pose(&mut self, reason: String) {
        let editor = &mut self.session.workspace_mut().schematic_authoring;
        editor.refusal = Some(reason);
        editor.position = None;
        editor.circuit_preview = None;
        editor.preview_dirty = false;
        self.invalidate_scene();
    }
    pub(crate) fn place_design_clipboard(&mut self, at: PointNm) -> bool {
        let request = (|| -> Result<_> {
            let authoring = &self.workspace().schematic_authoring;
            let draft = authoring
                .design_placement
                .as_ref()
                .context("Design placement ended")?;
            let base = authoring
                .base
                .as_ref()
                .context("Design placement source unavailable")?;
            let root = &self
                .workspace()
                .backing
                .as_ref()
                .context("Open a native Project")?
                .request
                .project_root;
            let request = draft.request(authoring, at).map_err(anyhow::Error::msg)?;
            let scene = draft.scene_at(authoring, base, root, at)?;
            Ok((request, scene))
        })();
        match request {
            Ok((request, scene)) => {
                self.session.workspace_mut().schematic_authoring.position = Some(at);
                self.publish_schematic_preview_scene(scene);
                self.session
                    .workspace_mut()
                    .schematic_authoring
                    .pending_request = Some(request);
                self.queue_native_edit(NativeEditAction::Schematic)
            }
            Err(error) => {
                self.refuse_design_placement_pose(error.to_string());
                self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
                true
            }
        }
    }
}
