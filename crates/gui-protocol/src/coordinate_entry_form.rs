//! Shared Move coordinate form state; input/rendering retain their owners.
use super::{AuthoringViewBasis, CoordinateEntry, PointNm};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordinateEntryControl {
    Absolute,
    Delta,
    X,
    Y,
    Apply,
    Cancel,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoordinateEntryForm {
    pub entry: CoordinateEntry,
    pub focus: CoordinateEntryControl,
    pub label: String,
    pub refusal: Option<String>,
    pub restore_position: PointNm,
    pub basis: AuthoringViewBasis,
    pub pending: bool,
    pub stale: bool,
}
impl CoordinateEntryForm {
    pub fn focus_next(&mut self, reverse: bool) {
        use CoordinateEntryControl::*;
        let order = [Absolute, Delta, X, Y, Apply, Cancel];
        let current = order.iter().position(|c| *c == self.focus).unwrap_or(0);
        self.focus = order[(current + if reverse { 5 } else { 1 }) % 6];
    }
}
