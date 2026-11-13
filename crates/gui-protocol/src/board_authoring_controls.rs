//! PCB tool configuration over the shared gesture, units and text editor.
//! These buffers hold designer intent; canonical drafts and the engine own geometry.
use super::board_zones::ZoneSettings;
use crate::single_line_editor::SingleLineEditor;
use crate::{AuthoringViewBasis, PaneContent, ReviewWorkspaceState};
use eda_engine::ir::units::{
    GuiProjectLengthInput, LengthEditSession, LengthQuantity, Nanometers, ResolvedUnitsProfile,
    resolve_gui_project_length,
};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardField {
    Width,
    Clearance,
    RulePriority,
    Net,
    Priority,
    ThermalGap,
    SpokeWidth,
}
impl BoardField {
    pub fn index(self) -> usize {
        self as usize
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardControl {
    Field(BoardField),
    Route,
    Zone,
    Outline,
    Thermal,
    Posture,
    Finish,
    Cancel,
    ApplyRule,
    Fill,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardToolKind {
    Route,
    Zone,
    Outline,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BoardFieldBuffer {
    pub text: String,
    pub editor: SingleLineEditor,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardAuthoringControls {
    pub basis: AuthoringViewBasis,
    pub root: std::path::PathBuf,
    pub tip: Option<Uuid>,
    pub units: ResolvedUnitsProfile,
    pub layer: i32,
    pub layer_name: String,
    pub kind: BoardToolKind,
    pub fields: [BoardFieldBuffer; 7],
    pub focus: Option<BoardControl>,
    pub thermal: Option<bool>,
    pub resolved_zone_net: Result<Uuid, String>,
    pub zone_clearance:
        Result<eda_engine::api::native_write::board_routing::zone::ZoneClearanceReadback, String>,
    pub needs_clearance_rule: bool,
    pub refusal: Option<String>,
    pub pending: bool,
}
impl BoardAuthoringControls {
    pub fn capture(state: &ReviewWorkspaceState, kind: BoardToolKind) -> Result<Self, String> {
        let basis = AuthoringViewBasis::capture(state, state.ui.layout.focused)?;
        if basis.content != PaneContent::Board {
            return Err("Focus the native Board".into());
        }
        let root = super::board_routing::native_root(state)?;
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Native source unavailable")?;
        let units = eda_engine::api::native_write::project::project_display_units(model)
            .map_err(|e| e.to_string())?
            .resolve()
            .map_err(|e| format!("Project units unavailable: {e:?}"))?;
        let (id, layer_name) = if kind == BoardToolKind::Outline {
            (0, "board · Edge.Cuts".to_string())
        } else {
            let selected = state
                .ui
                .filters
                .active_layer_id
                .as_deref()
                .ok_or("Choose a copper layer")?;
            let layer = state
                .scene
                .layers
                .iter()
                .find(|l| l.layer_id == selected)
                .ok_or("Selected layer is unavailable")?;
            let id: i32 = selected
                .strip_prefix('L')
                .ok_or("Choose a native copper layer")?
                .parse()
                .map_err(|_| "Invalid native layer")?;
            let board = model
                .capture_source_shard_value(
                    model
                        .source_shards
                        .iter()
                        .find(|s| s.kind == eda_engine::substrate::SourceShardKind::BoardRoot)
                        .ok_or("Captured Board source unavailable")?
                        .shard_id,
                )
                .map_err(|e| e.to_string())?;
            let stackup: eda_engine::board::Stackup =
                serde_json::from_value(board["stackup"].clone()).map_err(|e| e.to_string())?;
            if stackup
                .layers
                .iter()
                .filter(|l| {
                    l.id == id && l.layer_type == eda_engine::board::StackupLayerType::Copper
                })
                .count()
                != 1
            {
                return Err("Choose one existing copper layer".into());
            }
            (id, layer.name.clone())
        };
        Ok(Self {
            basis,
            root,
            tip: eda_engine::revision::transaction_tip(&model.journal),
            units,
            layer: id,
            layer_name,
            kind,
            fields: std::array::from_fn(|_| BoardFieldBuffer::default()),
            focus: None,
            thermal: None,
            resolved_zone_net: Err("Enter one existing Net name or UUID".into()),
            zone_clearance: Err("Choose a Zone Net first".into()),
            needs_clearance_rule: false,
            refusal: None,
            pending: false,
        })
    }
    /// Consumer input/Inspector ownership is local to the captured pane. A
    /// paused draft never takes keys or chrome from another editor leaf.
    pub fn owns_pane(&self, state: &ReviewWorkspaceState) -> bool {
        state.ui.layout.focused == self.basis.pane
            && state.ui.layout.focused_content() == PaneContent::Board
            && state.inspector_context.owner != crate::inspector_context::InspectorOwner::Navigator
    }
    pub fn current(&self, state: &ReviewWorkspaceState) -> bool {
        !self.pending
            && super::board_routing::native_root(state).is_ok_and(|root| root == self.root)
            && state.library.basis.as_ref().is_some_and(|model| {
                eda_engine::revision::transaction_tip(&model.journal) == self.tip
            })
            && self.basis.is_current(state, state.ui.layout.focused)
            && (self.kind == BoardToolKind::Outline
                || state.ui.filters.active_layer_id.as_deref()
                    == Some(format!("L{}", self.layer).as_str()))
    }
    /// Refresh this transient projection on Net/source changes, never in paint.
    /// It enables presentation only; Finish recaptures its canonical intent.
    pub fn refresh_zone_projection(&mut self, state: &ReviewWorkspaceState) {
        if self.kind != BoardToolKind::Zone {
            return;
        }
        if !self.current(state) {
            self.resolved_zone_net = Err("Zone tool source changed".into());
            self.zone_clearance = Err("Zone tool source changed".into());
            self.needs_clearance_rule = false;
            return;
        }
        self.needs_clearance_rule = false;
        let net = self.zone_net(state);
        let result = (|| -> Result<_, String> {
            let net = *net.as_ref().map_err(Clone::clone)?;
            let model = state
                .library
                .basis
                .as_ref()
                .ok_or("Native source unavailable")?;
            let basis = eda_engine::api::native_write::board_routing::zone::capture_zone_basis(
                model, net, self.layer,
            )
            .map_err(|e| e.to_string())?;
            self.needs_clearance_rule = basis.clearance.is_none();
            eda_engine::api::native_write::board_routing::zone::capture_zone_clearance_readback(
                model, net, self.layer,
            )
            .map_err(|e| e.to_string())
        })();
        self.resolved_zone_net = net;
        self.zone_clearance = result;
    }
    pub fn zone_clearance_label(&self) -> Result<String, String> {
        let readback = self.zone_clearance.as_ref().map_err(Clone::clone)?;
        let formatted =
            eda_engine::ir::units::format_length(eda_engine::ir::units::FormatLengthRequest {
                value: Nanometers::new(readback.minimum_nm),
                quantity: eda_engine::ir::units::QuantityKind::Length,
                display_unit: self.units.board.unit,
                measurement_system: self.units.system,
                precision: self.units.board.precision,
            })
            .map_err(|e| format!("Length display unavailable: {:?}", e.reason))?;
        Ok(format!(
            "{}{}",
            formatted.rendered,
            if formatted.rounded { " (rounded)" } else { "" }
        ))
    }
    pub fn field(&self, field: BoardField) -> &BoardFieldBuffer {
        &self.fields[field.index()]
    }
    pub fn field_mut(&mut self, field: BoardField) -> &mut BoardFieldBuffer {
        &mut self.fields[field.index()]
    }
    pub fn set_length(&mut self, field: BoardField, value: i64) {
        let text = LengthEditSession::begin(
            Nanometers::new(value),
            self.units.board.unit,
            self.units.system,
        )
        .buffer()
        .to_owned();
        *self.field_mut(field) = BoardFieldBuffer {
            editor: SingleLineEditor::at_end(&text),
            text,
        };
    }
    pub fn length(&self, field: BoardField) -> Result<i64, String> {
        resolve_gui_project_length(GuiProjectLengthInput {
            token: &self.field(field).text,
            project_id: &self.basis.project.to_string(),
            field_id: "pcb.tool",
            quantity: LengthQuantity::BoardLayout,
            units: self.units,
        })
        .map(|v| v.canonical_nm)
        .map_err(|e| format!("Invalid PCB dimension: {e}"))
    }
    pub fn rule_priority(&self) -> Result<u64, String> {
        self.field(BoardField::RulePriority)
            .text
            .trim()
            .parse()
            .map_err(|_| "Enter an unsigned integer rule priority".into())
    }
    pub fn zone_settings(&self) -> Result<ZoneSettings, String> {
        let priority = self
            .field(BoardField::Priority)
            .text
            .trim()
            .parse::<u32>()
            .map_err(|_| "Enter an unsigned 32-bit Zone priority")?;
        let thermal_relief = self
            .thermal
            .ok_or("Choose solid or thermal relief explicitly")?;
        let thermal_gap_nm = self.length(BoardField::ThermalGap)?;
        let thermal_spoke_width_nm = self.length(BoardField::SpokeWidth)?;
        if thermal_gap_nm < 0 || thermal_spoke_width_nm < 0 {
            return Err("Thermal dimensions must be nonnegative".into());
        }
        Ok(ZoneSettings {
            priority,
            thermal_relief,
            thermal_gap_nm,
            thermal_spoke_width_nm,
        })
    }
    pub fn zone_net(&self, state: &ReviewWorkspaceState) -> Result<Uuid, String> {
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Native source unavailable")?;
        let board = model
            .capture_source_shard_value(
                model
                    .source_shards
                    .iter()
                    .find(|s| s.kind == eda_engine::substrate::SourceShardKind::BoardRoot)
                    .ok_or("Captured Board source unavailable")?
                    .shard_id,
            )
            .map_err(|e| e.to_string())?;
        let nets: std::collections::BTreeMap<String, eda_engine::board::Net> =
            serde_json::from_value(board["nets"].clone()).map_err(|e| e.to_string())?;
        let token = self.field(BoardField::Net).text.trim();
        let matching: Vec<_> = nets
            .values()
            .filter(|n| n.name == token || n.uuid.to_string() == token)
            .collect();
        if matching.len() != 1 {
            return Err("Enter one existing Net name or UUID".into());
        }
        Ok(matching[0].uuid)
    }
}

impl ReviewWorkspaceState {
    /// Finish revalidates the displayed intent, even if no canvas click followed
    /// the last field edit. A boundary cannot silently keep a different Net.
    pub fn prepare_configured_board_zone(
        &mut self,
    ) -> Result<crate::native_edit_client::NativeEditIntent, String> {
        let setup = self
            .authoring
            .board_controls
            .as_ref()
            .ok_or("Start Zone first")?;
        if setup.kind != BoardToolKind::Zone || !setup.current(self) {
            return Err("Zone tool source changed; cancel and restart".into());
        }
        let net = setup.zone_net(self)?;
        let settings = setup.zone_settings()?;
        let draft = self
            .authoring
            .board_zone
            .as_mut()
            .ok_or("Draw the Zone boundary first")?;
        if draft.basis().net != net {
            return Err("Cancel the boundary before changing its Net".into());
        }
        draft.configure(settings)?;
        self.prepare_board_zone()
    }
}

#[cfg(test)]
#[path = "board_authoring_controls_tests.rs"]
mod tests;
