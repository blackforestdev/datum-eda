//! Inspector controls configure the shared consumer; they have no writer.
use super::*;
impl Runtime {
    pub(super) fn activate_selection_inspector_target(
        &mut self,
        target: &HitTarget,
    ) -> Option<bool> {
        Some(match target {
            HitTarget::SelectionInventoryToggle => self.toggle_selection_inventory(),
            HitTarget::SelectionFiltersToggle => self.toggle_selection_filters(),
            HitTarget::SelectionClassToggle(class) => self.toggle_selection_class(*class),
            HitTarget::SelectionInspectorScroll { .. } => false,
            _ => return None,
        })
    }
    pub(super) fn toggle_selection_inventory(&mut self) -> bool {
        self.session
            .workspace_mut()
            .ui
            .selection_inspector
            .toggle_inventory();
        self.invalidate_frame();
        true
    }
    pub(super) fn toggle_selection_filters(&mut self) -> bool {
        self.session
            .workspace_mut()
            .ui
            .selection_inspector
            .toggle_filters();
        self.invalidate_frame();
        true
    }
    pub(super) fn toggle_selection_class(
        &mut self,
        class: datum_gui_protocol::AuthoredSelectionClass,
    ) -> bool {
        let excluded = &mut self
            .session
            .workspace_mut()
            .ui
            .filters
            .excluded_selection_classes;
        if !excluded.remove(&class) {
            excluded.insert(class);
        }
        // Selection filters gate future acquisition only, never visibility or
        // current complete membership; no retained scene rebuilding is needed.
        self.invalidate_frame();
        true
    }
    pub(super) fn handle_selection_inspector_scroll(&mut self, lines: f32) -> Option<bool> {
        let (x, y) = self.last_cursor_pos?;
        let (total, visible) = self.presented_hits.regions().iter().find_map(|region| {
            if !region.rect.contains(x, y) {
                return None;
            }
            match region.target {
                HitTarget::SelectionInspectorScroll { total, visible } => Some((total, visible)),
                _ => None,
            }
        })?;
        let offset = &mut self
            .session
            .workspace_mut()
            .ui
            .selection_inspector
            .scroll_offset;
        let next = datum_gui_viewport::scroll::scrolled_row_offset(*offset, total, visible, lines);
        let changed = next != *offset;
        *offset = next;
        if changed {
            self.invalidate_frame();
        }
        Some(changed)
    }
}
