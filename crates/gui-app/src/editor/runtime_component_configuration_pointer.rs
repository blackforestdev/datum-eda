//! Component overlay hit and primary-gesture ownership, shared by all entrypoints.
use super::*;
impl Runtime {
    fn component_pointer_owner(
        &self,
    ) -> Option<(
        eda_engine::substrate::ObjectId,
        Option<datum_gui_protocol::PaneId>,
    )> {
        let library = &self.workspace().library;
        if let Some(c) = library
            .configuration
            .as_ref()
            .filter(|c| c.target.is_some() || c.footprint_chooser.is_some())
        {
            return Some((
                c.session,
                c.target
                    .as_ref()
                    .map(|t| t.basis.pane)
                    .or_else(|| c.picker.as_ref().and_then(|p| p.placement_pane)),
            ));
        }
        let a = library
            .acquisition
            .as_ref()
            .filter(|a| a.component_picker)?;
        Some((a.session, a.placement_pane))
    }

    pub(crate) fn component_configuration_hit(&self) -> Option<HitTarget> {
        self.component_pointer_owner()?;
        if self.workspace().ui.active_menu.is_some()
            || self.marking_menu_active()
            || self.terminal_clipboard_menu_active()
        {
            return None;
        }
        let (x, y) = self.last_cursor_pos?;
        let regions = self.presented_hits.regions();
        let (surface, _) = regions.iter().enumerate().rev().find(|(_, region)| {
            region.target == HitTarget::SchematicPicker && region.rect.contains(x, y)
        })?;
        Some(
            regions[surface + 1..]
                .iter()
                .rev()
                .find(|region| {
                    matches!(region.target, HitTarget::Library(_)) && region.rect.contains(x, y)
                })
                .map_or(HitTarget::SchematicPicker, |region| region.target.clone()),
        )
    }
    pub(crate) fn begin_component_configuration_press(&mut self) -> bool {
        if self.pan_gesture.is_active()
            || self.dock_drag_active
            || self.divider_drag.is_some()
            || self.terminal_tab_drag.is_some()
            || self.terminal_split_drag.is_some()
            || self.terminal_text_selection_drag.is_some()
            || self.terminal_mouse_button.is_some()
            || self.selection_gesture.is_some()
            || self.selection_move.is_some()
        {
            return false;
        }
        let Some(hit) = self.component_configuration_hit() else {
            return false;
        };
        let Some((session, pane)) = self.component_pointer_owner() else {
            return false;
        };
        self.trace_click(format!(
            "component pointer press owner={session} target={hit:?}"
        ));
        self.component_pointer_press = Some((session, hit));
        if let Some(pane) = pane.filter(|pane| *pane == self.workspace().ui.layout.focused) {
            self.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(pane));
        }
        true
    }
    pub(crate) fn finish_component_configuration_press(&mut self) -> bool {
        let Some((session, hit)) = self.component_pointer_press.take() else {
            return false;
        };
        self.trace_click(format!(
            "component pointer release owner={session} pressed={hit:?} current={:?}",
            self.component_configuration_hit()
        ));
        if self
            .component_pointer_owner()
            .is_some_and(|(owner, _)| owner == session)
            && self.component_configuration_hit().as_ref() == Some(&hit)
        {
            self.select_hit_target(&hit);
        }
        true
    }
}
