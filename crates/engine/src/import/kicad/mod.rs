use std::collections::HashMap;
use std::path::Path;

use uuid::Uuid;

use crate::board::{
    Board, Net, NetClass, PlacedPackage, PlacedPad, Stackup, StackupLayer, StackupLayerType, Track,
    Via, Zone,
};
use crate::error::EngineError;
use crate::import::{ImportKind, ImportObjectCounts, ImportReport};
use crate::ir::geometry::{Point, Polygon};
use crate::schematic::{
    Bus, CheckWaiver, HiddenPowerBehavior, HierarchicalPort, Junction, LabelKind, NetLabel,
    NoConnectMarker, PinElectricalType, PlacedSymbol, PortDirection, Schematic, SchematicWire,
    Sheet, SheetDefinition, SheetInstance, SymbolDisplayMode, SymbolField, SymbolPin, Variant,
};

// KiCad importer — see specs/IMPORT_SPEC.md §3

pub fn import_board_file(path: &Path) -> Result<ImportReport, EngineError> {
    let (_board, report) = import_board_document(path)?;
    Ok(report)
}

pub fn import_board_document(path: &Path) -> Result<(Board, ImportReport), EngineError> {
    let contents = std::fs::read_to_string(path)?;
    let board = parse_board_skeleton(path, &contents)?;
    let mut report =
        ImportReport::new(ImportKind::KiCadBoard, path, ImportObjectCounts::default()).with_warning(
            "parsed KiCad board skeleton into canonical nets, packages, tracks, vias, zones, and stackup; full geometry and rule import is not implemented yet",
        );

    if let Some(version) = extract_kicad_board_version(&contents) {
        report = report.with_metadata("kicad_version", version);
    }

    report = report
        .with_metadata(
            "footprint_count",
            count_top_level_form_lines(&contents, "footprint").to_string(),
        )
        .with_metadata(
            "segment_count",
            count_top_level_form_lines(&contents, "segment").to_string(),
        )
        .with_metadata(
            "via_count",
            count_top_level_form_lines(&contents, "via").to_string(),
        )
        .with_metadata(
            "zone_count",
            count_top_level_form_lines(&contents, "zone").to_string(),
        )
        .with_metadata(
            "net_count",
            count_top_level_form_lines(&contents, "net").to_string(),
        )
        .with_metadata(
            "gr_line_count",
            count_top_level_form_lines(&contents, "gr_line").to_string(),
        )
        .with_metadata(
            "gr_arc_count",
            count_top_level_form_lines(&contents, "gr_arc").to_string(),
        )
        .with_metadata(
            "dimension_count",
            count_top_level_form_lines(&contents, "dimension").to_string(),
        )
        .with_metadata(
            "gr_text_count",
            count_top_level_form_lines(&contents, "gr_text").to_string(),
        );

    Ok((board, report))
}

pub fn import_schematic_file(path: &Path) -> Result<ImportReport, EngineError> {
    let (_schematic, report) = import_schematic_document(path)?;
    Ok(report)
}

pub fn import_schematic_document(path: &Path) -> Result<(Schematic, ImportReport), EngineError> {
    let contents = std::fs::read_to_string(path)?;
    let schematic = parse_schematic_skeleton(&contents)?;
    let mut report = ImportReport::new(
        ImportKind::KiCadSchematic,
        path,
        ImportObjectCounts::default(),
    )
    .with_warning(
        "parsed KiCad schematic header and skeleton forms only; full symbol/connectivity import is not implemented yet",
    );

    if let Some(version) = extract_kicad_schematic_version(&contents) {
        report = report.with_metadata("kicad_version", version);
    }

    report = report
        .with_metadata(
            "symbol_count",
            count_top_level_form_lines(&contents, "symbol").to_string(),
        )
        .with_metadata(
            "wire_count",
            count_top_level_form_lines(&contents, "wire").to_string(),
        )
        .with_metadata(
            "junction_count",
            count_top_level_form_lines(&contents, "junction").to_string(),
        )
        .with_metadata(
            "label_count",
            count_top_level_form_lines(&contents, "label").to_string(),
        )
        .with_metadata(
            "global_label_count",
            count_top_level_form_lines(&contents, "global_label").to_string(),
        )
        .with_metadata(
            "hierarchical_label_count",
            count_top_level_form_lines(&contents, "hierarchical_label").to_string(),
        )
        .with_metadata(
            "bus_count",
            count_top_level_form_lines(&contents, "bus").to_string(),
        )
        .with_metadata(
            "sheet_count",
            count_top_level_form_lines(&contents, "sheet").to_string(),
        )
        .with_metadata(
            "no_connect_count",
            count_top_level_form_lines(&contents, "no_connect").to_string(),
        );

    Ok((schematic, report))
}

pub fn import_project_file(path: &Path) -> Result<ImportReport, EngineError> {
    let contents = std::fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&contents).map_err(|err| {
        EngineError::Import(format!(
            "failed to parse KiCad project JSON {}: {err}",
            path.display()
        ))
    })?;

    let mut report = ImportReport::new(
        ImportKind::KiCadProject,
        path,
        ImportObjectCounts::default(),
    )
    .with_warning(
        "parsed KiCad project metadata only; board and schematic import are not implemented yet",
    );

    if let Some(meta) = value.get("meta").and_then(|v| v.as_object()) {
        if let Some(filename) = meta.get("filename").and_then(|v| v.as_str()) {
            report = report.with_metadata("project_name", filename);
        }
        if let Some(version) = meta.get("version") {
            report = report.with_metadata("project_version", version.to_string());
        }
    }

    if !report.metadata.contains_key("project_name")
        && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
    {
        report = report.with_metadata("project_name", stem);
    }

    Ok(report)
}

fn extract_kicad_schematic_version(contents: &str) -> Option<String> {
    let marker = "(version ";
    let start = contents.find(marker)? + marker.len();
    let rest = &contents[start..];
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_string())
}

fn extract_kicad_board_version(contents: &str) -> Option<String> {
    let marker = "(version ";
    let start = contents.find(marker)? + marker.len();
    let rest = &contents[start..];
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_string())
}

fn parse_schematic_skeleton(contents: &str) -> Result<Schematic, EngineError> {
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

#[derive(Debug, Clone)]
struct LibraryPinTemplate {
    number: String,
    name: String,
    electrical_type: PinElectricalType,
    position: Point,
}

fn parse_board_skeleton(path: &Path, contents: &str) -> Result<Board, EngineError> {
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
            })
        })
        .collect()
}

fn count_top_level_form_lines(contents: &str, form: &str) -> usize {
    let prefix = format!("({form}");
    contents
        .lines()
        .filter(|line| {
            let indent = line.len() - line.trim_start().len();
            let trimmed = line.trim_start();
            indent <= 2
                && trimmed.starts_with(&prefix)
                && matches!(
                    trimmed.as_bytes().get(prefix.len()),
                    Some(b' ') | Some(b'\t') | Some(b')') | None
                )
        })
        .count()
}

fn top_level_blocks(contents: &str, form: &str) -> Vec<String> {
    nested_blocks_with_max_indent(contents, form, 2)
}

fn nested_blocks(contents: &str, form: &str) -> Vec<String> {
    nested_blocks_with_max_indent(contents, form, usize::MAX)
}

fn nested_blocks_with_max_indent(contents: &str, form: &str, max_indent: usize) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut capturing = false;
    let mut depth: i32 = 0;
    let prefix = format!("({form}");

    for line in contents.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();

        if !capturing
            && indent <= max_indent
            && trimmed.starts_with(&prefix)
            && matches!(
                trimmed.as_bytes().get(prefix.len()),
                Some(b' ') | Some(b'\t') | Some(b')') | None
            )
        {
            capturing = true;
            current.clear();
            depth = 0;
        }

        if capturing {
            current.push(line.to_string());
            depth += paren_delta(line);
            if depth <= 0 {
                blocks.push(current.join("\n"));
                current.clear();
                capturing = false;
            }
        }
    }
    blocks
}

fn paren_delta(line: &str) -> i32 {
    let opens = line.chars().filter(|c| *c == '(').count() as i32;
    let closes = line.chars().filter(|c| *c == ')').count() as i32;
    opens - closes
}

fn find_top_level_uuid(contents: &str) -> Option<Uuid> {
    for line in contents.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();
        if indent <= 2
            && trimmed.starts_with("(uuid ")
            && let Some(uuid) = parse_uuid_line(trimmed)
        {
            return Some(uuid);
        }
    }
    None
}

fn block_uuid(block: &str) -> Option<Uuid> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if trimmed.starts_with("(uuid ") {
            parse_uuid_line(trimmed)
        } else {
            None
        }
    })
}

fn parse_uuid_line(trimmed: &str) -> Option<Uuid> {
    if let Some(token) = parse_quoted_token(trimmed) {
        return Uuid::parse_str(&token).ok();
    }

    let token = trimmed
        .trim_start_matches("(uuid ")
        .trim_end_matches(')')
        .split_whitespace()
        .next()?;
    Uuid::parse_str(token).ok()
}

fn block_at_point(block: &str) -> Option<Point> {
    block
        .lines()
        .find_map(|line| parse_at_point(line.trim_start()))
}

fn parse_at_point(trimmed: &str) -> Option<Point> {
    if !trimmed.starts_with("(at ") {
        return None;
    }
    let rest = trimmed.trim_start_matches("(at ").trim_end_matches(')');
    let mut parts = rest.split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    Some(mm_point_to_nm(x, y))
}

fn block_xy_points(block: &str) -> Vec<Point> {
    let mut points = Vec::new();
    for line in block.lines() {
        let trimmed = line.trim_start();
        points.extend(parse_xy_points_from_line(trimmed));
    }
    points
}

fn parse_xy_points_from_line(line: &str) -> Vec<Point> {
    let mut points = Vec::new();
    let mut rest = line;
    let marker = "(xy ";

    while let Some(start) = rest.find(marker) {
        let after = &rest[start + marker.len()..];
        let Some(end) = after.find(')') else {
            break;
        };
        let mut parts = after[..end].split_whitespace();
        let Some(x) = parts.next().and_then(|v| v.parse::<f64>().ok()) else {
            rest = &after[end + 1..];
            continue;
        };
        let Some(y) = parts.next().and_then(|v| v.parse::<f64>().ok()) else {
            rest = &after[end + 1..];
            continue;
        };
        points.push(mm_point_to_nm(x, y));
        rest = &after[end + 1..];
    }

    points
}

fn block_head_string(block: &str, form: &str) -> Option<String> {
    let first = block.lines().next()?.trim_start();
    let prefix = format!("({form} ");
    if !first.starts_with(&prefix) {
        return None;
    }
    let after = &first[prefix.len()..];
    let start = after.find('"')?;
    let rest = &after[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn parse_net_block(block: &str) -> Option<(i32, String)> {
    let first = block.lines().next()?.trim_start();
    if !first.starts_with("(net ") {
        return None;
    }
    let after = first.trim_start_matches("(net ").trim_end_matches(')');
    let mut chars = after.chars().peekable();
    let mut code = String::new();
    while let Some(ch) = chars.peek() {
        if ch.is_ascii_whitespace() {
            break;
        }
        code.push(*ch);
        chars.next();
    }
    let code = code.parse::<i32>().ok()?;
    let rest: String = chars.collect();
    let start = rest.find('"')?;
    let rest = &rest[start + 1..];
    let end = rest.find('"')?;
    Some((code, rest[..end].to_string()))
}

fn block_rotation(block: &str) -> Option<i32> {
    let first = block
        .lines()
        .find_map(|line| parse_at_rotation(line.trim_start()))?;
    Some(first)
}

fn parse_at_rotation(trimmed: &str) -> Option<i32> {
    if !trimmed.starts_with("(at ") {
        return None;
    }
    let rest = trimmed.trim_start_matches("(at ").trim_end_matches(')');
    let mut parts = rest.split_whitespace();
    parts.next()?;
    parts.next()?;
    let rotation = parts.next()?.parse::<f64>().ok()?;
    Some(rotation.round() as i32)
}

fn extract_footprint_property(block: &str, key: &str) -> Option<String> {
    let needle = format!("(property \"{key}\" ");
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with(&needle) {
            return None;
        }
        let after = &trimmed[needle.len()..];
        let start = after.find('"')?;
        let rest = &after[start + 1..];
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    })
}

fn block_layer_name(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("(layer ") {
            return None;
        }
        parse_quoted_token(trimmed)
    })
}

fn block_layers_pair(block: &str) -> Option<(String, String)> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("(layers ") {
            return None;
        }
        let tokens = quoted_tokens(trimmed);
        if tokens.len() >= 2 {
            Some((tokens[0].clone(), tokens[1].clone()))
        } else {
            None
        }
    })
}

fn block_net_code(block: &str) -> Option<i32> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("(net ") {
            return None;
        }
        trimmed
            .trim_start_matches("(net ")
            .trim_end_matches(')')
            .split_whitespace()
            .next()?
            .parse::<i32>()
            .ok()
    })
}

fn block_width_mm(block: &str) -> Option<f64> {
    block
        .lines()
        .find_map(|line| parse_scalar_mm(line.trim_start(), "width"))
}

fn block_size_mm(block: &str) -> Option<f64> {
    block
        .lines()
        .find_map(|line| parse_scalar_mm(line.trim_start(), "size"))
}

fn block_drill_mm(block: &str) -> Option<f64> {
    block
        .lines()
        .find_map(|line| parse_scalar_mm(line.trim_start(), "drill"))
}

fn parse_scalar_mm(trimmed: &str, form: &str) -> Option<f64> {
    let prefix = format!("({form} ");
    if !trimmed.starts_with(&prefix) {
        return None;
    }
    trimmed[prefix.len()..]
        .trim_end_matches(')')
        .split_whitespace()
        .next()?
        .parse::<f64>()
        .ok()
}

fn block_start_end_points(block: &str) -> Option<(Point, Point)> {
    let mut start = None;
    let mut end = None;
    for line in block.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("(start ") {
            start = parse_xy_like(trimmed, "start");
        } else if trimmed.starts_with("(end ") {
            end = parse_xy_like(trimmed, "end");
        }
    }
    Some((start?, end?))
}

fn parse_xy_like(trimmed: &str, form: &str) -> Option<Point> {
    let prefix = format!("({form} ");
    if !trimmed.starts_with(&prefix) {
        return None;
    }
    let rest = trimmed[prefix.len()..].trim_end_matches(')');
    let mut parts = rest.split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    Some(mm_point_to_nm(x, y))
}

fn block_polygon(block: &str) -> Option<Polygon> {
    let points = block_xy_points(block);
    if points.is_empty() {
        None
    } else {
        Some(Polygon::new(points))
    }
}

fn parse_board_layers(contents: &str) -> Stackup {
    let mut layers = Vec::new();
    for block in top_level_blocks(contents, "layers") {
        for line in block.lines() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with('(') || trimmed.starts_with("(layers") {
                continue;
            }
            if let Some(layer) = parse_layer_line(trimmed) {
                layers.push(layer);
            }
        }
    }

    if layers.is_empty() {
        layers.push(StackupLayer {
            id: 0,
            name: "F.Cu".into(),
            layer_type: StackupLayerType::Copper,
            thickness_nm: 35_000,
        });
    }

    layers.sort_by_key(|layer| layer.id);
    Stackup { layers }
}

fn parse_layer_line(trimmed: &str) -> Option<StackupLayer> {
    let inner = trimmed.strip_prefix('(')?.trim_end_matches(')');
    let mut parts = inner.split_whitespace();
    let id = parts.next()?.parse::<i32>().ok()?;
    let name = parse_next_quoted_from(inner)?;
    let layer_type = if inner.contains(" signal") {
        StackupLayerType::Copper
    } else {
        StackupLayerType::Mechanical
    };
    Some(StackupLayer {
        id,
        name,
        layer_type,
        thickness_nm: 0,
    })
}

fn outline_from_edge_cuts(contents: &str) -> Option<Polygon> {
    let mut points = Vec::new();
    for form in ["gr_line", "gr_arc"] {
        for block in top_level_blocks(contents, form) {
            let Some(layer_name) = block_layer_name(&block) else {
                continue;
            };
            if layer_name != "Edge.Cuts" {
                continue;
            }
            for line in block.lines() {
                let trimmed = line.trim_start();
                if (trimmed.starts_with("(start ")
                    || trimmed.starts_with("(end ")
                    || trimmed.starts_with("(mid "))
                    && let Some(point) = parse_xy_like(
                        trimmed,
                        if trimmed.starts_with("(start ") {
                            "start"
                        } else if trimmed.starts_with("(end ") {
                            "end"
                        } else {
                            "mid"
                        },
                    )
                {
                    points.push(point);
                }
            }
        }
    }

    if points.is_empty() {
        return None;
    }

    let min_x = points.iter().map(|p| p.x).min()?;
    let min_y = points.iter().map(|p| p.y).min()?;
    let max_x = points.iter().map(|p| p.x).max()?;
    let max_y = points.iter().map(|p| p.y).max()?;
    Some(Polygon::new(vec![
        Point::new(min_x, min_y),
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
        Point::new(min_x, max_y),
    ]))
}

fn default_outline() -> Polygon {
    Polygon::new(vec![
        Point::new(0, 0),
        Point::new(10_000_000, 0),
        Point::new(10_000_000, 10_000_000),
        Point::new(0, 10_000_000),
    ])
}

fn parse_quoted_token(trimmed: &str) -> Option<String> {
    let start = trimmed.find('"')?;
    let rest = &trimmed[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn quoted_tokens(trimmed: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut rest = trimmed;
    while let Some(start) = rest.find('"') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('"') else {
            break;
        };
        tokens.push(after_start[..end].to_string());
        rest = &after_start[end + 1..];
    }
    tokens
}

fn parse_next_quoted_from(inner: &str) -> Option<String> {
    let start = inner.find('"')?;
    let rest = &inner[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn deterministic_kicad_board_uuid(kind: &str, key: &str) -> Uuid {
    crate::ir::ids::import_uuid(
        &crate::ir::ids::namespace_kicad(),
        &format!("board/{kind}/{key}"),
    )
}

fn kicad_layer_name_to_id(name: &str) -> i32 {
    match name {
        "F.Cu" => 0,
        "B.Cu" => 31,
        "B.SilkS" => 36,
        "F.SilkS" => 37,
        "Edge.Cuts" => 44,
        _ => 0,
    }
}

fn mm_to_nm(mm: f64) -> i64 {
    (mm * 1_000_000.0).round() as i64
}

fn transform_symbol_pin(origin: Point, rotation_deg: i32, mirrored_y: bool, local: Point) -> Point {
    let local = if mirrored_y {
        // KiCad `mirror y` reflects the symbol about the Y axis, so the
        // local X coordinate changes sign before rotation is applied.
        Point::new(-local.x, local.y)
    } else {
        local
    };
    let rotated = match rotation_deg.rem_euclid(360) {
        90 => Point::new(-local.y, local.x),
        180 => Point::new(-local.x, -local.y),
        270 => Point::new(local.y, -local.x),
        _ => local,
    };
    Point::new(origin.x + rotated.x, origin.y + rotated.y)
}

fn transform_board_local_point(origin: Point, rotation_deg: i32, local: Point) -> Point {
    let rotated = match rotation_deg.rem_euclid(360) {
        90 => Point::new(-local.y, local.x),
        180 => Point::new(-local.x, -local.y),
        270 => Point::new(local.y, -local.x),
        _ => local,
    };
    Point::new(origin.x + rotated.x, origin.y + rotated.y)
}

fn extract_sheet_property(block: &str, key: &str) -> Option<String> {
    let needle = format!("(property \"{key}\" ");
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with(&needle) {
            return None;
        }
        let after = &trimmed[needle.len()..];
        let start = after.find('"')?;
        let rest = &after[start + 1..];
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    })
}

fn extract_symbol_property(block: &str, key: &str) -> Option<String> {
    extract_sheet_property(block, key)
}

fn symbol_is_mirrored_y(block: &str) -> bool {
    block.lines().any(|line| line.trim() == "(mirror y)")
}

fn extract_symbol_lib_id(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("(lib_id ") {
            return None;
        }
        parse_quoted_token(trimmed)
    })
}

fn extract_symbol_unit(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("(unit ") {
            return None;
        }
        Some(
            trimmed
                .trim_start_matches("(unit ")
                .trim_end_matches(')')
                .trim()
                .to_string(),
        )
    })
}

fn symbol_fields(block: &str) -> Vec<SymbolField> {
    let mut fields = Vec::new();
    for line in block.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("(property ") {
            continue;
        }
        let tokens = quoted_tokens(trimmed);
        if tokens.len() < 2 {
            continue;
        }
        fields.push(SymbolField {
            uuid: Uuid::new_v4(),
            key: tokens[0].clone(),
            value: tokens[1].clone(),
            position: parse_at_point(trimmed),
            visible: true,
        });
    }
    fields
}

fn parse_library_symbol_pins(contents: &str) -> HashMap<String, Vec<LibraryPinTemplate>> {
    let mut libraries = HashMap::new();
    let Some(lib_symbols_block) = top_level_blocks(contents, "lib_symbols").into_iter().next()
    else {
        return libraries;
    };

    for symbol_block in nested_blocks(&lib_symbols_block, "symbol") {
        let Some(lib_id) = block_head_string(&symbol_block, "symbol") else {
            continue;
        };
        if !lib_id.contains(':') {
            continue;
        }
        let mut pins = Vec::new();
        for pin_block in nested_blocks(&symbol_block, "pin") {
            let number = extract_named_subfield(&pin_block, "number").unwrap_or_else(|| "?".into());
            let name = extract_named_subfield(&pin_block, "name").unwrap_or_else(|| number.clone());
            let electrical_type = parse_kicad_pin_electrical_type(&pin_block);
            let position = block_at_point(&pin_block).unwrap_or_else(Point::zero);
            pins.push(LibraryPinTemplate {
                number,
                name,
                electrical_type,
                position,
            });
        }
        libraries.insert(lib_id, pins);
    }

    libraries
}

fn extract_named_subfield(block: &str, field: &str) -> Option<String> {
    let needle = format!("({field} ");
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with(&needle) {
            return None;
        }
        parse_quoted_token(trimmed)
    })
}

fn parse_kicad_pin_electrical_type(pin_block: &str) -> PinElectricalType {
    let first = pin_block
        .lines()
        .next()
        .map(str::trim_start)
        .unwrap_or_default();
    if first.contains("(pin output ") || first.contains("(pin tri_state ") {
        PinElectricalType::Output
    } else if first.contains("(pin input ") {
        PinElectricalType::Input
    } else if first.contains("(pin bidirectional ") {
        PinElectricalType::Bidirectional
    } else if first.contains("(pin power_in ") {
        PinElectricalType::PowerIn
    } else if first.contains("(pin power_out ") {
        PinElectricalType::PowerOut
    } else {
        PinElectricalType::Passive
    }
}

fn extract_sheet_pins(block: &str) -> Vec<HierarchicalPort> {
    let mut ports = Vec::new();
    for pin_block in nested_blocks(block, "pin") {
        let Some(first_line) = pin_block.lines().next() else {
            continue;
        };
        let trimmed = first_line.trim_start();
        let tokens = quoted_tokens(trimmed);
        let Some(name) = tokens.first().cloned() else {
            continue;
        };
        let direction = if trimmed.contains(" output") {
            PortDirection::Output
        } else if trimmed.contains(" bidirectional") {
            PortDirection::Bidirectional
        } else if trimmed.contains(" passive") {
            PortDirection::Passive
        } else {
            PortDirection::Input
        };
        let position = block_at_point(&pin_block).unwrap_or_else(Point::zero);
        ports.push(HierarchicalPort {
            uuid: Uuid::new_v4(),
            name,
            direction,
            position,
        });
    }
    ports
}

fn mm_point_to_nm(x_mm: f64, y_mm: f64) -> Point {
    Point::new(
        (x_mm * 1_000_000.0).round() as i64,
        (y_mm * 1_000_000.0).round() as i64,
    )
}

fn pin_at_position(symbols: &HashMap<Uuid, PlacedSymbol>, position: Point) -> Option<(Uuid, Uuid)> {
    let mut matches: Vec<(Uuid, Uuid)> = symbols
        .iter()
        .flat_map(|(symbol_uuid, symbol)| {
            symbol
                .pins
                .iter()
                .filter(|pin| pin.position == position)
                .map(|pin| (*symbol_uuid, pin.uuid))
                .collect::<Vec<_>>()
        })
        .collect();
    matches.sort();
    matches.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/import/kicad")
            .join(name)
    }

    fn optional_doa2526_board_path() -> Option<std::path::PathBuf> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_pcb");
        path.exists().then_some(path)
    }

    fn optional_doa2526_schematic_path() -> Option<std::path::PathBuf> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_sch");
        path.exists().then_some(path)
    }

    #[test]
    fn imports_kicad_project_metadata() {
        let report = import_project_file(&fixture_path("simple-demo.kicad_pro"))
            .expect("fixture should parse");

        assert_eq!(report.kind, ImportKind::KiCadProject);
        assert!(report.counts.is_empty());
        assert_eq!(
            report.metadata.get("project_name").map(String::as_str),
            Some("simple-demo")
        );
        assert_eq!(
            report.metadata.get("project_version").map(String::as_str),
            Some("1")
        );
        assert_eq!(report.warnings.len(), 1);
    }

    #[test]
    fn imports_kicad_board_header_and_skeleton_counts() {
        let (board, report) = import_board_document(&fixture_path("simple-demo.kicad_pcb"))
            .expect("fixture should parse");

        assert_eq!(report.kind, ImportKind::KiCadBoard);
        assert!(report.counts.is_empty());
        assert_eq!(
            report.metadata.get("kicad_version").map(String::as_str),
            Some("20221018")
        );
        assert_eq!(
            report.metadata.get("footprint_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report.metadata.get("segment_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report.metadata.get("via_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report.metadata.get("zone_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report.metadata.get("net_count").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            report.metadata.get("gr_line_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report.metadata.get("gr_arc_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report.metadata.get("dimension_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report.metadata.get("gr_text_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(report.warnings.len(), 1);
        assert_eq!(board.name, "simple-demo");
        assert_eq!(board.packages.len(), 1);
        assert_eq!(board.tracks.len(), 1);
        assert_eq!(board.vias.len(), 1);
        assert_eq!(board.zones.len(), 1);
        assert_eq!(board.nets.len(), 2);
        assert_eq!(board.texts.len(), 1);
        assert_eq!(board.texts[0].text, "Demo");
        assert_eq!(board.texts[0].layer, 37);
        assert_eq!(board.stackup.layers.len(), 3);
    }

    #[test]
    fn imports_kicad_board_pads_for_unrouted_computation() {
        let (board, report) = import_board_document(&fixture_path("airwire-demo.kicad_pcb"))
            .expect("fixture should parse");

        assert_eq!(report.kind, ImportKind::KiCadBoard);
        assert_eq!(board.packages.len(), 2);
        assert_eq!(board.pads.len(), 2);
        assert_eq!(board.tracks.len(), 0);
        assert_eq!(board.vias.len(), 0);
        assert_eq!(board.zones.len(), 0);

        let nets = board.net_info();
        assert_eq!(nets.len(), 2);
        let sig = nets
            .iter()
            .find(|net| net.name == "SIG")
            .expect("SIG net should exist");
        assert_eq!(sig.pins.len(), 2);
        assert_eq!(sig.tracks, 0);

        let airwires = board.unrouted();
        assert_eq!(airwires.len(), 1);
        assert_eq!(airwires[0].net_name, "SIG");
        assert_eq!(airwires[0].from.component, "R1");
        assert_eq!(airwires[0].to.component, "R2");
    }

    #[test]
    fn imports_kicad_board_partial_route_still_reports_airwire() {
        let (board, report) = import_board_document(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("fixture should parse");

        assert_eq!(report.kind, ImportKind::KiCadBoard);
        assert_eq!(board.packages.len(), 2);
        assert_eq!(board.pads.len(), 2);
        assert_eq!(board.tracks.len(), 1);

        let nets = board.net_info();
        let sig = nets
            .iter()
            .find(|net| net.name == "SIG")
            .expect("SIG net should exist");
        assert_eq!(sig.pins.len(), 2);
        assert_eq!(sig.tracks, 1);

        let airwires = board.unrouted();
        assert_eq!(airwires.len(), 1);
        assert_eq!(airwires[0].net_name, "SIG");
        assert_eq!(airwires[0].from.component, "R1");
        assert_eq!(airwires[0].to.component, "R2");
    }

    #[test]
    fn imports_real_doa2526_board_with_copper_geometry() {
        let Some(path) = optional_doa2526_board_path() else {
            return;
        };

        let (board, report) = import_board_document(&path).expect("DOA2526 board should parse");

        assert_eq!(report.kind, ImportKind::KiCadBoard);
        assert!(!board.packages.is_empty());
        assert!(!board.pads.is_empty());
        assert!(
            !board.tracks.is_empty(),
            "real DOA2526 board should import tracks"
        );
        assert!(
            !board.vias.is_empty(),
            "real DOA2526 board should import vias"
        );
        assert!(
            !board.zones.is_empty(),
            "real DOA2526 board should import zones"
        );

        let diagnostics = board.diagnostics();
        assert!(
            diagnostics.iter().any(|d| d.kind != "net_without_copper"),
            "real DOA2526 board should not collapse to only empty-copper diagnostics"
        );

        let net_info = board.net_info();
        assert!(
            net_info
                .iter()
                .any(|net| net.tracks > 0 || net.vias > 0 || net.zones > 0),
            "real DOA2526 board should report imported copper on at least one net"
        );
    }

    #[test]
    fn imports_real_doa2526_schematic_without_collapsing_anonymous_nets() {
        let Some(path) = optional_doa2526_schematic_path() else {
            return;
        };

        let (schematic, report) =
            import_schematic_document(&path).expect("DOA2526 schematic should parse");

        assert_eq!(report.kind, ImportKind::KiCadSchematic);

        let nets = crate::connectivity::schematic_net_info(&schematic);
        assert!(!nets.is_empty(), "real DOA2526 schematic should yield nets");

        let unique_net_ids: std::collections::HashSet<_> =
            nets.iter().map(|net| net.uuid).collect();
        assert_eq!(
            unique_net_ids.len(),
            nets.len(),
            "real DOA2526 schematic should not reuse the same net UUID across distinct aggregates"
        );

        let anonymous_names: std::collections::HashSet<_> = nets
            .iter()
            .filter(|net| net.name.starts_with("N$"))
            .map(|net| net.name.clone())
            .collect();
        assert!(
            anonymous_names.len() > 1,
            "real DOA2526 schematic should produce distinct anonymous net identities"
        );
    }

    #[test]
    fn imports_real_doa2526_plusin_pin_at_expected_position() {
        let Some(path) = optional_doa2526_schematic_path() else {
            return;
        };

        let (schematic, _report) =
            import_schematic_document(&path).expect("DOA2526 schematic should parse");
        let root = schematic
            .sheets
            .values()
            .find(|sheet| sheet.name == "Root")
            .expect("root sheet should exist");
        let plus_in = root
            .symbols
            .values()
            .find(|symbol| symbol.reference == "+In1")
            .expect("+In1 should exist");
        let pin = plus_in
            .pins
            .iter()
            .find(|pin| pin.number == "1")
            .expect("+In1 pin 1 should exist");

        assert_eq!(pin.position.x, 39_370_000);
        assert_eq!(pin.position.y, 105_410_000);
    }

    #[test]
    fn mirrored_symbol_pin_transform_reflects_local_x_before_rotation() {
        let origin = Point::new(73_660_000, 105_410_000);

        let base = transform_symbol_pin(origin, 0, true, Point::new(-5_080_000, 0));
        let collector = transform_symbol_pin(origin, 0, true, Point::new(2_540_000, 5_080_000));
        let emitter = transform_symbol_pin(origin, 0, true, Point::new(2_540_000, -5_080_000));

        assert_eq!(base, Point::new(78_740_000, 105_410_000));
        assert_eq!(collector, Point::new(71_120_000, 110_490_000));
        assert_eq!(emitter, Point::new(71_120_000, 100_330_000));
    }

    #[test]
    fn real_doa2526_named_nets_attach_expected_pins() {
        let Some(path) = optional_doa2526_schematic_path() else {
            return;
        };

        let (schematic, _report) =
            import_schematic_document(&path).expect("DOA2526 schematic should parse");
        let nets = crate::connectivity::schematic_net_info(&schematic);

        let in_p = nets
            .iter()
            .find(|net| net.name == "IN_P")
            .expect("IN_P net should exist");
        assert!(
            in_p.pins
                .iter()
                .any(|pin| pin.component == "+In1" && pin.pin == "1")
        );
        assert!(
            in_p.pins
                .iter()
                .any(|pin| pin.component == "Q1" && pin.pin == "1")
        );

        let vcc = nets
            .iter()
            .find(|net| net.name == "VCC")
            .expect("VCC net should exist");
        assert!(
            vcc.pins
                .iter()
                .any(|pin| pin.component == "+Supply1" && pin.pin == "1")
        );
        assert!(
            vcc.pins
                .iter()
                .any(|pin| pin.component == "#FLG01" && pin.pin == "1")
        );
        assert!(
            vcc.pins.len() >= 4,
            "VCC should attach multiple component pins on DOA2526"
        );

        let q2_col = nets
            .iter()
            .find(|net| net.name == "IN_Q2_COL")
            .expect("IN_Q2_COL net should exist");
        assert!(
            q2_col
                .pins
                .iter()
                .any(|pin| pin.component == "Q2" && pin.pin == "2"),
            "Q2 emitter should land on IN_Q2_COL after mirrored pin transform"
        );

        let ff_q4_c = nets
            .iter()
            .find(|net| net.name == "FF_Q4_C")
            .expect("FF_Q4_C net should exist");
        assert!(
            ff_q4_c
                .pins
                .iter()
                .any(|pin| pin.component == "Q4" && pin.pin == "2"),
            "Q4 emitter should land on FF_Q4_C after mirrored pin transform"
        );

        let in_n = nets
            .iter()
            .find(|net| net.name == "IN_N")
            .expect("IN_N net should exist");
        assert!(
            in_n.pins
                .iter()
                .any(|pin| pin.component == "Q2" && pin.pin == "1"),
            "Q2 base should land on IN_N after mirrored pin transform"
        );

        let in_q1_col = nets
            .iter()
            .find(|net| net.name == "IN_Q1_COL")
            .expect("IN_Q1_COL net should exist");
        assert!(
            in_q1_col
                .pins
                .iter()
                .any(|pin| pin.component == "Q4" && pin.pin == "1"),
            "Q4 base should land on IN_Q1_COL after mirrored pin transform"
        );
    }

    #[test]
    fn real_doa2526_mirrored_transistors_are_not_reported_as_unconnected() {
        let Some(path) = optional_doa2526_schematic_path() else {
            return;
        };

        let (schematic, _report) =
            import_schematic_document(&path).expect("DOA2526 schematic should parse");
        let findings = crate::erc::run_prechecks(&schematic);

        for pin in ["Q2.1", "Q2.2", "Q2.3", "Q4.1", "Q4.2", "Q4.3"] {
            assert!(
                !findings.iter().any(|finding| {
                    finding.code == "unconnected_component_pin"
                        && finding
                            .objects
                            .iter()
                            .any(|object| object.kind == "pin" && object.key == pin)
                }),
                "{pin} should no longer be reported as an unconnected component pin on DOA2526"
            );
        }
    }

    #[test]
    fn imports_kicad_schematic_header_and_skeleton_counts() {
        let report = import_schematic_file(&fixture_path("simple-demo.kicad_sch"))
            .expect("fixture should parse");

        assert_eq!(report.kind, ImportKind::KiCadSchematic);
        assert!(report.counts.is_empty());
        assert_eq!(
            report.metadata.get("kicad_version").map(String::as_str),
            Some("20230121")
        );
        assert_eq!(
            report.metadata.get("symbol_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report.metadata.get("wire_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report.metadata.get("junction_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report.metadata.get("label_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report
                .metadata
                .get("global_label_count")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report
                .metadata
                .get("hierarchical_label_count")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report.metadata.get("bus_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report.metadata.get("sheet_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            report.metadata.get("no_connect_count").map(String::as_str),
            Some("1")
        );
        assert_eq!(report.warnings.len(), 1);
    }

    #[test]
    fn imports_kicad_schematic_into_canonical_objects() {
        let (schematic, report) = import_schematic_document(&fixture_path("simple-demo.kicad_sch"))
            .expect("fixture should parse");

        assert_eq!(report.kind, ImportKind::KiCadSchematic);
        assert_eq!(schematic.sheets.len(), 1);

        let root = schematic.sheets.values().next().unwrap();
        assert_eq!(root.symbols.len(), 1);
        assert_eq!(root.wires.len(), 1);
        assert_eq!(root.junctions.len(), 1);
        assert_eq!(root.labels.len(), 3);
        assert_eq!(root.buses.len(), 1);
        assert_eq!(root.ports.len(), 1);
        assert_eq!(root.noconnects.len(), 1);
        assert_eq!(schematic.sheet_instances.len(), 1);

        let symbol = root
            .symbols
            .values()
            .find(|symbol| symbol.reference == "R1")
            .expect("R1 symbol should exist");
        assert_eq!(symbol.value, "10k");
        assert_eq!(symbol.lib_id.as_deref(), Some("Device:R"));
        assert_eq!(symbol.position, Point::new(25_000_000, 20_000_000));
        assert_eq!(symbol.pins.len(), 2);
        assert!(symbol.pins.iter().any(|pin| {
            pin.number == "1"
                && pin.position == Point::new(20_000_000, 20_000_000)
                && pin.electrical_type == PinElectricalType::Passive
        }));

        let local = root
            .labels
            .values()
            .find(|label| label.kind == LabelKind::Local)
            .expect("local label should exist");
        assert_eq!(local.name, "SCL");
        assert_eq!(local.position, Point::new(20_000_000, 20_000_000));

        let hier = root
            .labels
            .values()
            .find(|label| label.kind == LabelKind::Hierarchical)
            .expect("hierarchical label should exist");
        assert_eq!(hier.name, "SUB_IN");
        assert_eq!(hier.position, Point::new(80_000_000, 15_000_000));

        let port = root
            .ports
            .values()
            .find(|port| port.name == "SUB_IN")
            .expect("sheet pin should exist");
        assert_eq!(port.direction, PortDirection::Input);
        assert_eq!(port.position, Point::new(60_000_000, 15_000_000));
    }

    #[test]
    fn imports_kicad_noconnect_with_pin_binding_when_marker_overlaps_pin() {
        let (schematic, _report) =
            import_schematic_document(&fixture_path("erc-coverage-demo.kicad_sch"))
                .expect("fixture should parse");
        let root = schematic
            .sheets
            .values()
            .next()
            .expect("root sheet should exist");
        let marker = root
            .noconnects
            .values()
            .next()
            .expect("fixture should include one no_connect marker");
        assert_ne!(marker.symbol, Uuid::nil());
        assert_ne!(marker.pin, Uuid::nil());
    }
}
