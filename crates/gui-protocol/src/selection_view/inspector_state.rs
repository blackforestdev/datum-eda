//! Transient Inspector navigation; never selection membership or authored data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelectionInspectorState {
    pub inventory_expanded: bool,
    pub filters_expanded: bool,
    pub scroll_offset: usize,
}
impl SelectionInspectorState {
    pub fn toggle_inventory(&mut self) {
        self.inventory_expanded = !self.inventory_expanded;
    }
    pub fn toggle_filters(&mut self) {
        self.filters_expanded = !self.filters_expanded;
    }
    pub fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
    }
}
