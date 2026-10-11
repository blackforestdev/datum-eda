//! Read-only ownership and canonical authored anchors for presentation LOD.
use super::*;
impl SelectionView {
    pub fn presentation_owner(
        &self,
        scene: &crate::BoardReviewSceneV1,
        surface: crate::PaneContent,
        key: &str,
    ) -> Option<&Id> {
        self.projection_keys(scene, surface)?
            .contains(key)
            .then_some(())?;
        let owner = match surface {
            crate::PaneContent::Board => self.board_presentations.get(key),
            crate::PaneContent::Schematic => self.schematic_presentations.get(key),
            crate::PaneContent::Library | crate::PaneContent::Revision(_) => None,
        }?;
        self.members.contains(owner).then_some(owner)
    }
    /// No bounds-center, text-layout or scene-name fallback can invent an anchor.
    pub fn authored_anchor(&self, owner: &Id) -> Option<crate::PointNm> {
        if !self.members.contains(owner) {
            return None;
        }
        if let Some(anchor) = self.authored_anchors.get(owner) {
            return Some(*anchor);
        }
        let values = self.values.get(owner)?;
        let field = match owner.class {
            Class::Footprint | Class::Symbol => "placement_position_nm",
            Class::Track => "section_start_nm",
            Class::Wire | Class::BusSection => "section_start_nm",
            Class::Pin => "terminal_position_nm",
            _ => "position_nm",
        };
        let value = values.get(field)?;
        Some(crate::PointNm {
            x: value.get("x")?.as_i64()?,
            y: value.get("y")?.as_i64()?,
        })
    }
}
