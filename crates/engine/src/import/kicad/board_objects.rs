//! Top-level routed-copper object parsing (segments, vias, zones) with
//! parse-or-account discipline.
//!
//! Policy (M7-IMP-005): the importer must never silently discard authored
//! board data. Every source block either imports, or emits an explicit
//! dropped-object warning naming the object and the fields that failed to
//! parse. A per-form conservation check backstops the accounting so a future
//! code path cannot reintroduce silent drops.

use std::collections::HashMap;
use std::path::Path;

use uuid::Uuid;

use crate::board::{Net, Track, Via, Zone};
use crate::error::EngineError;
use crate::substrate::{ImportKey, ImportMapEntry, allocate_import_identity};

use super::net_refs::*;
use super::parser_helpers::*;

pub(super) fn dropped_object_warning(
    form: &str,
    uuid: Option<Uuid>,
    missing: &[(&str, bool)],
) -> String {
    let missing_list = missing
        .iter()
        .filter(|(_, is_missing)| *is_missing)
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(", ");
    let identity = uuid
        .map(|u| u.to_string())
        .unwrap_or_else(|| "<no uuid>".to_string());
    format!("import dropped {form} {identity}: missing or unparseable {missing_list}")
}

/// Conservation backstop: imported + explicitly-dropped must account for
/// every source block of the form. A mismatch means some code path discarded
/// data without a warning, which is itself reported loudly.
pub(super) fn check_form_accounting(
    form: &str,
    source_blocks: usize,
    imported_blocks: usize,
    dropped_blocks: usize,
    warnings: &mut Vec<String>,
) {
    if imported_blocks + dropped_blocks != source_blocks {
        warnings.push(format!(
            "import accounting mismatch for {form}: {source_blocks} source blocks, \
             {imported_blocks} imported, {dropped_blocks} dropped-with-warning; \
             a code path is discarding data silently"
        ));
    }
}

// Import constructor threads many parsed board-object fields.
#[allow(clippy::too_many_arguments)]
pub(super) fn parse_tracks(
    path: &Path,
    blocks: &[String],
    import_map: Option<&std::collections::BTreeMap<ImportKey, ImportMapEntry>>,
    import_identities: Option<&mut Vec<KiCadBoardImportIdentity>>,
    net_lookup: &HashMap<i32, Uuid>,
    nets: &mut HashMap<Uuid, Net>,
    layer_table: &HashMap<String, i32>,
    warnings: &mut Vec<String>,
) -> Result<HashMap<Uuid, Track>, EngineError> {
    let mut tracks = HashMap::new();
    let mut dropped = 0usize;
    let mut import_identities = import_identities;
    for block in blocks {
        let uuid = block_uuid(block);
        let endpoints = block_start_end_points(block);
        let width = block_width_mm(block);
        let layer_name = block_layer_name(block);
        let net_ref = block_net_ref(block);
        let missing = [
            ("uuid", uuid.is_none()),
            ("start/end", endpoints.is_none()),
            ("width", width.is_none()),
            ("layer", layer_name.is_none()),
            ("net", net_ref.is_none()),
        ];
        let (Some(uuid), Some((from, to)), Some(width), Some(layer_name), Some(net_ref)) =
            (uuid, endpoints, width, layer_name, net_ref)
        else {
            dropped += 1;
            warnings.push(dropped_object_warning("segment", uuid, &missing));
            continue;
        };
        let allocation = import_map.map(|import_map| {
            allocate_import_identity(import_map, board_segment_import_key(path, uuid))
        });
        let track_uuid = allocation
            .as_ref()
            .map(|allocation| allocation.object_id)
            .unwrap_or(uuid);
        if let (Some(allocation), Some(identities)) =
            (&allocation, import_identities.as_deref_mut())
        {
            identities.push(KiCadBoardImportIdentity::new(
                "board_segment",
                allocation.import_key.clone(),
                allocation.object_id,
                uuid,
            ));
        }
        let net = resolve_board_net_ref(net_ref, net_lookup, nets);
        tracks.insert(
            track_uuid,
            Track {
                uuid: track_uuid,
                net,
                from,
                to,
                width: mm_to_nm(width),
                layer: resolve_layer_id(&layer_name, layer_table)?,
            },
        );
    }
    check_form_accounting("segment", blocks.len(), tracks.len(), dropped, warnings);
    Ok(tracks)
}

pub fn board_footprint_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    format!("kicad:board-footprint:{}:{source_uuid}", path.display())
}

pub fn board_pad_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    format!("kicad:board-pad:{}:{source_uuid}", path.display())
}

pub fn board_segment_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    format!("kicad:board-segment:{}:{source_uuid}", path.display())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KiCadBoardImportIdentity {
    pub object_family: &'static str,
    pub import_key: ImportKey,
    pub object_id: Uuid,
    pub source_uuid: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KiCadSchematicImportIdentity {
    pub object_family: &'static str,
    pub import_key: ImportKey,
    pub object_id: Uuid,
    pub source_uuid: Uuid,
}

impl KiCadSchematicImportIdentity {
    pub(super) fn new(
        object_family: &'static str,
        import_key: ImportKey,
        object_id: Uuid,
        source_uuid: Uuid,
    ) -> Self {
        Self {
            object_family,
            import_key,
            object_id,
            source_uuid,
        }
    }
}

impl KiCadBoardImportIdentity {
    pub(super) fn new(
        object_family: &'static str,
        import_key: ImportKey,
        object_id: Uuid,
        source_uuid: Uuid,
    ) -> Self {
        Self {
            object_family,
            import_key,
            object_id,
            source_uuid,
        }
    }
}

// Import constructor threads many parsed board-object fields.
#[allow(clippy::too_many_arguments)]
pub(super) fn parse_vias(
    path: &Path,
    blocks: &[String],
    import_map: Option<&std::collections::BTreeMap<ImportKey, ImportMapEntry>>,
    import_identities: Option<&mut Vec<KiCadBoardImportIdentity>>,
    net_lookup: &HashMap<i32, Uuid>,
    nets: &mut HashMap<Uuid, Net>,
    layer_table: &HashMap<String, i32>,
    warnings: &mut Vec<String>,
) -> Result<HashMap<Uuid, Via>, EngineError> {
    let mut vias = HashMap::new();
    let mut dropped = 0usize;
    let mut import_identities = import_identities;
    for block in blocks {
        let uuid = block_uuid(block);
        let position = block_at_point(block);
        let diameter = block_size_mm(block);
        let drill = block_drill_mm(block);
        let layers = block_layers_pair(block);
        let net_ref = block_net_ref(block);
        let missing = [
            ("uuid", uuid.is_none()),
            ("at", position.is_none()),
            ("size", diameter.is_none()),
            ("drill", drill.is_none()),
            ("layers", layers.is_none()),
            ("net", net_ref.is_none()),
        ];
        let (
            Some(uuid),
            Some(position),
            Some(diameter),
            Some(drill),
            Some((from_layer, to_layer)),
            Some(net_ref),
        ) = (uuid, position, diameter, drill, layers, net_ref)
        else {
            dropped += 1;
            warnings.push(dropped_object_warning("via", uuid, &missing));
            continue;
        };
        let allocation = import_map.map(|import_map| {
            allocate_import_identity(import_map, board_via_import_key(path, uuid))
        });
        let via_uuid = allocation
            .as_ref()
            .map(|allocation| allocation.object_id)
            .unwrap_or(uuid);
        if let (Some(allocation), Some(identities)) =
            (&allocation, import_identities.as_deref_mut())
        {
            identities.push(KiCadBoardImportIdentity::new(
                "board_via",
                allocation.import_key.clone(),
                allocation.object_id,
                uuid,
            ));
        }
        let net = resolve_board_net_ref(net_ref, net_lookup, nets);
        vias.insert(
            via_uuid,
            Via {
                uuid: via_uuid,
                net,
                position,
                drill: mm_to_nm(drill),
                diameter: mm_to_nm(diameter),
                from_layer: resolve_layer_id(&from_layer, layer_table)?,
                to_layer: resolve_layer_id(&to_layer, layer_table)?,
            },
        );
    }
    check_form_accounting("via", blocks.len(), vias.len(), dropped, warnings);
    Ok(vias)
}

pub fn board_via_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    format!("kicad:board-via:{}:{source_uuid}", path.display())
}

// Import constructor threads many parsed board-object fields.
#[allow(clippy::too_many_arguments)]
pub(super) fn parse_zones(
    path: &Path,
    blocks: &[String],
    import_map: Option<&std::collections::BTreeMap<ImportKey, ImportMapEntry>>,
    import_identities: Option<&mut Vec<KiCadBoardImportIdentity>>,
    net_lookup: &HashMap<i32, Uuid>,
    nets: &mut HashMap<Uuid, Net>,
    layer_table: &HashMap<String, i32>,
    warnings: &mut Vec<String>,
) -> Result<HashMap<Uuid, Zone>, EngineError> {
    let mut zones = HashMap::new();
    let mut imported_blocks = 0usize;
    let mut dropped = 0usize;
    let mut import_identities = import_identities;
    for block in blocks {
        let Some(net_ref) = block_net_ref(block) else {
            dropped += 1;
            warnings.push(dropped_object_warning(
                "zone",
                block_uuid(block),
                &[("net", true)],
            ));
            continue;
        };
        let net = resolve_board_net_ref(net_ref, net_lookup, nets);
        let zone_layers = block_layer_names(block);
        let zone_layer_names: Vec<String> = if zone_layers.is_empty() {
            block_layer_name(block).into_iter().collect()
        } else {
            zone_layers
        };
        if zone_layer_names.is_empty() {
            dropped += 1;
            warnings.push(dropped_object_warning(
                "zone",
                block_uuid(block),
                &[("layer", true)],
            ));
            continue;
        }
        // KiCad-generated fill zones (e.g. teardrop reinforcement copper)
        // carry no `(uuid ...)`. Derive a deterministic identity from net,
        // layer set, and first boundary point instead of dropping authored
        // copper that belongs in rendering and CAM output.
        let source_uuid = block_uuid(block).unwrap_or_else(|| {
            let first_point = block_xy_points(block)
                .first()
                .map(|point| format!("{}/{}", point.x, point.y))
                .unwrap_or_default();
            deterministic_kicad_board_uuid(
                "zone",
                &format!("{net}/{}/{first_point}", zone_layer_names.join(",")),
            )
        });
        let allocation = import_map.map(|import_map| {
            allocate_import_identity(import_map, board_zone_import_key(path, source_uuid))
        });
        let uuid = allocation
            .as_ref()
            .map(|allocation| allocation.object_id)
            .unwrap_or(source_uuid);
        if let (Some(allocation), Some(identities)) =
            (&allocation, import_identities.as_deref_mut())
        {
            identities.push(KiCadBoardImportIdentity::new(
                "board_zone",
                allocation.import_key.clone(),
                allocation.object_id,
                source_uuid,
            ));
        }
        let mut inserted_any = false;
        for (i, layer_name) in zone_layer_names.iter().enumerate() {
            let layer = resolve_layer_id(layer_name, layer_table)?;
            let mut islands = extract_filled_polygons_for_layer(block, layer_name);
            if islands.is_empty() {
                // No per-layer fill: fall back to the authored boundary
                // template so the zone is not lost, and say so loudly --
                // the boundary is wider/cruder than the true filled shape.
                if let Some(boundary) = block_polygon(block) {
                    warnings.push(format!(
                        "zone {uuid} on layer {layer_name}: no filled polygon; \
                         rendering authored boundary template instead"
                    ));
                    islands.push(boundary);
                } else {
                    warnings.push(format!(
                        "import dropped zone {uuid} on layer {layer_name}: no filled polygon \
                         or boundary polygon (unfilled zone, keepout, or rule area)"
                    ));
                    continue;
                }
            }
            for (island_index, polygon) in islands.into_iter().enumerate() {
                let zone_uuid = if i == 0 && island_index == 0 {
                    uuid
                } else {
                    deterministic_kicad_board_uuid(
                        "zone",
                        &format!("{uuid}/{layer}/{island_index}"),
                    )
                };
                zones.insert(
                    zone_uuid,
                    Zone {
                        uuid: zone_uuid,
                        net,
                        polygon,
                        layer,
                        priority: 0,
                        thermal_relief: true,
                        thermal_gap: 0,
                        thermal_spoke_width: 0,
                    },
                );
                inserted_any = true;
            }
        }
        if inserted_any {
            imported_blocks += 1;
        } else {
            dropped += 1;
        }
    }
    check_form_accounting("zone", blocks.len(), imported_blocks, dropped, warnings);
    Ok(zones)
}

pub fn board_zone_import_key(path: &Path, source_uuid: Uuid) -> ImportKey {
    format!("kicad:board-zone:{}:{source_uuid}", path.display())
}

/// Collect ALL filled-polygon islands for one layer of a zone block.
/// KiCad emits one `(filled_polygon ...)` per island; a same-net zone over
/// several pads (e.g. teardrop groups) carries several islands, and dropping
/// all but the first silently loses copper.
fn extract_filled_polygons_for_layer(
    zone_block: &str,
    layer_name: &str,
) -> Vec<crate::ir::geometry::Polygon> {
    use crate::ir::geometry::Polygon;
    let marker = "(filled_polygon";
    let mut islands = Vec::new();
    let mut search_from = 0;
    while let Some(start) = zone_block[search_from..].find(marker) {
        let abs_start = search_from + start;
        let rest = &zone_block[abs_start..];
        let Some(end) = find_matching_paren(rest) else {
            search_from = abs_start + marker.len();
            continue;
        };
        let section = &rest[..end];
        let layer_marker = format!("(layer \"{}\")", layer_name);
        if section.contains(&layer_marker) {
            let points = block_xy_points(section);
            if points.len() >= 3 {
                islands.push(Polygon::new(points));
            }
        }
        search_from = abs_start + end;
    }
    islands
}
