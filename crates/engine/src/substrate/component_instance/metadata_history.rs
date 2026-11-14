//! Reference/value epoch continuity over authenticated canonical transactions.
//! Frozen physical sources keep their original bytes. Only a proved metadata
//! transition can relate their recorded CI epoch to the current source epoch.
use super::PersistedComponentInstance;
use crate::substrate::{
    CommitSource, DesignModel, ElectricalIdentity, ElectricalIdentityRecord, EngineError,
    ObjectRevision, Operation, Proposal, ProposalSource, ProposalStatus, RevisionedRef,
    TransactionKind, TransactionRecord,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

mod compensation;
mod frame;
pub(in crate::substrate) use frame::{CanonicalMetadataPreview, MetadataFrame};

fn invalid(message: &str) -> EngineError {
    EngineError::Validation(format!("Component field history: {message}"))
}
fn next(value: ObjectRevision, count: u64) -> Result<ObjectRevision, EngineError> {
    value
        .0
        .checked_add(count)
        .map(ObjectRevision)
        .ok_or_else(|| invalid("source revision overflow"))
}

#[derive(Default)]
struct Changes {
    symbols: BTreeMap<Uuid, (Uuid, Value)>,
    components: BTreeMap<Uuid, (Value, Value)>,
    references: BTreeMap<Uuid, String>,
    values: BTreeMap<Uuid, String>,
    nets: BTreeMap<Uuid, (ElectricalIdentityRecord, ElectricalIdentityRecord)>,
}
impl Changes {
    fn capture(operations: &[Operation]) -> Result<Self, EngineError> {
        let mut out = Self::default();
        for operation in operations {
            let duplicate = match operation {
                Operation::GuardObjectRevision { .. } | Operation::SetProposalMetadata { .. } => {
                    false
                }
                Operation::SetSchematicSymbol {
                    sheet_id,
                    symbol_id,
                    symbol,
                } => out
                    .symbols
                    .insert(*symbol_id, (*sheet_id, symbol.clone()))
                    .is_some(),
                Operation::SetComponentInstance {
                    component_instance_id,
                    previous_component_instance,
                    component_instance,
                } => out
                    .components
                    .insert(
                        *component_instance_id,
                        (
                            previous_component_instance.clone(),
                            component_instance.clone(),
                        ),
                    )
                    .is_some(),
                Operation::SetBoardPackageReference {
                    package_id,
                    reference,
                } => out
                    .references
                    .insert(*package_id, reference.clone())
                    .is_some(),
                Operation::SetBoardPackageValue { package_id, value } => {
                    out.values.insert(*package_id, value.clone()).is_some()
                }
                Operation::SetElectricalIdentity { previous, record } => {
                    if previous.id != record.id {
                        return Err(invalid("Net identity changed"));
                    }
                    out.nets
                        .insert(record.id, ((**previous).clone(), record.clone()))
                        .is_some()
                }
                _ => return Err(invalid("an unrelated operation is mixed with metadata")),
            };
            if duplicate {
                return Err(invalid("duplicate metadata source operation"));
            }
        }
        Ok(out)
    }
}

struct Transition {
    changes: Changes,
    /// Exact current component refs after this one monotonic transaction.
    after: BTreeMap<Uuid, RevisionedRef>,
}

fn field_only(before: &Value, after: &Value) -> bool {
    let mut allowed = before.clone();
    for field in ["reference", "value"] {
        if !after[field].is_string() {
            return false;
        }
        allowed[field] = after[field].clone();
    }
    allowed == *after && before != after
}

/// Validate the whole pre/post operation grammar, not a reason marker or UUID.
fn transition(
    operations: &[Operation],
    inverse: &[Operation],
    prior: &[TransactionRecord],
) -> Result<Transition, EngineError> {
    let changes = Changes::capture(operations)?;
    let old = Changes::capture(inverse)?;
    if changes.components.is_empty()
        || changes.components.keys().ne(old.components.keys())
        || changes.symbols.keys().ne(old.symbols.keys())
        || changes.references.keys().ne(old.references.keys())
        || changes.values.keys().ne(old.values.keys())
        || changes.nets.keys().ne(old.nets.keys())
        || (!changes.symbols.is_empty()
            && (!changes.references.is_empty() || !changes.values.is_empty()))
    {
        return Err(invalid(
            "metadata source/inverse family is incomplete or mixed",
        ));
    }
    for (id, (before, after)) in &changes.components {
        if old.components.get(id) != Some(&(after.clone(), before.clone())) {
            return Err(invalid("component inverse preimage differs"));
        }
    }
    for (id, (sheet, payload)) in &changes.symbols {
        let (old_sheet, before) = &old.symbols[id];
        if sheet != old_sheet || !field_only(before, payload) {
            return Err(invalid("Symbol changes more than reference/value"));
        }
    }
    for (id, (before, after)) in &changes.nets {
        if old.nets.get(id) != Some(&(after.clone(), before.clone())) {
            return Err(invalid("Net evidence inverse does not match"));
        }
    }
    let mut baselines = BTreeMap::new();
    for (previous, _) in changes.components.values() {
        let ci: PersistedComponentInstance = serde_json::from_value(previous.clone())?;
        for package in &ci.placed_package_refs {
            if changes.references.contains_key(&package.object_id)
                || changes.values.contains_key(&package.object_id)
            {
                let (_, epoch) = super::placed_package_history::recorded_package(prior, package)?;
                baselines.insert(package.object_id, epoch);
            }
        }
    }
    let after = metadata_component_changes(&changes, &baselines)?;
    Ok(Transition { changes, after })
}

fn metadata_component_changes(
    changes: &Changes,
    package_baselines: &BTreeMap<Uuid, ObjectRevision>,
) -> Result<BTreeMap<Uuid, RevisionedRef>, EngineError> {
    if changes.components.is_empty() {
        return Err(invalid("metadata component source is missing"));
    }
    let mut symbols = BTreeSet::new();
    let mut packages = BTreeSet::new();
    let mut after = BTreeMap::new();
    for (id, (previous, updated)) in &changes.components {
        let ci: PersistedComponentInstance = serde_json::from_value(previous.clone())?;
        let ([symbol], [package]) = (
            ci.placed_symbol_refs.as_slice(),
            ci.placed_package_refs.as_slice(),
        ) else {
            return Err(invalid("metadata requires one-to-one placed members"));
        };
        if ci.uuid != *id
            || !symbols.insert(symbol.object_id)
            || !packages.insert(package.object_id)
        {
            return Err(invalid("component ownership or inverse preimage differs"));
        }
        let mut expected = previous.clone();
        let epoch = next(ci.object_revision, 1)?;
        expected["object_revision"] = serde_json::to_value(epoch)?;
        if let Some((_, payload)) = changes.symbols.get(&symbol.object_id) {
            if payload["uuid"] != serde_json::json!(symbol.object_id) {
                return Err(invalid("Symbol identity differs from its component member"));
            }
            expected["placed_symbol_refs"][0]["object_revision"] =
                serde_json::to_value(next(symbol.object_revision, 1)?)?;
        } else {
            let count = u64::from(changes.references.contains_key(&package.object_id))
                + u64::from(changes.values.contains_key(&package.object_id));
            if count == 0 {
                return Err(invalid("component epoch advanced without a field edit"));
            }
            expected["placed_package_refs"][0]["object_revision"] = serde_json::to_value(next(
                *package_baselines
                    .get(&package.object_id)
                    .ok_or_else(|| invalid("package field revision has no qualified baseline"))?,
                count,
            )?)?;
        }
        if expected != *updated {
            return Err(invalid(
                "component changes physical binding, membership or unrelated raw fields",
            ));
        }
        after.insert(
            *id,
            RevisionedRef {
                object_id: *id,
                object_revision: epoch,
            },
        );
    }
    if changes.symbols.keys().any(|id| !symbols.contains(id))
        || changes
            .references
            .keys()
            .chain(changes.values.keys())
            .any(|id| !packages.contains(id))
    {
        return Err(invalid("field edit targets an unowned member"));
    }
    for (previous, updated) in changes.nets.values() {
        let mut expected = previous.clone();
        expected.object_revision = next(previous.object_revision, 1)?;
        let ElectricalIdentity::NetRelationship { evidence, .. } = &mut expected.identity else {
            return Err(invalid(
                "metadata cannot change logical identity declarations",
            ));
        };
        let mut touched = false;
        for pin in evidence {
            if let Some(reference) = after.get(&pin.component_instance.object_id) {
                let before: PersistedComponentInstance =
                    serde_json::from_value(changes.components[&reference.object_id].0.clone())?;
                if pin.component_instance.object_revision != before.object_revision {
                    return Err(invalid("existing Net evidence is stale"));
                }
                pin.component_instance = reference.clone();
                touched = true;
            }
        }
        if !touched || expected != *updated {
            return Err(invalid(
                "Net changes identity, membership, mapping or non-CI evidence",
            ));
        }
    }
    Ok(after)
}

fn nonmetadata<'a>(operations: &'a [Operation], proposal_id: Uuid) -> Vec<&'a Operation> {
    operations.iter().filter(|op| !matches!(op, Operation::GuardObjectRevision { .. })
        && !matches!(op, Operation::SetProposalMetadata { proposal_id: id, .. } if *id == proposal_id))
        .collect()
}

fn latest_proposal(prior: &[TransactionRecord], id: Uuid) -> Option<Value> {
    let mut current = None;
    for op in prior.iter().flat_map(|tx| &tx.operations) {
        match op {
            Operation::CreateProposalMetadata {
                proposal_id,
                proposal,
                ..
            }
            | Operation::SetProposalMetadata {
                proposal_id,
                proposal,
                ..
            } if *proposal_id == id => current = Some(proposal.clone()),
            Operation::DeleteProposalMetadata { proposal_id, .. } if *proposal_id == id => {
                current = None
            }
            _ => {}
        }
    }
    current
}

/// Authenticate the immutable Draft -> Accepted -> Applied receipt and its exact
/// operation batch. A copied reason, fake inverse, or unrelated metadata cannot
/// qualify a component epoch.
fn receipt(
    prior: &[TransactionRecord],
    operations: &[Operation],
    transaction_id: Option<Uuid>,
    batch_id: Option<Uuid>,
) -> Result<Proposal, EngineError> {
    let metadata: Vec<_> = operations
        .iter()
        .filter_map(|op| match op {
            Operation::SetProposalMetadata {
                proposal_id,
                previous_proposal,
                proposal,
                ..
            } => Some((*proposal_id, previous_proposal, proposal)),
            _ => None,
        })
        .collect();
    let [(id, previous, applied)] = metadata.as_slice() else {
        return Err(invalid("one actual applied metadata receipt is required"));
    };
    let before: Proposal = serde_json::from_value((*previous).clone())?;
    let proposal: Proposal = serde_json::from_value((*applied).clone())?;
    let reason = proposal.batch.provenance.reason.as_str();
    if proposal.proposal_id != *id
        || before.status != ProposalStatus::Accepted
        || proposal.status != ProposalStatus::Applied
        || proposal.source != ProposalSource::Manual
        || proposal.batch.provenance.source != CommitSource::Manual
        || proposal.batch.batch_id != *id
        || batch_id.is_some_and(|id| id != proposal.batch.batch_id)
        || transaction_id.is_some_and(|id| proposal.applied_transaction_id != Some(id))
        || proposal.applied_transaction_id.is_none()
        || !matches!(
            reason,
            crate::api::native_write::schematic_field_batch::REASON
                | crate::api::native_write::schematic_board_transfer::field_revision::REASON
        )
        || nonmetadata(operations, *id) != nonmetadata(&proposal.batch.operations, *id)
        || latest_proposal(prior, *id).as_ref() != Some(*previous)
    {
        return Err(invalid(
            "metadata receipt provenance, identity or accepted source differs",
        ));
    }
    let mut expected = before.clone();
    expected.status = ProposalStatus::Applied;
    expected.applied_transaction_id = proposal.applied_transaction_id;
    if expected != proposal {
        return Err(invalid("applied receipt changes its accepted batch"));
    }
    let drafts: Vec<_> = prior
        .iter()
        .flat_map(|tx| &tx.operations)
        .filter_map(|op| match op {
            Operation::CreateProposalMetadata {
                proposal_id,
                proposal,
                ..
            } if proposal_id == id => serde_json::from_value::<Proposal>(proposal.clone()).ok(),
            _ => None,
        })
        .collect();
    let [draft] = drafts.as_slice() else {
        return Err(invalid("unique recorded Draft origin is unavailable"));
    };
    let mut accepted = draft.clone();
    if accepted.status != ProposalStatus::Draft || accepted.applied_transaction_id.is_some() {
        return Err(invalid("metadata proposal did not originate as a Draft"));
    }
    accepted.status = ProposalStatus::Accepted;
    if accepted != before {
        return Err(invalid("accepted receipt differs from its Draft"));
    }
    Ok(proposal)
}

fn normal_transition(
    tx: &TransactionRecord,
    prior: &[TransactionRecord],
) -> Result<Transition, EngineError> {
    let proposal = receipt(
        prior,
        &tx.operations,
        Some(tx.transaction_id),
        Some(tx.batch_id),
    )?;
    if tx.transaction_kind != TransactionKind::Normal
        || tx.provenance.source != CommitSource::Manual
        || tx.provenance != proposal.batch.provenance
    {
        return Err(invalid(
            "metadata origin is not its manual canonical transaction",
        ));
    }
    transition(&tx.operations, &tx.inverse_operations, prior)
}

pub(super) use compensation::validate_package_field_transaction;

fn latest_component(prior: &[TransactionRecord], id: Uuid) -> Result<Value, EngineError> {
    let mut current = None;
    for tx in prior {
        for op in &tx.operations {
            match op {
                Operation::CreateComponentInstance {
                    component_instance_id,
                    component_instance,
                } if *component_instance_id == id => {
                    if current.is_some() {
                        return Err(invalid("component identity was reused"));
                    }
                    current = Some(component_instance.clone());
                }
                Operation::SetComponentInstance {
                    component_instance_id,
                    previous_component_instance,
                    component_instance,
                } if *component_instance_id == id => {
                    if current
                        .as_ref()
                        .is_some_and(|old| old != previous_component_instance)
                        || previous_component_instance["object_revision"]
                            .as_u64()
                            .and_then(|n| n.checked_add(1))
                            != component_instance["object_revision"].as_u64()
                    {
                        return Err(invalid(
                            "component source history is discontinuous or reset",
                        ));
                    }
                    current = Some(component_instance.clone());
                }
                Operation::DeleteComponentInstance {
                    component_instance_id,
                    ..
                } if *component_instance_id == id => {
                    return Err(invalid("component source was retired"));
                }
                _ => {}
            }
        }
    }
    current.ok_or_else(|| invalid("recorded component source is unavailable"))
}

fn resolve_journal(
    journal: &[TransactionRecord],
    pinned: &RevisionedRef,
    project_id: Uuid,
) -> Result<RevisionedRef, EngineError> {
    let mut current: Option<Value> = None;
    let mut seeded = false;
    for (index, tx) in journal.iter().enumerate() {
        let relevant: Vec<_> = tx
            .operations
            .iter()
            .filter_map(|op| match op {
                Operation::CreateComponentInstance {
                    component_instance_id,
                    component_instance,
                } if *component_instance_id == pinned.object_id => Some((None, component_instance)),
                Operation::SetComponentInstance {
                    component_instance_id,
                    previous_component_instance,
                    component_instance,
                } if *component_instance_id == pinned.object_id => {
                    Some((Some(previous_component_instance), component_instance))
                }
                Operation::DeleteComponentInstance {
                    component_instance_id,
                    ..
                } if *component_instance_id == pinned.object_id => Some((None, &Value::Null)),
                _ => None,
            })
            .collect();
        if relevant.is_empty() {
            continue;
        }
        let [(before, after)] = relevant.as_slice() else {
            return Err(invalid("ambiguous component transition"));
        };
        if after.is_null() || before.is_some_and(|old| current.as_ref().is_some_and(|v| v != old)) {
            return Err(invalid(
                "component source was retired or its preimage changed",
            ));
        }
        let typed: PersistedComponentInstance = serde_json::from_value((*after).clone())?;
        if typed.uuid != pinned.object_id {
            return Err(invalid("component source identity differs"));
        }
        if let Some(before) = before {
            let old: PersistedComponentInstance = serde_json::from_value((*before).clone())?;
            if next(old.object_revision, 1)? != typed.object_revision {
                return Err(invalid("component epoch reset or discontinuity"));
            }
        }
        if seeded {
            let proof = compensation::transaction_transition(tx, &journal[..index])?;
            if !proof.after.contains_key(&pinned.object_id)
                || compensation::origin_project(tx, &journal[..index])? != project_id
            {
                return Err(invalid(
                    "intervening component transition is not authenticated metadata",
                ));
            }
        } else if typed.object_revision == pinned.object_revision {
            seeded = true;
        }
        current = Some((*after).clone());
    }
    let raw = current.ok_or_else(|| invalid("component origin is unavailable"))?;
    if !seeded {
        return Err(invalid("frozen component epoch has no canonical origin"));
    }
    let current: PersistedComponentInstance = serde_json::from_value(raw)?;
    Ok(RevisionedRef {
        object_id: pinned.object_id,
        object_revision: current.object_revision,
    })
}

impl DesignModel {
    pub(crate) fn validate_component_metadata_transaction(
        &self,
        proposal: &Proposal,
        transaction: &TransactionRecord,
    ) -> Result<(), EngineError> {
        let index = self
            .journal
            .iter()
            .position(|tx| tx == transaction)
            .ok_or_else(|| {
                invalid("applied metadata transaction differs from canonical history")
            })?;
        normal_transition(transaction, &self.journal[..index])?;
        let recorded = receipt(
            &self.journal[..index],
            &transaction.operations,
            Some(transaction.transaction_id),
            Some(transaction.batch_id),
        )?;
        if recorded.proposal_id != proposal.proposal_id
            || recorded.batch != proposal.batch
            || recorded.rationale != proposal.rationale
            || recorded.project_id != self.project.project_id
        {
            return Err(invalid(
                "applied metadata transaction has a different receipt",
            ));
        }
        Ok(())
    }

    /// Static receipt grammar only. Actual admission also requires the accepted
    /// source/preimage frame and the canonical inverse, never this check alone.
    pub(crate) fn validate_component_metadata_proposal(
        proposal: &Proposal,
    ) -> Result<(), EngineError> {
        let changes = Changes::capture(&proposal.batch.operations)?;
        let fields = proposal.batch.provenance.reason.as_str()
            == crate::api::native_write::schematic_field_batch::REASON;
        let board = proposal.batch.provenance.reason.as_str()
            == crate::api::native_write::schematic_board_transfer::field_revision::REASON;
        if proposal.source != ProposalSource::Manual
            || proposal.batch.provenance.source != CommitSource::Manual
            || proposal.batch.batch_id != proposal.proposal_id
            || !(fields
                && !changes.symbols.is_empty()
                && changes.references.is_empty()
                && changes.values.is_empty()
                || board
                    && changes.symbols.is_empty()
                    && (!changes.references.is_empty() || !changes.values.is_empty()))
        {
            return Err(invalid(
                "metadata receipt purpose or manual source is invalid",
            ));
        }
        let mut guards = BTreeMap::new();
        for op in &proposal.batch.operations {
            if let Operation::GuardObjectRevision {
                object_id,
                expected_object_revision,
            } = op
            {
                if guards
                    .insert(*object_id, *expected_object_revision)
                    .is_some()
                {
                    return Err(invalid("metadata receipt has duplicate guards"));
                }
            }
        }
        for (id, (before, _)) in &changes.components {
            let source: PersistedComponentInstance = serde_json::from_value(before.clone())?;
            if guards.get(id) != Some(&source.object_revision) {
                return Err(invalid("metadata receipt lacks its component source guard"));
            }
            for member in &source.placed_symbol_refs {
                if fields && guards.get(&member.object_id) != Some(&member.object_revision) {
                    return Err(invalid(
                        "metadata receipt lacks its changed Symbol source guard",
                    ));
                }
            }
            for member in &source.placed_package_refs {
                if board && !guards.contains_key(&member.object_id) {
                    return Err(invalid(
                        "metadata receipt lacks its changed PCB source guard",
                    ));
                }
            }
        }
        // Structural receipt validation uses its declared guards. Current apply
        // and history separately derive the baseline from the real prefix.
        metadata_component_changes(&changes, &guards)?;
        if changes
            .nets
            .iter()
            .any(|(id, (before, _))| guards.get(id) != Some(&before.object_revision))
        {
            return Err(invalid(
                "metadata receipt lacks its Net evidence source guard",
            ));
        }
        Ok(())
    }

    pub(in crate::substrate) fn prepare_component_metadata_inverse(
        &self,
        target: &TransactionRecord,
        batch: &mut crate::substrate::OperationBatch,
    ) -> Result<bool, EngineError> {
        compensation::prepare(self, target, batch)
    }

    pub(in crate::substrate) fn component_metadata_inverse_matches(
        tx: &TransactionRecord,
        target: &TransactionRecord,
        prior: &[TransactionRecord],
    ) -> bool {
        compensation::matches(tx, target, prior)
    }

    pub(crate) fn metadata_component_reference(
        &self,
        pinned: &RevisionedRef,
    ) -> Result<RevisionedRef, EngineError> {
        if self.journal_cursor.applied_transaction_count != self.journal.len() {
            return Err(invalid("journal cursor differs from the accepted source"));
        }
        let current = resolve_journal(&self.journal, pinned, self.project.project_id)?;
        let object = self
            .objects
            .get(&pinned.object_id)
            .ok_or_else(|| invalid("current component object is unavailable"))?;
        let owner = self
            .source_shards
            .iter()
            .find(|s| s.shard_id == object.source_shard_id)
            .ok_or_else(|| invalid("current component source owner is unavailable"))?;
        let raw = self.materialized_source_shard_value_by_relative_path(&owner.relative_path)?;
        if object.object_revision != current.object_revision
            || raw["component_instance"] != latest_component(&self.journal, pinned.object_id)?
            || self
                .component_instances
                .get(&pinned.object_id)
                .is_none_or(|ci| ci.object_revision != current.object_revision)
        {
            return Err(invalid(
                "current component source differs from canonical metadata history",
            ));
        }
        Ok(current)
    }
}
