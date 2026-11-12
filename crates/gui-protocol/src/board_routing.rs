//! A board route owns source intent and anchored vertices, not another input
//! controller. Cursor, contact/grid snapping and cancellation belong to the
//! shared workspace gesture; path geometry and write admission belong to engine.
use crate::{AuthoringViewBasis, PaneContent, PointNm, ReviewWorkspaceState, WorkspaceTool};
use eda_engine::api::native_write::{
    WriteProvenance,
    board_routing::route::{RouteBasis, RouteRequest, capture_route_basis, prepare_route},
    schematic_wire_edit::{WireRouteConfig, routed_wire_points},
};
use eda_engine::{ir::geometry::Point, substrate::CommitSource};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardRouteDraft {
    project_root: PathBuf,
    view: AuthoringViewBasis,
    basis: RouteBasis,
    edit_seed: Uuid,
    vertices: Vec<Point>,
    width_nm: Option<i64>,
    posture: WireRouteConfig,
    pending: Option<RouteRequest>,
}

impl BoardRouteDraft {
    fn capture(
        state: &ReviewWorkspaceState,
        start_pad: Uuid,
        layer: i32,
        posture: WireRouteConfig,
    ) -> Result<Self, String> {
        let root = native_root(state)?;
        let view = AuthoringViewBasis::capture(state, state.ui.layout.focused)?;
        if view.content != PaneContent::Board {
            return Err("Focus the native Board before routing".into());
        }
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Native source unavailable")?;
        let basis = capture_route_basis(model, start_pad, layer).map_err(|e| e.to_string())?;
        if basis.board != view.source {
            return Err("Board source view is stale".into());
        }
        Ok(Self {
            project_root: root,
            view,
            vertices: vec![basis.start],
            width_nm: basis.preferred_width_nm,
            basis,
            edit_seed: Uuid::new_v4(),
            posture,
            pending: None,
        })
    }

    pub fn basis(&self) -> &RouteBasis {
        &self.basis
    }

    pub fn vertices(&self) -> &[Point] {
        &self.vertices
    }

    pub fn width_nm(&self) -> Option<i64> {
        self.width_nm
    }

    pub fn is_current(&self, state: &ReviewWorkspaceState) -> bool {
        native_root(state).is_ok_and(|root| root == self.project_root)
            && self.view.is_current(state, state.ui.layout.focused)
            && state.library.basis.as_ref().is_some_and(|model| {
                model.project.project_id == self.basis.project
                    && model.model_revision == self.basis.revision
                    && eda_engine::revision::transaction_tip(&model.journal) == self.basis.tip
            })
    }

    fn editable(&self) -> Result<(), String> {
        if self.pending.is_some() {
            Err("Wait for the pending Board route edit".into())
        } else {
            Ok(())
        }
    }

    pub fn set_width_nm(&mut self, width: i64) -> Result<(), String> {
        self.editable()?;
        if width <= 0
            || self.basis.minimum_width_nm.is_some_and(|min| width < min)
            || self.basis.maximum_width_nm.is_some_and(|max| width > max)
        {
            return Err(
                "Route width must be positive and satisfy current width constraints".into(),
            );
        }
        self.width_nm = Some(width);
        Ok(())
    }

    pub fn set_posture(&mut self, posture: WireRouteConfig) -> Result<(), String> {
        self.editable()?;
        self.posture = posture;
        Ok(())
    }

    /// Endpoints are already resolved by shared snapping. Existing anchored
    /// corners remain fixed when the designer changes the next span's posture.
    pub fn preview_points(&self, cursor: Option<PointNm>) -> Result<Vec<Point>, String> {
        let mut points = self.vertices.clone();
        if let Some(to) = cursor {
            let from = *points.last().ok_or("Route has no start")?;
            points.extend(
                routed_wire_points(from, Point::new(to.x, to.y), self.posture)
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .skip(1),
            );
        }
        Ok(points)
    }

    pub fn append_vertex(&mut self, point: PointNm) -> Result<(), String> {
        self.editable()?;
        self.vertices = self.preview_points(Some(point))?;
        Ok(())
    }

    pub fn remove_last_vertex(&mut self) -> bool {
        if self.pending.is_some() || self.vertices.len() <= 1 {
            return false;
        }
        self.vertices.pop();
        true
    }

    /// Retain one immutable request for worker submission. Pure canonical
    /// preparation checks the whole chain before returning it; the writer still
    /// revalidates against its fresh model under the project lease.
    pub fn prepare_intent(
        &mut self,
        state: &ReviewWorkspaceState,
        cursor: Option<PointNm>,
        end_pad: Option<Uuid>,
    ) -> Result<crate::native_edit_client::NativeEditIntent, String> {
        self.editable()?;
        if !self.is_current(state) {
            return Err("Board route source changed; start a new route".into());
        }
        let request = RouteRequest {
            basis: self.basis.clone(),
            end_pad,
            points: self.preview_points(cursor)?,
            width_nm: self.width_nm.ok_or("Enter an explicit route width")?,
            edit_seed: self.edit_seed,
        };
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Native source unavailable")?;
        prepare_route(
            model,
            WriteProvenance::new("datum-gui", CommitSource::Manual, "Manual Board route"),
            &request,
        )
        .map_err(|e| e.to_string())?;
        let mut intent = crate::native_edit_client::NativeEditIntent::new(
            self.project_root.clone(),
            self.basis.project,
            self.basis.revision.0.clone(),
            self.basis.tip,
            crate::native_edit_client::NativeEditAction::BoardRoute,
        );
        intent.board_route = Some(request.clone());
        self.pending = Some(request);
        Ok(intent)
    }
}

fn native_root(state: &ReviewWorkspaceState) -> Result<PathBuf, String> {
    let backing = state
        .backing
        .as_ref()
        .ok_or("Open a native Project before routing")?;
    if state.project_unopened
        || backing.request.board_file.is_some()
        || backing.request.kicad_board_source.is_some()
    {
        return Err("Routing requires a native Project".into());
    }
    Ok(backing.request.project_root.clone())
}

impl ReviewWorkspaceState {
    pub fn begin_board_route(
        &mut self,
        start_pad: Uuid,
        layer: i32,
        posture: WireRouteConfig,
    ) -> Result<(), String> {
        if self.authoring.board_route.is_some() {
            return Err("Finish or cancel the current Board route first".into());
        }
        let draft = BoardRouteDraft::capture(self, start_pad, layer, posture)?;
        let start = PointNm {
            x: draft.basis.start.x,
            y: draft.basis.start.y,
        };
        self.set_tool(WorkspaceTool::DrawBoardTrack);
        self.begin_authoring_gesture_resolved(start, Some(start_pad.to_string()));
        self.authoring.gesture.view_basis = Some(draft.view.clone());
        self.authoring.board_route = Some(draft);
        Ok(())
    }

    /// Shared lifecycle hook. A camera-only change keeps the same consumer and
    /// source basis; switching pane, document, root or journal tip retires it.
    pub fn retire_stale_board_route(&mut self) -> bool {
        if self
            .authoring
            .board_route
            .as_ref()
            .is_some_and(|draft| !draft.is_current(self))
        {
            self.cancel_authoring_gesture()
        } else {
            false
        }
    }
}

#[cfg(test)]
#[path = "board_routing_tests.rs"]
mod tests;
