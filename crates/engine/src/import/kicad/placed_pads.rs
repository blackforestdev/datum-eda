//! Placed pad import construction, preserving parsed process and source identity.
use super::*;
// Import constructor threads many parsed board-object fields.
#[allow(clippy::too_many_arguments)]
pub(super) fn footprint_pads(
    path: &Path,
    block: &str,
    footprint_source_uuid: Uuid,
    package_uuid: Uuid,
    import_map: Option<&std::collections::BTreeMap<ImportKey, ImportMapEntry>>,
    import_identities: Option<&mut Vec<KiCadBoardImportIdentity>>,
    package_position: Point,
    package_rotation_deg: i32,
    package_layer: i32,
    net_lookup: &HashMap<i32, Uuid>,
    nets: &mut HashMap<Uuid, Net>,
    layer_table: &HashMap<String, i32>,
    pad_expansion_setup: &PadExpansionSetup,
) -> Result<Vec<PlacedPad>, EngineError> {
    let mut out = Vec::new();
    let mut import_identities = import_identities;
    let package_flipped = is_back_copper_layer(package_layer, layer_table);
    let footprint_mask_margin_nm =
        parse_footprint_mm_value_before_pads(block, "solder_mask_margin");
    let footprint_paste_margin_nm =
        parse_footprint_mm_value_before_pads(block, "solder_paste_margin");
    let footprint_paste_margin_ratio_ppm =
        parse_footprint_ratio_ppm_before_pads(block, "solder_paste_margin_ratio")
            .or_else(|| parse_footprint_ratio_ppm_before_pads(block, "solder_paste_ratio"));
    for pad_block in nested_blocks(block, "pad") {
        let Some(name) = block_head_string(&pad_block, "pad") else {
            continue;
        };
        let pad_kind = parse_pad_kind_anywhere(&pad_block).ok_or_else(|| {
            EngineError::Import(format!(
                "imported pad {package_uuid}/{name} is missing a supported pad kind in the pad head"
            ))
        })?;
        let local = block_at_point_anywhere(&pad_block).unwrap_or_else(Point::zero);
        let source_uuid = pad_uuid_anywhere(&pad_block).unwrap_or_else(|| {
            deterministic_kicad_board_uuid("pad", &format!("{footprint_source_uuid}/{name}"))
        });
        let allocation = import_map.map(|import_map| {
            allocate_import_identity(import_map, board_pad_import_key(path, source_uuid))
        });
        let uuid = allocation
            .as_ref()
            .map(|allocation| allocation.object_id)
            .unwrap_or(source_uuid);
        if let (Some(allocation), Some(identities)) =
            (&allocation, import_identities.as_deref_mut())
        {
            identities.push(KiCadBoardImportIdentity::new(
                "board_pad",
                allocation.import_key.clone(),
                allocation.object_id,
                source_uuid,
            ));
        }
        let net = block_net_ref(&pad_block)
            .map(|net_ref| resolve_board_net_ref(net_ref, net_lookup, nets));
        let shape = parse_pad_shape_anywhere(&pad_block).ok_or_else(|| {
            EngineError::Import(format!(
                "imported pad {package_uuid}/{name} has unsupported or missing shape in the pad head"
            ))
        })?;
        let (diameter, width, height) = parse_pad_geometry_anywhere(&pad_block, shape);
        let roundrect_rratio_ppm = parse_pad_roundrect_rratio_ppm_anywhere(&pad_block);
        let pad_local_rotation = block_at_rotation_anywhere(&pad_block).unwrap_or(0);
        // M7-IMP-010: pad primary copper layer comes from the pad's own
        // `(layers ...)` list under the accepted Option A encoding set; we no
        // longer silently fall back to the footprint placement layer.
        let copper_layers = parse_pad_copper_layers_anywhere(&pad_block, layer_table)?;
        let layer = resolve_pad_primary_copper_layer(&copper_layers)?;
        let mask_layers = parse_pad_mask_layers_anywhere(&pad_block, layer_table)?;
        let paste_layers = parse_pad_paste_layers_anywhere(&pad_block, layer_table)?;
        let solder_mask_margin_nm = parse_block_mm_value_anywhere(&pad_block, "solder_mask_margin")
            .or(footprint_mask_margin_nm)
            .unwrap_or(pad_expansion_setup.pad_to_mask_clearance_nm);
        let solder_paste_margin_nm =
            parse_block_mm_value_anywhere(&pad_block, "solder_paste_margin")
                .or(footprint_paste_margin_nm)
                .unwrap_or(pad_expansion_setup.pad_to_paste_clearance_nm);
        let solder_paste_margin_ratio_ppm =
            parse_block_ratio_ppm_anywhere(&pad_block, "solder_paste_margin_ratio")
                .or_else(|| parse_block_ratio_ppm_anywhere(&pad_block, "solder_paste_ratio"))
                .or(footprint_paste_margin_ratio_ppm)
                .unwrap_or(pad_expansion_setup.pad_to_paste_ratio_ppm);
        let drill = parse_pad_drill_anywhere(&pad_block, pad_kind).ok_or_else(|| {
            EngineError::Import(format!(
                "imported pad {package_uuid}/{name} has pad kind {pad_kind} but no supported drill definition"
            ))
        })?;
        let position = transform_footprint_local_point(
            package_position,
            package_rotation_deg,
            package_flipped,
            local,
        );
        // KiCad PCB pad `(at x y rot)` is emitted in the board-file pad schema
        // with a final authored pad angle even though the pad center remains
        // footprint-local. Do not compose footprint rotation again here or the
        // pad orientation is double-rotated on import.
        let rotation = normalize_board_rotation_deg(pad_local_rotation);
        out.push(PlacedPad {
            layer_connection: match pad_kind {
                KiCadPadKind::ThruHole => crate::board::PadLayerConnection::PlatedThrough,
                KiCadPadKind::NpThruHole | KiCadPadKind::Smd | KiCadPadKind::Connect => {
                    crate::board::PadLayerConnection::Separate
                }
            },
            uuid,
            package: package_uuid,
            name,
            net,
            position,
            layer,
            copper_layers,
            shape,
            diameter,
            width,
            height,
            drill,
            rotation,
            roundrect_rratio_ppm,
            mask_layers,
            paste_layers,
            solder_mask_margin_nm,
            solder_paste_margin_nm,
            solder_paste_margin_ratio_ppm,
        });
    }
    Ok(out)
}
