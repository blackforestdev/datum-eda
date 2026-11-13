//! Native Zone boundaries and explicitly requested, source-qualified derived fill.
//! Preparation is read-only; only the canonical commit persists either operation.
use super::{PreparedWrite, WriteProvenance, build_place_board_zone, build_set_zone_fills};
use crate::api::native_write::registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params};
use crate::board::nominal_geometry::DistanceBoundary;
use crate::board::query_pad::QueryPadGeometry;
use crate::board::{Keepout, Net, PadShape, Stackup, StackupLayerType, Zone, occupied_copper};
use crate::error::EngineError;
use crate::ir::geometry::{Polygon, Rect};
use crate::substrate::{
    DesignModel, ElectricalSelectionSnapshot, ModelRevision, SourceShardKind,
    ZONE_FILL_SCHEMA_VERSION, ZoneFill, ZoneFillCopperContext, ZoneFillState,
    compute_bounded_zone_fill,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

pub(crate) const VERBS: &[NativeWriteVerb] = &[
    NativeWriteVerb {
        id: "datum.pcb.place_zone",
        build: place,
    },
    NativeWriteVerb {
        id: "datum.check.fill_zones",
        build: fill,
    },
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardBasis {
    pub project: Uuid,
    pub board: Uuid,
    pub revision: ModelRevision,
    pub tip: Option<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneBasis {
    pub source: BoardBasis,
    pub net: Uuid,
    pub layer: i32,
    pub clearance: Option<super::stop::ClearancePolicy>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneRequest {
    pub basis: ZoneBasis,
    pub polygon: Polygon,
    pub priority: u32,
    pub thermal_relief: bool,
    pub thermal_gap_nm: i64,
    pub thermal_spoke_width_nm: i64,
    pub edit_seed: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FillRequest {
    pub basis: BoardBasis,
    pub zone: Option<Uuid>,
    pub net: Option<Uuid>,
}

fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(format!("Native PCB Zone: {}", reason.into()))
}

pub fn capture_board_basis(model: &DesignModel) -> Result<BoardBasis, EngineError> {
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    Ok(BoardBasis {
        project: model.project.project_id,
        board: serde_json::from_value(board["uuid"].clone())?,
        revision: model.model_revision.clone(),
        tip: crate::revision::transaction_tip(&model.journal),
    })
}

pub fn capture_zone_basis(
    model: &DesignModel,
    net: Uuid,
    layer: i32,
) -> Result<ZoneBasis, EngineError> {
    let source = capture_board_basis(model)?;
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let net_source: Net = serde_json::from_value(board["nets"][net.to_string()].clone())?;
    if net.is_nil() || net_source.uuid != net {
        return Err(invalid("Net identity differs from source"));
    }
    let stackup: Stackup = serde_json::from_value(board["stackup"].clone())?;
    if stackup.layers.iter().filter(|l| l.id == layer).count() != 1
        || !stackup
            .layers
            .iter()
            .any(|l| l.id == layer && l.layer_type == StackupLayerType::Copper)
    {
        return Err(invalid("Choose one existing copper layer"));
    }
    Ok(ZoneBasis {
        source,
        net,
        layer,
        clearance: super::stop::clearance_policy(model, &net_source, layer)?,
    })
}

/// Transient readback of the existing bounded fill policy, never authored data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneClearanceReadback {
    pub minimum_nm: i64,
    pub sources: Vec<super::stop::ClearancePolicy>,
}
fn required_zone_clearance(basis: &ZoneBasis) -> Result<&super::stop::ClearancePolicy, String> {
    basis
        .clearance
        .as_ref()
        .filter(|p| p.minimum_nm > 0)
        .ok_or_else(|| {
            "Configure a positive persisted clearance Rule or NetClass before filling".into()
        })
}
fn conservative_clearance<'a>(
    model: &DesignModel,
    own: &super::stop::ClearancePolicy,
    layer: i32,
    nets: impl Iterator<Item = &'a Net>,
) -> Result<ZoneClearanceReadback, String> {
    let mut result = ZoneClearanceReadback {
        minimum_nm: own.minimum_nm,
        sources: vec![own.clone()],
    };
    for net in nets {
        if let Some(policy) = super::stop::clearance_policy(model, net, layer)
            .map_err(|e| format!("Native Zone fill source unavailable: {e}"))?
        {
            if policy.minimum_nm <= 0 {
                return Err(format!("Net {} has a nonpositive clearance rule", net.uuid));
            }
            if policy.minimum_nm > result.minimum_nm {
                result.minimum_nm = policy.minimum_nm;
                result.sources.clear();
            }
            if policy.minimum_nm == result.minimum_nm && !result.sources.contains(&policy) {
                result.sources.push(policy);
            }
        }
    }
    Ok(result)
}
/// Read the same conservative policy as Fill, without a boundary, geometry
/// calculation or write. Every source read is qualified by the captured model.
pub fn capture_zone_clearance_readback(
    model: &DesignModel,
    net: Uuid,
    layer: i32,
) -> Result<ZoneClearanceReadback, EngineError> {
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::BoardRoot)
        .ok_or_else(|| invalid("Captured Board source unavailable"))?;
    model.capture_source_shard_value(shard.shard_id)?;
    let basis = capture_zone_basis(model, net, layer)?;
    let own = required_zone_clearance(&basis).map_err(invalid)?;
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Electrical source unavailable: {e:?}")))?;
    let board = snapshot
        .board_source()
        .map_err(|e| invalid(format!("Current copper source unavailable: {e:?}")))?;
    if board.uuid != basis.source.board {
        return Err(invalid("Board source identity changed"));
    }
    conservative_clearance(model, own, layer, board.nets.values()).map_err(invalid)
}

/// One authored boundary only. Unsupported fill geometry may still be authored;
/// its subsequent explicit fill reports Unsupported rather than inventing copper.
pub fn prepare_zone(
    model: &DesignModel,
    provenance: WriteProvenance,
    request: &ZoneRequest,
) -> Result<PreparedWrite, EngineError> {
    if capture_zone_basis(model, request.basis.net, request.basis.layer)? != request.basis {
        return Err(invalid(
            "Zone basis changed; reacquire current board source",
        ));
    }
    occupied_copper::validate_polygon(&request.polygon)
        .map_err(|e| invalid(format!("Boundary unavailable: {e:?}")))?;
    if request.edit_seed.is_nil()
        || !request.polygon.closed
        || request.thermal_gap_nm < 0
        || request.thermal_spoke_width_nm < 0
    {
        return Err(invalid(
            "Boundary needs a retained edit identity, closed polygon and nonnegative thermal dimensions",
        ));
    }
    let id = Uuid::new_v5(&request.edit_seed, b"datum.native.board.zone");
    if model.objects.contains_key(&id)
        || model
            .journal
            .iter()
            .any(|t| t.diff.created.contains(&id) || t.diff.deleted.contains(&id))
    {
        return Err(invalid("Zone edit identity is already used"));
    }
    build_place_board_zone(
        model,
        provenance,
        &Zone {
            uuid: id,
            net: request.basis.net,
            polygon: request.polygon.clone(),
            layer: request.basis.layer,
            priority: request.priority,
            thermal_relief: request.thermal_relief,
            thermal_gap: request.thermal_gap_nm,
            thermal_spoke_width: request.thermal_spoke_width_nm,
        },
    )
}

/// Captures all generated results without writing. The existing bounded solver
/// is used only with geometry its context can faithfully represent.
pub fn preview_fills(
    model: &DesignModel,
    request: &FillRequest,
) -> Result<Vec<ZoneFill>, EngineError> {
    if capture_board_basis(model)? != request.basis {
        return Err(invalid(
            "Fill basis changed; reacquire current board source",
        ));
    }
    let value = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let zones: std::collections::BTreeMap<Uuid, Zone> =
        serde_json::from_value(value["zones"].clone())?;
    if request.zone.is_some_and(|id| !zones.contains_key(&id))
        || request
            .net
            .is_some_and(|id| value["nets"].get(id.to_string()).is_none())
    {
        return Err(invalid("Fill scope has no current source"));
    }
    let selected: Vec<_> = zones
        .values()
        .filter(|z| {
            request.zone.is_none_or(|id| z.uuid == id) && request.net.is_none_or(|id| z.net == id)
        })
        .collect();
    if selected.is_empty() {
        return Err(invalid("No Zone matches the requested scope"));
    }
    selected
        .into_iter()
        .map(|zone| {
            if zone.uuid.is_nil() || zones.get(&zone.uuid) != Some(zone) {
                return Err(invalid("Zone identity differs from source key"));
            }
            let basis = capture_zone_basis(model, zone.net, zone.layer)?;
            let (state, islands, provenance) =
                match fill_context(model, &value, &zones, zone, &basis) {
                    Ok(context) => {
                        let (state, islands, reason) = compute_bounded_zone_fill(zone, &context);
                        (state, islands, format!("{reason}; conservative cutout clearance {} nm is the maximum actual persisted minimum across current board Nets, not a minimal per-pair cut", context.net_clearance_nm[&zone.net]))
                    },
                    Err(reason) => (ZoneFillState::Unsupported, Vec::new(), reason),
                };
            let object = model
                .objects
                .get(&zone.uuid)
                .ok_or_else(|| invalid("Zone revision is unavailable"))?;
            Ok(ZoneFill {
                schema_version: ZONE_FILL_SCHEMA_VERSION,
                zone_id: zone.uuid,
                state,
                islands,
                source_zone_revision: object.object_revision,
                model_revision: model.model_revision.clone(),
                provenance: Some(provenance),
            })
        })
        .collect()
}

pub fn prepare_fill(
    model: &DesignModel,
    provenance: WriteProvenance,
    request: &FillRequest,
) -> Result<PreparedWrite, EngineError> {
    build_set_zone_fills(model, provenance, &preview_fills(model, request)?)
}

fn fill_context(
    model: &DesignModel,
    value: &serde_json::Value,
    zones: &std::collections::BTreeMap<Uuid, Zone>,
    zone: &Zone,
    basis: &ZoneBasis,
) -> Result<ZoneFillCopperContext, String> {
    let own_clearance = required_zone_clearance(basis)?;
    let outline: Polygon =
        serde_json::from_value(value["outline"].clone()).map_err(|e| e.to_string())?;
    let bounds = rectangular_outline(&outline)?;
    occupied_copper::validate_polygon(&zone.polygon)
        .map_err(|e| format!("Zone boundary unavailable: {e:?}"))?;
    if zone.polygon.vertices.iter().any(|p| {
        p.x < bounds.min.x || p.x > bounds.max.x || p.y < bounds.min.y || p.y > bounds.max.y
    }) {
        return Err("Zone requires board-outline clipping unsupported by this fill context".into());
    }
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| format!("Electrical source unavailable: {e:?}"))?;
    let board = snapshot
        .board_source()
        .map_err(|e| format!("Current copper source unavailable: {e:?}"))?;
    if board.uuid != basis.source.board {
        return Err("Board source identity changed".into());
    }
    let mut context = ZoneFillCopperContext {
        tracks: board.tracks.values().cloned().collect(),
        vias: board.vias.values().cloned().collect(),
        keepouts: serde_json::from_value::<Vec<Keepout>>(value["keepouts"].clone())
            .map_err(|e| e.to_string())?,
        ..Default::default()
    };
    for keepout in &context.keepouts {
        occupied_copper::validate_polygon(&keepout.polygon)
            .map_err(|e| format!("Keepout geometry unavailable: {e:?}"))?;
        if !matches!(
            keepout.kind.as_str(),
            "copper" | "route" | "via" | "component"
        ) {
            return Err(format!("Keepout {} has an unsupported kind", keepout.uuid));
        }
    }
    context.keepouts.retain(|k| k.kind == "copper");
    let maximum_clearance =
        conservative_clearance(model, own_clearance, zone.layer, board.nets.values())?.minimum_nm;
    for other in zones
        .values()
        .filter(|other| other.uuid != zone.uuid && other.layer == zone.layer)
    {
        if occupied_copper::polygons_within(
            &zone.polygon,
            &other.polygon,
            maximum_clearance,
            DistanceBoundary::Inclusive,
        )
        .map_err(|e| format!("Zone interaction unavailable: {e:?}"))?
        {
            return Err(format!(
                "Interacting Zone {} requires priority/clearance composition unsupported by this fill context",
                other.uuid
            ));
        }
    }
    // The bounded solver accepts one cutout distance. The largest actual
    // applicable Net minimum conservatively satisfies both sides of a contact.
    context.net_clearance_nm.insert(zone.net, maximum_clearance);
    let mut geometries = board.query_pads.clone();
    for (id, pad) in &board.pads {
        geometries
            .entry(*id)
            .or_insert_with(|| QueryPadGeometry::Integer(pad.clone()));
    }
    for geometry in geometries.values() {
        let layers = occupied_copper::pad_layers(&board.stackup, geometry.terminal())
            .map_err(|e| format!("Pad copper layers unavailable: {e:?}"))?;
        if !layers.contains(&zone.layer) {
            continue;
        }
        let QueryPadGeometry::Integer(pad) = geometry else {
            return Err("Source-relative Pad geometry requires a fill context that can represent it exactly".into());
        };
        let mut pad = pad.clone();
        // This copied obstacle represents the already qualified current layer.
        // The legacy helper's layer-or-membership shortcut cannot add copper.
        pad.layer = zone.layer;
        pad.copper_layers = vec![zone.layer];
        occupied_copper::validate_pad(&pad)
            .map_err(|e| format!("Pad {} geometry unavailable: {e:?}", pad.uuid))?;
        if pad.shape != PadShape::Circle {
            if pad.rotation.rem_euclid(90) != 0 {
                return Err(format!("Pad {} needs rotated obstacle support", pad.uuid));
            }
            if pad.rotation.rem_euclid(180) != 0 {
                std::mem::swap(&mut pad.width, &mut pad.height);
            }
            pad.rotation = 0;
        }
        if (pad.shape == PadShape::Circle && pad.diameter % 2 != 0)
            || (pad.shape != PadShape::Circle && (pad.width % 2 != 0 || pad.height % 2 != 0))
        {
            return Err(format!(
                "Pad {} requires half-nanometre cutout bounds",
                pad.uuid
            ));
        }
        context.pads.push(pad);
    }
    for track in &context.tracks {
        occupied_copper::track_layers(&board.stackup, track)
            .map_err(|e| format!("Track copper layer unavailable: {e:?}"))?;
        occupied_copper::validate_track(track)
            .map_err(|e| format!("Track geometry unavailable: {e:?}"))?;
        if track.width % 2 != 0 {
            return Err(format!(
                "Track {} requires half-nanometre cutout bounds",
                track.uuid
            ));
        }
    }
    context.tracks.retain(|t| t.layer == zone.layer);
    for mut via in std::mem::take(&mut context.vias) {
        let layers = occupied_copper::via_layers(&board.stackup, &via)
            .map_err(|e| format!("Via copper layers unavailable: {e:?}"))?;
        if !layers.contains(&zone.layer) {
            continue;
        }
        occupied_copper::validate_via(&via)
            .map_err(|e| format!("Via geometry unavailable: {e:?}"))?;
        if via.diameter % 2 != 0 {
            return Err(format!(
                "Via {} requires half-nanometre cutout bounds",
                via.uuid
            ));
        }
        // Physical stack order is authoritative, never numeric LayerId order.
        // Adapt only this per-layer copy for the existing bounded solver.
        via.from_layer = zone.layer;
        via.to_layer = zone.layer;
        context.vias.push(via);
    }
    // Legacy bounded arithmetic inflates integer bounds; prove its additions
    // cannot overflow rather than allowing a wrapped obstacle to disappear.
    let margin = context
        .pads
        .iter()
        .map(|p| p.diameter.max(p.width).max(p.height))
        .chain(context.tracks.iter().map(|t| t.width))
        .chain(context.vias.iter().map(|v| v.diameter))
        .max()
        .unwrap_or(0);
    let safe = |n: i64| {
        i128::from(n).abs() + i128::from(margin) + i128::from(maximum_clearance)
            <= i128::from(i64::MAX)
    };
    if context
        .pads
        .iter()
        .any(|p| !safe(p.position.x) || !safe(p.position.y))
        || context
            .tracks
            .iter()
            .any(|t| !safe(t.from.x) || !safe(t.from.y) || !safe(t.to.x) || !safe(t.to.y))
        || context
            .vias
            .iter()
            .any(|v| !safe(v.position.x) || !safe(v.position.y))
        || context
            .keepouts
            .iter()
            .flat_map(|k| &k.polygon.vertices)
            .any(|p| !safe(p.x) || !safe(p.y))
    {
        return Err("Obstacle inflation exceeds bounded integer arithmetic".into());
    }
    Ok(context)
}

fn rectangular_outline(polygon: &Polygon) -> Result<Rect, String> {
    occupied_copper::validate_polygon(polygon)
        .map_err(|e| format!("Define a supported board outline before fill: {e:?}"))?;
    let bounds = polygon.bounding_box().ok_or("Board outline is undefined")?;
    let mut points = polygon.vertices.clone();
    if points.first() == points.last() {
        points.pop();
    }
    let corners: BTreeSet<_> = points.iter().map(|p| (p.x, p.y)).collect();
    if !polygon.closed
        || points.len() != 4
        || corners
            != BTreeSet::from([
                (bounds.min.x, bounds.min.y),
                (bounds.max.x, bounds.min.y),
                (bounds.max.x, bounds.max.y),
                (bounds.min.x, bounds.max.y),
            ])
        || (0..4).any(|i| {
            let a = points[i];
            let b = points[(i + 1) % 4];
            a.x != b.x && a.y != b.y
        })
    {
        return Err(
            "Board outline requires unsupported general clipping; no bounding-box substitution"
                .into(),
        );
    }
    Ok(bounds)
}

fn place(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    prepare_zone(
        context.model,
        provenance,
        &parse_verb_params("datum.pcb.place_zone", params)?,
    )
}
fn fill(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    prepare_fill(
        context.model,
        provenance,
        &parse_verb_params("datum.check.fill_zones", params)?,
    )
}

#[cfg(test)]
#[path = "zone_tests.rs"]
mod tests;
