//! One complete member compatibility/state rule for inventory consumers.
use super::*;

impl SelectionView {
    fn lacks_inventory_field(&self, id: &Id, field: &str) -> bool {
        !self
            .values
            .get(id)
            .is_some_and(|values| values.contains_key(field))
    }
    /// Exact per-member blocker detail for complete inventory/accessibility.
    pub fn unavailable_fields(&self, id: &Id) -> Vec<&'static str> {
        properties::fields(id.class)
            .iter()
            .filter_map(|(field, _)| self.lacks_inventory_field(id, field).then_some(*field))
            .collect()
    }
    /// Allocation-free counterpart for clipped inventory row structure.
    pub fn has_unavailable_fields(&self, id: &Id) -> bool {
        properties::fields(id.class)
            .iter()
            .any(|(field, _)| self.lacks_inventory_field(id, field))
    }
    pub fn has_inventory_states(&self, id: &Id) -> bool {
        self.hidden.contains(id) || self.locked.contains(id) || self.unprojected.contains(id)
    }
}
