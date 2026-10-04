//! Zone source and saved occupied copper have separate native owners.
use super::board_objects::{
    KiCadBoardImportIdentity, board_zone_import_key, check_form_accounting, dropped_object_warning,
};
use super::net_refs::*;
use super::parser_helpers::*;
use crate::board::{Net, Zone};
use crate::error::EngineError;
use crate::substrate::{ImportKey, ImportMapEntry, allocate_import_identity};
use std::{collections::HashMap, path::Path};
use uuid::Uuid;
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
    parse_zones_with_fills(
        path,
        blocks,
        import_map,
        import_identities,
        net_lookup,
        nets,
        layer_table,
        warnings,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn parse_zones_with_fills(
    path: &Path,
    blocks: &[String],
    import_map: Option<&std::collections::BTreeMap<ImportKey, ImportMapEntry>>,
    import_identities: Option<&mut Vec<KiCadBoardImportIdentity>>,
    net_lookup: &HashMap<i32, Uuid>,
    nets: &mut HashMap<Uuid, Net>,
    layer_table: &HashMap<String, i32>,
    warnings: &mut Vec<String>,
    native_fills: Option<&mut HashMap<Uuid, Vec<crate::ir::geometry::Polygon>>>,
) -> Result<HashMap<Uuid, Zone>, EngineError> {
    let mut native_fills = native_fills;
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
        let native_saved = if native_fills.is_some() {
            Some(saved_by_layer(block, &zone_layer_names)?)
        } else {
            None
        };
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
            if let Some(fills) = native_fills.as_deref_mut() {
                let marker = block.find("(polygon").ok_or_else(|| {
                    EngineError::Import(format!("zone {uuid}: authored boundary missing"))
                })?;
                let section = &block[marker..];
                let end = find_matching_paren(section)
                    .ok_or_else(|| EngineError::Import("zone boundary malformed".into()))?;
                let boundary = crate::ir::geometry::Polygon::new(
                    super::saved_fill_geometry::points(&section[..end])?,
                );
                let id = if i == 0 {
                    uuid
                } else {
                    deterministic_kicad_board_uuid("zone", &format!("{uuid}/{layer}/0"))
                };
                if let Some(saved) = native_saved
                    .as_ref()
                    .and_then(|layers| layers.get(layer_name))
                {
                    fills.insert(id, saved.clone());
                }
                islands = vec![boundary];
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
    if native_fills.is_some() && imported_blocks != blocks.len() {
        return Err(EngineError::Import(
            "incomplete native Zone source acquisition".into(),
        ));
    }
    check_form_accounting("zone", blocks.len(), imported_blocks, dropped, warnings);
    Ok(zones)
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

/// Native adoption retains one Zone per authored layer, with saved fill separate.
pub fn native_zone_sources(
    path: &Path,
    import_map: &std::collections::BTreeMap<ImportKey, ImportMapEntry>,
    board: &mut crate::board::Board,
) -> Result<HashMap<Uuid, Vec<crate::ir::geometry::Polygon>>, EngineError> {
    let contents = std::fs::read_to_string(path)?;
    let blocks = top_level_blocks(&contents, "zone");
    let net_lookup: HashMap<i32, Uuid> = top_level_blocks(&contents, "net")
        .iter()
        .filter_map(|block| parse_net_block(block))
        .filter_map(|(number, name)| {
            board
                .nets
                .values()
                .find(|net| net.name == name)
                .map(|net| (number, net.uuid))
        })
        .collect();
    let mut warnings = Vec::new();
    let mut fills = HashMap::new();
    board.zones = parse_zones_with_fills(
        path,
        &blocks,
        Some(import_map),
        None,
        &net_lookup,
        &mut board.nets,
        &parse_kicad_layer_table(&contents),
        &mut warnings,
        Some(&mut fills),
    )?;
    Ok(fills)
}

fn saved_by_layer(
    block: &str,
    declared: &[String],
) -> Result<HashMap<String, Vec<crate::ir::geometry::Polygon>>, EngineError> {
    let mut result: HashMap<String, Vec<crate::ir::geometry::Polygon>> = HashMap::new();
    let mut tail = block;
    while let Some(i) = tail.find("(filled_polygon") {
        let section = &tail[i..];
        let end = find_matching_paren(section)
            .ok_or_else(|| EngineError::Import("malformed saved filled_polygon".into()))?;
        let section = &section[..end];
        let layer = block_layer_name(section)
            .ok_or_else(|| EngineError::Import("saved fill layer missing".into()))?;
        if !declared.contains(&layer) {
            return Err(EngineError::Import(
                "saved fill layer is not declared by source Zone".into(),
            ));
        }
        let polygon =
            crate::ir::geometry::Polygon::new(super::saved_fill_geometry::points(section)?);
        result
            .entry(layer)
            .or_default()
            .extend(super::saved_fill_geometry::certify(&polygon)?);
        tail = &tail[i + end..];
    }
    Ok(result)
}
