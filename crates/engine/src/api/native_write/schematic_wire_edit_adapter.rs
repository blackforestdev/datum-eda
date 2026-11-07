//! Canonical raw-payload adapter over the shared immutable wire edit plan.
use super::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    registry::NativeWriteContext,
    schematic_edit::prepare_native_birth,
    schematic_selected_source::SelectedSchematicSource,
    schematic_symbol_payload::{member_payload, set_point, source_sheet_payload},
    schematic_wire_edit::{WireEditAction, WireEditBasis, WireEditPlan, WireHandle, WireRouting},
};
use crate::{error::EngineError, ir::geometry::Point, substrate::Operation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use uuid::Uuid;

/// One explicit handle and immutable ID seed shared by preview and commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireEditParams {
    #[serde(default)]
    pub action: WireEditAction,
    pub handle: WireHandle,
    #[serde(default = "free_routing")]
    pub routing: WireRouting,
    pub edit_seed: Uuid,
}
fn free_routing() -> WireRouting {
    WireRouting::Free
}

impl WireEditParams {
    /// Preview and canonical commit share one handle/action validation path.
    pub fn plan(
        &self,
        basis: &WireEditBasis,
        destination: Point,
    ) -> Result<WireEditPlan, EngineError> {
        if self.edit_seed.is_nil() {
            return Err(invalid(
                "Wire editing requires a nonempty gesture identity seed",
            ));
        }
        let anchor = basis.anchor(&self.handle)?;
        match (self.action, self.handle) {
            (WireEditAction::Move, _) => basis.plan_move(&self.handle, destination, self.routing),
            (
                WireEditAction::Split,
                WireHandle::Segment { from, to } | WireHandle::InsertVertex { from, to },
            ) => {
                if destination == from
                    || destination == to
                    || !crate::connectivity::exact_segment_intersection(
                        destination,
                        destination,
                        from,
                        to,
                    )
                    .is_ok_and(|point| point.is_some())
                {
                    return Err(invalid(
                        "Split requires an exact interior point of the captured span",
                    ));
                }
                basis.plan_split(destination)
            }
            (WireEditAction::RemoveSpan, WireHandle::Segment { from, to }) => {
                if destination != anchor {
                    return Err(invalid("Span removal must retain its captured anchor"));
                }
                basis.plan_remove_span(from, to)
            }
            (WireEditAction::RemoveVertex, WireHandle::Vertex { point }) => {
                if destination != anchor {
                    return Err(invalid("Vertex removal must retain its captured anchor"));
                }
                basis.plan_remove_vertex(point)
            }
            _ => Err(invalid(
                "Wire lifecycle action does not match its captured handle",
            )),
        }
    }
}

pub(super) fn move_handle(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    object: Uuid,
    class: &str,
    path: &[Uuid],
    destination: Point,
    edit: &WireEditParams,
) -> Result<PreparedWrite, EngineError> {
    if edit.edit_seed.is_nil() {
        return Err(invalid(
            "Wire editing requires a nonempty gesture identity seed",
        ));
    }
    let selected = SelectedSchematicSource::capture(context)?;
    let sheet_id = selected.sheet(class, object, path)?;
    let source = &selected.source;
    let sheet = &source.sheets[&sheet_id];
    let basis = if class == "junctions" {
        let junction = &sheet.junctions[&object];
        if !matches!(&edit.handle, WireHandle::Vertex { point } if *point == junction.position) {
            return Err(invalid(
                "Junction wire editing requires its captured vertex",
            ));
        }
        WireEditBasis::capture_junction(sheet, object, edit.edit_seed)?
    } else if class == "wires" {
        WireEditBasis::capture(sheet, object, edit.edit_seed)?
    } else {
        return Err(invalid("Wire edit handles require a Wire or Junction"));
    };
    let plan = edit.plan(&basis, destination)?;
    prepare_plan(
        context, provenance, &selected, sheet_id, object, class, path, &plan,
    )
}

fn prepare_plan(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    selected: &SelectedSchematicSource,
    sheet_id: Uuid,
    object: Uuid,
    class: &str,
    path: &[Uuid],
    plan: &WireEditPlan,
) -> Result<PreparedWrite, EngineError> {
    let source = &selected.source;
    let sheet = &source.sheets[&sheet_id];
    let raw = source_sheet_payload(context.model, sheet_id, object)?;
    let mut touched: BTreeSet<_> = plan.removed_wire_ids.iter().copied().collect();
    touched.extend(
        plan.wires
            .keys()
            .filter(|id| sheet.wires.contains_key(id))
            .copied(),
    );
    touched.extend(plan.new_wire_origins.values().copied());
    touched.extend(plan.coalesced_wire_origins.values().flatten().copied());
    if class == "wires" {
        touched.insert(object);
    }
    // A wire referenced by an entry cannot be reshaped or replaced without an
    // explicit entry edit. This also checks foreign/dangling sheet references.
    if source
        .sheets
        .values()
        .flat_map(|s| s.bus_entries.values())
        .any(|entry| entry.wire.is_some_and(|wire| touched.contains(&wire)))
    {
        return Err(invalid(
            "Wire is referenced by a bus entry; remove the entry first",
        ));
    }
    for id in &touched {
        if selected.sheet("wires", *id, path)? != sheet_id {
            return Err(invalid("Wire edit spans different source occurrences"));
        }
        let original = sheet
            .wires
            .get(id)
            .ok_or_else(|| invalid("Wire edit origin is unavailable"))?;
        unlocked(&member_payload(&raw, "wires", *id, original)?)?;
    }
    let mut junctions: BTreeSet<_> = plan.removed_junction_ids.iter().copied().collect();
    junctions.extend(
        plan.junctions
            .keys()
            .filter(|id| sheet.junctions.contains_key(id))
            .copied(),
    );
    if class == "junctions" {
        junctions.insert(object);
    }
    for id in junctions {
        if selected.sheet("junctions", id, path)? != sheet_id {
            return Err(invalid("Junction edit spans different source occurrences"));
        }
        unlocked(&member_payload(
            &raw,
            "junctions",
            id,
            &sheet.junctions[&id],
        )?)?;
    }
    for origins in plan.coalesced_wire_origins.values() {
        compatible_coalescence(&raw, origins)?;
    }
    validate_plan_ids(context, sheet, plan)?;
    let mut operations = Vec::new();
    for id in &plan.removed_wire_ids {
        operations.push(Operation::DeleteSchematicWire {
            sheet_id,
            wire_id: *id,
            wire: member_payload(&raw, "wires", *id, &sheet.wires[id])?,
        });
    }
    for (id, wire) in &plan.wires {
        let mut payload = if let Some(original) = sheet.wires.get(id) {
            let payload = member_payload(&raw, "wires", *id, original)?;
            if original == wire {
                continue;
            }
            operations.push(Operation::DeleteSchematicWire {
                sheet_id,
                wire_id: *id,
                wire: payload.clone(),
            });
            payload
        } else {
            let origin = plan
                .new_wire_origins
                .get(id)
                .ok_or_else(|| invalid("New wire has no captured payload origin"))?;
            member_payload(&raw, "wires", *origin, &sheet.wires[origin])?
        };
        payload["uuid"] = serde_json::to_value(id)?;
        if let Some(origins) = plan.coalesced_wire_origins.get(id) {
            payload["from"] = outer_point_payload(&raw, origins, wire.from)?;
            payload["to"] = outer_point_payload(&raw, origins, wire.to)?;
        }
        set_point(&mut payload["from"], wire.from);
        set_point(&mut payload["to"], wire.to);
        operations.push(Operation::CreateSchematicWire {
            sheet_id,
            wire_id: *id,
            wire: payload,
        });
    }
    for id in &plan.removed_junction_ids {
        let original = sheet
            .junctions
            .get(id)
            .ok_or_else(|| invalid("Junction edit origin is unavailable"))?;
        let payload = member_payload(&raw, "junctions", *id, original)?;
        unlocked(&payload)?;
        operations.push(Operation::DeleteSchematicJunction {
            sheet_id,
            junction_id: *id,
            junction: payload,
        });
    }
    for (id, junction) in &plan.junctions {
        let payload = if let Some(original) = sheet.junctions.get(id) {
            let mut payload = member_payload(&raw, "junctions", *id, original)?;
            if original == junction {
                continue;
            }
            operations.push(Operation::DeleteSchematicJunction {
                sheet_id,
                junction_id: *id,
                junction: payload.clone(),
            });
            set_point(&mut payload["position"], junction.position);
            payload
        } else {
            serde_json::to_value(junction)?
        };
        operations.push(Operation::PlaceSchematicMarker {
            sheet_id,
            marker_id: *id,
            marker_kind: crate::substrate::SchematicMarkerKind::Junction,
            marker: payload,
        });
    }
    if operations.is_empty() {
        return Err(invalid("Wire edit destination is unchanged"));
    }
    let prepared = BatchComposer::compose(context.model, provenance)
        .push_ops(operations)
        .primary_object(object)
        .finish()?;
    prepare_native_birth(context.model, prepared)
}

fn compatible_coalescence(raw: &Value, origins: &BTreeSet<Uuid>) -> Result<(), EngineError> {
    let mut expected = None;
    for id in origins {
        let mut member = raw["wires"][id.to_string()]
            .as_object()
            .ok_or_else(|| invalid("Coalesced wire payload is unavailable"))?
            .clone();
        for key in ["uuid", "from", "to"] {
            member.remove(key);
        }
        if expected
            .as_ref()
            .is_some_and(|previous| previous != &member)
        {
            return Err(invalid("Coalesced wires have incompatible source metadata"));
        }
        expected = Some(member);
    }
    Ok(())
}
fn outer_point_payload(
    raw: &Value,
    origins: &BTreeSet<Uuid>,
    point: Point,
) -> Result<Value, EngineError> {
    let mut result = None;
    for id in origins {
        for end in ["from", "to"] {
            let value = &raw["wires"][id.to_string()][end];
            if value["x"].as_i64() == Some(point.x) && value["y"].as_i64() == Some(point.y) {
                if result.as_ref().is_some_and(|previous| previous != value) {
                    return Err(invalid("Coalesced endpoint metadata is ambiguous"));
                }
                result = Some(value.clone());
            }
        }
    }
    if let Some(result) = result {
        return Ok(result);
    }
    // A virtual outer contact may lie inside its original physical edge.
    // It can inherit source extensions only when every candidate endpoint has
    // identical extension data; differing unknown point metadata must refuse.
    let mut common = None;
    for id in origins {
        for end in ["from", "to"] {
            let mut value = raw["wires"][id.to_string()][end]
                .as_object()
                .ok_or_else(|| invalid("Coalesced point payload is unavailable"))?
                .clone();
            value.remove("x");
            value.remove("y");
            if common.as_ref().is_some_and(|previous| previous != &value) {
                return Err(invalid("Coalesced virtual endpoint metadata is ambiguous"));
            }
            common = Some(value);
        }
    }
    let mut result = Value::Object(
        common.ok_or_else(|| invalid("Coalesced endpoint has no captured payload origin"))?,
    );
    set_point(&mut result, point);
    Ok(result)
}

fn validate_plan_ids(
    context: &NativeWriteContext<'_>,
    sheet: &crate::schematic::Sheet,
    plan: &WireEditPlan,
) -> Result<(), EngineError> {
    if plan
        .removed_wire_ids
        .iter()
        .any(|id| plan.wires.contains_key(id) || !sheet.wires.contains_key(id))
        || plan
            .removed_junction_ids
            .iter()
            .any(|id| plan.junctions.contains_key(id) || !sheet.junctions.contains_key(id))
        || plan.wires.iter().any(|(id, wire)| *id != wire.uuid)
        || plan
            .junctions
            .iter()
            .any(|(id, junction)| *id != junction.uuid)
    {
        return Err(invalid(
            "Wire edit plan does not match captured source identities",
        ));
    }
    if plan.coalesced_wire_origins.iter().any(|(id, origins)| {
        !plan.wires.contains_key(id)
            || origins.len() < 2
            || origins
                .iter()
                .any(|origin| !sheet.wires.contains_key(origin))
    }) {
        return Err(invalid(
            "Coalesced payload origins do not match captured source",
        ));
    }
    let new_ids: BTreeSet<_> = plan
        .wires
        .keys()
        .filter(|id| !sheet.wires.contains_key(id))
        .copied()
        .collect();
    if plan
        .new_wire_origins
        .keys()
        .copied()
        .collect::<BTreeSet<_>>()
        != new_ids
        || plan
            .new_wire_origins
            .values()
            .any(|id| !sheet.wires.contains_key(id))
    {
        return Err(invalid(
            "New wire payload origins do not match the captured edit",
        ));
    }
    let new_junctions: BTreeSet<_> = plan
        .junctions
        .keys()
        .filter(|id| !sheet.junctions.contains_key(id))
        .copied()
        .collect();
    if !new_ids.is_disjoint(&new_junctions) {
        return Err(invalid("New wire and junction identities collide"));
    }
    let new_ids: BTreeSet<_> = new_ids.union(&new_junctions).copied().collect();
    validate_fresh_ids(context, &new_ids)
}

/// Shared fresh physical-ID admission for graph edits and design-bundle paste.
pub(super) fn validate_fresh_ids(
    context: &NativeWriteContext<'_>,
    new_ids: &BTreeSet<Uuid>,
) -> Result<(), EngineError> {
    if !new_ids.is_empty() {
        // Check all model namespaces, raw members and external references,
        // including persisted history: a gesture never reuses an authored ID.
        let model = serde_json::to_value(context.model)?;
        let mut authored_payloads = Vec::new();
        for shard in &context.model.source_shards {
            if shard.relative_path.ends_with(".json") {
                authored_payloads.push(
                    context
                        .model
                        .materialized_source_shard_value_by_relative_path(&shard.relative_path)?,
                );
            }
        }
        for id in new_ids {
            if id.is_nil()
                || contains_id(&model, &id.to_string())
                || authored_payloads
                    .iter()
                    .any(|value| contains_id(value, &id.to_string()))
            {
                return Err(invalid(
                    "New design identity already exists or is externally referenced",
                ));
            }
        }
    }
    Ok(())
}
pub(super) fn contains_id(value: &Value, id: &str) -> bool {
    match value {
        Value::String(value) => value == id,
        Value::Array(values) => values.iter().any(|v| contains_id(v, id)),
        Value::Object(values) => values
            .iter()
            .any(|(key, value)| key == id || contains_id(value, id)),
        _ => false,
    }
}
fn unlocked(payload: &Value) -> Result<(), EngineError> {
    if payload.get("locked").and_then(Value::as_bool) == Some(true) {
        Err(invalid("A wire edit source object is locked"))
    } else {
        Ok(())
    }
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(reason.into())
}
