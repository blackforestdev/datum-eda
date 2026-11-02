//! Source-qualified Label/Power move using the shared attachment geometry guard.
use super::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, parse_verb_params},
};
use crate::{
    connectivity::SymbolContactBasis,
    error::EngineError,
    ir::geometry::Point,
    schematic::{NetLabel, SchematicWire},
    substrate::{ElectricalSelectionSnapshot, Operation, SelectionSourceDomain},
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct LabelMoveBasis {
    pub sheet_id: Uuid,
    pub label: NetLabel,
    contacts: SymbolContactBasis,
}
#[derive(Debug, Clone, PartialEq)]
pub struct LabelMoveDestination {
    pub label: NetLabel,
    pub wires: Vec<SchematicWire>,
}
impl LabelMoveBasis {
    pub fn capture(
        snapshot: &ElectricalSelectionSnapshot,
        label: Uuid,
        instance_path: &[Uuid],
    ) -> Result<Self, EngineError> {
        let sources = snapshot
            .authored_sources()
            .map_err(|e| invalid(format!("Label occurrence basis unavailable: {e:?}")))?;
        let occurrences: Vec<_> = sources
            .iter()
            .filter(|s| {
                s.domain == SelectionSourceDomain::Schematic
                    && s.occurrence.class == "labels"
                    && s.occurrence.source_id == label
            })
            .collect();
        if occurrences.len() != 1 || occurrences[0].occurrence.instance_path != instance_path {
            return Err(invalid(
                "Label move requires exactly one complete matching occurrence",
            ));
        }
        let source = snapshot
            .schematic_source()
            .ok_or_else(|| invalid("Native schematic source unavailable"))?;
        let sheets: Vec<_> = source
            .sheets
            .values()
            .filter(|s| s.labels.contains_key(&label))
            .collect();
        if sheets.len() != 1 {
            return Err(invalid("Label source ownership is ambiguous"));
        }
        let sheet = sheets[0];
        if sheet.labels.iter().any(|(id, label)| *id != label.uuid) {
            return Err(invalid("Label source identity is inconsistent"));
        }
        Ok(Self {
            sheet_id: sheet.uuid,
            label: sheet.labels[&label].clone(),
            contacts: SymbolContactBasis::capture_label(sheet, label)?,
        })
    }
    pub fn destination_edit(&self, position: Point) -> Result<LabelMoveDestination, EngineError> {
        let mut label = self.label.clone();
        label.position = position;
        let wires = self.contacts.moved_label_wires(&label)?;
        Ok(LabelMoveDestination { label, wires })
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveParams {
    label: Uuid,
    instance_path: Vec<Uuid>,
    x_nm: i64,
    y_nm: i64,
}
pub(super) fn move_label(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: MoveParams = parse_verb_params("datum.schematic.move_label", params)?;
    let snapshot = ElectricalSelectionSnapshot::capture(context.model)
        .map_err(|e| invalid(format!("Label source/contact basis unavailable: {e:?}")))?;
    let basis = LabelMoveBasis::capture(&snapshot, params.label, &params.instance_path)?;
    let destination = basis.destination_edit(Point::new(params.x_nm, params.y_nm))?;
    if destination.label == basis.label {
        return Err(invalid("Label move destination is unchanged"));
    }
    let mut operations = vec![Operation::SetSchematicLabel {
        sheet_id: basis.sheet_id,
        label_id: destination.label.uuid,
        label: serde_json::to_value(&destination.label)?,
    }];
    for wire in destination.wires {
        operations.push(Operation::DeleteSchematicWire {
            sheet_id: basis.sheet_id,
            wire_id: wire.uuid,
            wire: serde_json::to_value(basis.contacts.original_wire(wire.uuid))?,
        });
        operations.push(Operation::CreateSchematicWire {
            sheet_id: basis.sheet_id,
            wire_id: wire.uuid,
            wire: serde_json::to_value(wire)?,
        });
    }
    BatchComposer::compose(context.model, provenance)
        .push_ops(operations)
        .primary_object(destination.label.uuid)
        .finish()
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}
#[cfg(test)]
#[path = "schematic_label_move_tests.rs"]
mod tests;
