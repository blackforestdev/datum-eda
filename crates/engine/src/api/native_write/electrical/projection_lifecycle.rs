//! A source drawing's deletion removes its projections, not semantic Bus identity.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub(in crate::api::native_write) fn projection_removals(
    model: &DesignModel,
    source_id: Uuid,
    class: &str,
) -> Result<Vec<Operation>, EngineError> {
    projection_removals_for_sources(model, &BTreeSet::from([(class.to_string(), source_id)]))
}

fn projection_removals_for_sources(
    model: &DesignModel,
    sources: &BTreeSet<(String, Uuid)>,
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
            .any(|r| sources.contains(&(r.class.clone(), r.source_id)))
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
        representations.retain(|r| !sources.contains(&(r.class.clone(), r.source_id)));
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

/// Combine lifecycle updates from constituent source builders against one preimage.
/// Only removal of the declared deleted projections is admitted; other lifecycle
/// edits, missing removals, conflicting preimages or revision changes refuse.
pub(in crate::api::native_write) fn merge_projection_removals(
    model: &DesignModel,
    sources: &BTreeSet<(String, Uuid)>,
    emitted: Vec<Operation>,
) -> Result<Vec<Operation>, EngineError> {
    let invalid = || {
        EngineError::Validation(
            "Deletion projection lifecycle does not match the complete source set".into(),
        )
    };
    let expected = projection_removals_for_sources(model, sources)?;
    let mut merged = BTreeMap::new();
    for op in emitted {
        let Operation::SetElectricalIdentity { previous, record } = op else {
            return Err(invalid());
        };
        let Some(Operation::SetElectricalIdentity {
            previous: expected_previous,
            record: expected_record,
        }) = expected.iter().find(
            |op| matches!(op,Operation::SetElectricalIdentity{record:r,..} if r.id==record.id),
        )
        else {
            return Err(invalid());
        };
        if &previous != expected_previous {
            return Err(invalid());
        }
        let ElectricalIdentity::Bus {
            representations: old,
            ..
        } = &previous.identity
        else {
            return Err(invalid());
        };
        let ElectricalIdentity::Bus {
            representations: next,
            ..
        } = &record.identity
        else {
            return Err(invalid());
        };
        let ElectricalIdentity::Bus {
            representations: finals,
            ..
        } = &expected_record.identity
        else {
            return Err(invalid());
        };
        if next == old || !next.is_subset(old) || !finals.is_subset(next) {
            return Err(invalid());
        }
        let mut normalized = record.clone();
        let ElectricalIdentity::Bus {
            representations, ..
        } = &mut normalized.identity
        else {
            unreachable!()
        };
        *representations = finals.clone();
        if &normalized != expected_record {
            return Err(invalid());
        }
        let merged_record = merged.entry(record.id).or_insert_with(|| record.clone());
        let ElectricalIdentity::Bus {
            representations, ..
        } = &mut merged_record.identity
        else {
            unreachable!()
        };
        representations.retain(|r| next.contains(r));
    }
    for op in &expected {
        let Operation::SetElectricalIdentity { record, .. } = op else {
            unreachable!()
        };
        if merged.get(&record.id) != Some(record) {
            return Err(invalid());
        }
    }
    Ok(expected)
}
