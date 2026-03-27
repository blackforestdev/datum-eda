use std::collections::HashMap;
use std::path::Path;

use uuid::Uuid;

use crate::board::{Board, Net, NetClass, PlacedPackage, PlacedPad, Track, Via, Zone};
use crate::error::EngineError;
use crate::ir::geometry::Point;
use crate::schematic::{
    Bus, CheckWaiver, HiddenPowerBehavior, Junction, LabelKind, NetLabel, NoConnectMarker,
    PlacedSymbol, Schematic, SchematicWire, Sheet, SheetDefinition, SheetInstance,
    SymbolDisplayMode, SymbolPin, Variant,
};

use super::parser_helpers::*;
use super::symbol_helpers::*;

pub(super) fn parse_schematic_skeleton(contents: &str) -> Result<Schematic, EngineError> {
    let root_uuid = find_top_level_uuid(contents).unwrap_or_else(Uuid::new_v4);
    let root_sheet_uuid = Uuid::new_v4();
    let library_pins = parse_library_symbol_pins(contents);

    let mut symbols = HashMap::new();
    for block in top_level_blocks(contents, "symbol") {
        let uuid = block_uuid(&block).unwrap_or_else(Uuid::new_v4);
        let position = block_at_point(&block).unwrap_or_else(Point::zero);
        let rotation = block_rotation(&block).unwrap_or(0);
        let mirrored = symbol_is_mirrored_y(&block);
        let lib_id = extract_symbol_lib_id(&block);
        let reference = extract_symbol_property(&block, "Reference").unwrap_or_else(|| "?".into());
        let value = extract_symbol_property(&block, "Value").unwrap_or_default();
        let fields = symbol_fields(&block);
        let pins = lib_id
            .as_ref()
            .and_then(|lib_id| library_pins.get(lib_id))
            .map(|templates| {
                templates
                    .iter()
                    .map(|template| SymbolPin {
                        uuid: Uuid::new_v4(),
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
        let uuid = block_uuid(&block).unwrap_or_else(Uuid::new_v4);
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
        if let (Some(uuid), Some(position)) = (block_uuid(&block), block_at_point(&block)) {
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
            if let (Some(uuid), Some(position), Some(name)) = (
                block_uuid(&block),
                block_at_point(&block),
                block_head_string(&block, form),
            ) {
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

    let mut buses = HashMap::new();
    for block in top_level_blocks(contents, "bus") {
        let uuid = block_uuid(&block).unwrap_or_else(Uuid::new_v4);
        buses.insert(
            uuid,
            Bus {
                uuid,
                name: format!("BUS_{uuid}"),
                members: Vec::new(),
            },
        );
    }

    let mut noconnects = HashMap::new();
    for block in top_level_blocks(contents, "no_connect") {
        if let (Some(uuid), Some(position)) = (block_uuid(&block), block_at_point(&block)) {
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

    let mut sheet_definitions = HashMap::new();
    let mut sheet_instances = HashMap::new();
    let mut ports = HashMap::new();
    for block in top_level_blocks(contents, "sheet") {
        let uuid = block_uuid(&block).unwrap_or_else(Uuid::new_v4);
        let name = extract_sheet_property(&block, "Sheetname").unwrap_or_else(|| "Sheet".into());
        let definition_uuid = Uuid::new_v4();
        sheet_definitions.insert(
            definition_uuid,
            SheetDefinition {
                uuid: definition_uuid,
                root_sheet: uuid,
                name: name.clone(),
            },
        );
        sheet_instances.insert(
            uuid,
            SheetInstance {
                uuid,
                definition: definition_uuid,
                parent_sheet: Some(root_sheet_uuid),
                position: block_at_point(&block).unwrap_or_else(Point::zero),
                name,
            },
        );

        for port in extract_sheet_pins(&block) {
            ports.insert(port.uuid, port);
        }
    }

    let root_sheet = Sheet {
        uuid: root_sheet_uuid,
        name: "Root".into(),
        frame: None,
        symbols,
        wires,
        junctions,
        labels,
        buses,
        bus_entries: HashMap::new(),
        ports,
        noconnects,
        texts: HashMap::new(),
        drawings: HashMap::new(),
    };

    Ok(Schematic {
        uuid: root_uuid,
        sheets: HashMap::from([(root_sheet_uuid, root_sheet)]),
        sheet_definitions,
        sheet_instances,
        variants: HashMap::<Uuid, Variant>::new(),
        waivers: Vec::<CheckWaiver>::new(),
    })
}

pub(super) fn parse_board_skeleton(path: &Path, contents: &str) -> Result<Board, EngineError> {
    let board_uuid = find_top_level_uuid(contents).unwrap_or_else(Uuid::new_v4);
    let name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("board")
        .to_string();

    let stackup = parse_board_layers(contents);
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

    let mut nets = HashMap::new();
    let mut net_lookup = HashMap::new();
    for block in top_level_blocks(contents, "net") {
        if let Some((net_code, net_name)) = parse_net_block(&block) {
            let uuid = deterministic_kicad_board_uuid("net", &net_code.to_string());
            net_lookup.insert(net_code, uuid);
            nets.insert(
                uuid,
                Net {
                    uuid,
                    name: net_name,
                    class: Uuid::nil(),
                },
            );
        }
    }

    let mut packages = HashMap::new();
    let mut pads = HashMap::new();
    for block in top_level_blocks(contents, "footprint") {
        let uuid = block_uuid(&block).unwrap_or_else(Uuid::new_v4);
        let reference =
            extract_footprint_property(&block, "Reference").unwrap_or_else(|| "?".into());
        let value = extract_footprint_property(&block, "Value").unwrap_or_default();
        let position = block_at_point(&block).unwrap_or_else(Point::zero);
        let rotation = block_rotation(&block).unwrap_or(0);
        let layer = block_layer_name(&block)
            .as_deref()
            .map(kicad_layer_name_to_id)
            .unwrap_or(0);
        packages.insert(
            uuid,
            PlacedPackage {
                uuid,
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
        for pad in footprint_pads(&block, uuid, position, rotation, layer, &net_lookup) {
            pads.insert(pad.uuid, pad);
        }
    }

    let mut tracks = HashMap::new();
    for block in top_level_blocks(contents, "segment") {
        if let (Some(uuid), Some((from, to)), Some(width), Some(layer_name), Some(net_code)) = (
            block_uuid(&block),
            block_start_end_points(&block),
            block_width_mm(&block),
            block_layer_name(&block),
            block_net_code(&block),
        ) {
            let net = net_lookup
                .get(&net_code)
                .copied()
                .unwrap_or_else(|| deterministic_kicad_board_uuid("net", &net_code.to_string()));
            tracks.insert(
                uuid,
                Track {
                    uuid,
                    net,
                    from,
                    to,
                    width: mm_to_nm(width),
                    layer: kicad_layer_name_to_id(&layer_name),
                },
            );
        }
    }

    let mut vias = HashMap::new();
    for block in top_level_blocks(contents, "via") {
        if let (
            Some(uuid),
            Some(position),
            Some(diameter),
            Some(drill),
            Some((from_layer, to_layer)),
            Some(net_code),
        ) = (
            block_uuid(&block),
            block_at_point(&block),
            block_size_mm(&block),
            block_drill_mm(&block),
            block_layers_pair(&block),
            block_net_code(&block),
        ) {
            let net = net_lookup
                .get(&net_code)
                .copied()
                .unwrap_or_else(|| deterministic_kicad_board_uuid("net", &net_code.to_string()));
            vias.insert(
                uuid,
                Via {
                    uuid,
                    net,
                    position,
                    drill: mm_to_nm(drill),
                    diameter: mm_to_nm(diameter),
                    from_layer: kicad_layer_name_to_id(&from_layer),
                    to_layer: kicad_layer_name_to_id(&to_layer),
                },
            );
        }
    }

    let mut zones = HashMap::new();
    for block in top_level_blocks(contents, "zone") {
        if let (Some(uuid), Some(net_code), Some(layer_name), Some(polygon)) = (
            block_uuid(&block),
            block_net_code(&block),
            block_layer_name(&block),
            block_polygon(&block),
        ) {
            let net = net_lookup
                .get(&net_code)
                .copied()
                .unwrap_or_else(|| deterministic_kicad_board_uuid("net", &net_code.to_string()));
            zones.insert(
                uuid,
                Zone {
                    uuid,
                    net,
                    polygon,
                    layer: kicad_layer_name_to_id(&layer_name),
                    priority: 0,
                    thermal_relief: true,
                    thermal_gap: 0,
                    thermal_spoke_width: 0,
                },
            );
        }
    }

    let mut texts = Vec::new();
    for block in top_level_blocks(contents, "gr_text") {
        let Some(position) = block_at_point(&block) else {
            continue;
        };
        let layer = block_layer_name(&block)
            .as_deref()
            .map(kicad_layer_name_to_id)
            .unwrap_or(0);
        let text = block_head_string(&block, "gr_text").unwrap_or_default();
        let uuid = block_uuid(&block).unwrap_or_else(|| {
            deterministic_kicad_board_uuid(
                "gr_text",
                &format!("{text}/{}/{}/{}", position.x, position.y, layer),
            )
        });
        texts.push(crate::board::BoardText {
            uuid,
            text,
            position,
            rotation: block_rotation(&block).unwrap_or(0),
            layer,
        });
    }
    texts.sort_by_key(|text| text.uuid);

    let outline = outline_from_edge_cuts(contents).unwrap_or_else(default_outline);

    Ok(Board {
        uuid: board_uuid,
        name,
        stackup,
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
    })
}

fn footprint_pads(
    block: &str,
    package_uuid: Uuid,
    package_position: Point,
    package_rotation_deg: i32,
    package_layer: i32,
    net_lookup: &HashMap<i32, Uuid>,
) -> Vec<PlacedPad> {
    nested_blocks(block, "pad")
        .into_iter()
        .filter_map(|pad_block| {
            let name = block_head_string(&pad_block, "pad")?;
            let local = block_at_point(&pad_block).unwrap_or_else(Point::zero);
            let uuid = block_uuid(&pad_block).unwrap_or_else(|| {
                deterministic_kicad_board_uuid("pad", &format!("{package_uuid}/{name}"))
            });
            let net = block_net_code(&pad_block).map(|code| {
                net_lookup
                    .get(&code)
                    .copied()
                    .unwrap_or_else(|| deterministic_kicad_board_uuid("net", &code.to_string()))
            });
            Some(PlacedPad {
                uuid,
                package: package_uuid,
                name,
                net,
                position: transform_board_local_point(
                    package_position,
                    package_rotation_deg,
                    local,
                ),
                layer: package_layer,
                diameter: 0,
            })
        })
        .collect()
}
