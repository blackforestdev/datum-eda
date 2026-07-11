use std::collections::HashMap;
use std::path::Path;

use uuid::Uuid;

use crate::board::{Board, Net, NetClass, PadExpansionSetup, PlacedPackage, PlacedPad};
use crate::error::EngineError;
use crate::ir::geometry::Point;
use crate::ir::ids::{import_uuid, namespace_kicad};
use crate::schematic::{
    Bus, HiddenPowerBehavior, Junction, LabelKind, NetLabel, NoConnectMarker, PlacedSymbol,
    SchematicWire, Sheet, SymbolDisplayMode, SymbolPin,
};
use crate::substrate::{ImportKey, ImportMapEntry, allocate_import_identity};

use super::board_objects::*;
use super::net_refs::*;
use super::parser_helpers::*;
use super::pad_expansion::{
    NonCopperLayerKind, parse_block_mm_value_anywhere, parse_block_ratio_ppm_anywhere,
    parse_footprint_mm_value_before_pads, parse_footprint_ratio_ppm_before_pads,
    parse_pad_expansion_setup, parse_pad_non_copper_layers_anywhere,
};
use super::schematic_bus::{
    ParsedBusEntrySkeleton, ParsedBusSegment, attached_bus_specs, resolve_bus_entry_attachment,
};
use super::schematic_graphics::{parse_schematic_drawings, parse_schematic_texts};
use super::schematic_identity::schematic_import_id;
use super::symbol_helpers::*;

pub(super) struct ChildSheetRef {
    pub(super) instance_uuid: Uuid,
    pub(super) name: String,
    pub(super) position: Point,
    pub(super) sheetfile: Option<String>,
    pub(super) ports: Vec<Uuid>,
}

pub(super) struct ParsedSchematicSkeleton {
    pub(super) root_sheet: Sheet,
    pub(super) child_sheets: Vec<ChildSheetRef>,
}
pub(super) fn parse_schematic_skeleton(
    path: &Path,
    contents: &str,
    root_sheet_name: &str,
    import_map: Option<&std::collections::BTreeMap<ImportKey, ImportMapEntry>>,
    import_identities: Option<&mut Vec<KiCadSchematicImportIdentity>>,
) -> Result<ParsedSchematicSkeleton, EngineError> {
    let root_uuid = find_top_level_uuid(contents).unwrap_or_else(Uuid::new_v4);
    let root_sheet_uuid = root_uuid;
    let library_pins = parse_library_symbol_pins(contents);

    let mut symbols = HashMap::new();
    let mut import_identities = import_identities;
    for block in top_level_blocks(contents, "symbol") {
        let source_uuid = block_uuid(&block).unwrap_or_else(Uuid::new_v4);
        let uuid = schematic_import_id(
            path,
            source_uuid,
            "symbol",
            "schematic_symbol",
            import_map,
            import_identities.as_deref_mut(),
        );
        let position = block_at_point(&block).unwrap_or_else(Point::zero);
        let rotation = block_rotation(&block).unwrap_or(0);
        let mirrored = symbol_is_mirrored_y(&block);
        let lib_id = extract_symbol_lib_id(&block);
        let reference = extract_symbol_property(&block, "Reference").unwrap_or_else(|| "?".into());
        let value = extract_symbol_property(&block, "Value").unwrap_or_default();
        let fields = symbol_fields(uuid, &block);
        let pins = lib_id
            .as_ref()
            .and_then(|lib_id| library_pins.get(lib_id))
            .map(|templates| {
                templates
                    .iter()
                    .enumerate()
                    .map(|(index, template)| SymbolPin {
                        uuid: import_uuid(
                            &namespace_kicad(),
                            &format!("schematic-symbol-pin/{uuid}/{index}/{}", template.number),
                        ),
                        number: template.number.clone(),
                        name: template.name.clone(),
                        electrical_type: template.electrical_type.clone(),
                        position: transform_symbol_pin(
                            position,
                            rotation,
                            mirrored,
                            template.position,
                        ),
                    })
                    .collect()
            })
            .unwrap_or_default();
        symbols.insert(
            uuid,
            PlacedSymbol {
                uuid,
                part: None,
                entity: None,
                gate: None,
                lib_id,
                reference,
                value,
                fields,
                pins,
                position,
                rotation,
                mirrored,
                unit_selection: extract_symbol_unit(&block),
                display_mode: SymbolDisplayMode::LibraryDefault,
                pin_overrides: Vec::new(),
                hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
            },
        );
    }

    let mut wires = HashMap::new();
    for block in top_level_blocks(contents, "wire") {
        let source_uuid = block_uuid(&block).unwrap_or_else(Uuid::new_v4);
        let uuid = schematic_import_id(
            path,
            source_uuid,
            "wire",
            "schematic_wire",
            import_map,
            import_identities.as_deref_mut(),
        );
        let points = block_xy_points(&block);
        if points.len() >= 2 {
            wires.insert(
                uuid,
                SchematicWire {
                    uuid,
                    from: points[0],
                    to: points[1],
                },
            );
        }
    }

    let mut junctions = HashMap::new();
    for block in top_level_blocks(contents, "junction") {
        if let (Some(source_uuid), Some(position)) = (block_uuid(&block), block_at_point(&block)) {
            let uuid = schematic_import_id(
                path,
                source_uuid,
                "junction",
                "schematic_junction",
                import_map,
                import_identities.as_deref_mut(),
            );
            junctions.insert(uuid, Junction { uuid, position });
        }
    }

    let mut labels = HashMap::new();
    for (form, kind) in [
        ("label", LabelKind::Local),
        ("global_label", LabelKind::Global),
        ("hierarchical_label", LabelKind::Hierarchical),
    ] {
        for block in top_level_blocks(contents, form) {
            if let (Some(source_uuid), Some(position), Some(name)) = (
                block_uuid(&block),
                block_at_point(&block),
                block_head_string(&block, form),
            ) {
                let uuid = schematic_import_id(
                    path,
                    source_uuid,
                    "label",
                    "schematic_label",
                    import_map,
                    import_identities.as_deref_mut(),
                );
                labels.insert(
                    uuid,
                    NetLabel {
                        uuid,
                        kind: kind.clone(),
                        name,
                        position,
                    },
                );
            }
        }
    }

    let mut parsed_buses = Vec::new();
    for block in top_level_blocks(contents, "bus") {
        let source_uuid = block_uuid(&block).unwrap_or_else(Uuid::new_v4);
        let uuid = schematic_import_id(
            path,
            source_uuid,
            "bus",
            "schematic_bus",
            import_map,
            import_identities.as_deref_mut(),
        );
        let points = block_xy_points(&block);
        if points.len() >= 2 {
            parsed_buses.push(ParsedBusSegment { uuid, points });
        }
    }
    parsed_buses.sort_by_key(|bus| bus.uuid);

    let mut buses = HashMap::new();
    for bus in &parsed_buses {
        let specs = attached_bus_specs(bus, labels.values());
        let (name, members) = if let Some((name, members)) = specs.first() {
            (name.clone(), members.clone())
        } else {
            (format!("BUS_{}", bus.uuid), Vec::new())
        };
        buses.insert(
            bus.uuid,
            Bus {
                uuid: bus.uuid,
                name,
                members,
                segments: bus.points.clone(),
            },
        );
    }

    let mut bus_entries = HashMap::new();
    let mut parsed_entries = Vec::new();
    for block in top_level_blocks(contents, "bus_entry") {
        let Some(position) = block_at_point(&block) else {
            continue;
        };
        parsed_entries.push(ParsedBusEntrySkeleton {
            uuid: schematic_import_id(
                path,
                block_uuid(&block).unwrap_or_else(Uuid::new_v4),
                "bus-entry",
                "schematic_bus_entry",
                import_map,
                import_identities.as_deref_mut(),
            ),
            position,
            size: block_size_point(&block).unwrap_or_else(Point::zero),
        });
    }
    parsed_entries.sort_by_key(|entry| entry.uuid);
    for entry in parsed_entries {
        let (bus, wire) = resolve_bus_entry_attachment(&entry, &parsed_buses, &wires);
        let Some(bus) = bus else {
            continue;
        };
        bus_entries.insert(
            entry.uuid,
            crate::schematic::BusEntry {
                uuid: entry.uuid,
                bus,
                wire,
                position: entry.position,
                size: entry.size,
            },
        );
    }

    let mut noconnects = HashMap::new();
    for block in top_level_blocks(contents, "no_connect") {
        if let (Some(source_uuid), Some(position)) = (block_uuid(&block), block_at_point(&block)) {
            let uuid = schematic_import_id(
                path,
                source_uuid,
                "no-connect",
                "schematic_no_connect",
                import_map,
                import_identities.as_deref_mut(),
            );
            let (symbol, pin) =
                pin_at_position(&symbols, position).unwrap_or((Uuid::nil(), Uuid::nil()));
            noconnects.insert(
                uuid,
                NoConnectMarker {
                    uuid,
                    symbol,
                    pin,
                    position,
                },
            );
        }
    }

    let mut child_sheets = Vec::new();
    let mut ports = HashMap::new();
    for block in top_level_blocks(contents, "sheet") {
        let instance_uuid = block_uuid(&block).unwrap_or_else(Uuid::new_v4);
        let name = extract_sheet_property(&block, "Sheetname").unwrap_or_else(|| "Sheet".into());
        let mut port_uuids = Vec::new();
        for port in extract_sheet_pins(instance_uuid, &block) {
            port_uuids.push(port.uuid);
            ports.insert(port.uuid, port);
        }
        port_uuids.sort();
        child_sheets.push(ChildSheetRef {
            instance_uuid,
            name,
            position: block_at_point(&block).unwrap_or_else(Point::zero),
            sheetfile: extract_sheet_property(&block, "Sheetfile"),
            ports: port_uuids,
        });
    }
    child_sheets.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then_with(|| a.instance_uuid.cmp(&b.instance_uuid))
    });

    let root_sheet = Sheet {
        uuid: root_sheet_uuid,
        name: root_sheet_name.into(),
        frame: None,
        symbols,
        wires,
        junctions,
        labels,
        buses,
        bus_entries,
        ports,
        noconnects,
        texts: parse_schematic_texts(path, contents, import_map, import_identities.as_deref_mut()),
        drawings: parse_schematic_drawings(
            path,
            contents,
            import_map,
            import_identities,
        ),
    };

    Ok(ParsedSchematicSkeleton {
        root_sheet,
        child_sheets,
    })
}

pub(super) fn parse_board_skeleton(
    path: &Path,
    contents: &str,
    import_map: Option<&std::collections::BTreeMap<ImportKey, ImportMapEntry>>,
    import_identities: Option<&mut Vec<KiCadBoardImportIdentity>>,
) -> Result<(Board, Vec<String>), EngineError> {
    let board_uuid = find_top_level_uuid(contents).unwrap_or_else(Uuid::new_v4);
    let name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("board")
        .to_string();

    let layer_table = parse_kicad_layer_table(contents);
    let resolve_layer =
        |name: &str| -> Result<i32, EngineError> { resolve_layer_id(name, &layer_table) };
    let stackup = parse_board_layers(contents);
    let pad_expansion_setup = parse_pad_expansion_setup(contents);
    let net_classes = HashMap::from([(
        Uuid::nil(),
        NetClass {
            uuid: Uuid::nil(),
            name: "Default".into(),
            clearance: 0,
            track_width: 0,
            via_drill: 0,
            via_diameter: 0,
            diffpair_width: 0,
            diffpair_gap: 0,
        },
    )]);
    let top_blocks = top_level_blocks_by_form(
        contents,
        &["net", "footprint", "segment", "via", "zone", "gr_text"],
    );

    let mut warnings = Vec::new();
    let mut nets = HashMap::new();
    let mut net_lookup = HashMap::new();
    let mut import_identities = import_identities;
    for block in top_blocks.get("net").into_iter().flatten() {
        if let Some((net_code, net_name)) = parse_net_block(block) {
            let uuid = deterministic_kicad_board_uuid("net", &net_code.to_string());
            net_lookup.insert(net_code, uuid);
            nets.insert(
                uuid,
                Net {
                    uuid,
                    name: net_name,
                    class: Uuid::nil(),
                    controlled_impedance: None,
                },
            );
        } else {
            warnings.push(dropped_object_warning(
                "net",
                block_uuid(block),
                &[("code/name", true)],
            ));
        }
    }

    let mut packages = HashMap::new();
    let mut pads = HashMap::new();
    for block in top_blocks.get("footprint").into_iter().flatten() {
        let source_uuid = block_uuid(block).unwrap_or_else(Uuid::new_v4);
        let allocation = import_map.map(|import_map| {
            allocate_import_identity(import_map, board_footprint_import_key(path, source_uuid))
        });
        let package_uuid = allocation
            .as_ref()
            .map(|allocation| allocation.object_id)
            .unwrap_or(source_uuid);
        if let (Some(allocation), Some(identities)) =
            (&allocation, import_identities.as_deref_mut())
        {
            identities.push(KiCadBoardImportIdentity::new(
                "board_footprint",
                allocation.import_key.clone(),
                allocation.object_id,
                source_uuid,
            ));
        }
        let reference =
            extract_footprint_property(block, "Reference").unwrap_or_else(|| "?".into());
        let value = extract_footprint_property(block, "Value").unwrap_or_default();
        let position = block_at_point(block).unwrap_or_else(Point::zero);
        let rotation = block_rotation(block).unwrap_or(0);
        let layer = match block_layer_name(block).as_deref() {
            Some(name) => resolve_layer(name)?,
            None => {
                return Err(EngineError::Import(format!(
                    "imported footprint {reference} ({package_uuid}) has no placement layer"
                )));
            }
        };
        packages.insert(
            package_uuid,
            PlacedPackage {
                uuid: package_uuid,
                part: Uuid::nil(),
                package: Uuid::nil(),
                reference,
                value,
                position,
                rotation,
                layer,
                locked: block.contains("(locked)"),
            },
        );
        for pad in footprint_pads(
            path,
            block,
            source_uuid,
            package_uuid,
            import_map,
            import_identities.as_deref_mut(),
            position,
            rotation,
            layer,
            &net_lookup,
            &mut nets,
            &layer_table,
            &pad_expansion_setup,
        )? {
            pads.insert(pad.uuid, pad);
        }
    }

    let no_blocks: Vec<String> = Vec::new();
    let blocks_for = |form: &str| top_blocks.get(form).unwrap_or(&no_blocks);

    let tracks = parse_tracks(
        path,
        blocks_for("segment"),
        import_map,
        import_identities.as_deref_mut(),
        &net_lookup,
        &mut nets,
        &layer_table,
        &mut warnings,
    )?;

    let vias = parse_vias(
        path,
        blocks_for("via"),
        import_map,
        import_identities.as_deref_mut(),
        &net_lookup,
        &mut nets,
        &layer_table,
        &mut warnings,
    )?;

    let zones = parse_zones(
        path,
        blocks_for("zone"),
        import_map,
        import_identities,
        &net_lookup,
        &mut nets,
        &layer_table,
        &mut warnings,
    )?;

    let mut texts = Vec::new();
    let mut dropped_texts = 0usize;
    for block in top_blocks.get("gr_text").into_iter().flatten() {
        let Some(position) = block_at_point(block) else {
            dropped_texts += 1;
            warnings.push(dropped_object_warning(
                "gr_text",
                block_uuid(block),
                &[("at", true)],
            ));
            continue;
        };
        let Some(layer_name) = block_layer_name(block) else {
            // gr_text without a layer has no well-defined placement; drop with
            // accounting rather than silently collapse onto F.Cu.
            dropped_texts += 1;
            warnings.push(dropped_object_warning(
                "gr_text",
                block_uuid(block),
                &[("layer", true)],
            ));
            continue;
        };
        let layer = resolve_layer(&layer_name)?;
        let text = block_head_string(block, "gr_text").unwrap_or_default();
        let uuid = block_uuid(block).unwrap_or_else(|| {
            deterministic_kicad_board_uuid(
                "gr_text",
                &format!("{text}/{}/{}/{}", position.x, position.y, layer),
            )
        });
        texts.push(crate::board::BoardText {
            uuid,
            text,
            position,
            rotation: block_rotation(block).unwrap_or(0),
            render_intent: crate::text::TextRenderIntent::Manufacturing,
            family: crate::text::TextFamilyId::default(),
            family_source: crate::text::TextFamilySource::ImplicitDefault,
            style: crate::text::TextStyleId::default(),
            height_nm: 1_000_000,
            stroke_width_nm: crate::text::default_stroke_width_nm(1_000_000),
            layer,
            h_align: crate::text::TextHAlign::Left,
            v_align: crate::text::TextVAlign::Bottom,
            mirrored: false,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_000_000,
            italic: false,
            bold: false,
            style_class: None,
        });
    }
    texts.sort_by_key(|text| text.uuid);
    check_form_accounting(
        "gr_text",
        top_blocks.get("gr_text").map(Vec::len).unwrap_or(0),
        texts.len(),
        dropped_texts,
        &mut warnings,
    );

    let (outline, outline_warning) = outline_from_edge_cuts(contents);
    if let Some(msg) = outline_warning {
        warnings.push(msg);
    }

    Ok((
        Board {
            uuid: board_uuid,
            name,
            stackup,
            pad_expansion_setup,
            outline,
            packages,
            pads,
            tracks,
            vias,
            zones,
            nets,
            net_classes,
            rules: Vec::new(),
            keepouts: Vec::new(),
            dimensions: Vec::new(),
            texts,
        },
        warnings,
    ))
}

// Import constructor threads many parsed board-object fields.
#[allow(clippy::too_many_arguments)]
fn footprint_pads(
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

fn pad_uuid_anywhere(pad_block: &str) -> Option<Uuid> {
    let start = pad_block.find("(uuid ")?;
    let rest = &pad_block[start..];
    let end = rest.find(')')?;
    parse_uuid_line(&rest[..=end])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KiCadPadKind {
    Smd,
    ThruHole,
    NpThruHole,
    Connect,
}

impl std::fmt::Display for KiCadPadKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KiCadPadKind::Smd => write!(f, "smd"),
            KiCadPadKind::ThruHole => write!(f, "thru_hole"),
            KiCadPadKind::NpThruHole => write!(f, "np_thru_hole"),
            KiCadPadKind::Connect => write!(f, "connect"),
        }
    }
}

fn is_back_copper_layer(layer: i32, layer_table: &HashMap<String, i32>) -> bool {
    resolve_layer_id("B.Cu", layer_table)
        .map(|id| id == layer)
        .unwrap_or(layer == 31)
}

fn mirror_footprint_local_x(local: Point) -> Point {
    Point::new(-local.x, local.y)
}

fn transform_footprint_local_point(
    origin: Point,
    rotation_deg: i32,
    flipped: bool,
    local: Point,
) -> Point {
    let local = if flipped {
        mirror_footprint_local_x(local)
    } else {
        local
    };
    transform_board_local_point(origin, rotation_deg, local)
}

fn normalize_board_rotation_deg(rotation_deg: i32) -> i32 {
    let normalized = rotation_deg.rem_euclid(360);
    if normalized > 180 {
        normalized - 360
    } else {
        normalized
    }
}

fn block_at_rotation_anywhere(block: &str) -> Option<i32> {
    for line in block.lines() {
        let trimmed = line.trim_start();
        if let Some(start) = trimmed.find("(at ") {
            let rest = &trimmed[start + 4..];
            let end = rest.find(')').unwrap_or(rest.len());
            let mut parts = rest[..end].split_whitespace();
            parts.next()?; // x
            parts.next()?; // y
            if let Some(rot) = parts.next().and_then(|s| s.parse::<f64>().ok()) {
                return Some(rot.round() as i32);
            }
            return None; // (at x y) with no rotation
        }
    }
    None
}

fn block_at_point_anywhere(block: &str) -> Option<Point> {
    block
        .lines()
        .find_map(|line| parse_xy_like_anywhere(line.trim_start(), "at"))
}

fn parse_xy_like_anywhere(trimmed: &str, form: &str) -> Option<Point> {
    let marker = format!("({form} ");
    let start = trimmed.find(&marker)? + marker.len();
    let rest = &trimmed[start..];
    let end = rest.find(')').unwrap_or(rest.len());
    let mut parts = rest[..end].split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    Some(Point::new(mm_to_nm(x), mm_to_nm(y)))
}

pub(super) fn parse_pad_shape_anywhere(block: &str) -> Option<crate::board::PadShape> {
    let mut chars = pad_head_after_name(block)?;
    while chars.peek().is_some_and(|ch| ch.is_ascii_whitespace()) {
        chars.next();
    }
    while chars.peek().is_some_and(|ch| !ch.is_ascii_whitespace()) {
        chars.next();
    }
    while chars.peek().is_some_and(|ch| ch.is_ascii_whitespace()) {
        chars.next();
    }
    let mut shape = String::new();
    while let Some(ch) = chars.peek() {
        if ch.is_ascii_whitespace() || *ch == ')' {
            break;
        }
        shape.push(*ch);
        chars.next();
    }
    match shape.as_str() {
        "circle" => Some(crate::board::PadShape::Circle),
        "rect" => Some(crate::board::PadShape::Rect),
        "oval" => Some(crate::board::PadShape::Oval),
        "roundrect" => Some(crate::board::PadShape::RoundRect),
        _ => None,
    }
}

pub(super) fn parse_pad_kind_anywhere(block: &str) -> Option<KiCadPadKind> {
    let mut chars = pad_head_after_name(block)?;
    while chars.peek().is_some_and(|ch| ch.is_ascii_whitespace()) {
        chars.next();
    }
    let mut kind = String::new();
    while let Some(ch) = chars.peek() {
        if ch.is_ascii_whitespace() || *ch == ')' {
            break;
        }
        kind.push(*ch);
        chars.next();
    }
    match kind.as_str() {
        "smd" => Some(KiCadPadKind::Smd),
        "thru_hole" => Some(KiCadPadKind::ThruHole),
        "np_thru_hole" => Some(KiCadPadKind::NpThruHole),
        "connect" => Some(KiCadPadKind::Connect),
        _ => None,
    }
}

fn pad_head_after_name(block: &str) -> Option<std::iter::Peekable<std::str::Chars<'_>>> {
    let first = block.lines().next()?.trim_start();
    let prefix = "(pad ";
    if !first.starts_with(prefix) {
        return None;
    }
    let rest = &first[prefix.len()..];
    let mut chars = rest.chars().peekable();
    if chars.peek() == Some(&'"') {
        chars.next();
        for ch in chars.by_ref() {
            if ch == '"' {
                break;
            }
        }
    } else {
        while let Some(ch) = chars.peek() {
            if ch.is_ascii_whitespace() {
                break;
            }
            chars.next();
        }
    }
    while chars.peek().is_some_and(|ch| ch.is_ascii_whitespace()) {
        chars.next();
    }
    Some(chars)
}

fn parse_pad_geometry_anywhere(block: &str, shape: crate::board::PadShape) -> (i64, i64, i64) {
    let size = block
        .lines()
        .find_map(|line| parse_xy_like_anywhere(line.trim_start(), "size"));
    match (shape, size) {
        (crate::board::PadShape::Circle, Some(size)) => {
            let diameter = size.x.max(size.y).max(1);
            (diameter, 0, 0)
        }
        (_, Some(size)) => (0, size.x.max(1), size.y.max(1)),
        (crate::board::PadShape::Circle, None) => (1, 0, 0),
        _ => (0, 1, 1),
    }
}

fn parse_pad_roundrect_rratio_ppm_anywhere(block: &str) -> u32 {
    for line in block.lines() {
        let trimmed = line.trim_start();
        let Some(start) = trimmed.find("(roundrect_rratio ") else {
            continue;
        };
        let rest = &trimmed[start + "(roundrect_rratio ".len()..];
        let rest = rest.split(')').next().unwrap_or(rest);
        if let Some(value) = rest
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
        {
            return (value.clamp(0.0, 0.5) * 1_000_000.0).round() as u32;
        }
    }
    250_000
}

fn parse_pad_mask_layers_anywhere(
    block: &str,
    layer_table: &HashMap<String, i32>,
) -> Result<Vec<i32>, EngineError> {
    parse_pad_non_copper_layers_anywhere(block, layer_table, NonCopperLayerKind::Mask)
}

fn parse_pad_paste_layers_anywhere(
    block: &str,
    layer_table: &HashMap<String, i32>,
) -> Result<Vec<i32>, EngineError> {
    parse_pad_non_copper_layers_anywhere(block, layer_table, NonCopperLayerKind::Paste)
}

pub(super) fn parse_pad_drill_anywhere(block: &str, pad_kind: KiCadPadKind) -> Option<i64> {
    match pad_kind {
        KiCadPadKind::Smd | KiCadPadKind::Connect => return Some(0),
        KiCadPadKind::ThruHole | KiCadPadKind::NpThruHole => {}
    }
    for line in block.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("(drill ") {
            let rest = trimmed.trim_start_matches("(drill ").trim_end_matches(')');
            if let Some(val) = rest
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok())
            {
                return Some(mm_to_nm(val));
            }
        }
    }
    None
}

fn all_copper_layers(layer_table: &HashMap<String, i32>) -> Vec<i32> {
    let mut layers: Vec<i32> = layer_table
        .iter()
        .filter_map(|(name, id)| name.ends_with(".Cu").then_some(*id))
        .collect();
    if layers.is_empty() {
        // No parsed layer table: bounded fallback to the classic KiCad
        // F.Cu/B.Cu ids. Do NOT inject these ids when a table exists --
        // KiCad 20260206+ renumbers layers (e.g. B.Cu=2, F.CrtYd=31), so
        // hardcoded ids would put non-copper layers into copper sets.
        layers.extend([0, 31]);
    }
    layers.sort_unstable();
    layers.dedup();
    layers
}

/// Pure helper: resolve a KiCad pad's explicit copper-layer membership under
/// M7-IMP-011.
///
/// - `"F.Cu"` / `"B.Cu"` / named inner-copper layers: included directly
/// - `"*.Cu"`: expands to all copper layers present in the PCB layer table
///   (or the bounded F.Cu/B.Cu fallback when no table is available)
/// - `"F&B.Cu"`: expands to front and back copper only
/// - combinations with multiple copper entries: sorted union of the recognized copper layer ids
///
/// Unsupported encodings return an explicit import error rather than silently
/// falling back to the footprint's placement layer.
pub(super) fn resolve_pad_copper_layers(
    layers_tokens: &[String],
    layer_table: &HashMap<String, i32>,
) -> Result<Vec<i32>, EngineError> {
    let mut resolved: Vec<i32> = Vec::new();
    let mut saw_supported = false;
    for entry in layers_tokens {
        match entry.as_str() {
            "*.Cu" => {
                saw_supported = true;
                resolved.extend(all_copper_layers(layer_table));
            }
            "F&B.Cu" => {
                saw_supported = true;
                resolved.push(resolve_layer_id("F.Cu", layer_table)?);
                resolved.push(resolve_layer_id("B.Cu", layer_table)?);
            }
            name if name.ends_with(".Cu") => {
                saw_supported = true;
                resolved.push(resolve_layer_id(name, layer_table)?);
            }
            _ => {}
        }
    }
    resolved.sort_unstable();
    resolved.dedup();
    if !resolved.is_empty() {
        return Ok(resolved);
    }
    if saw_supported {
        return Err(EngineError::Import(format!(
            "imported pad resolved no copper-layer membership from (layers ...) encoding {layers_tokens:?}"
        )));
    }
    Err(EngineError::Import(format!(
        "imported pad has unsupported (layers ...) encoding {layers_tokens:?}; accepted values are F.Cu, B.Cu, *.Cu, F&B.Cu, or named copper layers present in the PCB layer table"
    )))
}

pub(super) fn resolve_pad_primary_copper_layer(copper_layers: &[i32]) -> Result<i32, EngineError> {
    copper_layers.iter().copied().min().ok_or_else(|| {
        EngineError::Import("imported pad resolved no primary copper layer".to_string())
    })
}

fn parse_pad_copper_layers_anywhere(
    block: &str,
    layer_table: &HashMap<String, i32>,
) -> Result<Vec<i32>, EngineError> {
    let Some(layers) = block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.contains("(layers ") {
            return None;
        }
        Some(quoted_tokens(trimmed))
    }) else {
        return Err(EngineError::Import(
            "imported pad has no (layers ...) list; cannot determine primary copper layer"
                .to_string(),
        ));
    };
    resolve_pad_copper_layers(&layers, layer_table)
}
