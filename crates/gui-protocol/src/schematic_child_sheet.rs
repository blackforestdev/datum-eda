//! Blank child placement configures the shared name, pose and Proposal state.
use super::delete_review::DeleteReview;
use crate::{BoardReviewSceneV1, PointNm, ReviewWorkspaceState};
use eda_engine::{
    api::native_write::{
        schematic_child_sheet::{self as engine, ChildSheetRequest, ChildSheetSummary},
        schematic_delete_proposal::NativeProposalAction,
    },
    ir::geometry::Point,
};
use uuid::Uuid;
#[derive(Debug, Clone, PartialEq)]
pub struct ChildSheetDraft {
    pub review: Box<DeleteReview>,
    pub request: ChildSheetRequest,
    pub choosing_pose: bool,
    pub source: Option<eda_engine::schematic::Schematic>,
    pub scene: Option<BoardReviewSceneV1>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ChildSheetEvidence {
    pub summary: ChildSheetSummary,
    pub preview: BoardReviewSceneV1,
}
impl ChildSheetDraft {
    pub fn capture(state: &ReviewWorkspaceState) -> Result<Self, String> {
        let basis =
            crate::AuthoringViewBasis::capture_qualified_schematic(state, state.ui.layout.focused)?;
        let review = DeleteReview::capture_source_basis(state, basis)?;
        let key = review
            .basis
            .document
            .as_ref()
            .ok_or("Choose a qualified parent sheet")?;
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Native source unavailable")?;
        if model.model_revision.0 != review.revision
            || model.project.project_id != review.basis.project
        {
            return Err("Parent source changed".into());
        }
        engine::parent_occurrence(model, key.source_sheet, &key.instance_path)
            .map_err(|e| e.to_string())?;
        let request = ChildSheetRequest {
            proposal_id: review.proposal_id,
            parent: key.source_sheet,
            instance_path: key.instance_path.clone(),
            sheet: Uuid::new_v4(),
            definition: Uuid::new_v4(),
            instance: Uuid::new_v4(),
            name: String::new(),
            position: Point::new(0, 0),
        };
        Ok(Self {
            review: Box::new(review),
            request,
            choosing_pose: false,
            source: None,
            scene: None,
        })
    }
    pub fn name(&mut self, state: &ReviewWorkspaceState, name: String) -> Result<(), String> {
        if !self.review.current(state) {
            return Err("Parent changed; cancel and try again".into());
        }
        self.request.name = name;
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Native source unavailable")?;
        self.source = Some(
            engine::preview_source(model, &self.request)
                .map_err(|e| e.to_string())?
                .1,
        );
        self.choosing_pose = true;
        Ok(())
    }
    pub fn scene_at(
        &self,
        authoring: &super::SchematicAuthoring,
        base: &BoardReviewSceneV1,
        root: &std::path::Path,
        at: PointNm,
    ) -> anyhow::Result<BoardReviewSceneV1> {
        let mut source = self
            .source
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Child preview unavailable"))?;
        source
            .sheet_instances
            .get_mut(&self.request.instance)
            .ok_or_else(|| anyhow::anyhow!("Child preview identity unavailable"))?
            .position = Point::new(at.x, at.y);
        authoring.project_source(&source, self.request.parent, base, root)
    }
    pub fn preview(
        &self,
        state: &ReviewWorkspaceState,
        at: PointNm,
    ) -> Result<(ChildSheetSummary, BoardReviewSceneV1), String> {
        if !self.review.current(state) {
            return Err("Parent view changed; cancel and place the child again".into());
        }
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Native source unavailable")?;
        if model.model_revision.0 != self.review.revision
            || model.project.project_id != self.review.basis.project
        {
            return Err("Parent source changed".into());
        }
        let mut request = self.request.clone();
        request.position = Point::new(at.x, at.y);
        let (summary, source) =
            engine::preview_source(model, &request).map_err(|e| e.to_string())?;
        let root = &state
            .backing
            .as_ref()
            .ok_or("Native project unavailable")?
            .request
            .project_root;
        let scene = state
            .schematic_authoring
            .project_source(&source, request.parent, &self.review.scene, root)
            .map_err(|e| e.to_string())?;
        Ok((summary, scene))
    }
    pub fn stage(
        mut self,
        state: &ReviewWorkspaceState,
        at: PointNm,
    ) -> Result<DeleteReview, String> {
        let (summary, preview) = self.preview(state, at)?;
        self.request.position = Point::new(at.x, at.y);
        self.review.child_sheet = Some(ChildSheetEvidence { summary, preview });
        self.review.pending = Some(NativeProposalAction::CreateChildSheet(self.request));
        Ok(*self.review)
    }
}

#[cfg(test)]
#[path = "schematic_child_sheet_tests.rs"]
mod tests;
