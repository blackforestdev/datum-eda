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
    pub(super) fn navigate_selection_inspector(
        &mut self,
        page: datum_gui_protocol::selection_view::inspector_state::InspectorPage,
    ) -> bool {
        use datum_gui_protocol::selection_view::inspector_state::InspectorPresentationBasis;
        let navigation = &self.workspace().ui.selection_inspector;
        if !navigation.inventory_expanded {
            self.selection_notice("Expand complete inventory first".into());
            return true;
        }
        let datum_gui_protocol::SelectionTarget::Native(view) = &self.workspace().selection else {
            return false;
        };
        let current_basis = InspectorPresentationBasis::capture(view, navigation);
        let right = self.current_layout().right_sidebar;
        let sidebar = [
            right.x.to_bits(),
            right.y.to_bits(),
            right.width.to_bits(),
            right.height.to_bits(),
        ];
        // Menu-only composition damage preserves the actual presented Inspector.
        // Expansion, scope, notices, filters and geometry must match its own basis.
        let Some((total, visible)) =
            self.presented_hits
                .regions()
                .iter()
                .find_map(|region| match &region.target {
                    HitTarget::SelectionInspectorScroll {
                        total,
                        visible,
                        basis,
                        sidebar: published,
                        ..
                    } if *visible > 0 && basis == &current_basis && published == &sidebar => {
                        Some((*total, *visible))
                    }
                    _ => None,
                })
        else {
            self.selection_notice("Inspector presentation changed; reopen the menu".into());
            return true;
        };
        let offset = &mut self
            .session
            .workspace_mut()
            .ui
            .selection_inspector
            .scroll_offset;
        let next = inspector_page_offset(*offset, total, visible, page);
        *offset = next;
        self.invalidate_frame();
        true
    }
    pub(super) fn handle_selection_inspector_scroll(&mut self, lines: f32) -> Option<bool> {
        let (x, y) = self.last_cursor_pos?;
        let (total, visible) = self.presented_hits.regions().iter().find_map(|region| {
            if !region.rect.contains(x, y) {
                return None;
            }
            match &region.target {
                HitTarget::SelectionInspectorScroll { total, visible, .. } => {
                    Some((*total, *visible))
                }
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

fn inspector_page_offset(
    offset: usize,
    total: usize,
    visible: usize,
    page: datum_gui_protocol::selection_view::inspector_state::InspectorPage,
) -> usize {
    use datum_gui_protocol::selection_view::inspector_state::InspectorPage;
    match page {
        InspectorPage::First => 0,
        InspectorPage::Last if visible > 0 => total.saturating_sub(visible),
        InspectorPage::Last => 0,
        InspectorPage::Previous | InspectorPage::Next => {
            datum_gui_viewport::scroll::scrolled_row_offset(
                offset,
                total,
                visible,
                if page == InspectorPage::Previous {
                    visible as f32
                } else {
                    -(visible as f32)
                },
            )
        }
    }
}
#[cfg(test)]
mod tests {
    use super::inspector_page_offset;
    use datum_gui_protocol::selection_view::inspector_state::InspectorPage::*;
    #[test]
    fn inspector_pages_cover_complete_inventory_and_clamp_after_reflow() {
        for total in [256, 257] {
            let mut offset = 0;
            let mut seen = std::collections::BTreeSet::new();
            loop {
                seen.extend(offset..(offset + 11).min(total));
                let next = inspector_page_offset(offset, total, 11, Next);
                if next == offset {
                    break;
                }
                offset = next;
            }
            assert_eq!(seen, (0..total).collect());
            assert_eq!(offset, total - 11);
            assert_eq!(inspector_page_offset(0, total, 11, Last), offset);
            assert_eq!(
                inspector_page_offset(offset, total, 11, Previous),
                total - 22
            );
            assert_eq!(inspector_page_offset(offset, total, 11, First), 0);
            assert_eq!(inspector_page_offset(offset, 3, 11, Next), 0);
            assert_eq!(inspector_page_offset(offset, total, 0, Last), 0);
        }
    }
}
