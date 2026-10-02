#[path = "electrical_selection_query.rs"]
mod query;
pub use query::{
    ElectricalMembership, ElectricalQueryFailure, ElectricalSelectionSnapshot, SelectionSourceBasis,
};
// Internal current/final occurrence authority shared by writes and diagnostics.
use super::{DesignModel, ElectricalIdentity, ElectricalOccurrence, EngineError, Operation};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub(super) fn index(
    model: &DesignModel,
    operations: &[Operation],
) -> Result<BTreeMap<Uuid, BTreeSet<ElectricalOccurrence>>, EngineError> {
    if !model.electrical_identities.values().any(|r| {
        matches!(&r.identity,
        ElectricalIdentity::Net { anchor, retired: false, .. }
            if super::electrical_transaction::schematic_class(&anchor.class))
    }) {
        return Ok(BTreeMap::new());
    }
    let groups = super::electrical_topology_source::partitions(model, operations)?;
    Ok(super::electrical_transaction::previous(model, &groups)?
        .into_iter()
        .map(|group| (group.net_id, group.members))
        .collect())
}
