use super::*;

impl Engine {

    /// M3 save entry point for the current implemented write-back slice.
    ///
    /// Current scope: imported KiCad boards can be written back byte-identically
    /// when unmodified, and can persist the current `delete_track` slice by
    /// removing deleted top-level KiCad `segment` forms from the imported
    /// source text.
    pub fn save(&self, path: &Path) -> Result<(), EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let imported = self.imported_source.as_ref().ok_or_else(|| {
            EngineError::Operation("save requires imported design provenance".to_string())
        })?;

        if design.board.is_none() {
            return Err(EngineError::Operation(
                "save is currently implemented only for imported KiCad boards".to_string(),
            ));
        }
        if imported.kind != ImportKind::KiCadBoard {
            return Err(EngineError::Operation(format!(
                "save is currently implemented only for imported KiCad boards; current design kind is {}",
                imported.kind.as_str()
            )));
        }

        let serialized = serialize_current_kicad_board_slice(
            &imported.original_contents,
            design
                .board
                .as_ref()
                .ok_or_else(|| EngineError::Operation("save requires imported board".to_string()))?,
            &self.pool,
            &self.undo_stack,
        )?;
        std::fs::write(path, serialized.as_bytes())?;
        persist_rule_sidecar(
            path,
            &serialized,
            design
                .board
                .as_ref()
                .map(|board| board.rules.clone())
                .unwrap_or_default(),
            imported.loaded_rule_sidecar,
        )?;
        persist_part_assignment_sidecar(
            path,
            &serialized,
            design
                .board
                .as_ref()
                .map(|board| {
                    board
                        .packages
                        .values()
                        .filter(|package| package.part != uuid::Uuid::nil())
                        .map(|package| (package.uuid, package.part))
                        .collect()
                })
                .unwrap_or_default(),
            imported.loaded_part_assignment_sidecar,
        )?;
        persist_package_assignment_sidecar(
            path,
            &serialized,
            design
                .board
                .as_ref()
                .map(|board| {
                    board
                        .packages
                        .values()
                        .filter(|package| package.package != uuid::Uuid::nil())
                        .map(|package| (package.uuid, package.package))
                        .collect()
                })
                .unwrap_or_default(),
            imported.loaded_package_assignment_sidecar,
        )?;
        persist_net_class_sidecar(
            path,
            &serialized,
            design
                .board
                .as_ref()
                .map(net_class_sidecar_payload)
                .unwrap_or_default(),
            imported.loaded_net_class_sidecar,
        )?;
        Ok(())
    }

    pub fn save_to_original(&self) -> Result<std::path::PathBuf, EngineError> {
        let imported = self.imported_source.as_ref().ok_or_else(|| {
            EngineError::Operation("save requires imported design provenance".to_string())
        })?;
        self.save(&imported.source_path)?;
        Ok(imported.source_path.clone())
    }
}

fn serialize_current_kicad_board_slice(
    original_contents: &str,
    board: &Board,
    pool: &Pool,
    undo_stack: &[TransactionRecord],
) -> Result<String, EngineError> {
    let deleted_tracks = active_deleted_track_uuids(undo_stack);
    let deleted_vias = active_deleted_via_uuids(undo_stack);
    let deleted_components = active_deleted_component_uuids(undo_stack);
    let moved_components = active_moved_components(undo_stack);
    let valued_components = active_set_value_components(undo_stack);
    let referenced_components = active_set_reference_components(undo_stack);
    let assigned_components = active_assigned_part_components(undo_stack);
    let package_rewritten_components = active_package_rewritten_components(board);
    let forms = [
        ("segment", &deleted_tracks),
        ("via", &deleted_vias),
        ("footprint", &deleted_components),
    ];
    let without_removed = if deleted_tracks.is_empty()
        && deleted_vias.is_empty()
        && deleted_components.is_empty()
    {
        original_contents.to_string()
    } else {
        remove_kicad_top_level_forms(original_contents, &forms)?
    };
    let package_rewritten = rewrite_package_footprints(
        &without_removed,
        board,
        pool,
        &package_rewritten_components,
    )?;
    let moved = rewrite_moved_footprints(
        &package_rewritten,
        &filter_component_map(&moved_components, &package_rewritten_components),
    )?;
    let assigned_values = merge_component_value_overrides(&valued_components, &assigned_components);
    let valued = rewrite_value_footprints(
        &moved,
        &filter_component_map(&assigned_values, &package_rewritten_components),
    )?;
    rewrite_reference_footprints(
        &valued,
        &filter_component_map(&referenced_components, &package_rewritten_components),
    )
}

fn active_deleted_track_uuids(undo_stack: &[TransactionRecord]) -> BTreeSet<uuid::Uuid> {
    let mut deleted = BTreeSet::new();
    for transaction in undo_stack {
        if let TransactionRecord::DeleteTrack { track } = transaction {
            deleted.insert(track.uuid);
        }
    }
    deleted
}

fn active_deleted_via_uuids(undo_stack: &[TransactionRecord]) -> BTreeSet<uuid::Uuid> {
    let mut deleted = BTreeSet::new();
    for transaction in undo_stack {
        if let TransactionRecord::DeleteVia { via } = transaction {
            deleted.insert(via.uuid);
        }
    }
    deleted
}

fn active_deleted_component_uuids(undo_stack: &[TransactionRecord]) -> BTreeSet<uuid::Uuid> {
    let mut deleted = BTreeSet::new();
    for transaction in undo_stack {
        if let TransactionRecord::DeleteComponent { package, .. } = transaction {
            deleted.insert(package.uuid);
        }
    }
    deleted
}

fn active_moved_components(
    undo_stack: &[TransactionRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut moved = BTreeMap::new();
    for transaction in undo_stack {
        match transaction {
            TransactionRecord::MoveComponent { after, .. }
            | TransactionRecord::RotateComponent { after, .. } => {
                moved.insert(after.uuid, after.clone());
            }
            _ => {}
        }
    }
    moved
}

fn active_set_value_components(
    undo_stack: &[TransactionRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut valued = BTreeMap::new();
    for transaction in undo_stack {
        if let TransactionRecord::SetValue { after, .. } = transaction {
            valued.insert(after.uuid, after.clone());
        }
    }
    valued
}

fn active_set_reference_components(
    undo_stack: &[TransactionRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut referenced = BTreeMap::new();
    for transaction in undo_stack {
        if let TransactionRecord::SetReference { after, .. } = transaction {
            referenced.insert(after.uuid, after.clone());
        }
    }
    referenced
}

fn active_assigned_part_components(
    undo_stack: &[TransactionRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut assigned = BTreeMap::new();
    for transaction in undo_stack {
        if let TransactionRecord::AssignPart { after, .. } = transaction {
            assigned.insert(after.uuid, after.clone());
        }
    }
    assigned
}

fn merge_component_value_overrides(
    valued_components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
    assigned_components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut merged = valued_components.clone();
    for (uuid, package) in assigned_components {
        merged.insert(*uuid, package.clone());
    }
    merged
}

fn active_package_rewritten_components(board: &Board) -> BTreeSet<uuid::Uuid> {
    board.packages
        .values()
        .filter(|package| package.package != uuid::Uuid::nil())
        .map(|package| package.uuid)
        .collect()
}

fn filter_component_map(
    components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
    exclude: &BTreeSet<uuid::Uuid>,
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    components
        .iter()
        .filter(|(uuid, _)| !exclude.contains(uuid))
        .map(|(uuid, package)| (*uuid, package.clone()))
        .collect()
}

fn remove_kicad_top_level_forms(
    original_contents: &str,
    forms: &[(&str, &BTreeSet<uuid::Uuid>)],
) -> Result<String, EngineError> {
    let removal_ranges = find_kicad_top_level_form_ranges(original_contents, forms)?;
    if removal_ranges.is_empty() {
        return Err(EngineError::Operation(
            "save could not locate deleted KiCad board form in imported source".to_string(),
        ));
    }

    let mut output = String::with_capacity(original_contents.len());
    let mut cursor = 0;
    for (start, end) in removal_ranges {
        if start < cursor || end < start || end > original_contents.len() {
            return Err(EngineError::Operation(
                "save generated invalid KiCad segment removal range".to_string(),
            ));
        }
        output.push_str(&original_contents[cursor..start]);
        cursor = end;
    }
    output.push_str(&original_contents[cursor..]);
    Ok(output)
}

fn find_kicad_top_level_form_ranges(
    original_contents: &str,
    forms: &[(&str, &BTreeSet<uuid::Uuid>)],
) -> Result<Vec<(usize, usize)>, EngineError> {
    let mut ranges = Vec::new();
    let mut found: BTreeMap<&str, BTreeSet<uuid::Uuid>> = forms
        .iter()
        .map(|(name, _)| (*name, BTreeSet::new()))
        .collect();
    let mut depth = 0usize;
    let mut top_level_start = None;

    for (idx, ch) in original_contents.char_indices() {
        match ch {
            '(' => {
                depth += 1;
                if depth == 2 {
                    top_level_start = Some(idx);
                }
            }
            ')' => {
                if depth == 2 {
                    let start = top_level_start.ok_or_else(|| {
                        EngineError::Operation(
                            "save encountered malformed KiCad top-level form boundaries"
                                .to_string(),
                        )
                    })?;
                    let end = idx + ch.len_utf8();
                    let form = &original_contents[start..end];
                    for (form_name, uuids) in forms {
                        if uuids.is_empty() {
                            continue;
                        }
                        if let Some(uuid) = top_level_form_uuid(form, form_name)?
                            && uuids.contains(&uuid)
                        {
                            let line_start = original_contents[..start]
                                .rfind('\n')
                                .map(|pos| pos + 1)
                                .unwrap_or(0);
                            let line_end = original_contents[end..]
                                .find('\n')
                                .map(|offset| end + offset + 1)
                                .unwrap_or(original_contents.len());
                            ranges.push((line_start, line_end));
                            found.entry(form_name).or_default().insert(uuid);
                        }
                    }
                    top_level_start = None;
                }
                depth = depth.checked_sub(1).ok_or_else(|| {
                    EngineError::Operation(
                        "save encountered malformed KiCad board structure".to_string(),
                    )
                })?;
            }
            _ => {}
        }
    }

    for (form_name, expected) in forms {
        if found.get(form_name).unwrap_or(&BTreeSet::new()) != *expected {
            return Err(EngineError::Operation(format!(
                "save could not map deleted KiCad {form_name} UUIDs into imported source; expected {:?}, found {:?}",
                expected,
                found.get(form_name).unwrap_or(&BTreeSet::new())
            )));
        }
    }

    Ok(ranges)
}

fn top_level_form_uuid(form: &str, form_name: &str) -> Result<Option<uuid::Uuid>, EngineError> {
    let trimmed = form.trim_start();
    if !trimmed.starts_with(&format!("({form_name}")) {
        return Ok(None);
    }

    let uuid_marker = "(uuid ";
    let uuid_start = match form.find(uuid_marker) {
        Some(index) => index + uuid_marker.len(),
        None => {
            return Err(EngineError::Operation(format!(
                "save found KiCad {form_name} without UUID in imported source"
            )));
        }
    };
    let uuid_end = form[uuid_start..]
        .find(')')
        .map(|offset| uuid_start + offset)
        .ok_or_else(|| {
            EngineError::Operation(format!(
                "save found unterminated KiCad {form_name} UUID in imported source"
            ))
        })?;
    let uuid = uuid::Uuid::parse_str(form[uuid_start..uuid_end].trim()).map_err(|err| {
        EngineError::Operation(format!(
            "save found invalid KiCad {form_name} UUID in imported source: {err}"
        ))
    })?;
    Ok(Some(uuid))
}

fn rewrite_moved_footprints(
    contents: &str,
    moved_components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
) -> Result<String, EngineError> {
    let mut updated = contents.to_string();
    for (uuid, package) in moved_components {
        updated = rewrite_footprint_at_line(&updated, *uuid, package.position, package.rotation)?;
    }
    Ok(updated)
}

fn rewrite_value_footprints(
    contents: &str,
    valued_components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
) -> Result<String, EngineError> {
    let mut updated = contents.to_string();
    for (uuid, package) in valued_components {
        updated = rewrite_footprint_property_line(&updated, *uuid, "Value", &package.value)?;
    }
    Ok(updated)
}

fn rewrite_reference_footprints(
    contents: &str,
    referenced_components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
) -> Result<String, EngineError> {
    let mut updated = contents.to_string();
    for (uuid, package) in referenced_components {
        updated =
            rewrite_footprint_property_line(&updated, *uuid, "Reference", &package.reference)?;
    }
    Ok(updated)
}

fn rewrite_package_footprints(
    contents: &str,
    board: &Board,
    pool: &Pool,
    component_uuids: &BTreeSet<uuid::Uuid>,
) -> Result<String, EngineError> {
    if component_uuids.is_empty() {
        return Ok(contents.to_string());
    }
    let net_codes = kicad_net_code_map(contents)?;
    let mut updated = contents.to_string();
    for component_uuid in component_uuids {
        let component = board
            .packages
            .get(component_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: *component_uuid,
            })?;
        let package = pool
            .packages
            .get(&component.package)
            .ok_or(EngineError::NotFound {
                object_type: "package",
                uuid: component.package,
            })?;
        let replacement = render_kicad_footprint_block(component, package, board, &net_codes)?;
        updated = replace_kicad_top_level_form(&updated, "footprint", *component_uuid, &replacement)?;
    }
    Ok(updated)
}

fn rewrite_footprint_at_line(
    contents: &str,
    package_uuid: uuid::Uuid,
    position: crate::ir::geometry::Point,
    rotation: i32,
) -> Result<String, EngineError> {
    let lookup = BTreeSet::from([package_uuid]);
    let ranges = find_kicad_top_level_form_ranges(contents, &[("footprint", &lookup)])?;
    let (start, end) = *ranges.first().ok_or_else(|| {
        EngineError::Operation(format!(
            "save could not locate moved component {} in imported KiCad source",
            package_uuid
        ))
    })?;
    let block = &contents[start..end];
    let at_line = block
        .lines()
        .find(|line| line.trim_start().starts_with("(at "))
        .ok_or_else(|| {
            EngineError::Operation(format!(
                "save found moved footprint {} without (at ...) line",
                package_uuid
            ))
        })?;
    let indent_len = at_line.len() - at_line.trim_start().len();
    let replacement = format!(
        "{}(at {} {} {})",
        " ".repeat(indent_len),
        format_mm(position.x),
        format_mm(position.y),
        rotation
    );
    let replaced_block = block.replacen(at_line, &replacement, 1);
    Ok(format!(
        "{}{}{}",
        &contents[..start],
        replaced_block,
        &contents[end..]
    ))
}

fn rewrite_footprint_property_line(
    contents: &str,
    package_uuid: uuid::Uuid,
    property_name: &str,
    property_value: &str,
) -> Result<String, EngineError> {
    let lookup = BTreeSet::from([package_uuid]);
    let ranges = find_kicad_top_level_form_ranges(contents, &[("footprint", &lookup)])?;
    let (start, end) = *ranges.first().ok_or_else(|| {
        EngineError::Operation(format!(
            "save could not locate component {} in imported KiCad source",
            package_uuid
        ))
    })?;
    let block = &contents[start..end];
    let marker = format!("(property \"{property_name}\" ");
    let property_line = block
        .lines()
        .find(|line| line.trim_start().starts_with(&marker))
        .ok_or_else(|| {
            EngineError::Operation(format!(
                "save found footprint {} without {property_name} property line",
                package_uuid
            ))
        })?;
    let indent_len = property_line.len() - property_line.trim_start().len();
    let remainder = property_line
        .trim_start()
        .strip_prefix(&marker)
        .ok_or_else(|| {
            EngineError::Operation(format!(
                "save could not parse {property_name} property line for footprint {}",
                package_uuid
            ))
        })?;
    let tail_start = remainder.find('"').ok_or_else(|| {
        EngineError::Operation(format!(
            "save could not locate property value terminator for footprint {}",
            package_uuid
        ))
    })? + 1;
    let replacement = format!(
        "{}(property \"{}\" \"{}\"{}",
        " ".repeat(indent_len),
        property_name,
        property_value,
        &remainder[tail_start..]
    );
    let replaced_block = block.replacen(property_line, &replacement, 1);
    Ok(format!(
        "{}{}{}",
        &contents[..start],
        replaced_block,
        &contents[end..]
    ))
}

fn replace_kicad_top_level_form(
    contents: &str,
    form_name: &str,
    form_uuid: uuid::Uuid,
    replacement: &str,
) -> Result<String, EngineError> {
    let lookup = BTreeSet::from([form_uuid]);
    let ranges = find_kicad_top_level_form_ranges(contents, &[(form_name, &lookup)])?;
    let (start, end) = *ranges.first().ok_or_else(|| {
        EngineError::Operation(format!(
            "save could not locate {form_name} {} in imported KiCad source",
            form_uuid
        ))
    })?;
    Ok(format!(
        "{}{}{}",
        &contents[..start],
        replacement,
        &contents[end..]
    ))
}

fn render_kicad_footprint_block(
    component: &crate::board::PlacedPackage,
    package: &crate::pool::Package,
    board: &Board,
    net_codes: &BTreeMap<uuid::Uuid, i32>,
) -> Result<String, EngineError> {
    let mut lines = Vec::new();
    lines.push(format!("  (footprint \"{}\"", package.name));
    lines.push(format!(
        "    (layer \"{}\")",
        kicad_layer_name_for_id(component.layer)
    ));
    lines.push(format!("    (uuid {})", component.uuid));
    lines.push(format!(
        "    (at {} {} {})",
        format_mm(component.position.x),
        format_mm(component.position.y),
        component.rotation
    ));
    lines.push(format!(
        "    (property \"Reference\" \"{}\" (at 0 -2 0) (layer \"F.SilkS\"))",
        component.reference
    ));
    lines.push(format!(
        "    (property \"Value\" \"{}\" (at 0 2 0) (layer \"F.Fab\"))",
        component.value
    ));
    let mut pads: Vec<_> = package.pads.values().collect();
    pads.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    for pad in pads {
        lines.push(format!("    (pad \"{}\" smd rect", pad.name));
        lines.push(format!(
            "      (at {} {})",
            format_mm(pad.position.x),
            format_mm(pad.position.y)
        ));
        lines.push("      (size 1 1)".to_string());
        lines.push(format!(
            "      (layers \"{}\" \"F.Paste\" \"F.Mask\")",
            kicad_layer_name_for_id(pad.layer)
        ));
        let pad_state = board
            .pads
            .values()
            .find(|candidate| candidate.package == component.uuid && candidate.name == pad.name);
        if let Some(net_uuid) = pad_state.and_then(|pad| pad.net)
            && let Some(net_code) = net_codes.get(&net_uuid)
        {
            let net_name = board
                .nets
                .get(&net_uuid)
                .map(|net| net.name.as_str())
                .unwrap_or("");
            lines.push(format!("      (net {} \"{}\")", net_code, net_name));
        }
        lines.push("    )".to_string());
    }
    lines.push("  )".to_string());
    Ok(format!("{}\n", lines.join("\n")))
}

fn kicad_net_code_map(contents: &str) -> Result<BTreeMap<uuid::Uuid, i32>, EngineError> {
    let mut net_codes = BTreeMap::new();
    for block in contents
        .lines()
        .collect::<Vec<_>>()
        .join("\n")
        .split("\n  (net ")
    {
        let candidate = if block.starts_with("(kicad_pcb") {
            continue;
        } else {
            format!("  (net {block}")
        };
        if let Some((code, _name)) = parse_simple_net_block(&candidate)
            && code >= 0
        {
            let uuid = deterministic_kicad_net_uuid(code);
            net_codes.insert(uuid, code);
        }
    }
    Ok(net_codes)
}

fn parse_simple_net_block(block: &str) -> Option<(i32, String)> {
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

fn deterministic_kicad_net_uuid(code: i32) -> uuid::Uuid {
    crate::ir::ids::import_uuid(
        &crate::ir::ids::namespace_kicad(),
        &format!("board/net/{code}"),
    )
}

fn kicad_layer_name_for_id(layer: i32) -> &'static str {
    match layer {
        31 => "B.Cu",
        36 => "B.SilkS",
        37 => "F.SilkS",
        44 => "Edge.Cuts",
        _ => "F.Cu",
    }
}

fn format_mm(nm: i64) -> String {
    let value = nm_to_mm(nm);
    if (value.fract()).abs() < f64::EPSILON {
        format!("{}", value.round() as i64)
    } else {
        let mut text = format!("{value:.6}");
        while text.contains('.') && text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
        text
    }
}
