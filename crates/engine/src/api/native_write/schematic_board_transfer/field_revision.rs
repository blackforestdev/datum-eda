//! Reviewed display-field propagation over existing component identity.
//!
//! This planner never allocates physical identities or repairs connectivity.
//! Library and electrical correspondence must already be qualified; changing
//! either belongs to a different, explicitly reviewed revision intent.
use crate::{
    api::native_write::{
        BatchComposer, PreparedWrite, WriteProvenance,
        board_components::{BoardPackageEdit, build_edit_board_package},
        schematic_selection_edit::validate_field_values,
    },
    board::PlacedPackage,
    error::EngineError,
    substrate::{
        ComponentInstanceAuthority, DesignModel, ElectricalIdentity, ElectricalOccurrence,
        ElectricalSelectionSnapshot, ModelRevision, NetRelationshipIntent, ObjectRevision,
        Operation, SourceShardKind, SourceShardTaxon,
        component_pad_source::{
            self as pads, ComponentPadMaterializationStatus, ComponentPadSource,
        },
    },
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub(crate) const REASON: &str = "Review schematic reference and value changes on the PCB";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardFieldChange {
    pub component_instance: Uuid,
    pub symbol: Uuid,
    pub package: Uuid,
    pub previous_reference: String,
    pub previous_value: String,
    pub reference: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardFieldRevision {
    pub project_id: Uuid,
    pub sheet: Uuid,
    pub board: Uuid,
    pub model_revision: ModelRevision,
    pub changes: Vec<BoardFieldChange>,
    pub source_revisions: BTreeMap<Uuid, ObjectRevision>,
}

fn invalid(message: impl Into<String>) -> EngineError {
    EngineError::Validation(format!("PCB field revision: {}", message.into()))
}

fn guard(
    model: &DesignModel,
    guards: &mut BTreeMap<Uuid, ObjectRevision>,
    id: Uuid,
) -> Result<(), EngineError> {
    let source = model
        .objects
        .get(&id)
        .ok_or_else(|| invalid("A declared source object is unavailable"))?;
    guards.insert(id, source.object_revision);
    Ok(())
}

fn current_pin(
    model: &DesignModel,
    revisions: &mut BTreeMap<Uuid, ObjectRevision>,
    pin: &pads::PinnedPadLibrarySource,
) -> Result<(), EngineError> {
    pads::check_pin(model, pin)?;
    let owner = model
        .objects
        .get(&pin.owner_id)
        .ok_or_else(|| invalid("A frozen physical source owner is unavailable"))?;
    if model
        .objects
        .get(&pin.object_id)
        .is_none_or(|object| object.object_revision != pin.object_revision)
    {
        return Err(invalid(
            "A physical source revision changed, including a reverted library edit",
        ));
    }
    let taxon = match owner.kind.as_str() {
        "parts" => SourceShardTaxon::PoolPart,
        "packages" => SourceShardTaxon::PoolPackage,
        "footprints" => SourceShardTaxon::PoolFootprint,
        "padstacks" => SourceShardTaxon::PoolPadstack,
        "pin_pad_maps" => SourceShardTaxon::PoolPinPadMap,
        _ => return Err(invalid("Unsupported physical source owner")),
    };
    let current = pads::capture_root(model, pin.owner_id, taxon)?;
    if current.owner_value_hash != pin.owner_value_hash
        || current.source_shard_id != pin.source_shard_id
        || current.incarnation != pin.incarnation
    {
        return Err(invalid(
            "A physical-library definition changed; field propagation cannot adopt it",
        ));
    }
    guard(model, revisions, pin.owner_id)?;
    guard(model, revisions, pin.object_id)
}

/// Capture both sides from the same immutable model, using CI membership rather
/// than matching reference text. A changed or incomplete graph is a refusal.
pub fn preview(
    model: &DesignModel,
    sheet: Uuid,
    instance_path: &[Uuid],
) -> Result<BoardFieldRevision, EngineError> {
    if !instance_path.is_empty() {
        return Err(invalid(
            "This metadata revision supports the root occurrence only",
        ));
    }
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Schematic source unavailable: {e:?}")))?;
    let schematic = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("A native schematic is required"))?;
    super::super::schematic_edit::require_unique_occurrence(schematic, sheet, instance_path)?;
    let source = schematic
        .sheets
        .get(&sheet)
        .ok_or_else(|| invalid("The captured schematic sheet is unavailable"))?;
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let board_id: Uuid = serde_json::from_value(board["uuid"].clone())?;
    let packages: BTreeMap<Uuid, PlacedPackage> =
        serde_json::from_value(board["packages"].clone())?;
    let projection = pads::component_pad_projection(model, &board)?;
    let mut revisions = BTreeMap::new();
    guard(model, &mut revisions, sheet)?;
    guard(model, &mut revisions, board_id)?;
    let mut changes = Vec::new();
    let mut seen_symbols = BTreeSet::new();
    let mut seen_packages = BTreeSet::new();
    let mut current_pins: BTreeMap<Uuid, BTreeSet<ElectricalOccurrence>> = BTreeMap::new();
    for source in snapshot
        .authored_sources()
        .map_err(|e| invalid(format!("Current connectivity source is unavailable: {e:?}")))?
    {
        if source.occurrence.class == "pins"
            && let Ok(net) = snapshot.schematic_member_net(&source.occurrence)
        {
            current_pins
                .entry(net)
                .or_default()
                .insert(source.occurrence);
        }
    }
    let mut recorded_pins: BTreeMap<Uuid, BTreeSet<ElectricalOccurrence>> = BTreeMap::new();
    for record in model.electrical_identities.values() {
        if let ElectricalIdentity::NetRelationship {
            logical_net,
            board_net: Some(_),
            intent: NetRelationshipIntent::Implemented,
            evidence,
        } = &record.identity
        {
            recorded_pins
                .entry(*logical_net)
                .or_default()
                .extend(evidence.iter().map(|pin| pin.schematic_terminal.clone()));
        }
    }

    for component in model.component_instances.values().filter(|ci| {
        ci.placed_symbol_refs
            .iter()
            .any(|id| source.symbols.contains_key(id))
            && !ci.placed_package_refs.is_empty()
    }) {
        let ([symbol_id], [package_id]) = (
            component.placed_symbol_refs.as_slice(),
            component.placed_package_refs.as_slice(),
        ) else {
            return Err(invalid(
                "This revision requires one Symbol and one PCB counterpart per component",
            ));
        };
        if component.authority != ComponentInstanceAuthority::Authored
            || !seen_symbols.insert(*symbol_id)
            || !seen_packages.insert(*package_id)
        {
            return Err(invalid("Component counterpart ownership is ambiguous"));
        }
        let symbol = source.symbols.get(symbol_id).ok_or_else(|| {
            invalid("Component membership crosses an unsupported sheet occurrence")
        })?;
        let placement = packages
            .get(package_id)
            .ok_or_else(|| invalid("The declared PCB counterpart is missing"))?;
        if model
            .component_instances
            .values()
            .filter(|ci| {
                ci.placed_symbol_refs.contains(symbol_id)
                    || ci.placed_package_refs.contains(package_id)
            })
            .count()
            != 1
        {
            return Err(invalid(
                "A Symbol or PCB counterpart has more than one component owner",
            ));
        }
        if component.part_ref != symbol.part || component.part_ref != Some(placement.part) {
            return Err(invalid(
                "Component Part identity changed; review a physical-library update separately",
            ));
        }
        for (id, revision) in
            model.declared_bound_component_part_sources(component.id, placement.part)?
        {
            if model
                .objects
                .get(&id)
                .is_none_or(|source| source.object_revision != revision)
            {
                return Err(invalid(
                    "A declared logical or physical library source revision changed",
                ));
            }
            guard(model, &mut revisions, id)?;
        }
        for binding in component.library_bindings.values() {
            let current = model
                .objects
                .get(&binding.target_object_id)
                .ok_or_else(|| invalid("A pinned library source is unavailable"))?;
            if current.object_revision != binding.pinned_object_revision {
                return Err(invalid(
                    "A library revision changed; field propagation cannot adopt it",
                ));
            }
            guard(model, &mut revisions, binding.target_object_id)?;
        }
        for id in [component.id, *symbol_id, *package_id] {
            guard(model, &mut revisions, id)?;
        }
        let physical: Vec<ComponentPadSource> =
            serde_json::from_value(board["component_pads"][package_id.to_string()].clone())?;
        if physical.is_empty() || projection.unadopted_packages.contains(package_id) {
            return Err(invalid(
                "PCB counterpart has no qualified materialized Pad sources",
            ));
        }
        let mut terminals = BTreeSet::new();
        let mut physical_ids = BTreeSet::new();
        let mut covered_pads = BTreeSet::new();
        for pad in &physical {
            if projection.statuses.get(&pad.uuid)
                != Some(&ComponentPadMaterializationStatus::Current)
                || !physical_ids.insert(pad.uuid)
            {
                return Err(invalid(
                    "A placed Pad or physical-library source is stale or unavailable",
                ));
            }
            let binding = pad
                .source_binding
                .as_ref()
                .ok_or_else(|| invalid("A placed Pad lacks its source binding"))?;
            if binding.package_id != *package_id
                || binding.board_id != board_id
                || binding.component_instance.as_ref().map(|r| r.object_id) != Some(component.id)
            {
                return Err(invalid(
                    "Placed Pad ownership differs from the component counterpart",
                ));
            }
            for pin in [
                &binding.part,
                &binding.package,
                &binding.land_pattern,
                &binding.library_pad,
                &binding.padstack,
            ] {
                current_pin(model, &mut revisions, pin)?;
            }
            if let Some(map) = &binding.pin_pad_map {
                current_pin(model, &mut revisions, map)?;
            } else {
                return Err(invalid("A placed Pad lacks its explicit Pin/Pad Map"));
            }
        }
        for relation in model.electrical_identities.values() {
            let ElectricalIdentity::NetRelationship {
                logical_net,
                board_net,
                intent,
                evidence,
            } = &relation.identity
            else {
                continue;
            };
            let owned: Vec<_> = evidence
                .iter()
                .filter(|e| e.component_instance.object_id == component.id)
                .collect();
            if owned.is_empty() {
                continue;
            }
            if *intent != NetRelationshipIntent::Implemented || board_net.is_none() {
                return Err(invalid(
                    "PCB correspondence is not an implemented connection",
                ));
            }
            if current_pins.get(logical_net) != recorded_pins.get(logical_net) {
                return Err(invalid(
                    "Connected schematic Pin membership changed; review connectivity separately",
                ));
            }
            snapshot
                .global_net(&model.model_revision, *logical_net)
                .map_err(|e| invalid(format!("Connectivity changed or is unqualified: {e:?}")))?;
            guard(model, &mut revisions, relation.id)?;
            guard(model, &mut revisions, *logical_net)?;
            for evidence in owned {
                if !physical_ids.contains(&evidence.board_pad.object_id)
                    || !covered_pads.insert(evidence.board_pad.object_id)
                    || snapshot.schematic_member_net(&evidence.schematic_terminal)
                        != Ok(*logical_net)
                {
                    return Err(invalid(
                        "Schematic topology or Pin/Pad correspondence changed",
                    ));
                }
                terminals.insert(evidence.schematic_terminal.clone());
            }
        }
        let expected: BTreeSet<_> = symbol
            .pins
            .iter()
            .map(|pin| ElectricalOccurrence {
                class: "pins".into(),
                source_id: pin.uuid,
                instance_path: instance_path.to_vec(),
            })
            .collect();
        if terminals != expected || covered_pads != physical_ids {
            return Err(invalid(
                "Every logical Pin and placed Pad needs unchanged explicit correspondence",
            ));
        }
        validate_field_values(Some(&symbol.reference), Some(&symbol.value))?;
        if placement.reference != symbol.reference || placement.value != symbol.value {
            changes.push(BoardFieldChange {
                component_instance: component.id,
                symbol: *symbol_id,
                package: *package_id,
                previous_reference: placement.reference.clone(),
                previous_value: placement.value.clone(),
                reference: symbol.reference.clone(),
                value: symbol.value.clone(),
            });
        }
    }
    changes.sort_by_key(|row| row.component_instance);
    let replacements: BTreeMap<_, _> = changes.iter().map(|c| (c.package, &c.reference)).collect();
    let mut references = BTreeSet::new();
    for placement in packages.values() {
        let reference = replacements
            .get(&placement.uuid)
            .copied()
            .unwrap_or(&placement.reference);
        if !references.insert(reference) {
            return Err(invalid(
                "Updated PCB references would collide; no changes were prepared",
            ));
        }
    }
    Ok(BoardFieldRevision {
        project_id: model.project.project_id,
        sheet,
        board: board_id,
        model_revision: model.model_revision.clone(),
        changes,
        source_revisions: revisions,
    })
}

/// Compose existing display edits and explicit component/evidence epoch repins.
/// The caller creates a Proposal; frozen physical sources remain unchanged.
pub(crate) fn prepare(
    model: &DesignModel,
    provenance: WriteProvenance,
    reviewed: &BoardFieldRevision,
    batch_id: Uuid,
) -> Result<PreparedWrite, EngineError> {
    if preview(model, reviewed.sheet, &[])? != *reviewed {
        return Err(invalid(
            "The reviewed source or PCB fields changed; reopen the review",
        ));
    }
    if reviewed.changes.is_empty() {
        return Err(invalid(
            "PCB reference and value fields already match the schematic",
        ));
    }
    let mut operations = Vec::new();
    for row in &reviewed.changes {
        let mut add = |edit| -> Result<(), EngineError> {
            operations.extend(
                build_edit_board_package(model, provenance.clone(), row.package, edit)?
                    .batch
                    .operations,
            );
            Ok(())
        };
        if row.reference != row.previous_reference {
            add(BoardPackageEdit::Reference {
                reference: row.reference.clone(),
            })?;
        }
        if row.value != row.previous_value {
            add(BoardPackageEdit::Value {
                value: row.value.clone(),
            })?;
        }
    }
    let advances = reviewed
        .changes
        .iter()
        .map(|row| {
            (
                row.package,
                u64::from(row.reference != row.previous_reference)
                    + u64::from(row.value != row.previous_value),
            )
        })
        .collect();
    operations.extend(
        crate::api::native_write::schematic_assignment::component::repin_board_fields(
            model, &advances,
        )?,
    );
    operations.retain(|op| !matches!(op, Operation::GuardObjectRevision { .. }));
    operations.splice(
        0..0,
        reviewed
            .source_revisions
            .iter()
            .map(|(id, revision)| Operation::GuardObjectRevision {
                object_id: *id,
                expected_object_revision: *revision,
            }),
    );
    let mut result = BatchComposer::compose(model, provenance)
        .push_ops(operations)
        .finish()?;
    result.batch.batch_id = batch_id;
    Ok(result)
}

pub(in crate::api::native_write) fn is_revision(proposal: &crate::substrate::Proposal) -> bool {
    proposal.batch.provenance.reason == REASON
}

fn field_operations(reviewed: &BoardFieldRevision) -> Vec<Operation> {
    let mut operations = Vec::new();
    for row in &reviewed.changes {
        if row.previous_reference != row.reference {
            operations.push(Operation::SetBoardPackageReference {
                package_id: row.package,
                reference: row.reference.clone(),
            });
        }
        if row.previous_value != row.value {
            operations.push(Operation::SetBoardPackageValue {
                package_id: row.package,
                value: row.value.clone(),
            });
        }
    }
    operations
}

fn recorded(proposal: &crate::substrate::Proposal) -> Result<BoardFieldRevision, EngineError> {
    if !is_revision(proposal) {
        return Err(invalid("This is not a PCB display-field revision"));
    }
    let reviewed: BoardFieldRevision = serde_json::from_str(&proposal.rationale)?;
    if reviewed.project_id != proposal.project_id
        || reviewed.model_revision != proposal.prepared_against
        || proposal.batch.expected_model_revision.as_ref() != Some(&reviewed.model_revision)
        || reviewed.changes.is_empty()
    {
        return Err(invalid(
            "Revision receipt has a different project or source basis",
        ));
    }
    let mut components = BTreeSet::new();
    let mut symbols = BTreeSet::new();
    let mut packages = BTreeSet::new();
    for row in &reviewed.changes {
        validate_field_values(Some(&row.reference), Some(&row.value))?;
        if !components.insert(row.component_instance)
            || !symbols.insert(row.symbol)
            || !packages.insert(row.package)
            || (row.reference == row.previous_reference && row.value == row.previous_value)
            || [row.component_instance, row.symbol, row.package]
                .iter()
                .any(|id| !reviewed.source_revisions.contains_key(id))
        {
            return Err(invalid(
                "Revision receipt has duplicate, unchanged or unguarded rows",
            ));
        }
    }
    if !reviewed.source_revisions.contains_key(&reviewed.sheet)
        || !reviewed.source_revisions.contains_key(&reviewed.board)
    {
        return Err(invalid(
            "Revision receipt lacks its schematic or board source guard",
        ));
    }
    let mut guards = BTreeMap::new();
    let mut operations = Vec::new();
    for operation in &proposal.batch.operations {
        match operation {
            Operation::GuardObjectRevision {
                object_id,
                expected_object_revision,
            } => {
                if guards
                    .insert(*object_id, *expected_object_revision)
                    .is_some()
                {
                    return Err(invalid("Revision receipt has duplicate source guards"));
                }
            }
            Operation::SetBoardPackageReference { .. } | Operation::SetBoardPackageValue { .. } => {
                operations.push(operation.clone());
            }
            Operation::SetComponentInstance { .. } | Operation::SetElectricalIdentity { .. } => {}
            _ => {
                return Err(invalid(
                    "Revision receipt changes more than PCB reference/value fields",
                ));
            }
        }
    }
    if guards != reviewed.source_revisions || operations != field_operations(&reviewed) {
        return Err(invalid(
            "Revision receipt does not describe the exact guarded field batch",
        ));
    }
    DesignModel::validate_component_metadata_proposal(proposal)?;
    Ok(reviewed)
}

pub(in crate::api::native_write) fn validate_recorded(
    proposal: &crate::substrate::Proposal,
) -> Result<(), EngineError> {
    recorded(proposal).map(|_| ())
}

/// Draft/acceptance validation rebuilds the complete source-qualified batch.
/// Applied receipts recover their before fields from the canonical inverse,
/// rather than trusting display text supplied in a proposal rationale.
pub(in crate::api::native_write) fn summary(
    model: &DesignModel,
    proposal: &crate::substrate::Proposal,
    current: bool,
) -> Result<BoardFieldRevision, EngineError> {
    let reviewed = recorded(proposal)?;
    if reviewed.project_id != model.project.project_id {
        return Err(invalid("Revision receipt belongs to another project"));
    }
    if current || proposal.status == crate::substrate::ProposalStatus::Draft {
        let provenance = &proposal.batch.provenance;
        let rebuilt = prepare(
            model,
            WriteProvenance::new(
                provenance.actor.clone(),
                provenance.source.clone(),
                provenance.reason.clone(),
            ),
            &reviewed,
            proposal.batch.batch_id,
        )?;
        if rebuilt.batch.operations != proposal.batch.operations {
            return Err(invalid(
                "The pending revision differs from its complete current batch",
            ));
        }
    }
    if let Some(id) = proposal.applied_transaction_id {
        let transaction = model
            .journal
            .iter()
            .find(|tx| tx.transaction_id == id)
            .ok_or_else(|| invalid("Applied revision transaction is unavailable"))?;
        if transaction.batch_id != proposal.batch.batch_id {
            return Err(invalid(
                "Applied revision names a different operation batch",
            ));
        }
        model.validate_component_metadata_transaction(proposal, transaction)?;
        let mut before = BTreeMap::new();
        for operation in &transaction.inverse_operations {
            let item = match operation {
                Operation::SetBoardPackageReference {
                    package_id,
                    reference,
                } => Some(((*package_id, false), reference)),
                Operation::SetBoardPackageValue { package_id, value } => {
                    Some(((*package_id, true), value))
                }
                Operation::SetProposalMetadata { .. }
                | Operation::SetComponentInstance { .. }
                | Operation::SetElectricalIdentity { .. } => None,
                _ => return Err(invalid("Applied revision inverse changes unrelated source")),
            };
            if let Some((key, value)) = item {
                if before.insert(key, value).is_some() {
                    return Err(invalid("Applied revision has duplicate field inverses"));
                }
            }
        }
        let mut expected = BTreeMap::new();
        for row in &reviewed.changes {
            if row.reference != row.previous_reference {
                expected.insert((row.package, false), &row.previous_reference);
            }
            if row.value != row.previous_value {
                expected.insert((row.package, true), &row.previous_value);
            }
        }
        if before != expected {
            return Err(invalid(
                "Revision before fields differ from their recorded canonical inverse",
            ));
        }
    }
    Ok(reviewed)
}
