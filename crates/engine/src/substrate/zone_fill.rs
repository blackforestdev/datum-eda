use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Keepout, PlacedPad, Track, Via, Zone};
use crate::error::EngineError;
use crate::ir::geometry::{LayerId, Point, Polygon, Rect};

use super::generated_evidence::{persist_generated_evidence, validate_filename_uuid};
use super::{
    DesignModel, DomainObject, ModelRevision, ObjectId, ObjectRevision, ResolveDiagnostic,
    SourceShardDirtyState, SourceShardKind, SourceShardRef, materialized_shard_value,
    read_json_value, sha256_hex, source_shard_authority_for_kind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoneFillState {
    Filled,
    Unfilled,
    Stale,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneFill {
    pub zone_id: ObjectId,
    pub state: ZoneFillState,
    pub source_zone_revision: ObjectRevision,
    pub model_revision: ModelRevision,
    pub islands: Vec<Polygon>,
    pub provenance: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ZoneFillCopperContext {
    pub pads: Vec<PlacedPad>,
    pub tracks: Vec<Track>,
    pub vias: Vec<Via>,
    pub keepouts: Vec<Keepout>,
    pub net_clearance_nm: BTreeMap<Uuid, i64>,
    pub has_unresolved_component_pads: bool,
}

pub fn compute_bounded_zone_fill(
    zone: &Zone,
    context: &ZoneFillCopperContext,
) -> (ZoneFillState, Vec<Polygon>, String) {
    if zone.thermal_relief || zone.thermal_gap != 0 || zone.thermal_spoke_width != 0 {
        return unsupported(
            "datum-eda fill-zones: unsupported because thermal relief generation is not implemented",
        );
    }
    if !polygon_has_area(&zone.polygon) {
        return unsupported(
            "datum-eda fill-zones: unsupported because zone polygon is open or degenerate",
        );
    }
    if context.has_unresolved_component_pads {
        return unsupported(
            "datum-eda fill-zones: unsupported because component pads need resolved net attribution before fill",
        );
    }
    if layer_keepout_intersects_zone(&context.keepouts, zone) {
        return unsupported(
            "datum-eda fill-zones: unsupported because a keepout intersects the zone layer",
        );
    }
    let mut foreign_obstacles = Vec::new();
    if let Some(reason) = collect_foreign_pad_obstacles(context, zone, &mut foreign_obstacles) {
        return unsupported(reason);
    }
    if let Some(reason) = collect_foreign_track_obstacles(context, zone, &mut foreign_obstacles) {
        return unsupported(reason);
    }
    collect_foreign_via_obstacles(context, zone, &mut foreign_obstacles);
    if let Some(obstacle) = foreign_obstacles.first() {
        let Some(clearance_nm) = context.net_clearance_nm.get(&zone.net).copied() else {
            return unsupported(
                "datum-eda fill-zones: unsupported because zone net clearance is unavailable",
            );
        };
        if clearance_nm <= 0 {
            return unsupported(
                "datum-eda fill-zones: unsupported because zone net clearance is not positive",
            );
        }
        if foreign_obstacles.len() > 1 {
            let Some(islands) =
                rectangular_multi_cutout_islands(zone, &foreign_obstacles, clearance_nm)
            else {
                return unsupported(
                    "datum-eda fill-zones: unsupported because obstacle cutouts are outside the bounded non-overlapping rectangular solver envelope",
                );
            };
            return (
                ZoneFillState::Filled,
                islands,
                "datum-eda fill-zones: bounded rectangular obstacle cutout fill v3; multiple non-overlapping foreign pads/vias/orthogonal tracks inflated by netclass clearance".to_string(),
            );
        }
        let Some(islands) = rectangular_cutout_islands(zone, *obstacle, clearance_nm) else {
            return unsupported(
                "datum-eda fill-zones: unsupported because obstacle cutout is outside the bounded rectangular solver envelope",
            );
        };
        return (
            ZoneFillState::Filled,
            islands,
            "datum-eda fill-zones: bounded rectangular obstacle cutout fill v2; one foreign pad/via/orthogonal track inflated by netclass clearance".to_string(),
        );
    }
    (
        ZoneFillState::Filled,
        vec![zone.polygon.clone()],
        "datum-eda fill-zones: bounded same-net polygon island fill v1; no clearance subtraction required".to_string(),
    )
}

fn unsupported(reason: &str) -> (ZoneFillState, Vec<Polygon>, String) {
    (ZoneFillState::Unsupported, Vec::new(), reason.to_string())
}

fn polygon_has_area(polygon: &Polygon) -> bool {
    if !polygon.closed || polygon.vertices.len() < 3 {
        return false;
    }
    let Some(bounds) = polygon.bounding_box() else {
        return false;
    };
    if bounds.min.x == bounds.max.x || bounds.min.y == bounds.max.y {
        return false;
    }
    let mut unique = Vec::new();
    for point in &polygon.vertices {
        if !unique.contains(point) {
            unique.push(*point);
        }
    }
    unique.len() >= 3 && !polygon_self_intersects(polygon)
}

fn layer_keepout_intersects_zone(keepouts: &[Keepout], zone: &Zone) -> bool {
    keepouts.iter().any(|keepout| {
        keepout.layers.contains(&zone.layer)
            && polygons_may_intersect(&keepout.polygon, &zone.polygon)
    })
}

fn collect_foreign_pad_obstacles<'a>(
    context: &'a ZoneFillCopperContext,
    zone: &Zone,
    obstacles: &mut Vec<Rect>,
) -> Option<&'a str> {
    for pad in &context.pads {
        if !pad_applies_to_layer(pad, zone.layer) {
            continue;
        }
        let Some(bounds) = pad_bounds(pad) else {
            continue;
        };
        if !rect_intersects_polygon(&bounds, &zone.polygon) {
            continue;
        }
        match pad.net {
            Some(net) if net == zone.net => {}
            Some(_) => obstacles.push(bounds),
            None => {
                return Some(
                    "datum-eda fill-zones: unsupported because an unresolved pad intersects the zone",
                );
            }
        }
    }
    None
}

fn collect_foreign_track_obstacles<'a>(
    context: &'a ZoneFillCopperContext,
    zone: &Zone,
    obstacles: &mut Vec<Rect>,
) -> Option<&'a str> {
    for track in &context.tracks {
        if track.layer != zone.layer || track.net == zone.net {
            continue;
        }
        let bounds = track_bounds(track);
        if !rect_intersects_polygon(&bounds, &zone.polygon) {
            continue;
        }
        if track.width <= 0 {
            return Some(
                "datum-eda fill-zones: unsupported because a different-net track has nonpositive width",
            );
        }
        if track.from.x != track.to.x && track.from.y != track.to.y {
            return Some(
                "datum-eda fill-zones: unsupported because a non-orthogonal different-net track intersects the zone",
            );
        }
        obstacles.push(bounds);
    }
    None
}

fn collect_foreign_via_obstacles(
    context: &ZoneFillCopperContext,
    zone: &Zone,
    obstacles: &mut Vec<Rect>,
) {
    for via in &context.vias {
        if via_applies_to_layer(via, zone.layer) && via.net != zone.net {
            let bounds = via_bounds(via);
            if rect_intersects_polygon(&bounds, &zone.polygon) {
                obstacles.push(bounds);
            }
        }
    }
}

fn pad_applies_to_layer(pad: &PlacedPad, layer: LayerId) -> bool {
    pad.layer == layer || pad.copper_layers.contains(&layer)
}

fn via_applies_to_layer(via: &Via, layer: LayerId) -> bool {
    layer >= via.from_layer.min(via.to_layer) && layer <= via.from_layer.max(via.to_layer)
}

fn pad_bounds(pad: &PlacedPad) -> Option<Rect> {
    let half_width = match pad.shape {
        crate::board::PadShape::Circle => pad.diameter / 2,
        crate::board::PadShape::Rect
        | crate::board::PadShape::Oval
        | crate::board::PadShape::RoundRect => pad.width / 2,
    };
    let half_height = match pad.shape {
        crate::board::PadShape::Circle => pad.diameter / 2,
        crate::board::PadShape::Rect
        | crate::board::PadShape::Oval
        | crate::board::PadShape::RoundRect => pad.height / 2,
    };
    if half_width <= 0 || half_height <= 0 {
        return None;
    }
    Some(Rect::new(
        Point::new(pad.position.x - half_width, pad.position.y - half_height),
        Point::new(pad.position.x + half_width, pad.position.y + half_height),
    ))
}

fn track_bounds(track: &Track) -> Rect {
    let half = (track.width / 2).max(0);
    Rect::new(
        Point::new(
            track.from.x.min(track.to.x) - half,
            track.from.y.min(track.to.y) - half,
        ),
        Point::new(
            track.from.x.max(track.to.x) + half,
            track.from.y.max(track.to.y) + half,
        ),
    )
}

fn via_bounds(via: &Via) -> Rect {
    let half = (via.diameter / 2).max(0);
    Rect::new(
        Point::new(via.position.x - half, via.position.y - half),
        Point::new(via.position.x + half, via.position.y + half),
    )
}

fn rectangular_cutout_islands(
    zone: &Zone,
    obstacle: Rect,
    clearance_nm: i64,
) -> Option<Vec<Polygon>> {
    let zone_bounds = rectangular_polygon_bounds(&zone.polygon)?;
    let inflated = inflate_rect(obstacle, clearance_nm);
    if inflated.min.x <= zone_bounds.min.x
        || inflated.max.x >= zone_bounds.max.x
        || inflated.min.y <= zone_bounds.min.y
        || inflated.max.y >= zone_bounds.max.y
    {
        return None;
    }
    let mut islands = Vec::new();
    push_rect_island(
        &mut islands,
        Rect::new(
            zone_bounds.min,
            Point::new(inflated.min.x, zone_bounds.max.y),
        ),
    );
    push_rect_island(
        &mut islands,
        Rect::new(
            Point::new(inflated.max.x, zone_bounds.min.y),
            zone_bounds.max,
        ),
    );
    push_rect_island(
        &mut islands,
        Rect::new(
            Point::new(inflated.min.x, zone_bounds.min.y),
            Point::new(inflated.max.x, inflated.min.y),
        ),
    );
    push_rect_island(
        &mut islands,
        Rect::new(
            Point::new(inflated.min.x, inflated.max.y),
            Point::new(inflated.max.x, zone_bounds.max.y),
        ),
    );
    (!islands.is_empty()).then_some(islands)
}

fn rectangular_multi_cutout_islands(
    zone: &Zone,
    obstacles: &[Rect],
    clearance_nm: i64,
) -> Option<Vec<Polygon>> {
    let zone_bounds = rectangular_polygon_bounds(&zone.polygon)?;
    let mut inflated_obstacles = Vec::new();
    for obstacle in obstacles {
        let inflated = inflate_rect(*obstacle, clearance_nm);
        if inflated.min.x <= zone_bounds.min.x
            || inflated.max.x >= zone_bounds.max.x
            || inflated.min.y <= zone_bounds.min.y
            || inflated.max.y >= zone_bounds.max.y
        {
            return None;
        }
        inflated_obstacles.push(inflated);
    }
    if rects_overlap_or_touch(&inflated_obstacles) {
        return None;
    }

    let mut x_edges = vec![zone_bounds.min.x, zone_bounds.max.x];
    let mut y_edges = vec![zone_bounds.min.y, zone_bounds.max.y];
    for obstacle in &inflated_obstacles {
        x_edges.push(obstacle.min.x);
        x_edges.push(obstacle.max.x);
        y_edges.push(obstacle.min.y);
        y_edges.push(obstacle.max.y);
    }
    x_edges.sort_unstable();
    x_edges.dedup();
    y_edges.sort_unstable();
    y_edges.dedup();

    let mut islands = Vec::new();
    for x_pair in x_edges.windows(2) {
        for y_pair in y_edges.windows(2) {
            let cell = Rect::new(
                Point::new(x_pair[0], y_pair[0]),
                Point::new(x_pair[1], y_pair[1]),
            );
            if rect_is_covered_by_any(&cell, &inflated_obstacles) {
                continue;
            }
            push_rect_island(&mut islands, cell);
        }
    }
    (!islands.is_empty()).then_some(islands)
}

fn rects_overlap_or_touch(rects: &[Rect]) -> bool {
    for (index, rect) in rects.iter().enumerate() {
        if rects
            .iter()
            .skip(index + 1)
            .any(|other| rect.intersects(other))
        {
            return true;
        }
    }
    false
}

fn rect_is_covered_by_any(cell: &Rect, obstacles: &[Rect]) -> bool {
    obstacles.iter().any(|obstacle| {
        cell.min.x >= obstacle.min.x
            && cell.max.x <= obstacle.max.x
            && cell.min.y >= obstacle.min.y
            && cell.max.y <= obstacle.max.y
    })
}

fn rectangular_polygon_bounds(polygon: &Polygon) -> Option<Rect> {
    if !polygon.closed || polygon.vertices.len() != 4 {
        return None;
    }
    let bounds = polygon.bounding_box()?;
    let corners = [
        bounds.min,
        Point::new(bounds.max.x, bounds.min.y),
        bounds.max,
        Point::new(bounds.min.x, bounds.max.y),
    ];
    corners
        .iter()
        .all(|corner| polygon.vertices.contains(corner))
        .then_some(bounds)
}

fn inflate_rect(rect: Rect, amount: i64) -> Rect {
    Rect::new(
        Point::new(rect.min.x - amount, rect.min.y - amount),
        Point::new(rect.max.x + amount, rect.max.y + amount),
    )
}

fn push_rect_island(islands: &mut Vec<Polygon>, rect: Rect) {
    if rect.min.x >= rect.max.x || rect.min.y >= rect.max.y {
        return;
    }
    islands.push(Polygon {
        vertices: vec![
            rect.min,
            Point::new(rect.max.x, rect.min.y),
            rect.max,
            Point::new(rect.min.x, rect.max.y),
        ],
        closed: true,
    });
}

fn rect_intersects_polygon(rect: &Rect, polygon: &Polygon) -> bool {
    polygon
        .bounding_box()
        .is_some_and(|bounds| rect.intersects(&bounds))
}

fn polygons_may_intersect(a: &Polygon, b: &Polygon) -> bool {
    match (a.bounding_box(), b.bounding_box()) {
        (Some(a_bounds), Some(b_bounds)) => a_bounds.intersects(&b_bounds),
        _ => false,
    }
}

pub fn persist_zone_fill(
    project_root: &Path,
    fill: &ZoneFill,
) -> Result<PathBuf, crate::error::EngineError> {
    persist_generated_evidence(project_root, ".datum/zone_fills", &fill.zone_id, fill)
}

pub(super) fn validated_zone_fill_payload(
    expected_zone_id: Uuid,
    value: &serde_json::Value,
) -> Result<ZoneFill, EngineError> {
    let fill = serde_json::from_value::<ZoneFill>(value.clone())?;
    if fill.zone_id != expected_zone_id {
        return Err(EngineError::Validation(format!(
            "zone fill payload id {} does not match operation zone_id {}",
            fill.zone_id, expected_zone_id
        )));
    }
    validate_zone_fill(&fill).map_err(EngineError::Validation)?;
    Ok(fill)
}

pub fn zone_fill_copper_projection_zones(
    authored_zones: &[Zone],
    zone_fills: &BTreeMap<Uuid, ZoneFill>,
) -> (Vec<Zone>, Vec<String>) {
    let mut rendered_zones = Vec::new();
    let mut blocked_zone_ids = Vec::new();
    for zone in authored_zones {
        let Some(fill) = zone_fills.get(&zone.uuid) else {
            blocked_zone_ids.push(zone.uuid.to_string());
            continue;
        };
        if fill.state != ZoneFillState::Filled {
            blocked_zone_ids.push(zone.uuid.to_string());
            continue;
        }
        for (index, island) in fill.islands.iter().enumerate() {
            let mut rendered = zone.clone();
            rendered.uuid = Uuid::new_v5(
                &zone.uuid,
                format!("datum-eda:zone-fill-island:{index}").as_bytes(),
            );
            rendered.polygon = island.clone();
            rendered_zones.push(rendered);
        }
    }
    rendered_zones.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.polygon.vertices.len().cmp(&b.polygon.vertices.len()))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    blocked_zone_ids.sort();
    (rendered_zones, blocked_zone_ids)
}

#[derive(Debug, Deserialize)]
struct BoardZoneRoot {
    #[serde(default)]
    zones: BTreeMap<Uuid, Zone>,
}

pub(super) fn derive_model_zone_fills(
    model: &DesignModel,
    persisted_fills: BTreeMap<Uuid, ZoneFill>,
) -> Result<BTreeMap<Uuid, ZoneFill>, EngineError> {
    let Some(board_shard) = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
    else {
        return Ok(BTreeMap::new());
    };
    let value = materialized_shard_value(model, board_shard)?;
    let board: BoardZoneRoot = serde_json::from_value(value)?;
    Ok(derive_zone_fills(
        &board.zones,
        &model.objects,
        &model.model_revision,
        persisted_fills,
    ))
}

fn derive_zone_fills(
    zones: &BTreeMap<Uuid, Zone>,
    objects: &BTreeMap<ObjectId, DomainObject>,
    model_revision: &ModelRevision,
    persisted_fills: BTreeMap<Uuid, ZoneFill>,
) -> BTreeMap<Uuid, ZoneFill> {
    let mut fills = BTreeMap::new();
    for zone_id in zones.keys() {
        let source_zone_revision = objects
            .get(zone_id)
            .map(|object| object.object_revision)
            .unwrap_or(ObjectRevision(0));
        if let Some(mut fill) = persisted_fills.get(zone_id).cloned() {
            if fill.model_revision != *model_revision
                || fill.source_zone_revision != source_zone_revision
            {
                fill.state = ZoneFillState::Stale;
            }
            fills.insert(*zone_id, fill);
            continue;
        }
        fills.insert(
            *zone_id,
            ZoneFill {
                zone_id: *zone_id,
                state: ZoneFillState::Unfilled,
                source_zone_revision,
                model_revision: model_revision.clone(),
                islands: Vec::new(),
                provenance: None,
            },
        );
    }
    fills
}

pub(super) fn read_zone_fill_shards(
    project_root: &Path,
) -> (
    Vec<SourceShardRef>,
    BTreeMap<Uuid, ZoneFill>,
    Vec<ResolveDiagnostic>,
) {
    let fill_dir = project_root.join(".datum/zone_fills");
    let mut shards = Vec::new();
    let mut fills = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let Ok(entries) = std::fs::read_dir(&fill_dir) else {
        return (shards, fills, diagnostics);
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        let Some(filename) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let relative_path = format!(".datum/zone_fills/{filename}");
        let path = project_root.join(&relative_path);
        match read_zone_fill_shard(path, relative_path) {
            Ok((shard, fill)) => {
                fills.insert(fill.zone_id, fill);
                shards.push(shard);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    (shards, fills, diagnostics)
}

fn read_zone_fill_shard(
    path: PathBuf,
    relative_path: String,
) -> Result<(SourceShardRef, ZoneFill), ResolveDiagnostic> {
    let bytes = std::fs::read(&path).map_err(|error| ResolveDiagnostic {
        code: "missing_zone_fill".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let value = read_json_value(&path).map_err(|error| ResolveDiagnostic {
        code: "invalid_zone_fill".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let schema_version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64);
    let shard = SourceShardRef {
        shard_id: Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:source-shard:{relative_path}").as_bytes(),
        ),
        kind: SourceShardKind::ZoneFill,
        path,
        relative_path,
        authority: source_shard_authority_for_kind(&SourceShardKind::ZoneFill),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    };
    let fill = serde_json::from_value::<ZoneFill>(value).map_err(|error| ResolveDiagnostic {
        code: "invalid_zone_fill".to_string(),
        message: error.to_string(),
        path: Some(shard.path.clone()),
    })?;
    validate_filename_uuid(&shard.path, fill.zone_id, "invalid_zone_fill")?;
    validate_zone_fill(&fill).map_err(|message| ResolveDiagnostic {
        code: "invalid_zone_fill".to_string(),
        message,
        path: Some(shard.path.clone()),
    })?;
    Ok((shard, fill))
}

fn validate_zone_fill(fill: &ZoneFill) -> Result<(), String> {
    match fill.state {
        ZoneFillState::Filled => {
            if fill.islands.is_empty() {
                return Err("filled zone fill must contain at least one island".to_string());
            }
            if fill.provenance.as_deref().unwrap_or("").trim().is_empty() {
                return Err("filled zone fill must record provenance".to_string());
            }
            for (index, island) in fill.islands.iter().enumerate() {
                validate_filled_island(island, index)?;
            }
        }
        ZoneFillState::Unfilled | ZoneFillState::Unsupported => {
            if !fill.islands.is_empty() {
                return Err(format!(
                    "{:?} zone fill must not contain renderable islands",
                    fill.state
                ));
            }
        }
        ZoneFillState::Stale => {}
    }
    Ok(())
}

fn validate_filled_island(island: &Polygon, index: usize) -> Result<(), String> {
    if !island.closed {
        return Err(format!("filled zone island {index} must be closed"));
    }
    if island.vertices.len() < 3 {
        return Err(format!(
            "filled zone island {index} must have at least three vertices"
        ));
    }
    let Some(bounds) = island.bounding_box() else {
        return Err(format!("filled zone island {index} must have bounds"));
    };
    if bounds.min.x == bounds.max.x || bounds.min.y == bounds.max.y {
        return Err(format!(
            "filled zone island {index} must have non-zero area bounds"
        ));
    }
    let mut unique = Vec::new();
    for point in &island.vertices {
        if !unique.contains(point) {
            unique.push(*point);
        }
    }
    if unique.len() < 3 {
        return Err(format!(
            "filled zone island {index} must have at least three distinct vertices"
        ));
    }
    if polygon_self_intersects(island) {
        return Err(format!(
            "filled zone island {index} must not self-intersect"
        ));
    }
    Ok(())
}

fn polygon_self_intersects(polygon: &Polygon) -> bool {
    let edge_count = polygon.vertices.len();
    if edge_count < 4 {
        return false;
    }
    for edge_a in 0..edge_count {
        let a0 = polygon.vertices[edge_a];
        let a1 = polygon.vertices[(edge_a + 1) % edge_count];
        for edge_b in (edge_a + 1)..edge_count {
            if edges_are_adjacent(edge_a, edge_b, edge_count) {
                continue;
            }
            let b0 = polygon.vertices[edge_b];
            let b1 = polygon.vertices[(edge_b + 1) % edge_count];
            if segments_intersect(a0, a1, b0, b1) {
                return true;
            }
        }
    }
    false
}

fn edges_are_adjacent(a: usize, b: usize, edge_count: usize) -> bool {
    a == b || a + 1 == b || (a == 0 && b + 1 == edge_count)
}

fn segments_intersect(a0: Point, a1: Point, b0: Point, b1: Point) -> bool {
    let o1 = orientation(a0, a1, b0);
    let o2 = orientation(a0, a1, b1);
    let o3 = orientation(b0, b1, a0);
    let o4 = orientation(b0, b1, a1);

    if o1 != o2 && o3 != o4 {
        return true;
    }

    (o1 == 0 && point_on_segment(b0, a0, a1))
        || (o2 == 0 && point_on_segment(b1, a0, a1))
        || (o3 == 0 && point_on_segment(a0, b0, b1))
        || (o4 == 0 && point_on_segment(a1, b0, b1))
}

fn point_on_segment(point: Point, from: Point, to: Point) -> bool {
    point.x >= from.x.min(to.x)
        && point.x <= from.x.max(to.x)
        && point.y >= from.y.min(to.y)
        && point.y <= from.y.max(to.y)
}

fn orientation(a: Point, b: Point, c: Point) -> i32 {
    let cross =
        (b.y - a.y) as i128 * (c.x - b.x) as i128 - (b.x - a.x) as i128 * (c.y - b.y) as i128;
    if cross == 0 {
        0
    } else if cross > 0 {
        1
    } else {
        2
    }
}
