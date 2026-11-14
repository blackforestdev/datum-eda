//! Authenticated value evidence for complete-component assignment proposals.
use super::*;

pub(super) fn recorded_value(
    model: &DesignModel,
    proposal: &Proposal,
    part: Uuid,
) -> Result<Option<String>, EngineError> {
    let revisions: Vec<_> = proposal
        .batch
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::GuardObjectRevision {
                object_id,
                expected_object_revision,
            } if *object_id == part => Some(*expected_object_revision),
            _ => None,
        })
        .collect();
    let revision = match revisions.as_slice() {
        [] => return Ok(None),
        [revision] => *revision,
        _ => {
            return Err(invalid(
                "Complete component Part source evidence is ambiguous",
            ));
        }
    };
    let raw = model.materialized_pool_object_revision(part, "parts", revision)?;
    let source: crate::pool::Part = serde_json::from_value(raw)?;
    if source.uuid != part
        || source.value.trim().is_empty()
        || source.default_footprint.is_none()
        || source.default_pin_pad_map.is_none()
    {
        return Err(invalid("Complete component Part source is unresolved"));
    }
    Ok(Some(source.value))
}
