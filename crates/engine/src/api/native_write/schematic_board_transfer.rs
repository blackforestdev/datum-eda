//! Initial, source-qualified schematic-to-PCB transfer over the canonical writer.
//! Planning is read-only. Creation is keyed by authored component identity,
//! never reference text, and is distinct from later board reconciliation.
use super::registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params};
use super::{PreparedWrite, WriteProvenance};
use crate::error::EngineError;
use crate::ir::geometry::Point;
use crate::substrate::{ElectricalOccurrence, ModelRevision};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod compose;
pub mod field_revision;
mod plan;
pub(crate) use compose::initial_bindings;
pub use plan::plan_transfer;

pub(crate) const VERBS: &[NativeWriteVerb] = &[
    NativeWriteVerb {
        id: "datum.schematic.transfer_to_pcb",
        build: transfer,
    },
    NativeWriteVerb {
        id: "datum.pcb.prepare_footprint_sources",
        build: prepare_sources,
    },
];
fn prepare_sources(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Request {
        board: Uuid,
    }
    let request: Request = parse_verb_params("datum.pcb.prepare_footprint_sources", params)?;
    super::board_layout::source_profile::build_adopt_board_source_profile(
        context.model,
        provenance,
        request.board,
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferChoice {
    pub component_instance: Uuid,
    pub pin_pad_map: Uuid,
}

/// The retained edit seed qualifies prospective first-birth IDs across review
/// and acceptance. It does not register anything during a query or preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferRequest {
    pub sheet: Uuid,
    pub choices: Vec<TransferChoice>,
    pub edit_seed: Uuid,
    pub origin: Point,
    pub pitch_nm: i64,
    pub layer: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferStatus {
    Ready,
    Existing,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferMapOption {
    pub pin_pad_map: Uuid,
    pub footprint: Uuid,
    pub footprint_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferConnection {
    pub library_pad: Uuid,
    pub library_pin: Uuid,
    pub schematic_terminal: ElectricalOccurrence,
    pub logical_net: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferComponent {
    pub component_instance: Option<Uuid>,
    pub symbols: Vec<Uuid>,
    pub reference: String,
    pub value: Option<String>,
    pub part: Option<Uuid>,
    pub package: Option<Uuid>,
    pub footprint: Option<Uuid>,
    pub pin_pad_map: Option<Uuid>,
    pub map_options: Vec<TransferMapOption>,
    pub connections: Vec<TransferConnection>,
    pub existing_packages: Vec<Uuid>,
    pub status: TransferStatus,
    pub reason: Option<String>,
    #[serde(default)]
    pub component_choice: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferPlan {
    pub project_id: Uuid,
    pub expected_model_revision: ModelRevision,
    pub expected_tip: Option<Uuid>,
    pub board_id: Uuid,
    pub sheet: Uuid,
    pub requires_profile_adoption: bool,
    pub components: Vec<TransferComponent>,
}
impl TransferPlan {
    pub fn ready_count(&self) -> usize {
        self.components
            .iter()
            .filter(|c| c.status == TransferStatus::Ready)
            .count()
    }
}

pub fn prepare_transfer(
    model: &crate::substrate::DesignModel,
    provenance: WriteProvenance,
    request: &TransferRequest,
) -> Result<PreparedWrite, EngineError> {
    let plan = plan_transfer(model, request)?;
    if plan.requires_profile_adoption {
        return Err(invalid(
            "Explicitly prepare the board for placed footprint sources before transfer",
        ));
    }
    if plan.ready_count() == 0 {
        return Err(invalid(
            "No eligible untransferred components; the design is unchanged",
        ));
    }
    compose::prepare(model, provenance, request, &plan)
}

fn transfer(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    mut params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    // Existing-counterpart propagation is proposal-first even for a manual
    // caller. Do not let the ordinary direct native.write route bypass review.
    if let Some(intent) = params.get("intent") {
        match intent.as_str() {
            Some("initial") => {
                params.as_object_mut().unwrap().remove("intent");
            }
            Some("reference_value_revision") => {
                return Err(invalid(
                    "Reference/value revision requires native.proposal create_board_field_revision; preview the Draft and explicitly accept or reject it",
                ));
            }
            _ => return Err(invalid("Unsupported PCB transfer intent")),
        }
    }
    let request = parse_verb_params("datum.schematic.transfer_to_pcb", params)?;
    prepare_transfer(context.model, provenance, &request)
}

fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(format!("Initial PCB transfer: {}", reason.into()))
}

#[cfg(test)]
mod tests;
