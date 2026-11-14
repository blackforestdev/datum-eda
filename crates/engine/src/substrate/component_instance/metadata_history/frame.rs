//! Exact accepted-model plus actually applied metadata operations.
use super::*;
use crate::substrate::component_instance::pool_history::PoolHistoryBasis;

pub(super) fn announces_metadata(operations: &[Operation]) -> bool {
    operations.iter().any(|op| matches!(op,
        Operation::SetProposalMetadata { proposal, .. }
        if matches!(proposal["batch"]["provenance"]["reason"].as_str(),
            Some(crate::api::native_write::schematic_field_batch::REASON
                | crate::api::native_write::schematic_board_transfer::field_revision::REASON))))
        && operations.iter().any(|op| matches!(op,
            Operation::SetComponentInstance { previous_component_instance, .. }
                if previous_component_instance["placed_package_refs"].as_array().is_some_and(|r| r.len() == 1)))
}

/// Return a proof only for this metadata family. An announced but invalid frame
/// is an error, never permission to fall back to a less restrictive reader.
fn prove(
    basis: &PoolHistoryBasis<'_>,
    staged: &DesignModel,
) -> Result<Option<Transition>, EngineError> {
    if !announces_metadata(basis.operations) {
        return Ok(None);
    }
    if basis.operations != basis.applied_operations
        || staged.project.project_id != basis.model.project.project_id
        || staged.model_revision != basis.model.model_revision
        || staged.journal != basis.model.journal
        || staged.journal_cursor != basis.model.journal_cursor
        || staged.source_shards != basis.model.source_shards
    {
        return Err(invalid(
            "metadata frame is not the complete applied accepted-model frame",
        ));
    }
    let metadata = basis
        .operations
        .iter()
        .find_map(|op| match op {
            Operation::SetProposalMetadata { proposal, .. } => {
                serde_json::from_value::<Proposal>(proposal.clone()).ok()
            }
            _ => None,
        })
        .ok_or_else(|| invalid("metadata frame receipt is unavailable"))?;
    let mut batch = metadata.batch.clone();
    batch.operations = basis.operations.to_vec();
    let inverse = crate::substrate::journal::inverse_operations_for_batch(basis.model, &batch)?;
    let proof = if metadata.status == ProposalStatus::Applied {
        if let Some(proof) = compensation::pending_redo_transition(
            basis.model,
            metadata.proposal_id,
            basis.operations,
            &inverse,
        )? {
            proof
        } else {
            let authenticated = receipt(
                &basis.model.journal,
                basis.operations,
                None,
                Some(batch.batch_id),
            )?;
            if basis
                .model
                .proposals
                .get(&authenticated.proposal_id)
                .is_none_or(|p| p.status != ProposalStatus::Accepted)
            {
                return Err(invalid("accepted metadata proposal is not current"));
            }
            transition(basis.operations, &inverse, &basis.model.journal)?
        }
    } else {
        compensation::pending_transition(basis.model, basis.operations, &inverse)?
    };
    prove_candidate(basis, staged, &proof)?;
    Ok(Some(proof))
}

fn prove_candidate(
    basis: &PoolHistoryBasis<'_>,
    staged: &DesignModel,
    proof: &Transition,
) -> Result<(), EngineError> {
    if basis.operations != basis.applied_operations
        || staged.project.project_id != basis.model.project.project_id
        || staged.model_revision != basis.model.model_revision
        || staged.journal != basis.model.journal
        || staged.journal_cursor != basis.model.journal_cursor
        || staged.source_shards != basis.model.source_shards
    {
        return Err(invalid(
            "metadata candidate is not a complete accepted-model frame",
        ));
    }
    let mut objects = basis.model.objects.clone();
    let mut components = basis.model.component_instances.clone();
    let mut nets = basis.model.electrical_identities.clone();
    let mut proposals = basis.model.proposals.clone();
    for (id, (before, after)) in &proof.changes.components {
        let object = basis
            .model
            .objects
            .get(id)
            .ok_or_else(|| invalid("accepted component source object is unavailable"))?;
        let owner = basis
            .model
            .source_shards
            .iter()
            .find(|s| s.shard_id == object.source_shard_id)
            .ok_or_else(|| invalid("accepted component owner is unavailable"))?;
        let raw = basis
            .model
            .materialized_source_shard_value_by_relative_path(&owner.relative_path)?;
        if raw["component_instance"] != *before {
            return Err(invalid(
                "metadata component preimage differs from accepted raw source",
            ));
        }
        let typed: PersistedComponentInstance = serde_json::from_value(before.clone())?;
        if typed.object_revision != object.object_revision {
            return Err(invalid(
                "accepted component counter differs from its source",
            ));
        }
        for package in &typed.placed_package_refs {
            if proof.changes.references.contains_key(&package.object_id)
                || proof.changes.values.contains_key(&package.object_id)
            {
                let (raw, epoch) = super::super::placed_package_history::recorded_package(
                    &basis.model.journal,
                    package,
                )?;
                let board = basis.model.materialized_source_shard_value(
                    crate::substrate::SourceShardKind::BoardRoot,
                )?;
                if basis
                    .model
                    .objects
                    .get(&package.object_id)
                    .is_none_or(|object| object.object_revision != epoch)
                    || board["packages"][package.object_id.to_string()] != raw
                {
                    return Err(invalid(
                        "package prefix differs from the accepted source/counter",
                    ));
                }
            }
        }
        components.insert(
            *id,
            super::super::persisted_component_instance_from_value(after)?,
        );
        objects.get_mut(id).unwrap().object_revision = proof.after[id].object_revision;
    }
    for id in proof.changes.symbols.keys() {
        let object = objects
            .get_mut(id)
            .ok_or_else(|| invalid("accepted Symbol is unavailable"))?;
        object.object_revision = next(object.object_revision, 1)?;
    }
    for id in proof
        .changes
        .references
        .keys()
        .chain(proof.changes.values.keys())
    {
        let object = objects
            .get_mut(id)
            .ok_or_else(|| invalid("accepted PCB counterpart is unavailable"))?;
        object.object_revision = next(object.object_revision, 1)?;
    }
    for (id, (before, after)) in &proof.changes.nets {
        if basis.model.electrical_identities.get(id) != Some(before) {
            return Err(invalid(
                "metadata Net evidence preimage differs from accepted source",
            ));
        }
        let ElectricalIdentity::NetRelationship { logical_net, .. } = &before.identity else {
            unreachable!()
        };
        if crate::substrate::net_correspondence_status(basis.model, *logical_net)
            != crate::substrate::NetCorrespondenceStatus::Complete
        {
            return Err(invalid(
                "metadata cannot recertify an unqualified connection",
            ));
        }
        nets.insert(*id, after.clone());
        objects
            .get_mut(id)
            .ok_or_else(|| invalid("Net evidence source object is unavailable"))?
            .object_revision = after.object_revision;
    }
    // Every affected connection must participate; dropping an evidence update
    // cannot strand another Pad or silently become a partial qualification.
    for record in basis.model.electrical_identities.values() {
        if let ElectricalIdentity::NetRelationship { evidence, .. } = &record.identity
            && evidence
                .iter()
                .any(|pin| proof.after.contains_key(&pin.component_instance.object_id))
            && !proof.changes.nets.contains_key(&record.id)
        {
            return Err(invalid("metadata omits affected Net evidence"));
        }
    }
    for op in basis.operations {
        if let Operation::SetProposalMetadata {
            proposal_id,
            previous_proposal,
            proposal,
            ..
        } = op
        {
            if basis
                .model
                .proposals
                .get(proposal_id)
                .map(serde_json::to_value)
                .transpose()?
                .as_ref()
                != Some(previous_proposal)
            {
                return Err(invalid(
                    "metadata proposal preimage differs from accepted source",
                ));
            }
            proposals.insert(*proposal_id, serde_json::from_value(proposal.clone())?);
        }
    }
    // Compare every model field, including all unrelated registries. This is a
    // transient verification value, not a second model, source cache or replay.
    let mut expected = serde_json::to_value(basis.model)?;
    expected["objects"] = serde_json::to_value(objects)?;
    expected["component_instances"] = serde_json::to_value(components)?;
    expected["electrical_identities"] = serde_json::to_value(nets)?;
    expected["proposals"] = serde_json::to_value(proposals)?;
    if expected != serde_json::to_value(staged)? {
        return Err(invalid(
            "candidate differs from the exact accepted metadata operation frame",
        ));
    }
    Ok(())
}

/// Unforgeable outside this owner. It borrows the actual accepted model and
/// retains the full canonical batch, including its original revision guards.
pub(in crate::substrate) struct CanonicalMetadataPreview<'a> {
    accepted: &'a DesignModel,
    batch: crate::substrate::OperationBatch,
}

impl<'a> CanonicalMetadataPreview<'a> {
    pub(in crate::substrate) fn capture(
        model: &'a DesignModel,
        batch: &crate::substrate::OperationBatch,
    ) -> Result<Option<Self>, EngineError> {
        use crate::api::native_write::{
            WriteProvenance, schematic_board_transfer::field_revision as board,
            schematic_field_batch as fields,
        };
        if !batch.operations.iter().any(|op| matches!(op,
            Operation::SetComponentInstance { previous_component_instance, .. }
                if previous_component_instance["placed_package_refs"].as_array().is_some_and(|refs| refs.len() == 1))) {
            return Ok(None);
        }
        let mut canonical = match batch.provenance.reason.as_str() {
            fields::REASON => {
                let mut sheet = None;
                let mut edits = Vec::new();
                for op in &batch.operations {
                    if let Operation::SetSchematicSymbol {
                        sheet_id,
                        symbol_id,
                        symbol,
                    } = op
                    {
                        if sheet.replace(*sheet_id).is_some_and(|old| old != *sheet_id) {
                            return Err(invalid("metadata preview spans source sheets"));
                        }
                        let typed: crate::schematic::PlacedSymbol =
                            serde_json::from_value(symbol.clone())?;
                        edits.push(fields::FieldRowEdit {
                            symbol: *symbol_id,
                            reference: typed.reference,
                            value: typed.value,
                        });
                    }
                }
                fields::build(
                    model,
                    sheet.ok_or_else(|| invalid("metadata preview has no source sheet"))?,
                    &[],
                    &edits,
                )?
            }
            board::REASON => {
                let snapshot = crate::substrate::ElectricalSelectionSnapshot::capture(model)
                    .map_err(|error| invalid(&format!("preview source unavailable: {error:?}")))?;
                let source = snapshot
                    .schematic_source()
                    .ok_or_else(|| invalid("preview has no schematic"))?;
                let sheets: BTreeSet<_> = batch
                    .operations
                    .iter()
                    .filter_map(|op| match op {
                        Operation::GuardObjectRevision { object_id, .. }
                            if source.sheets.contains_key(object_id) =>
                        {
                            Some(*object_id)
                        }
                        _ => None,
                    })
                    .collect();
                if sheets.len() != 1 {
                    return Err(invalid("preview source sheet is ambiguous"));
                }
                let reviewed = board::preview(model, *sheets.first().unwrap(), &[])?;
                board::prepare(
                    model,
                    WriteProvenance::new(
                        batch.provenance.actor.clone(),
                        batch.provenance.source,
                        board::REASON,
                    ),
                    &reviewed,
                    batch.batch_id,
                )?
            }
            _ => return Ok(None),
        }
        .batch;
        canonical.batch_id = batch.batch_id;
        if canonical != *batch || batch.provenance.source != CommitSource::Manual {
            return Err(invalid(
                "metadata preview differs from the complete canonical builder batch",
            ));
        }
        Ok(Some(Self {
            accepted: model,
            batch: canonical,
        }))
    }

    pub(in crate::substrate) fn validate_input(
        &self,
        model: &DesignModel,
        batch: &crate::substrate::OperationBatch,
    ) -> Result<(), EngineError> {
        if *batch != self.batch
            || serde_json::to_value(model)? != serde_json::to_value(self.accepted)?
        {
            return Err(invalid("metadata preview batch or accepted model changed"));
        }
        Ok(())
    }

    pub(in crate::substrate) fn frame<'frame>(
        &'frame self,
        staged: &'frame DesignModel,
        operations: &[Operation],
    ) -> Result<MetadataFrame<'frame>, EngineError> {
        let expected: Vec<_> = self
            .batch
            .operations
            .iter()
            .filter(|op| !matches!(op, Operation::GuardObjectRevision { .. }))
            .cloned()
            .collect();
        if operations != expected {
            return Err(invalid(
                "metadata preview applied an altered or incomplete batch",
            ));
        }
        let inverse =
            crate::substrate::journal::inverse_operations_for_batch(self.accepted, &self.batch)?;
        let proof = transition(operations, &inverse, &self.accepted.journal)?;
        let basis = PoolHistoryBasis {
            model: self.accepted,
            operations,
            applied_operations: operations,
        };
        prove_candidate(&basis, staged, &proof)?;
        Ok(MetadataFrame {
            accepted: self.accepted,
            staged,
            proof,
        })
    }
}

pub(in crate::substrate) struct MetadataFrame<'a> {
    accepted: &'a DesignModel,
    staged: &'a DesignModel,
    proof: Transition,
}

impl MetadataFrame<'_> {
    pub(in crate::substrate) fn accepted(&self) -> &DesignModel {
        self.accepted
    }
    pub(in crate::substrate) fn reference(
        &self,
        pinned: &RevisionedRef,
    ) -> Result<RevisionedRef, EngineError> {
        let accepted = self.accepted.metadata_component_reference(pinned)?;
        match self.proof.after.get(&pinned.object_id) {
            Some(after) => {
                let before: PersistedComponentInstance = serde_json::from_value(
                    self.proof.changes.components[&pinned.object_id].0.clone(),
                )?;
                if before.object_revision != accepted.object_revision
                    || self
                        .staged
                        .component_instances
                        .get(&pinned.object_id)
                        .is_none_or(|ci| ci.object_revision != after.object_revision)
                {
                    return Err(invalid(
                        "prospective metadata does not continue the accepted component epoch",
                    ));
                }
                Ok(after.clone())
            }
            None => Ok(accepted),
        }
    }
}

impl<'a> PoolHistoryBasis<'a> {
    pub(in crate::substrate) fn metadata_frame(
        &self,
        staged: &'a DesignModel,
    ) -> Result<Option<MetadataFrame<'a>>, EngineError> {
        Ok(prove(self, staged)?.map(|proof| MetadataFrame {
            accepted: self.model,
            staged,
            proof,
        }))
    }
}
