//! Zone intent over the shared workspace gesture and canonical native client.
//! No menu/control, cursor, snap controller or fill solver lives here.
use super::board_routing::native_root;
use crate::native_edit_client::{NativeEditAction, NativeEditIntent};
use crate::{AuthoringViewBasis, PaneContent, PointNm, ReviewWorkspaceState, WorkspaceTool};
use eda_engine::api::native_write::{
    WriteProvenance,
    board_routing::zone::{
        BoardBasis, FillRequest, ZoneBasis, ZoneRequest, capture_board_basis, capture_zone_basis,
        prepare_fill, prepare_zone,
    },
};
use eda_engine::{
    ir::geometry::{Point, Polygon},
    substrate::CommitSource,
};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum NativeZoneEdit {
    Boundary(Box<ZoneRequest>),
    Fill(FillRequest),
}
impl NativeZoneEdit {
    pub fn basis(&self) -> &BoardBasis {
        match self {
            Self::Boundary(r) => &r.basis.source,
            Self::Fill(r) => &r.basis,
        }
    }
    pub fn verb_params(&self) -> Result<(&'static str, serde_json::Value), serde_json::Error> {
        match self {
            Self::Boundary(r) => Ok(("datum.pcb.place_zone", serde_json::to_value(r)?)),
            Self::Fill(r) => Ok(("datum.check.fill_zones", serde_json::to_value(r)?)),
        }
    }
}

/// All fields express explicit designer intent. No Net, priority, thermal
/// dimensions or transient clearance is manufactured by an editor default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneSettings {
    pub priority: u32,
    pub thermal_relief: bool,
    pub thermal_gap_nm: i64,
    pub thermal_spoke_width_nm: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardZoneDraft {
    root: PathBuf,
    view: AuthoringViewBasis,
    basis: ZoneBasis,
    edit_seed: Uuid,
    vertices: Vec<Point>,
    settings: Option<ZoneSettings>,
    pending: bool,
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new("datum-gui", CommitSource::Manual, "Native Zone edit")
}
fn view(state: &ReviewWorkspaceState) -> Result<AuthoringViewBasis, String> {
    let view = AuthoringViewBasis::capture(state, state.ui.layout.focused)?;
    if view.content != PaneContent::Board {
        return Err("Focus the native Board before authoring Zones".into());
    }
    Ok(view)
}
fn intent(root: PathBuf, edit: NativeZoneEdit) -> NativeEditIntent {
    let basis = edit.basis();
    let mut intent = NativeEditIntent::new(
        root,
        basis.project,
        basis.revision.0.clone(),
        basis.tip,
        NativeEditAction::BoardZone,
    );
    intent.board_zone = Some(edit);
    intent
}
impl BoardZoneDraft {
    pub fn basis(&self) -> &ZoneBasis {
        &self.basis
    }
    pub fn vertices(&self) -> &[Point] {
        &self.vertices
    }
    pub fn settings(&self) -> Option<&ZoneSettings> {
        self.settings.as_ref()
    }
    pub fn pending(&self) -> bool {
        self.pending
    }
    pub fn is_current(&self, state: &ReviewWorkspaceState) -> bool {
        native_root(state).is_ok_and(|root| root == self.root)
            && self.view.is_current(state, state.ui.layout.focused)
            && state.library.basis.as_ref().is_some_and(|model| {
                capture_zone_basis(model, self.basis.net, self.basis.layer)
                    .is_ok_and(|b| b == self.basis)
            })
    }
    fn editable(&self) -> Result<(), String> {
        if self.pending {
            Err("Wait for the pending Zone edit".into())
        } else {
            Ok(())
        }
    }
    pub fn configure(&mut self, settings: ZoneSettings) -> Result<(), String> {
        self.editable()?;
        if settings.thermal_gap_nm < 0 || settings.thermal_spoke_width_nm < 0 {
            return Err("Thermal dimensions must be nonnegative".into());
        }
        self.settings = Some(settings);
        Ok(())
    }
    fn request(&self) -> Result<ZoneRequest, String> {
        let settings = self
            .settings
            .as_ref()
            .ok_or("Set explicit Zone priority and thermal intent")?;
        Ok(ZoneRequest {
            basis: self.basis.clone(),
            polygon: Polygon::new(self.vertices.clone()),
            priority: settings.priority,
            thermal_relief: settings.thermal_relief,
            thermal_gap_nm: settings.thermal_gap_nm,
            thermal_spoke_width_nm: settings.thermal_spoke_width_nm,
            edit_seed: self.edit_seed,
        })
    }
}
impl ReviewWorkspaceState {
    pub fn begin_board_zone(&mut self, net: Uuid, layer: i32) -> Result<(), String> {
        if !crate::check_runs::producer::CheckProducerRequest::editor_preview_idle(self) {
            return Err("Finish or cancel the active edit before authoring a Zone".into());
        }
        let root = native_root(self)?;
        let view = view(self)?;
        let backing = self.backing.as_ref().ok_or("Open a native Project")?;
        if !backing.native_write_verbs.contains("datum.pcb.place_zone") {
            return Err("Native Zone boundary authoring unavailable".into());
        }
        let model = self
            .library
            .basis
            .as_ref()
            .ok_or("Native source unavailable")?;
        let basis = capture_zone_basis(model, net, layer).map_err(|e| e.to_string())?;
        if basis.source.board != view.source {
            return Err("Board source view is stale".into());
        }
        let draft = BoardZoneDraft {
            root,
            view,
            basis,
            edit_seed: Uuid::new_v4(),
            vertices: Vec::new(),
            settings: None,
            pending: false,
        };
        self.set_tool(WorkspaceTool::DrawBoardZone);
        self.authoring.gesture.view_basis = Some(draft.view.clone());
        self.authoring.board_zone = Some(draft);
        Ok(())
    }
    pub fn append_board_zone_vertex(&mut self, point: PointNm) -> Result<(), String> {
        let point = self.snap_authoring_point(point);
        if self
            .authoring
            .board_zone
            .as_ref()
            .is_none_or(|d| !d.is_current(self))
        {
            return Err("Zone draft source changed".into());
        }
        let draft = self
            .authoring
            .board_zone
            .as_mut()
            .ok_or("Begin a Zone first")?;
        draft.editable()?;
        let vertex = Point::new(point.x, point.y);
        if draft.vertices.last() == Some(&vertex) {
            return Err("Choose a different boundary vertex".into());
        }
        draft.vertices.push(vertex);
        if self.authoring.gesture.anchor.is_none() {
            self.authoring.gesture.anchor = Some(point);
        }
        self.authoring.gesture.preview = Some(point);
        Ok(())
    }
    pub fn remove_board_zone_vertex(&mut self) -> Result<bool, String> {
        if self
            .authoring
            .board_zone
            .as_ref()
            .is_none_or(|d| !d.is_current(self))
        {
            return Err("Zone draft source changed".into());
        }
        let draft = self
            .authoring
            .board_zone
            .as_mut()
            .ok_or("Begin a Zone first")?;
        draft.editable()?;
        let removed = draft.vertices.pop().is_some();
        self.authoring.gesture.anchor = draft.vertices.first().map(|p| PointNm { x: p.x, y: p.y });
        self.authoring.gesture.preview = draft.vertices.last().map(|p| PointNm { x: p.x, y: p.y });
        Ok(removed)
    }
    pub fn prepare_board_zone(&mut self) -> Result<NativeEditIntent, String> {
        let draft = self
            .authoring
            .board_zone
            .as_ref()
            .ok_or("Begin a Zone first")?;
        draft.editable()?;
        if !draft.is_current(self) {
            return Err("Zone draft source changed".into());
        }
        let request = draft.request()?;
        let model = self
            .library
            .basis
            .as_ref()
            .ok_or("Native source unavailable")?;
        prepare_zone(model, provenance(), &request).map_err(|e| e.to_string())?;
        let prepared = intent(
            draft.root.clone(),
            NativeZoneEdit::Boundary(Box::new(request)),
        );
        self.authoring.board_zone.as_mut().unwrap().pending = true;
        Ok(prepared)
    }
    pub fn prepare_board_zone_fill(
        &self,
        zone: Option<Uuid>,
        net: Option<Uuid>,
    ) -> Result<NativeEditIntent, String> {
        if !crate::check_runs::producer::CheckProducerRequest::editor_preview_idle(self) {
            return Err("Finish or cancel the active edit before filling Zones".into());
        }
        let root = native_root(self)?;
        let view = view(self)?;
        if !self
            .backing
            .as_ref()
            .is_some_and(|b| b.native_write_verbs.contains("datum.check.fill_zones"))
        {
            return Err("Native explicit Zone fill unavailable".into());
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
        let request = FillRequest { basis, zone, net };
        prepare_fill(model, provenance(), &request).map_err(|e| e.to_string())?;
        Ok(intent(root, NativeZoneEdit::Fill(request)))
    }
    pub fn retire_stale_board_zone(&mut self) -> bool {
        if self
            .authoring
            .board_zone
            .as_ref()
            .is_some_and(|d| !d.is_current(self))
        {
            self.cancel_authoring_gesture()
        } else {
            false
        }
    }
}

#[cfg(test)]
#[path = "board_zones_tests.rs"]
mod tests;
