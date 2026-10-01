//! A source drawing's deletion removes its projections, not semantic Bus identity.
use super::*;
use uuid::Uuid;

pub(in crate::api::native_write) fn projection_removals(
    model: &DesignModel,
    source_id: Uuid,
    class: &str,
) -> Result<Vec<Operation>, EngineError> {
    let mut operations = Vec::new();
    for previous in model.electrical_identities.values() {
        let ElectricalIdentity::Bus {
            representations,
            retired: false,
            ..
        } = &previous.identity
        else {
            continue;
        };
        if !representations
            .iter()
            .any(|r| r.source_id == source_id && r.class == class)
        {
            continue;
        }
        let mut record = previous.clone();
        let ElectricalIdentity::Bus {
            representations, ..
        } = &mut record.identity
        else {
            unreachable!()
        };
        representations.retain(|r| r.source_id != source_id || r.class != class);
        record.object_revision = ObjectRevision(next_revision(previous.object_revision)?);
        operations.push(Operation::SetElectricalIdentity {
            previous: Box::new(previous.clone()),
            record,
        });
    }
    // Final-state validation refuses any remaining interface dependency. Its
    // declared semantic intent must be explicitly rebound/removed by the caller.
    Ok(operations)
}
