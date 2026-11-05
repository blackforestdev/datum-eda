//! Project annotation receipts preserve every changed sheet and reference preimage.
use super::super::schematic_symbol_edit_receipt::{self as shared, Recorded};
use super::*;
use crate::schematic::PlacedSymbol;
use serde_json::json;
use std::collections::BTreeMap;

fn recorded(proposal: &Proposal) -> Result<Recorded<'_>, EngineError> {
    if !is_project_annotation(proposal) {
        return Err(invalid("Project annotation Proposal family unavailable"));
    }
    let recorded = shared::recorded(proposal)?;
    for payload in recorded.symbols.values() {
        let symbol: PlacedSymbol = serde_json::from_value((*payload).clone())?;
        super::super::schematic_annotation::prefix(&symbol.reference)?;
    }
    Ok(recorded)
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
) -> Result<ProjectAnnotationSummary, EngineError> {
    let recorded = recorded(proposal)?;
    if current {
        if proposal.prepared_against != model.model_revision
            || proposal.batch.expected_model_revision.as_ref() != Some(&model.model_revision)
        {
            return Err(invalid("Design changed; review project annotation again"));
        }
        if build(model)?.batch.operations != proposal.batch.operations {
            return Err(invalid(
                "Project annotation differs from the complete current census",
            ));
        }
    }
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Project annotation sheet names unavailable: {e:?}")))?;
    let source = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("Project schematic unavailable"))?;
    let mut sheets: BTreeMap<_, _> = source
        .sheets
        .values()
        .map(|s| {
            (
                s.uuid,
                ProjectAnnotationSheet {
                    sheet: s.uuid,
                    name: s.name.clone(),
                    symbol_count: s.symbols.len(),
                    changes: vec![],
                },
            )
        })
        .collect();
    if sheets.values().map(|s| s.symbol_count).sum::<usize>() != recorded.symbol_count {
        return Err(invalid("Project annotation complete Symbol census changed"));
    }
    let old_payloads = shared::old_payloads(model, proposal, &recorded)?;
    for (id, next) in recorded.symbols {
        let old = old_payloads
            .get(&id)
            .ok_or_else(|| invalid("Project annotation preimage unavailable"))?;
        let before: PlacedSymbol = serde_json::from_value(old.clone())?;
        let after: PlacedSymbol = serde_json::from_value(next.clone())?;
        let mut expected = old.clone();
        expected["reference"] = json!(after.reference);
        if before.uuid != id
            || after.uuid != id
            || before.reference == after.reference
            || expected != *next
        {
            return Err(invalid(
                "Project annotation changes unrelated Symbol source",
            ));
        }
        sheets
            .get_mut(&recorded.sheets[&id])
            .ok_or_else(|| invalid("Project annotation sheet disappeared"))?
            .changes
            .push(AnnotationChange {
                symbol: id,
                previous_reference: before.reference,
                reference: after.reference,
            });
    }
    Ok(ProjectAnnotationSummary {
        sheets: sheets.into_values().collect(),
    })
}
