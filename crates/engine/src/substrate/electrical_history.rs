//! Normal writes cannot recycle retired semantic identity. Undo/redo uses the
//! exact recorded operations and is explicitly distinguished by the journal owner.
use super::{DesignModel, ElectricalIdentity, EngineError, Operation, TransactionKind};

pub(super) fn validate(
    model: &DesignModel,
    operations: &[Operation],
    kind: TransactionKind,
) -> Result<(), EngineError> {
    if kind != TransactionKind::Normal {
        return Ok(());
    }
    for op in operations {
        match op {
            Operation::CreateElectricalIdentity { record } => {
                if record.object_revision.0 != 0
                    || model
                        .journal
                        .iter()
                        .flat_map(|t| &t.operations)
                        .filter_map(super::electrical_identity_store::write)
                        .any(|(id, _, _)| id == record.id)
                {
                    return Err(invalid("new identity reuses history or nonzero revision"));
                }
            }
            Operation::SetElectricalIdentity { previous, record } => {
                if previous.object_revision.0.checked_add(1) != Some(record.object_revision.0) {
                    return Err(invalid("normal update must advance revision once"));
                }
                if retired(&previous.identity) && !retired(&record.identity) {
                    return Err(invalid(
                        "retired identity cannot be revived by normal write",
                    ));
                }
            }
            Operation::DeleteElectricalIdentity { record }
                if matches!(
                    record.identity,
                    ElectricalIdentity::Net { .. } | ElectricalIdentity::Bus { .. }
                ) =>
            {
                return Err(invalid(
                    "semantic deletion must retain retirement provenance",
                ));
            }
            _ => {}
        }
    }
    Ok(())
}
fn retired(identity: &ElectricalIdentity) -> bool {
    match identity {
        ElectricalIdentity::Net { retired, .. } | ElectricalIdentity::Bus { retired, .. } => {
            *retired
        }
        _ => false,
    }
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("electrical identity history refused: {reason}"))
}
