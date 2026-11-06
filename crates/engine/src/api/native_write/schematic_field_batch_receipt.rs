//! Exact before/after field review and recovery through shared guarded receipts.
use super::super::schematic_symbol_edit_receipt::{self as shared, Recorded};
use super::*;
use crate::{schematic::PlacedSymbol, substrate::Proposal};

fn recorded(proposal: &Proposal) -> Result<Recorded<'_>, EngineError> {
    if !is_field_batch(proposal) {
        return Err(invalid("Symbol Fields Table proposal family unavailable"));
    }
    let result = shared::recorded(proposal)?;
    result.single_sheet()?;
    for value in result.symbols.values() {
        let typed: PlacedSymbol = serde_json::from_value((*value).clone())?;
        validate_field_values(Some(&typed.reference), Some(&typed.value))?;
    }
    Ok(result)
}

pub(in crate::api::native_write) fn validate_recorded(
    proposal: &Proposal,
) -> Result<(), EngineError> {
    recorded(proposal).map(|_| ())
}

pub(in crate::api::native_write) fn summary(
    model: &DesignModel,
    proposal: &Proposal,
    current: bool,
) -> Result<FieldBatchSummary, EngineError> {
    let recorded = recorded(proposal)?;
    let sheet = recorded.single_sheet()?;
    let path = shared::occurrence_path(model, sheet)?;
    if current {
        if proposal.prepared_against != model.model_revision
            || proposal.batch.expected_model_revision.as_ref() != Some(&model.model_revision)
        {
            return Err(invalid(
                "Design changed; review the Symbol Fields Table again",
            ));
        }
        let edits = recorded
            .symbols
            .iter()
            .map(|(id, payload)| {
                let symbol: PlacedSymbol = serde_json::from_value((*payload).clone())?;
                Ok(FieldRowEdit {
                    symbol: *id,
                    reference: symbol.reference,
                    value: symbol.value,
                })
            })
            .collect::<Result<Vec<_>, EngineError>>()?;
        let rebuilt = build(model, sheet, &path, &edits)?;
        if rebuilt.batch.operations != proposal.batch.operations {
            return Err(invalid(
                "Symbol Fields Table proposal differs from the current complete batch",
            ));
        }
    }
    let old_payloads = shared::old_payloads(model, proposal, &recorded)?;
    let mut changes = Vec::new();
    for (id, next) in recorded.symbols {
        let old = old_payloads
            .get(&id)
            .ok_or_else(|| invalid("Symbol field preimage unavailable"))?;
        let before: PlacedSymbol = serde_json::from_value(old.clone())?;
        let after: PlacedSymbol = serde_json::from_value(next.clone())?;
        let mut expected = old.clone();
        expected["reference"] = json!(after.reference);
        expected["value"] = json!(after.value);
        if before.uuid != id
            || after.uuid != id
            || expected != *next
            || (before.reference == after.reference && before.value == after.value)
        {
            return Err(invalid(
                "Symbol Fields Table changes unrelated or unchanged source",
            ));
        }
        changes.push(FieldRowChange {
            symbol: id,
            previous_reference: before.reference,
            previous_value: before.value,
            reference: after.reference,
            value: after.value,
        });
    }
    Ok(FieldBatchSummary {
        sheet,
        instance_path: path,
        changes,
    })
}
