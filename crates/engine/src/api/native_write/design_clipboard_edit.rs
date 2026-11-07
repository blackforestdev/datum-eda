//! Canonical design-bundle placement and Cut through existing Proposal transport.
use super::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    design_clipboard::{
        ClipboardClass, ClipboardItem, ClipboardPlacement, ClipboardTarget, DesignClipboardBundle,
    },
    registry::NativeWriteContext,
    schematic_delete_proposal::{DeleteClass, DeleteTarget},
    schematic_edit::{prepare_native_birth, require_unique_occurrence, wire_t_contacts},
    schematic_symbol_payload::set_point,
    schematic_wire_edit::WiringTransformBasis,
    schematic_wire_edit_adapter::validate_fresh_ids,
};
use crate::{
    error::EngineError,
    ir::geometry::Point,
    schematic::{Junction, SchematicWire},
    substrate::{ElectricalSelectionSnapshot, Operation, SchematicMarkerKind},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

/// Pasted wiring follows exactly the existing authored endpoint-to-segment law.
/// A mere interior crossing has no new connection; selected Junctions remain
/// explicit. No disconnected-contact or rerouting mode is added here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardContactPolicy {
    DrawWireEndpoints,
}
/// Existing-object transforms preserve fixed contacts; reconnection remains
/// an explicit endpoint edit, never an implicit rigid-body transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExistingTransformContactPolicy {
    PreserveFixedContacts,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DesignEditIntent {
    TransformExisting {
        targets: Vec<ClipboardTarget>,
        reference: Point,
        placement: ClipboardPlacement,
        contact_policy: ExistingTransformContactPolicy,
    },
    Cut {
        targets: Vec<ClipboardTarget>,
    },
    PlaceBundle {
        bundle: DesignClipboardBundle,
        placement: ClipboardPlacement,
        edit_seed: Uuid,
        contact_policy: ClipboardContactPolicy,
    },
}

pub fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    sheet: Uuid,
    path: &[Uuid],
    edit: &DesignEditIntent,
) -> Result<PreparedWrite, EngineError> {
    build_with_id(context, provenance, None, sheet, path, edit)
}

pub(super) fn build_with_id(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    batch_id: Option<Uuid>,
    sheet: Uuid,
    path: &[Uuid],
    edit: &DesignEditIntent,
) -> Result<PreparedWrite, EngineError> {
    match edit {
        DesignEditIntent::TransformExisting {
            targets,
            reference,
            placement,
            contact_policy,
        } => {
            let captured =
                DesignClipboardBundle::capture(context.model, sheet, path, targets, *reference)?;
            let snapshot = ElectricalSelectionSnapshot::capture(context.model)
                .map_err(|e| invalid(format!("Transform source unavailable: {e:?}")))?;
            let source = snapshot
                .schematic_source()
                .ok_or_else(|| invalid("Transform source unavailable"))?;
            let wire_ids: BTreeSet<_> = targets
                .iter()
                .filter(|t| t.class == ClipboardClass::Wire)
                .map(|t| t.id)
                .collect();
            if source
                .sheets
                .values()
                .flat_map(|s| s.bus_entries.values())
                .any(|entry| entry.wire.is_some_and(|id| wire_ids.contains(&id)))
            {
                return Err(invalid("Transform wire is referenced by a Bus Entry"));
            }
            let target = source
                .sheets
                .get(&sheet)
                .ok_or_else(|| invalid("Transform sheet unavailable"))?;
            let operations = transform_operations(
                target,
                &captured,
                targets,
                *reference,
                *placement,
                *contact_policy,
            )?;
            if operations.is_empty() {
                return Err(invalid("Transform destination is unchanged"));
            }
            // Existing Move vocabulary replaces raw payloads atomically with
            // SAME UUIDs. This is not Cut+Paste or fresh identity nomination.
            let mut prepared = BatchComposer::compose(context.model, provenance)
                .push_ops(operations)
                .primary_object(targets[0].id)
                .finish()?;
            if let Some(id) = batch_id {
                prepared.batch.batch_id = id;
            }
            Ok(prepared)
        }
        DesignEditIntent::Cut { targets } => {
            let captured =
                DesignClipboardBundle::capture(context.model, sheet, path, targets, Point::zero())?;
            if captured.items.iter().any(|item| {
                item.raw.get("locked").and_then(serde_json::Value::as_bool) == Some(true)
            }) {
                return Err(invalid("Cut contains a locked design object"));
            }
            let targets = targets
                .iter()
                .map(|target| DeleteTarget {
                    id: target.id,
                    class: match target.class {
                        ClipboardClass::Wire => DeleteClass::Wire,
                        ClipboardClass::Junction => DeleteClass::Junction,
                    },
                })
                .collect::<Vec<_>>();
            // Exact raw inverse, Bus Entry dependencies and complete selection
            // admission are owned by the same deletion builder as ordinary Delete.
            let mut prepared =
                super::schematic_delete::build(context, provenance, sheet, path, &targets)?;
            if let Some(id) = batch_id {
                prepared.batch.batch_id = id;
            }
            Ok(prepared)
        }
        DesignEditIntent::PlaceBundle {
            bundle,
            placement,
            edit_seed,
            contact_policy: ClipboardContactPolicy::DrawWireEndpoints,
        } => {
            let snapshot = ElectricalSelectionSnapshot::capture(context.model)
                .map_err(|e| invalid(format!("Paste source unavailable: {e:?}")))?;
            let source = snapshot
                .schematic_source()
                .ok_or_else(|| invalid("Native paste source unavailable"))?;
            require_unique_occurrence(source, sheet, path)?;
            let target = source
                .sheets
                .get(&sheet)
                .ok_or_else(|| invalid("Paste sheet unavailable"))?;
            let (operations, ids) = place_operations(
                target,
                bundle,
                placement,
                edit_seed,
                ClipboardContactPolicy::DrawWireEndpoints,
            )?;
            validate_fresh_ids(context, &ids)?;
            let mut prepared = BatchComposer::compose(context.model, provenance)
                .push_ops(operations)
                .primary_object(
                    *ids.iter()
                        .next()
                        .ok_or_else(|| invalid("Paste contains no supported objects"))?,
                )
                .finish()?;
            if let Some(id) = batch_id {
                prepared.batch.batch_id = id;
            }
            prepare_native_birth(context.model, prepared)
        }
    }
}
/// Pure physical plan shared by clipboard preview, receipts and canonical build.
/// It does not allocate semantic IDs or evaluate a cached proposal against itself.
pub(super) fn place_operations(
    target: &crate::schematic::Sheet,
    bundle: &DesignClipboardBundle,
    placement: &ClipboardPlacement,
    edit_seed: &Uuid,
    policy: ClipboardContactPolicy,
) -> Result<(Vec<Operation>, BTreeSet<Uuid>), EngineError> {
    let ClipboardContactPolicy::DrawWireEndpoints = policy;
    let sheet = target.uuid;
    let items = bundle.placed(*placement, *edit_seed)?;
    let mut ids = items.iter().map(|item| item.id).collect::<BTreeSet<_>>();
    let mut occupied = target
        .junctions
        .values()
        .map(|j| (j.position.x, j.position.y))
        .collect::<BTreeSet<_>>();
    let edge_key = |a: Point, b: Point| {
        let (a, b) = ((a.x, a.y), (b.x, b.y));
        if a <= b { (a, b) } else { (b, a) }
    };
    let mut geometry = target
        .wires
        .values()
        .map(|wire| edge_key(wire.from, wire.to))
        .collect::<BTreeSet<_>>();
    let mut edges = Vec::new();
    let mut operations = Vec::new();
    for ClipboardItem { class, id, raw } in items {
        match class {
            ClipboardClass::Wire => {
                let wire: SchematicWire = serde_json::from_value(raw.clone())?;
                if wire.from == wire.to || !geometry.insert(edge_key(wire.from, wire.to)) {
                    return Err(invalid(
                        "Paste would create a zero-length or duplicate wire segment",
                    ));
                }
                edges.push((wire.from, wire.to));
                operations.push(Operation::CreateSchematicWire {
                    sheet_id: sheet,
                    wire_id: id,
                    wire: raw,
                });
            }
            ClipboardClass::Junction => {
                let junction: Junction = serde_json::from_value(raw.clone())?;
                if !occupied.insert((junction.position.x, junction.position.y)) {
                    return Err(invalid("A Junction already occupies the pasted position"));
                }
                operations.push(Operation::PlaceSchematicMarker {
                    sheet_id: sheet,
                    marker_id: id,
                    marker_kind: SchematicMarkerKind::Junction,
                    marker: raw,
                });
            }
        }
    }
    for (x, y) in wire_t_contacts(target, &edges)? {
        if !occupied.insert((x, y)) {
            continue;
        }
        let id = Uuid::new_v5(
            edit_seed,
            format!("design-clipboard:t-contact:{x}:{y}").as_bytes(),
        );
        if !ids.insert(id) {
            return Err(invalid("Pasted physical object identities collide"));
        }
        operations.push(Operation::PlaceSchematicMarker {
            sheet_id: sheet,
            marker_id: id,
            marker_kind: SchematicMarkerKind::Junction,
            marker: serde_json::to_value(Junction {
                uuid: id,
                position: Point::new(x, y),
            })?,
        });
    }
    Ok((operations, ids))
}
/// Pure source projection for the shared placement ghost. This checks known
/// target source IDs only; canonical build owns global freshness and nomination.
pub fn preview_placement(
    target: &crate::schematic::Sheet,
    bundle: &DesignClipboardBundle,
    placement: &ClipboardPlacement,
    seed: &Uuid,
    policy: ClipboardContactPolicy,
) -> Result<crate::schematic::Sheet, EngineError> {
    let (operations, ids) = place_operations(target, bundle, placement, seed, policy)?;
    let image = serde_json::to_value(target)?;
    if ids
        .iter()
        .any(|id| super::schematic_wire_edit_adapter::contains_id(&image, &id.to_string()))
    {
        return Err(invalid(
            "Clipboard preview would overwrite an existing source identity",
        ));
    }
    let mut preview = target.clone();
    for op in operations {
        match op {
            Operation::CreateSchematicWire { wire_id, wire, .. } => {
                preview.wires.insert(wire_id, serde_json::from_value(wire)?);
            }
            Operation::PlaceSchematicMarker {
                marker_id, marker, ..
            } => {
                preview
                    .junctions
                    .insert(marker_id, serde_json::from_value(marker)?);
            }
            _ => {
                return Err(invalid(
                    "Clipboard preview contains an unsupported operation",
                ));
            }
        }
    }
    Ok(preview)
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}

/// Pure transform plan shared by pointer previews and the recorded operation
/// materializer. Capturing a preview never allocates physical or semantic IDs.
pub fn preview_transform(
    target: &crate::schematic::Sheet,
    targets: &[ClipboardTarget],
    reference: Point,
    placement: ClipboardPlacement,
    policy: ExistingTransformContactPolicy,
) -> Result<crate::schematic::Sheet, EngineError> {
    let ExistingTransformContactPolicy::PreserveFixedContacts = policy;
    let plan = WiringTransformBasis::capture(target, targets)?.plan(reference, placement)?;
    Ok(plan.apply_to_sheet(target))
}

pub(super) fn transform_operations(
    target: &crate::schematic::Sheet,
    captured: &DesignClipboardBundle,
    targets: &[ClipboardTarget],
    reference: Point,
    placement: ClipboardPlacement,
    policy: ExistingTransformContactPolicy,
) -> Result<Vec<Operation>, EngineError> {
    let ExistingTransformContactPolicy::PreserveFixedContacts = policy;
    captured.validate()?;
    let requested: BTreeSet<_> = targets.iter().map(|t| (t.class, t.id)).collect();
    let actual: BTreeSet<_> = captured.items.iter().map(|t| (t.class, t.id)).collect();
    if requested.len() != targets.len()
        || requested != actual
        || captured.source_sheet != target.uuid
        || captured.reference != reference
    {
        return Err(invalid(
            "Transform raw capture differs from its explicit targets",
        ));
    }
    let plan = WiringTransformBasis::capture(target, targets)?.plan(reference, placement)?;
    if !plan.removed_wire_ids.is_empty()
        || !plan.removed_junction_ids.is_empty()
        || !plan.new_wire_origins.is_empty()
        || !plan.coalesced_wire_origins.is_empty()
        || plan
            .wires
            .keys()
            .any(|id| !requested.contains(&(ClipboardClass::Wire, *id)))
        || plan
            .junctions
            .keys()
            .any(|id| !requested.contains(&(ClipboardClass::Junction, *id)))
    {
        return Err(invalid(
            "Existing transform cannot create or remove physical identities",
        ));
    }
    let mut operations = Vec::new();
    for item in &captured.items {
        if item.raw.get("locked").and_then(serde_json::Value::as_bool) == Some(true) {
            return Err(invalid("Transform contains a locked design object"));
        }
        let mut next = item.raw.clone();
        match item.class {
            ClipboardClass::Wire => {
                let old: SchematicWire = serde_json::from_value(item.raw.clone())?;
                if target.wires.get(&item.id) != Some(&old) {
                    return Err(invalid("Transform wire differs from captured source"));
                }
                if let Some(wire) = plan.wires.get(&item.id).filter(|wire| **wire != old) {
                    set_point(&mut next["from"], wire.from);
                    set_point(&mut next["to"], wire.to);
                    operations.push(Operation::DeleteSchematicWire {
                        sheet_id: target.uuid,
                        wire_id: item.id,
                        wire: item.raw.clone(),
                    });
                    operations.push(Operation::CreateSchematicWire {
                        sheet_id: target.uuid,
                        wire_id: item.id,
                        wire: next,
                    });
                }
            }
            ClipboardClass::Junction => {
                let old: Junction = serde_json::from_value(item.raw.clone())?;
                if target.junctions.get(&item.id) != Some(&old) {
                    return Err(invalid("Transform Junction differs from captured source"));
                }
                if let Some(junction) = plan
                    .junctions
                    .get(&item.id)
                    .filter(|junction| **junction != old)
                {
                    set_point(&mut next["position"], junction.position);
                    operations.push(Operation::DeleteSchematicJunction {
                        sheet_id: target.uuid,
                        junction_id: item.id,
                        junction: item.raw.clone(),
                    });
                    operations.push(Operation::PlaceSchematicMarker {
                        sheet_id: target.uuid,
                        marker_id: item.id,
                        marker_kind: SchematicMarkerKind::Junction,
                        marker: next,
                    });
                }
            }
        }
    }
    Ok(operations)
}
