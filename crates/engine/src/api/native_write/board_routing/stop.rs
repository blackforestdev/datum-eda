//! Conservative manual routing over captured, current occupied copper.
//! This is Stop/refuse admission, not shove, automatic routing or full DRC.
use super::route::{RouteBasis, capture_route_basis, selected_rule};
use crate::board::{
    Keepout, Net, NetClass, Track, nominal_geometry::DistanceBoundary, occupied_copper,
    query_pad::QueryPadGeometry, track_contact::tracks_within,
};
use crate::connectivity::BoardCopperSource;
use crate::error::EngineError;
use crate::ir::geometry::{Point, Polygon, Rect};
use crate::substrate::{DesignModel, ElectricalSelectionSnapshot, SourceShardKind};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClearanceSource {
    Rule,
    NetClass,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClearancePolicy {
    pub minimum_nm: i64,
    pub source: Uuid,
    pub kind: ClearanceSource,
    pub name: String,
}

fn invalid(message: impl Into<String>) -> EngineError {
    EngineError::Validation(format!("PCB routing Stop: {}", message.into()))
}

pub(super) fn clearance_policy(
    model: &DesignModel,
    net: &Net,
    layer: i32,
) -> Result<Option<ClearancePolicy>, EngineError> {
    if let Some(rule) = selected_rule(model, "clearance_copper", net, layer)? {
        return Ok(Some(ClearancePolicy {
            minimum_nm: rule["params"]["min_nm"]
                .as_i64()
                .ok_or_else(|| invalid("Clearance rule has no integer minimum"))?,
            source: serde_json::from_value(rule["uuid"].clone())?,
            kind: ClearanceSource::Rule,
            name: rule["name"]
                .as_str()
                .ok_or_else(|| invalid("Clearance rule name is unavailable"))?
                .into(),
        }));
    }
    if net.class.is_nil() {
        return Ok(None);
    }
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let class: NetClass =
        serde_json::from_value(board["net_classes"][net.class.to_string()].clone())?;
    if class.uuid != net.class {
        return Err(invalid("Net class identity differs from source"));
    }
    if class.clearance <= 0 {
        return Ok(None);
    }
    Ok(Some(ClearancePolicy {
        minimum_nm: class.clearance,
        source: class.uuid,
        kind: ClearanceSource::NetClass,
        name: class.name,
    }))
}

/// Frozen read context reusable for transient preview. It grants no mutation
/// authority: acceptance rebuilds it against the exact current RouteBasis.
#[derive(Clone, Debug)]
pub struct StopContext {
    basis: RouteBasis,
    board: BoardCopperSource,
    clearances: BTreeMap<Uuid, Option<ClearancePolicy>>,
    islands: BTreeMap<Uuid, Vec<Polygon>>,
    keepouts: Vec<Keepout>,
    outline: Option<Rect>,
}

impl StopContext {
    pub fn capture(model: &DesignModel, basis: &RouteBasis) -> Result<Self, EngineError> {
        if capture_route_basis(model, basis.start_pad, basis.layer)? != *basis {
            return Err(invalid("Source changed; restart the route"));
        }
        if basis.clearance.as_ref().is_none_or(|p| p.minimum_nm <= 0) {
            return Err(invalid(format!(
                "Configure an explicit clearance rule for Net {} before routing",
                basis.net
            )));
        }
        let snapshot = ElectricalSelectionSnapshot::capture(model)
            .map_err(|e| invalid(format!("Native source unavailable: {e:?}")))?;
        let board = snapshot
            .board_source()
            .map_err(|e| invalid(format!("Board copper unavailable: {e:?}")))?;
        if board.uuid != basis.board {
            return Err(invalid("Board source identity changed"));
        }
        let shard = model
            .objects
            .get(&board.uuid)
            .ok_or_else(|| invalid("Board source is unregistered"))?
            .source_shard_id;
        let value = snapshot
            .source_shard_value(&basis.revision, shard)
            .map_err(|e| invalid(format!("Board source is unavailable: {e:?}")))?;
        let keepouts = serde_json::from_value(value["keepouts"].clone())?;
        let outline = rectangular_outline(&serde_json::from_value(value["outline"].clone())?)?;
        let mut clearances = BTreeMap::new();
        for net in board.nets.values() {
            clearances.insert(net.uuid, clearance_policy(model, net, basis.layer)?);
        }
        let mut islands = BTreeMap::new();
        for zone in board
            .zones
            .values()
            .filter(|z| z.layer == basis.layer && z.net != basis.net)
        {
            let fill = model.current_zone_fill(zone.uuid).ok_or_else(|| {
                invalid(format!(
                    "Zone {} has no authenticated current copper fill; refill it before routing",
                    zone.uuid
                ))
            })?;
            islands.insert(zone.uuid, fill.islands.clone());
        }
        Ok(Self {
            basis: basis.clone(),
            board,
            clearances,
            islands,
            keepouts,
            outline,
        })
    }

    /// An undefined outline is exposed to the caller, never represented as a
    /// successful board-edge check or a manufactured default board rectangle.
    pub fn outline_defined(&self) -> bool {
        self.outline.is_some()
    }

    pub fn basis(&self) -> &RouteBasis {
        &self.basis
    }

    pub fn preview_path(&self, points: &[Point], width_nm: i64) -> Result<(), EngineError> {
        if points.first() != Some(&self.basis.start) {
            return Err(invalid("Preview must start at the captured Pad centre"));
        }
        let tracks: Vec<_> = points
            .windows(2)
            .map(|pair| {
                Track::straight(
                    Uuid::nil(),
                    self.basis.net,
                    pair[0],
                    pair[1],
                    width_nm,
                    self.basis.layer,
                )
            })
            .collect();
        self.validate_tracks(&tracks)
    }

    fn clearance(&self, other: Option<Uuid>) -> Result<i64, EngineError> {
        let own = self
            .basis
            .clearance
            .as_ref()
            .ok_or_else(|| invalid("Clearance is unconfigured"))?
            .minimum_nm;
        let Some(other) = other.filter(|id| !id.is_nil()) else {
            return Ok(own);
        };
        let foreign = self
            .clearances
            .get(&other)
            .ok_or_else(|| invalid(format!("Foreign Net {other} has no current source")))?;
        // A missing foreign policy contributes no invented numeric default.
        // The route's persisted policy remains mandatory and any stronger
        // configured foreign requirement is retained, as in native pair checks.
        Ok(foreign
            .as_ref()
            .map_or(own, |policy| own.max(policy.minimum_nm)))
    }

    pub(super) fn validate_tracks(&self, tracks: &[Track]) -> Result<(), EngineError> {
        if tracks.is_empty() {
            return Err(invalid("The route has no segments"));
        }
        for (index, track) in tracks.iter().enumerate() {
            if track.net != self.basis.net
                || track.layer != self.basis.layer
                || track.midpoint.is_some()
                || track.from == track.to
                || track.width <= 0
                || self
                    .basis
                    .minimum_width_nm
                    .is_some_and(|min| track.width < min)
                || self
                    .basis
                    .maximum_width_nm
                    .is_some_and(|max| track.width > max)
            {
                return Err(invalid(format!(
                    "Segment {index} has unsupported identity, layer, width or geometry"
                )));
            }
            occupied_copper::validate_track(track)
                .map_err(|e| invalid(format!("Segment {index} geometry unavailable: {e:?}")))?;
            let radius = track.width / 2 + track.width % 2;
            if [track.from.x, track.from.y, track.to.x, track.to.y]
                .into_iter()
                .any(|v| v.checked_sub(radius).is_none() || v.checked_add(radius).is_none())
            {
                return Err(invalid(format!(
                    "Segment {index} copper extent exceeds integer coordinates"
                )));
            }
            if self
                .outline
                .is_some_and(|rect| !inside_rectangle(track, rect))
            {
                return Err(invalid(format!(
                    "Segment {index} copper extends outside the board outline"
                )));
            }
            self.track_obstacles(track, index)?;
            self.pad_obstacles(track, index)?;
            self.via_obstacles(track, index)?;
            for zone in self
                .board
                .zones
                .values()
                .filter(|z| z.layer == track.layer && z.net != track.net)
            {
                let clearance = self.clearance(Some(zone.net))?;
                let islands = self
                    .islands
                    .get(&zone.uuid)
                    .ok_or_else(|| invalid(format!("Zone {} copper is unavailable", zone.uuid)))?;
                for island in islands {
                    blocked(
                        index,
                        zone.uuid,
                        "filled Zone",
                        |gap, boundary| {
                            occupied_copper::track_polygon_within(track, island, gap, boundary)
                        },
                        clearance,
                    )?;
                }
            }
            for keepout in self
                .keepouts
                .iter()
                .filter(|k| k.layers.contains(&track.layer))
            {
                match keepout.kind.as_str() {
                    "route" | "copper" => {}
                    "via" | "component" => continue,
                    _ => {
                        return Err(invalid(format!(
                            "Keepout {} has unsupported kind {:?}; track applicability is unavailable",
                            keepout.uuid, keepout.kind
                        )));
                    }
                }
                let contact = occupied_copper::track_polygon_within(
                    track,
                    &keepout.polygon,
                    0,
                    DistanceBoundary::Inclusive,
                )
                .map_err(|e| {
                    invalid(format!(
                        "Keepout {} geometry unavailable: {e:?}",
                        keepout.uuid
                    ))
                })?;
                if contact {
                    return Err(invalid(format!(
                        "Segment {index} contacts Keepout {}",
                        keepout.uuid
                    )));
                }
            }
        }
        Ok(())
    }

    fn track_obstacles(&self, track: &Track, index: usize) -> Result<(), EngineError> {
        for other in self.board.tracks.values() {
            let layers = occupied_copper::track_layers(&self.board.stackup, other)
                .map_err(|e| invalid(format!("Track {} layers unavailable: {e:?}", other.uuid)))?;
            if other.net == track.net || !layers.contains(&track.layer) {
                continue;
            }
            blocked(
                index,
                other.uuid,
                "Track",
                |gap, boundary| tracks_within(track, other, gap, boundary),
                self.clearance(Some(other.net))?,
            )?;
        }
        Ok(())
    }

    fn pad_obstacles(&self, track: &Track, index: usize) -> Result<(), EngineError> {
        let ids: BTreeSet<_> = self
            .board
            .pads
            .keys()
            .chain(self.board.query_pads.keys())
            .copied()
            .collect();
        for id in ids {
            let geometry = self
                .board
                .query_pads
                .get(&id)
                .cloned()
                .or_else(|| {
                    self.board
                        .pads
                        .get(&id)
                        .cloned()
                        .map(QueryPadGeometry::Integer)
                })
                .ok_or_else(|| invalid(format!("Pad {id} geometry is unavailable")))?;
            let pad = geometry.terminal();
            let layers = occupied_copper::pad_layers(&self.board.stackup, pad)
                .map_err(|e| invalid(format!("Pad {id} layers unavailable: {e:?}")))?;
            if pad.net == Some(track.net) || !layers.contains(&track.layer) {
                continue;
            }
            let clearance = self.clearance(pad.net)?;
            blocked(
                index,
                id,
                "Pad",
                |gap, boundary| match &geometry {
                    QueryPadGeometry::Integer(pad) => {
                        occupied_copper::track_pad_within(track, pad, gap, boundary)
                    }
                    QueryPadGeometry::SourceRelative(pad) => {
                        occupied_copper::source_relative::track(pad, track, gap, boundary)
                    }
                },
                clearance,
            )?;
        }
        Ok(())
    }

    fn via_obstacles(&self, track: &Track, index: usize) -> Result<(), EngineError> {
        for via in self.board.vias.values() {
            let layers = occupied_copper::via_layers(&self.board.stackup, via)
                .map_err(|e| invalid(format!("Via {} span unavailable: {e:?}", via.uuid)))?;
            if via.net == track.net || !layers.contains(&track.layer) {
                continue;
            }
            blocked(
                index,
                via.uuid,
                "Via",
                |gap, boundary| occupied_copper::track_via_within(track, via, gap, boundary),
                self.clearance(Some(via.net))?,
            )?;
        }
        Ok(())
    }
}

fn blocked(
    segment: usize,
    object: Uuid,
    kind: &str,
    predicate: impl Fn(
        i64,
        DistanceBoundary,
    ) -> std::result::Result<bool, crate::board::nominal_geometry::GeometryError>,
    clearance: i64,
) -> Result<(), EngineError> {
    for (gap, boundary) in [
        (0, DistanceBoundary::Inclusive),
        (clearance, DistanceBoundary::Strict),
    ] {
        match predicate(gap, boundary) {
            Ok(true) => {
                return Err(invalid(format!(
                    "Segment {segment} contacts or violates clearance to {kind} {object}"
                )));
            }
            Ok(false) => {}
            Err(reason) => {
                return Err(invalid(format!(
                    "{kind} {object} geometry unavailable: {reason:?}"
                )));
            }
        }
    }
    Ok(())
}

fn rectangular_outline(polygon: &Polygon) -> Result<Option<Rect>, EngineError> {
    if polygon.vertices.is_empty() {
        return Ok(None);
    }
    occupied_copper::validate_polygon(polygon)
        .map_err(|e| invalid(format!("Board outline is unavailable: {e:?}")))?;
    let mut vertices = polygon.vertices.clone();
    if vertices.first() == vertices.last() {
        vertices.pop();
    }
    let bounds = polygon
        .bounding_box()
        .ok_or_else(|| invalid("Board outline bounds are unavailable"))?;
    let corners: BTreeSet<_> = vertices.iter().map(|p| (p.x, p.y)).collect();
    let expected = BTreeSet::from([
        (bounds.min.x, bounds.min.y),
        (bounds.max.x, bounds.min.y),
        (bounds.max.x, bounds.max.y),
        (bounds.min.x, bounds.max.y),
    ]);
    if !polygon.closed
        || vertices.len() != 4
        || corners != expected
        || (0..4).any(|i| {
            let a = vertices[i];
            let b = vertices[(i + 1) % 4];
            a.x != b.x && a.y != b.y
        })
    {
        return Err(invalid(
            "This board outline lacks supported full-stroke containment; no approximate edge check is used",
        ));
    }
    Ok(Some(bounds))
}

fn inside_rectangle(track: &Track, rectangle: Rect) -> bool {
    let doubled = |value: i64| 2 * i128::from(value);
    let width = i128::from(track.width);
    // Exact capsule extents, including odd widths. This proves containment in
    // this actual rectangle, not an arbitrary polygon or manufacturing edge gap.
    doubled(track.from.x.min(track.to.x)) - width >= doubled(rectangle.min.x)
        && doubled(track.from.x.max(track.to.x)) + width <= doubled(rectangle.max.x)
        && doubled(track.from.y.min(track.to.y)) - width >= doubled(rectangle.min.y)
        && doubled(track.from.y.max(track.to.y)) + width <= doubled(rectangle.max.y)
}

#[cfg(test)]
#[path = "stop_tests.rs"]
mod tests;
