//! Typed wheel actions acquire their captured source target through shared selection.
use super::*;
use crate::runtime_view_actions::action_evidence::EntrySurface;

impl Runtime {
    pub(crate) fn activate_schematic_marking_item(&mut self, menu_key: &str, slot: &str) -> bool {
        if !menu_key.starts_with("schematic.") {
            return false;
        }
        let prepare = (|| {
            if self.workspace().schematic_authoring.delete_review.is_some() {
                return Err("Accept or dismiss the Proposal first".into());
            }
            if self.native_edit.pending()
                || self.check_work.pending()
                || self.native_edit.preview.is_some()
                || self.workspace().library.draft.is_some()
                || self.workspace().library.acquisition.is_some()
            {
                return Err("Finish or cancel the current edit first".into());
            }
            let marking = self
                .workspace()
                .ui
                .marking_menu
                .as_ref()
                .filter(|m| m.menu_key == menu_key)
                .cloned()
                .ok_or("Context menu changed; reopen it")?;
            let model = datum_gui_protocol::gui_menu_model::default_gui_menu_model()
                .map_err(|_| "Context menu unavailable")?;
            let menu = model
                .marking_menus
                .get(menu_key)
                .ok_or("Context menu unavailable")?;
            let item = if let Some(submenu) = marking.submenu_path.last() {
                slot.strip_prefix("SUB:")
                    .and_then(|n| n.parse::<usize>().ok())
                    .and_then(|n| menu.submenus.get(submenu)?.get(n))
            } else {
                menu.cardinal
                    .get(slot)
                    .or_else(|| menu.secondary.get(slot))
                    .or_else(|| {
                        slot.strip_prefix("MORE:")
                            .and_then(|n| n.parse::<usize>().ok())
                            .and_then(|n| menu.overflow.get(n))
                    })
            }
            .cloned()
            .ok_or("Context action unavailable")?;
            if matches!(
                item.binding(),
                datum_gui_protocol::GuiMenuBinding::Submenu(_)
            ) {
                return Ok((item, None));
            }
            if let Some(reason) =
                marking.schematic_action_unavailable_reason(&item, self.workspace())
            {
                return Err(reason);
            }
            let target = marking.schematic_context_target().cloned();
            // Revalidate the captured presentation key against current native authority;
            // publishing selection below invalidates PreparedScene, so never re-hit then.
            let pane = self.workspace().ui.layout.focused;
            let basis = self
                .native_binding_basis(SceneSurface::Schematic, Some(pane))
                .map_err(|e| e.to_string())?
                .ok_or("Native context authority unavailable")?;
            if let Some(target) = &target {
                let binding = marking
                    .target_object_id
                    .as_deref()
                    .and_then(|key| basis.bindings.get(key));
                if !binding.is_some_and(|b| {
                    b.acquired == *target && b.eligible(&self.workspace().ui.filters)
                }) {
                    return Err("Context target changed; reopen the menu".into());
                }
            }
            Ok::<_, String>((item, target))
        })();
        let (item, target) = match prepare {
            Ok(prepared) => prepared,
            Err(reason) => {
                self.log_console_refusal(ConsoleFeedbackSource::Menu, reason);
                self.invalidate_frame();
                return true;
            }
        };
        if matches!(
            item.binding(),
            datum_gui_protocol::GuiMenuBinding::Submenu(_)
        ) {
            return false;
        }
        let move_symbol = item.verb.as_deref() == Some("datum.schematic.move_symbol");
        let pickup = self
            .workspace()
            .ui
            .marking_menu
            .as_ref()
            .and_then(|menu| menu.selection.as_ref())
            .map(|menu| menu.pickup_world);
        let preserve=move_symbol && target.as_ref().is_some_and(|target|
            matches!(&self.workspace().selection,datum_gui_protocol::SelectionTarget::Native(view)
                if !view.stale && datum_gui_protocol::footprint_move_preview::native::group::subject_contains(&view.subject,target)));
        self.dismiss_marking_menu();
        if let Some(target) = target.filter(|_| !preserve) {
            self.cancel_native_selection_gesture();
            self.native_selection.as_mut().unwrap().reset_click_depth();
            self.publish_native_acquisition(
                BTreeSet::new(),
                AcquisitionMode::Replace,
                AcquisitionEntry::Individual,
                Some(datum_gui_protocol::SelectionSubject::Object(target.clone())),
            );
            if !datum_gui_protocol::schematic_authoring::selected::target(self.workspace())
                .is_ok_and(|(_, selected)| selected == target)
            {
                self.log_console_refusal(
                    ConsoleFeedbackSource::Menu,
                    "Context target acquisition failed; select the object again",
                );
                return true;
            }
        }
        if move_symbol {
            if let Some(pickup) = pickup {
                self.dispatch_session_command(SessionCommand::SetTool(WorkspaceTool::Move));
                self.begin_selected_native_move_at(Some(pickup));
            } else {
                self.log_console_refusal(
                    ConsoleFeedbackSource::Menu,
                    "Move pickup source changed; reopen the menu",
                );
            }
            return true;
        }
        if let Some(action) = item.gui_local.as_deref() {
            return self.activate_gui_local_menu_action(action, EntrySurface::MenuPointer);
        }
        self.activate_native_edit_menu(item.verb.as_deref())
            .unwrap_or(true)
    }
}
