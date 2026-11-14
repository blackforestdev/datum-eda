//! Explicit correspondence revision between existing stable Nets.
//!
//! Copper and physical library choices are never rewritten by this family.
use crate::{
    api::native_write::{BatchComposer, PreparedWrite, WriteProvenance},
    connectivity::PhysicalQueryFailure,
    error::EngineError,
    substrate::{
        CommitSource, DesignModel, ElectricalIdentity, ElectricalIdentityRecord,
        ElectricalOccurrence, ElectricalSelectionSnapshot, ModelRevision, NetCorrespondence,
        NetRelationshipIntent, ObjectRevision, Operation, Proposal, ProposalSource,
        SourceShardKind, ZoneFillState,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

mod history;
mod physical;

pub(crate) const REASON: &str = "Review schematic pin and Net correspondence changes on the PCB";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PadNetChange {
    pub component_instance: Uuid,
    pub terminal: ElectricalOccurrence,
    pub pad: Uuid,
    pub previous_logical_net: Uuid,
    pub logical_net: Uuid,
    pub previous_board_net: Uuid,
    pub board_net: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetainedCopperKind {
    Track,
    Via,
    Zone,
}

/// Source identities to inspect/rework; not a list of geometry edits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedCopper {
    pub source_id: Uuid,
    pub kind: RetainedCopperKind,
    pub net: Option<Uuid>,
    pub impact: CopperImpact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CopperImpact {
    QualifiedNetChange,
    ForeignContact,
    ConservativeUnknownContacts,
    FillBasisInvalidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentView {
    Previous,
    Proposed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactReview {
    pub board_net: Uuid,
    pub assignment: AssignmentView,
    pub unavailable: Option<PhysicalQueryFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneFillReview {
    pub source_id: Uuid,
    pub source_revision: ObjectRevision,
    pub fill_source_shard: Option<Uuid>,
    pub fill_source_hash: Option<String>,
    pub current: bool,
    pub recorded_state: Option<ZoneFillState>,
    pub recorded_model_revision: Option<ModelRevision>,
    pub recorded_zone_revision: Option<ObjectRevision>,
    pub invalidated_by_revision: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationshipChange {
    pub previous: ElectricalIdentityRecord,
    pub record: ElectricalIdentityRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardConnectivityRevision {
    pub schema_version: u32,
    pub project_id: Uuid,
    pub sheet: Uuid,
    pub board: Uuid,
    pub model_revision: ModelRevision,
    pub changes: Vec<PadNetChange>,
    pub relationships: Vec<RelationshipChange>,
    pub retained_copper: Vec<RetainedCopper>,
    pub contact_basis: Vec<ContactReview>,
    pub zone_fills: Vec<ZoneFillReview>,
    pub source_revisions: BTreeMap<Uuid, ObjectRevision>,
}

fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(format!("PCB connectivity revision: {}", reason.into()))
}

fn guard(
    model: &DesignModel,
    guards: &mut BTreeMap<Uuid, ObjectRevision>,
    id: Uuid,
) -> Result<(), EngineError> {
    let object = model
        .objects
        .get(&id)
        .ok_or_else(|| invalid("Missing source object"))?;
    guards.insert(id, object.object_revision);
    Ok(())
}

/// Read both projections from one immutable model. Pending correspondence is
/// historical evidence, never a complete Global Net or a source of new mapping.
pub fn preview(model: &DesignModel, sheet: Uuid) -> Result<BoardConnectivityRevision, EngineError> {
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Unavailable electrical source: {e:?}")))?;
    let schematic = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("Native schematic required"))?;
    super::super::schematic_edit::require_unique_occurrence(schematic, sheet, &[])?;
    let schematic_sheet = schematic
        .sheets
        .get(&sheet)
        .ok_or_else(|| invalid("Missing sheet"))?;
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let mut copper = snapshot
        .board_source()
        .map_err(|e| invalid(format!("Unavailable PCB source: {e:?}")))?;
    let mut review = BoardConnectivityRevision {
        schema_version: 1,
        project_id: model.project.project_id,
        sheet,
        board: copper.uuid,
        model_revision: model.model_revision.clone(),
        changes: vec![],
        relationships: vec![],
        retained_copper: vec![],
        contact_basis: vec![],
        zone_fills: vec![],
        source_revisions: BTreeMap::new(),
    };
    let guards = &mut review.source_revisions;
    guard(model, guards, sheet)?;
    guard(model, guards, copper.uuid)?;
    let physical::QualifiedPhysical {
        sources: physical,
        terminals: expected_terminals,
    } = physical::qualify(model, schematic_sheet, &board, copper.uuid, guards)?;
    let mut logical_owners = BTreeMap::new();
    let mut board_owners = BTreeSet::new();
    let mut evidence = BTreeMap::new();
    let mut terminals = BTreeSet::new();
    let mut records = Vec::new();
    for record in model.electrical_identities.values() {
        let ElectricalIdentity::NetRelationship {
            logical_net,
            board_net,
            intent,
            evidence: pins,
        } = &record.identity
        else {
            continue;
        };
        let endpoint = board_net
            .as_ref()
            .ok_or_else(|| invalid("Correspondence lacks an existing board Net"))?;
        if !matches!(
            intent,
            NetRelationshipIntent::Implemented
                | NetRelationshipIntent::Pending
                | NetRelationshipIntent::Mismatch
        ) || !snapshot.net_ids().contains(logical_net)
            || !copper.nets.contains_key(&endpoint.object_id)
            || model
                .objects
                .get(&endpoint.object_id)
                .is_none_or(|o| o.object_revision != endpoint.object_revision)
            || logical_owners.insert(*logical_net, records.len()).is_some()
            || !board_owners.insert(endpoint.object_id)
            || pins.is_empty()
        {
            return Err(invalid(
                "Net birth, retirement, ambiguous endpoints or unsupported correspondence",
            ));
        }
        for id in [record.id, *logical_net, endpoint.object_id] {
            guard(model, guards, id)?;
        }
        for pin in pins {
            let (component, binding) = physical
                .get(&pin.board_pad.object_id)
                .ok_or_else(|| invalid("Evidence names an unowned Pad"))?;
            let map = binding
                .pin_pad_map
                .as_ref()
                .ok_or_else(|| invalid("Pad map missing"))?;
            let library_pin = pin
                .library_pin
                .as_ref()
                .ok_or_else(|| invalid("Explicit library Pin missing"))?;
            let placed_pin = schematic_sheet
                .symbols
                .values()
                .flat_map(|symbol| &symbol.pins)
                .find(|placed| placed.uuid == pin.schematic_terminal.source_id)
                .ok_or_else(|| invalid("Placed terminal missing"))?;
            if placed_pin.library_pin != Some(library_pin.object_id)
                || model
                    .objects
                    .get(&library_pin.object_id)
                    .is_none_or(|object| object.object_revision != library_pin.object_revision)
            {
                return Err(invalid("Library Pin identity or source revision changed"));
            }
            guard(model, guards, library_pin.object_id)?;
            if *component != pin.component_instance.object_id
                || model
                    .objects
                    .get(component)
                    .is_none_or(|o| o.object_revision != pin.component_instance.object_revision)
                || model
                    .objects
                    .get(&pin.board_pad.object_id)
                    .is_none_or(|o| o.object_revision != pin.board_pad.object_revision)
                || pin.pin_pad_map.object_id != map.object_id
                || pin.pin_pad_map.object_revision != map.object_revision
                || pin.library_pad.as_ref().is_none_or(|r| {
                    r.object_id != binding.library_pad.object_id
                        || r.object_revision != binding.library_pad.object_revision
                })
                || !expected_terminals.contains(&pin.schematic_terminal)
                || evidence
                    .insert(pin.board_pad.object_id, pin.clone())
                    .is_some()
            {
                return Err(invalid(
                    "Changed or incomplete terminal/Pad/library correspondence",
                ));
            }
            terminals.insert(pin.schematic_terminal.clone());
            if let Some(pad) = copper.pads.get_mut(&pin.board_pad.object_id) {
                pad.net = Some(endpoint.object_id);
            }
            if let Some(pad) = copper.query_pads.get_mut(&pin.board_pad.object_id) {
                pad.terminal_mut().net = Some(endpoint.object_id);
            }
        }
        records.push(record.clone());
    }
    if terminals != expected_terminals || evidence.keys().ne(physical.keys()) {
        return Err(invalid(
            "Every logical Pin and physical Pad requires complete correspondence",
        ));
    }
    let mut next_evidence: Vec<Vec<NetCorrespondence>> = vec![vec![]; records.len()];
    for pin in evidence.values() {
        let net = snapshot
            .schematic_member_net(&pin.schematic_terminal)
            .map_err(|e| invalid(format!("Unavailable current Pin membership: {e:?}")))?;
        let index = logical_owners
            .get(&net)
            .ok_or_else(|| invalid("Pin moved to a Net without an existing board counterpart"))?;
        next_evidence[*index].push(pin.clone());
        guard(model, guards, pin.schematic_terminal.source_id)?;
    }
    for source in snapshot
        .authored_sources()
        .map_err(|e| invalid(format!("Unavailable authored source: {e:?}")))?
    {
        if source.occurrence.class == "pins" && !expected_terminals.contains(&source.occurrence) {
            return Err(invalid(
                "Schematic includes unsupported or untransferred terminals",
            ));
        }
    }
    for (previous, pins) in records.into_iter().zip(next_evidence) {
        if pins.is_empty() {
            return Err(invalid(
                "Revision would empty an existing board correspondence",
            ));
        }
        let mut record = previous.clone();
        let ElectricalIdentity::NetRelationship {
            intent, evidence, ..
        } = &mut record.identity
        else {
            unreachable!()
        };
        let previous_ids: BTreeSet<_> =
            evidence.iter().map(|pin| pin.board_pad.object_id).collect();
        let next_ids: BTreeSet<_> = pins.iter().map(|pin| pin.board_pad.object_id).collect();
        // Do not reorder untouched evidence or clear unrelated Pending intent.
        if previous_ids == next_ids {
            continue;
        }
        let mut ordered: Vec<_> = evidence
            .iter()
            .filter(|pin| next_ids.contains(&pin.board_pad.object_id))
            .cloned()
            .collect();
        ordered.extend(
            pins.into_iter()
                .filter(|pin| !previous_ids.contains(&pin.board_pad.object_id)),
        );
        *intent = NetRelationshipIntent::Implemented;
        *evidence = ordered;
        record.object_revision = ObjectRevision(
            previous
                .object_revision
                .0
                .checked_add(1)
                .ok_or_else(|| invalid("Relationship revision overflow"))?,
        );
        review
            .relationships
            .push(RelationshipChange { previous, record });
    }
    review.changes = assignment_changes(&review)?;
    physical::impact(model, &snapshot, &copper, &mut review)?;
    Ok(review)
}

pub(crate) fn prepare(
    model: &DesignModel,
    provenance: WriteProvenance,
    review: &BoardConnectivityRevision,
    batch_id: Uuid,
) -> Result<PreparedWrite, EngineError> {
    if preview(model, review.sheet)? != *review {
        return Err(invalid(
            "Reviewed source changed; reopen the connectivity review",
        ));
    }
    let mut write = BatchComposer::compose(model, provenance)
        .push_ops(operations(review)?)
        .finish()?;
    write.batch.batch_id = batch_id;
    Ok(write)
}

type PadAssignments = BTreeMap<Uuid, (Uuid, Uuid, NetCorrespondence)>;

/// Check the *complete* old/new evidence sets. A pin-map edit, source rebind,
/// missing terminal or board-Net retarget cannot hide inside a Net change.
fn assignment_changes(
    review: &BoardConnectivityRevision,
) -> Result<Vec<PadNetChange>, EngineError> {
    let mut before = PadAssignments::new();
    let mut after = PadAssignments::new();
    let mut relationships = BTreeSet::new();
    let mut logical_nets = BTreeSet::new();
    let mut board_nets = BTreeSet::new();
    for row in &review.relationships {
        let ElectricalIdentity::NetRelationship {
            logical_net: old_logical,
            board_net: Some(old_board),
            intent: old_intent,
            evidence: old_evidence,
        } = &row.previous.identity
        else {
            return Err(invalid(
                "Previous correspondence is not an existing board relationship",
            ));
        };
        let ElectricalIdentity::NetRelationship {
            logical_net,
            board_net: Some(board_net),
            intent: NetRelationshipIntent::Implemented,
            evidence,
        } = &row.record.identity
        else {
            return Err(invalid(
                "Revised correspondence is not an implemented board relationship",
            ));
        };
        if row.previous.id != row.record.id
            || row.previous.object_revision.0.checked_add(1) != Some(row.record.object_revision.0)
            || old_logical != logical_net
            || old_board != board_net
            || !matches!(
                old_intent,
                NetRelationshipIntent::Implemented
                    | NetRelationshipIntent::Pending
                    | NetRelationshipIntent::Mismatch
            )
            || !relationships.insert(row.previous.id)
            || !logical_nets.insert(*logical_net)
            || !board_nets.insert(board_net.object_id)
            || review.source_revisions.get(&row.previous.id) != Some(&row.previous.object_revision)
            || old_evidence.is_empty()
            || evidence.is_empty()
        {
            return Err(invalid(
                "A relationship changes identity, endpoints, ownership or revision",
            ));
        }
        for (items, destination) in [(old_evidence, &mut before), (evidence, &mut after)] {
            for pin in items {
                if pin.schematic_terminal.class != "pins"
                    || !pin.schematic_terminal.instance_path.is_empty()
                    || destination
                        .insert(
                            pin.board_pad.object_id,
                            (*logical_net, board_net.object_id, pin.clone()),
                        )
                        .is_some()
                {
                    return Err(invalid(
                        "Duplicate Pad ownership or unsupported terminal occurrence",
                    ));
                }
            }
        }
    }
    if before.keys().ne(after.keys()) {
        return Err(invalid(
            "Revision adds or removes an existing pin/Pad correspondence",
        ));
    }
    let mut changes = Vec::new();
    for (pad, (previous_logical_net, previous_board_net, previous)) in before {
        let (logical_net, board_net, next) = &after[&pad];
        if previous != *next {
            return Err(invalid(
                "Revision rebinds a component, terminal, map or physical Pad source",
            ));
        }
        if previous_logical_net != *logical_net {
            changes.push(PadNetChange {
                component_instance: previous.component_instance.object_id,
                terminal: previous.schematic_terminal,
                pad,
                previous_logical_net,
                logical_net: *logical_net,
                previous_board_net,
                board_net: *board_net,
            });
        }
    }
    Ok(changes)
}

fn operations(review: &BoardConnectivityRevision) -> Result<Vec<Operation>, EngineError> {
    if review.schema_version != 1
        || review.changes.is_empty()
        || assignment_changes(review)? != review.changes
        || !review.source_revisions.contains_key(&review.sheet)
        || !review.source_revisions.contains_key(&review.board)
    {
        return Err(invalid(
            "Receipt does not contain its complete guarded assignment change",
        ));
    }
    let mut copper = BTreeSet::new();
    for row in &review.retained_copper {
        if !copper.insert(row.source_id) || !review.source_revisions.contains_key(&row.source_id) {
            return Err(invalid(
                "Retained copper has duplicate or unguarded source identity",
            ));
        }
    }
    let mut result: Vec<_> = review
        .source_revisions
        .iter()
        .map(|(id, revision)| Operation::GuardObjectRevision {
            object_id: *id,
            expected_object_revision: *revision,
        })
        .collect();
    result.extend(
        review
            .relationships
            .iter()
            .map(|row| Operation::SetElectricalIdentity {
                previous: Box::new(row.previous.clone()),
                record: row.record.clone(),
            }),
    );
    Ok(result)
}

pub(crate) fn is_revision(proposal: &Proposal) -> bool {
    proposal.batch.provenance.reason == REASON
}

/// A reason string is dispatch only. The typed receipt must certify the exact
/// batch, and current-source recomputation remains required before acceptance.
pub(crate) fn validate_recorded(
    proposal: &Proposal,
) -> Result<BoardConnectivityRevision, EngineError> {
    let review: BoardConnectivityRevision = serde_json::from_str(&proposal.rationale)?;
    if !is_revision(proposal)
        || proposal.batch.batch_id != proposal.proposal_id
        || proposal.source != ProposalSource::Manual
        || proposal.batch.provenance.source != CommitSource::Manual
        || review.project_id != proposal.project_id
        || review.model_revision != proposal.prepared_against
        || proposal.batch.expected_model_revision.as_ref() != Some(&review.model_revision)
        || operations(&review)? != proposal.batch.operations
    {
        return Err(invalid(
            "Proposal differs from the exact typed connectivity receipt",
        ));
    }
    Ok(review)
}

/// Source policy for both native commands and the generic Proposal gateway.
/// The family marker dispatches to canonical recomputation; it grants no trust.
pub(crate) fn validate_current(
    model: &DesignModel,
    proposal: &Proposal,
) -> Result<(), EngineError> {
    let review = validate_recorded(proposal)?;
    history::authenticate(model, proposal)?;
    let provenance = &proposal.batch.provenance;
    let rebuilt = prepare(
        model,
        WriteProvenance::new(
            provenance.actor.clone(),
            provenance.source.clone(),
            provenance.reason.clone(),
        ),
        &review,
        proposal.batch.batch_id,
    )?;
    if rebuilt.batch.operations != proposal.batch.operations {
        return Err(invalid(
            "Pending batch differs from the complete current revision",
        ));
    }
    Ok(())
}

pub(crate) fn summary(
    model: &DesignModel,
    proposal: &Proposal,
    current: bool,
) -> Result<BoardConnectivityRevision, EngineError> {
    let review = validate_recorded(proposal)?;
    if review.project_id != model.project.project_id {
        return Err(invalid("Receipt belongs to another project"));
    }
    history::authenticate(model, proposal)?;
    if current || proposal.status == crate::substrate::ProposalStatus::Draft {
        validate_current(model, proposal)?;
    }
    Ok(review)
}
