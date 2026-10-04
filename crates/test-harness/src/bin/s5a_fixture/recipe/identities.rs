//! Existing PM053 board-only identity and relationship authoring, not a private resolver.
use super::*;
use eda_engine::substrate::{
    DesignModel, ElectricalIdentity, ElectricalIdentityRecord, NetAnchorReason,
    NetRelationshipIntent, ObjectRevision, RevisionedRef,
};
pub(super) fn bind_board_net(
    model: &mut DesignModel,
    root: &Path,
    board_net: Uuid,
    anchor: ElectricalOccurrence,
) -> Result<Uuid> {
    let logical = id(0x60000000, 1);
    let relationship = id(0x60000000, 2);
    let board_ref = RevisionedRef {
        object_id: board_net,
        object_revision: model
            .objects
            .get(&board_net)
            .context("persisted board Net identity")?
            .object_revision,
    };
    let records = [
        ElectricalIdentityRecord {
            id: logical,
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::Net {
                anchor,
                anchor_reason: NetAnchorReason::Authored,
                retired: false,
                predecessors: BTreeSet::new(),
            },
        },
        ElectricalIdentityRecord {
            id: relationship,
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::NetRelationship {
                logical_net: logical,
                board_net: Some(board_ref),
                intent: NetRelationshipIntent::BoardOnly,
                evidence: vec![],
            },
        },
    ];
    let write = BatchComposer::compose(model, provenance())
        .push_ops(
            records
                .into_iter()
                .map(|record| Operation::CreateElectricalIdentity { record }),
        )
        .finish()?;
    native_write::commit_prepared(model, root, write)?;
    Ok(logical)
}
