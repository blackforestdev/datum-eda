//! Board-wide Outline intent over the existing workspace gesture and native queue.
use super::board_routing::native_root;
use crate::native_edit_client::{NativeEditAction, NativeEditIntent};
use crate::{AuthoringViewBasis, PaneContent, PointNm, ReviewWorkspaceState, WorkspaceTool};
use anyhow::Context;
use eda_engine::api::native_write::{
    WriteProvenance,
    board_layout::outline::{OutlineRequest, prepare_outline},
    board_routing::zone::{BoardBasis, capture_board_basis},
};
use eda_engine::{
    board::nominal_geometry::GeometryError,
    ir::geometry::{Point, Polygon},
    substrate::CommitSource,
};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardOutlineDraft {
    root: PathBuf,
    view: AuthoringViewBasis,
    basis: BoardBasis,
    vertices: Vec<Point>,
    geometry_error: Option<String>,
    pending: bool,
}
impl BoardOutlineDraft {
    pub fn vertices(&self) -> &[Point] {
        &self.vertices
    }
    pub fn pending(&self) -> bool {
        self.pending
    }
    pub fn release_pending(&mut self) {
        self.pending = false;
    }
    pub fn is_current(&self, state: &ReviewWorkspaceState) -> bool {
        native_root(state).is_ok_and(|root| root == self.root)
            && self.view.is_current(state, state.ui.layout.focused)
            && state.library.basis.as_ref().is_some_and(|model| {
                capture_board_basis(model).is_ok_and(|basis| basis == self.basis)
            })
    }
    pub fn polygon(&self) -> Polygon {
        Polygon::new(self.vertices.clone())
    }
    /// Cached placed-vertex admission; paint never clones or validates geometry.
    pub fn geometry_error(&self) -> Option<&str> {
        self.geometry_error.as_deref()
    }
    fn refresh_geometry_validation(&mut self) {
        self.geometry_error = eda_engine::board::occupied_copper::validate_polygon(&self.polygon())
            .err()
            .map(outline_geometry_error);
    }
    fn editable(&self) -> Result<(), String> {
        if self.pending {
            Err("Wait for the pending Outline edit".into())
        } else {
            Ok(())
        }
    }
}
fn outline_geometry_error(error: GeometryError) -> String {
    match error {
        GeometryError::InvalidPolygon => {
            "Outline must be a closed, simple, non-zero-area polygon; place at least three vertices"
                .into()
        }
        other => format!("Outline validation unavailable: {other:?}"),
    }
}
impl ReviewWorkspaceState {
    pub fn begin_board_outline(&mut self) -> Result<(), String> {
        if !crate::check_runs::producer::CheckProducerRequest::editor_preview_idle(self) {
            return Err("Finish or cancel the active edit before defining the Outline".into());
        }
        let root = native_root(self)?;
        let view = AuthoringViewBasis::capture(self, self.ui.layout.focused)?;
        if view.content != PaneContent::Board {
            return Err("Focus the native Board".into());
        }
        if !self
            .backing
            .as_ref()
            .is_some_and(|b| b.native_write_verbs.contains("datum.pcb.set_outline"))
        {
            return Err("Native Outline authoring unavailable".into());
        }
        let model = self
            .library
            .basis
            .as_ref()
            .ok_or("Native source unavailable")?;
        let basis = capture_board_basis(model).map_err(|e| e.to_string())?;
        if basis.board != view.source {
            return Err("Board source view is stale".into());
        }
        let draft = BoardOutlineDraft {
            root,
            view,
            basis,
            vertices: Vec::new(),
            geometry_error: Some(outline_geometry_error(GeometryError::InvalidPolygon)),
            pending: false,
        };
        self.set_tool(WorkspaceTool::DrawBoardOutline);
        self.authoring.gesture.view_basis = Some(draft.view.clone());
        self.authoring.board_outline = Some(draft);
        Ok(())
    }
    pub fn append_board_outline_vertex(&mut self, point: PointNm) -> Result<(), String> {
        let point = self.snap_authoring_point(point);
        if self
            .authoring
            .board_outline
            .as_ref()
            .is_none_or(|draft| !draft.is_current(self))
        {
            return Err("Outline draft source changed".into());
        }
        let draft = self
            .authoring
            .board_outline
            .as_mut()
            .ok_or("Begin an Outline first")?;
        draft.editable()?;
        let vertex = Point::new(point.x, point.y);
        if draft.vertices.last() == Some(&vertex) {
            return Err("Choose a different boundary vertex".into());
        }
        draft.vertices.push(vertex);
        draft.refresh_geometry_validation();
        self.authoring.gesture.anchor = draft.vertices.first().map(|p| PointNm { x: p.x, y: p.y });
        self.authoring.gesture.preview = Some(point);
        Ok(())
    }
    pub fn remove_board_outline_vertex(&mut self) -> Result<bool, String> {
        if self
            .authoring
            .board_outline
            .as_ref()
            .is_none_or(|draft| !draft.is_current(self))
        {
            return Err("Outline draft source changed".into());
        }
        let draft = self
            .authoring
            .board_outline
            .as_mut()
            .ok_or("Begin an Outline first")?;
        draft.editable()?;
        let removed = draft.vertices.pop().is_some();
        if removed {
            draft.refresh_geometry_validation();
        }
        self.authoring.gesture.anchor = draft.vertices.first().map(|p| PointNm { x: p.x, y: p.y });
        self.authoring.gesture.preview = draft.vertices.last().map(|p| PointNm { x: p.x, y: p.y });
        Ok(removed)
    }
    pub fn prepare_board_outline(&mut self) -> Result<NativeEditIntent, String> {
        let draft = self
            .authoring
            .board_outline
            .as_ref()
            .ok_or("Begin an Outline first")?;
        draft.editable()?;
        if !draft.is_current(self) {
            return Err("Outline draft source changed".into());
        }
        let polygon = draft.polygon();
        // Finish recertifies the intent; the cached preview is never authority.
        eda_engine::board::occupied_copper::validate_polygon(&polygon)
            .map_err(outline_geometry_error)?;
        let request = OutlineRequest {
            basis: draft.basis.clone(),
            polygon,
        };
        let model = self
            .library
            .basis
            .as_ref()
            .ok_or("Native source unavailable")?;
        prepare_outline(
            model,
            WriteProvenance::new("datum-gui", CommitSource::Manual, "Native Board Outline"),
            &request,
        )
        .map_err(|e| e.to_string())?;
        let mut intent = NativeEditIntent::new(
            draft.root.clone(),
            draft.basis.project,
            draft.basis.revision.0.clone(),
            draft.basis.tip,
            NativeEditAction::BoardOutline,
        );
        intent.board_outline = Some(request);
        self.authoring.board_outline.as_mut().unwrap().pending = true;
        Ok(intent)
    }
}
impl NativeEditIntent {
    pub(crate) fn validate_outline_payload(&self) -> anyhow::Result<()> {
        if !matches!(self.action, NativeEditAction::BoardOutline) {
            anyhow::ensure!(
                self.board_outline.is_none(),
                "Outline payload does not match edit action"
            );
            return Ok(());
        }
        let request = self
            .board_outline
            .as_ref()
            .context("Outline payload missing")?;
        anyhow::ensure!(
            request.basis.project == self.project_id
                && request.basis.revision.0 == self.expected_revision
                && request.basis.tip == self.expected_tip,
            "Outline source basis does not match prepared intent"
        );
        anyhow::ensure!(
            self.proposal.is_none()
                && self.schematic.is_none()
                && self.library.is_none()
                && self.library_acquisition.is_none()
                && self.board_transfer.is_none()
                && self.board_route.is_none()
                && self.board_zone.is_none()
                && self.clearance_rule.is_none()
                && self.symbol_group.is_none()
                && self.symbol_alignment.is_none()
                && self.instance_path.is_empty(),
            "Outline intent contains another editor's payload"
        );
        Ok(())
    }
}
