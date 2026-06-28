use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::board::{Keepout, PlacedPad, Track, Via, Zone};
use crate::error::EngineError;
use crate::ir::geometry::Polygon;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::generated_evidence::{persist_generated_evidence, validate_filename_uuid};
use super::zone_fill_geometry::{
    collect_foreign_pad_obstacles, collect_foreign_track_obstacles, collect_foreign_via_obstacles,
    collect_keepout_obstacles, has_same_net_thermal_anchor, polygon_has_area,
    polygon_self_intersects, rectangular_cutout_islands, rectangular_multi_cutout_islands,
};
use super::{
    DesignModel, DomainObject, ModelRevision, ObjectId, ObjectRevision, ResolveDiagnostic,
    SourceShardKind, SourceShardRef, materialized_shard_value, read_json_value,
    source_shard_ref_builders::source_shard_ref_for_bytes,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoneFillState {
    Filled,
    Unfilled,
    Stale,
    Unsupported,
}

pub const ZONE_FILL_SCHEMA_VERSION: u64 = 1;

fn default_zone_fill_schema_version() -> u64 {
    ZONE_FILL_SCHEMA_VERSION
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneFill {
    #[serde(default = "default_zone_fill_schema_version")]
    pub schema_version: u64,
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
}

pub fn compute_bounded_zone_fill(
    zone: &Zone,
    context: &ZoneFillCopperContext,
) -> (ZoneFillState, Vec<Polygon>, String) {
    let thermal_requested =
        zone.thermal_relief || zone.thermal_gap != 0 || zone.thermal_spoke_width != 0;
    if thermal_requested && has_same_net_thermal_anchor(context, zone) {
        return unsupported(
            "datum-eda fill-zones: unsupported because thermal relief generation for same-net pad/via anchors is not implemented",
        );
    }
    if !polygon_has_area(&zone.polygon) {
        return unsupported(
            "datum-eda fill-zones: unsupported because zone polygon is open or degenerate",
        );
    }
    let mut foreign_obstacles = Vec::new();
    let keepout_obstacle_count = collect_keepout_obstacles(context, zone, &mut foreign_obstacles);
    if let Some(reason) = collect_foreign_pad_obstacles(context, zone, &mut foreign_obstacles) {
        return unsupported(reason);
    }
    let has_non_orthogonal_track =
        match collect_foreign_track_obstacles(context, zone, &mut foreign_obstacles) {
            Ok(has_non_orthogonal_track) => has_non_orthogonal_track,
            Err(reason) => return unsupported(reason),
        };
    collect_foreign_via_obstacles(context, zone, &mut foreign_obstacles);
    if let Some(obstacle) = foreign_obstacles.first() {
        let has_foreign_copper_obstacles = foreign_obstacles.len() > keepout_obstacle_count;
        let clearance_nm = if has_foreign_copper_obstacles {
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
            clearance_nm
        } else {
            0
        };
        if foreign_obstacles.len() > 1 {
            let Some((islands, merged_obstacles, clipped_obstacles)) =
                rectangular_multi_cutout_islands(zone, &foreign_obstacles, clearance_nm)
            else {
                return unsupported(
                    "datum-eda fill-zones: unsupported because obstacle cutouts cover the bounded rectangular solver envelope",
                );
            };
            let provenance = if keepout_obstacle_count > 0 {
                "datum-eda fill-zones: bounded rectangular obstacle cutout fill v6; keepout bounds removed from copper before fill"
            } else if has_non_orthogonal_track {
                "datum-eda fill-zones: bounded rectangular obstacle cutout fill v7; non-orthogonal foreign track bounds conservatively removed before fill"
            } else if clipped_obstacles {
                "datum-eda fill-zones: bounded rectangular obstacle cutout fill v5; edge-crossing foreign pad/via/orthogonal-track clearances clipped to zone bounds before fill"
            } else if merged_obstacles {
                "datum-eda fill-zones: bounded rectangular obstacle cutout fill v4; overlapping foreign pad/via/orthogonal-track clearances conservatively unioned before fill"
            } else {
                "datum-eda fill-zones: bounded rectangular obstacle cutout fill v3; multiple non-overlapping foreign pads/vias/orthogonal tracks inflated by netclass clearance"
            };
            return (
                ZoneFillState::Filled,
                islands,
                zone_fill_provenance(provenance, thermal_requested),
            );
        }
        let Some((islands, clipped_obstacle)) =
            rectangular_cutout_islands(zone, *obstacle, clearance_nm)
        else {
            return unsupported(
                "datum-eda fill-zones: unsupported because obstacle cutout covers the bounded rectangular solver envelope",
            );
        };
        let provenance = if keepout_obstacle_count > 0 {
            "datum-eda fill-zones: bounded rectangular obstacle cutout fill v6; keepout bounds removed from copper before fill"
        } else if has_non_orthogonal_track {
            "datum-eda fill-zones: bounded rectangular obstacle cutout fill v7; non-orthogonal foreign track bounds conservatively removed before fill"
        } else if clipped_obstacle {
            "datum-eda fill-zones: bounded rectangular obstacle cutout fill v5; edge-crossing foreign pad/via/orthogonal-track clearances clipped to zone bounds before fill"
        } else {
            "datum-eda fill-zones: bounded rectangular obstacle cutout fill v2; one foreign pad/via/orthogonal track inflated by netclass clearance"
        };
        return (
            ZoneFillState::Filled,
            islands,
            zone_fill_provenance(provenance, thermal_requested),
        );
    }
    (
        ZoneFillState::Filled,
        vec![zone.polygon.clone()],
        zone_fill_provenance(
            "datum-eda fill-zones: bounded same-net polygon island fill v1; no clearance subtraction required",
            thermal_requested,
        ),
    )
}

fn unsupported(reason: &str) -> (ZoneFillState, Vec<Polygon>, String) {
    (ZoneFillState::Unsupported, Vec::new(), reason.to_string())
}

fn zone_fill_provenance(base: &str, thermal_passthrough: bool) -> String {
    if thermal_passthrough {
        format!(
            "{base}; thermal relief requested but no same-net pad/via anchors intersected the bounded fill"
        )
    } else {
        base.to_string()
    }
}

#[allow(dead_code)]
pub(super) fn persist_zone_fill(
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
                schema_version: ZONE_FILL_SCHEMA_VERSION,
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
    let shard = source_shard_ref_for_bytes(
        SourceShardKind::ZoneFill,
        path,
        relative_path,
        schema_version,
        &bytes,
        "invalid_zone_fill",
    )?;
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
    if fill.schema_version != ZONE_FILL_SCHEMA_VERSION {
        return Err(format!(
            "unsupported zone fill schema_version {}; supported {}",
            fill.schema_version, ZONE_FILL_SCHEMA_VERSION
        ));
    }
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
