//! Annotation-specific deterministic review over shared field receipts.
use super::super::schematic_symbol_edit_receipt::{self as shared, Recorded};
use super::*;
fn recorded(proposal: &Proposal) -> Result<Recorded<'_>, EngineError> {
    if !is_annotation(proposal) {
        return Err(invalid("Annotation family unavailable"));
    }
    let result = shared::recorded(proposal)?;
    result.single_sheet()?;
    for value in result.symbols.values() {
        let typed: PlacedSymbol = serde_json::from_value((*value).clone())?;
        prefix(&typed.reference)?;
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
) -> Result<AnnotationSummary, EngineError> {
    let recorded = recorded(proposal)?;
    let sheet = recorded.single_sheet()?;
    let path = shared::occurrence_path(model, sheet)?;
    if current {
        if proposal.prepared_against != model.model_revision
            || proposal.batch.expected_model_revision.as_ref() != Some(&model.model_revision)
        {
            return Err(invalid("Design changed; prepare sheet annotation again"));
        }
        let rebuilt = build(model, sheet, &path)?;
        if rebuilt.batch.operations != proposal.batch.operations {
            return Err(invalid(
                "Annotation differs from the complete current sheet",
            ));
        }
    }
    let old_payloads = shared::old_payloads(model, proposal, &recorded)?;
    let mut changes = vec![];
    for (id, next) in recorded.symbols {
        let old = old_payloads
            .get(&id)
            .ok_or_else(|| invalid("Annotation preimage unavailable"))?;
        let typed: PlacedSymbol = serde_json::from_value(old.clone())?;
        let reference = next["reference"]
            .as_str()
            .ok_or_else(|| invalid("Annotation reference unavailable"))?;
        let mut expected = old.clone();
        expected["reference"] = json!(reference);
        if typed.uuid != id || typed.reference == reference || expected != *next {
            return Err(invalid("Annotation changes unrelated Symbol source"));
        }
        changes.push(AnnotationChange {
            symbol: id,
            previous_reference: typed.reference,
            reference: reference.into(),
        });
    }
    Ok(AnnotationSummary {
        sheet,
        instance_path: path,
        symbol_count: recorded.symbol_count,
        changes,
    })
}
