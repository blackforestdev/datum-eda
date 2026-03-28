use super::*;

fn resolve_native_project_paste_context(
    root: &Path,
    layer: i32,
) -> Result<(StackupLayer, i32, Vec<PlacedPad>)> {
    let stackup = query_native_project_board_stackup(root)?;
    let paste_layer = stackup
        .iter()
        .find(|entry| entry.id == layer)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("board stackup layer not found in native project: {layer}")
        })?;
    if !matches!(paste_layer.layer_type, StackupLayerType::Paste) {
        bail!("board stackup layer is not a paste layer: {layer}");
    }

    let associated_copper_layer = stackup
        .iter()
        .filter(|entry| matches!(entry.layer_type, StackupLayerType::Copper))
        .min_by(|a, b| {
            (i64::from((a.id - layer).abs()), a.id).cmp(&(i64::from((b.id - layer).abs()), b.id))
        })
        .map(|entry| entry.id)
        .ok_or_else(|| anyhow::anyhow!("no copper layer available to derive paste openings"))?;

    let pads = query_native_project_emitted_copper_pads(root)?
        .into_iter()
        .filter(|pad| pad.layer == associated_copper_layer)
        .collect::<Vec<_>>();

    Ok((paste_layer, associated_copper_layer, pads))
}

pub(crate) fn export_native_project_gerber_outline(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectGerberOutlineExportView> {
    let project = load_native_project(root)?;
    let outline = native_outline_to_polygon(&project.board.outline);
    let gerber = render_rs274x_outline_default(&outline)
        .context("failed to render native board outline as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberOutlineExportView {
        action: "export_gerber_outline".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: output_path.display().to_string(),
        outline_vertex_count: project.board.outline.vertices.len(),
        outline_closed: project.board.outline.closed,
    })
}

pub(crate) fn export_native_project_gerber_copper_layer(
    root: &Path,
    layer: i32,
    output_path: &Path,
) -> Result<NativeProjectGerberCopperExportView> {
    let project = load_native_project(root)?;
    let pads = query_native_project_emitted_copper_pads(root)?
        .into_iter()
        .filter(|pad| pad.layer == layer)
        .collect::<Vec<_>>();
    let tracks = query_native_project_board_tracks(root)?
        .into_iter()
        .filter(|track| track.layer == layer)
        .collect::<Vec<_>>();
    let zones = query_native_project_board_zones(root)?
        .into_iter()
        .filter(|zone| zone.layer == layer)
        .collect::<Vec<_>>();
    let vias = query_native_project_board_vias(root)?
        .into_iter()
        .filter(|via| {
            let min_layer = via.from_layer.min(via.to_layer);
            let max_layer = via.from_layer.max(via.to_layer);
            layer >= min_layer && layer <= max_layer
        })
        .collect::<Vec<_>>();
    let gerber = render_rs274x_copper_layer(layer, &pads, &tracks, &zones, &vias)
        .context("failed to render native board copper layer as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberCopperExportView {
        action: "export_gerber_copper_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: output_path.display().to_string(),
        layer,
        pad_count: pads.len(),
        track_count: tracks.len(),
        zone_count: zones.len(),
        via_count: vias.len(),
    })
}

pub(crate) fn export_native_project_gerber_soldermask_layer(
    root: &Path,
    layer: i32,
    output_path: &Path,
) -> Result<NativeProjectGerberSoldermaskExportView> {
    let project = load_native_project(root)?;
    let (_mask_layer, source_copper_layer, pads) =
        resolve_native_project_soldermask_context(root, layer)?;
    let gerber = render_rs274x_soldermask_layer(layer, &pads)
        .context("failed to render native board soldermask layer as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberSoldermaskExportView {
        action: "export_gerber_soldermask_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: output_path.display().to_string(),
        layer,
        source_copper_layer,
        pad_count: pads.len(),
    })
}

pub(crate) fn export_native_project_gerber_silkscreen_layer(
    root: &Path,
    layer: i32,
    output_path: &Path,
) -> Result<NativeProjectGerberSilkscreenExportView> {
    let project = load_native_project(root)?;
    let component_text_count = count_native_component_silkscreen_texts(&project.board, layer);
    let component_line_count = count_native_component_silkscreen_lines(&project.board, layer);
    let component_arc_count = count_native_component_silkscreen_arcs(&project.board, layer);
    let component_circle_count = count_native_component_silkscreen_circles(&project.board, layer);
    let component_polygon_count = count_native_component_silkscreen_polygons(&project.board, layer);
    let component_polyline_count =
        count_native_component_silkscreen_polylines(&project.board, layer);
    let (_silk_layer, texts, component_strokes) =
        resolve_native_project_silkscreen_context(root, layer)?;
    let gerber = render_rs274x_silkscreen_layer(layer, &texts, &component_strokes)
        .context("failed to render native board silkscreen layer as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberSilkscreenExportView {
        action: "export_gerber_silkscreen_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: output_path.display().to_string(),
        layer,
        text_count: texts.len().saturating_sub(component_text_count),
        component_text_count,
        component_stroke_count: component_line_count,
        component_arc_count,
        component_circle_count,
        component_polygon_count,
        component_polyline_count,
    })
}

pub(crate) fn export_native_project_gerber_paste_layer(
    root: &Path,
    layer: i32,
    output_path: &Path,
) -> Result<NativeProjectGerberPasteExportView> {
    let project = load_native_project(root)?;
    let (_paste_layer, source_copper_layer, pads) =
        resolve_native_project_paste_context(root, layer)?;
    let gerber = render_rs274x_paste_layer(layer, &pads)
        .context("failed to render native board paste layer as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberPasteExportView {
        action: "export_gerber_paste_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: output_path.display().to_string(),
        layer,
        source_copper_layer,
        pad_count: pads.len(),
    })
}

pub(crate) fn validate_native_project_gerber_outline(
    root: &Path,
    gerber_path: &Path,
) -> Result<NativeProjectGerberOutlineValidationView> {
    let project = load_native_project(root)?;
    let outline = native_outline_to_polygon(&project.board.outline);
    let expected = render_rs274x_outline_default(&outline)
        .context("failed to render expected native board outline as RS-274X")?;
    let actual = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;

    Ok(NativeProjectGerberOutlineValidationView {
        action: "validate_gerber_outline".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        outline_vertex_count: project.board.outline.vertices.len(),
        outline_closed: project.board.outline.closed,
    })
}

pub(crate) fn validate_native_project_gerber_copper_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberCopperValidationView> {
    let project = load_native_project(root)?;
    let pads = query_native_project_emitted_copper_pads(root)?
        .into_iter()
        .filter(|pad| pad.layer == layer)
        .collect::<Vec<_>>();
    let tracks = query_native_project_board_tracks(root)?
        .into_iter()
        .filter(|track| track.layer == layer)
        .collect::<Vec<_>>();
    let zones = query_native_project_board_zones(root)?
        .into_iter()
        .filter(|zone| zone.layer == layer)
        .collect::<Vec<_>>();
    let vias = query_native_project_board_vias(root)?
        .into_iter()
        .filter(|via| {
            let min_layer = via.from_layer.min(via.to_layer);
            let max_layer = via.from_layer.max(via.to_layer);
            layer >= min_layer && layer <= max_layer
        })
        .collect::<Vec<_>>();
    let expected = render_rs274x_copper_layer(layer, &pads, &tracks, &zones, &vias)
        .context("failed to render expected native board copper layer as RS-274X")?;
    let actual = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;

    Ok(NativeProjectGerberCopperValidationView {
        action: "validate_gerber_copper_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        pad_count: pads.len(),
        track_count: tracks.len(),
        zone_count: zones.len(),
        via_count: vias.len(),
    })
}

pub(crate) fn validate_native_project_gerber_soldermask_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberSoldermaskValidationView> {
    let project = load_native_project(root)?;
    let (_mask_layer, source_copper_layer, pads) =
        resolve_native_project_soldermask_context(root, layer)?;
    let expected = render_rs274x_soldermask_layer(layer, &pads)
        .context("failed to render expected native board soldermask layer as RS-274X")?;
    let actual = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;

    Ok(NativeProjectGerberSoldermaskValidationView {
        action: "validate_gerber_soldermask_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        source_copper_layer,
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        pad_count: pads.len(),
    })
}

pub(crate) fn validate_native_project_gerber_silkscreen_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberSilkscreenValidationView> {
    let project = load_native_project(root)?;
    let component_text_count = count_native_component_silkscreen_texts(&project.board, layer);
    let component_line_count = count_native_component_silkscreen_lines(&project.board, layer);
    let component_arc_count = count_native_component_silkscreen_arcs(&project.board, layer);
    let component_circle_count = count_native_component_silkscreen_circles(&project.board, layer);
    let component_polygon_count = count_native_component_silkscreen_polygons(&project.board, layer);
    let component_polyline_count =
        count_native_component_silkscreen_polylines(&project.board, layer);
    let (_silk_layer, texts, component_strokes) =
        resolve_native_project_silkscreen_context(root, layer)?;
    let expected = render_rs274x_silkscreen_layer(layer, &texts, &component_strokes)
        .context("failed to render expected native board silkscreen layer as RS-274X")?;
    let actual = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;

    Ok(NativeProjectGerberSilkscreenValidationView {
        action: "validate_gerber_silkscreen_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        text_count: texts.len().saturating_sub(component_text_count),
        component_text_count,
        component_stroke_count: component_line_count,
        component_arc_count,
        component_circle_count,
        component_polygon_count,
        component_polyline_count,
    })
}

pub(crate) fn validate_native_project_gerber_paste_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberPasteValidationView> {
    let project = load_native_project(root)?;
    let (_paste_layer, source_copper_layer, pads) =
        resolve_native_project_paste_context(root, layer)?;
    let expected = render_rs274x_paste_layer(layer, &pads)
        .context("failed to render expected native board paste layer as RS-274X")?;
    let actual = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;

    Ok(NativeProjectGerberPasteValidationView {
        action: "validate_gerber_paste_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        source_copper_layer,
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        pad_count: pads.len(),
    })
}

pub(crate) fn compare_native_project_gerber_outline(
    root: &Path,
    gerber_path: &Path,
) -> Result<NativeProjectGerberOutlineComparisonView> {
    let project = load_native_project(root)?;
    let outline = native_outline_to_polygon(&project.board.outline);
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber outline for semantic comparison")?;

    let expected_entries = gerber_outline_expected_entries(&outline);
    let actual_entries = gerber_outline_actual_entries(&parsed);
    let (matched_count, missing_count, extra_count, matched, missing, extra) =
        compare_entry_views(expected_entries, actual_entries);

    Ok(NativeProjectGerberOutlineComparisonView {
        action: "compare_gerber_outline".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        expected_outline_count: 1,
        actual_geometry_count: parsed.geometries.len(),
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    })
}

pub(crate) fn compare_native_project_gerber_copper_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberCopperComparisonView> {
    let project = load_native_project(root)?;
    let pads = query_native_project_emitted_copper_pads(root)?
        .into_iter()
        .filter(|pad| pad.layer == layer)
        .collect::<Vec<_>>();
    let tracks = query_native_project_board_tracks(root)?
        .into_iter()
        .filter(|track| track.layer == layer)
        .collect::<Vec<_>>();
    let zones = query_native_project_board_zones(root)?
        .into_iter()
        .filter(|zone| zone.layer == layer)
        .collect::<Vec<_>>();
    let vias = query_native_project_board_vias(root)?
        .into_iter()
        .filter(|via| {
            let min_layer = via.from_layer.min(via.to_layer);
            let max_layer = via.from_layer.max(via.to_layer);
            layer >= min_layer && layer <= max_layer
        })
        .collect::<Vec<_>>();
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber copper layer for semantic comparison")?;

    let expected_pad_signatures = pads
        .iter()
        .map(render_pad_flash_geometry)
        .collect::<BTreeSet<_>>();
    let expected_via_signatures = vias
        .iter()
        .map(|via| render_circular_flash_geometry(via.diameter, &via.position))
        .collect::<BTreeSet<_>>();
    let expected_entries = gerber_copper_expected_entries(&pads, &tracks, &zones, &vias);
    let actual_entries =
        gerber_copper_actual_entries(&parsed, &expected_pad_signatures, &expected_via_signatures);
    let (matched_count, missing_count, extra_count, matched, missing, extra) =
        compare_entry_views(expected_entries, actual_entries);

    let actual_pad_count = parsed
        .geometries
        .iter()
        .filter(|geometry| match geometry {
            ParsedGerberGeometry::Flash { aperture, position } => {
                expected_pad_signatures.contains(&render_parsed_flash_geometry(aperture, position))
            }
            _ => false,
        })
        .count();
    let actual_track_count = parsed
        .geometries
        .iter()
        .filter(|geometry| matches!(geometry, ParsedGerberGeometry::Stroke { .. }))
        .count();
    let actual_zone_count = parsed
        .geometries
        .iter()
        .filter(|geometry| matches!(geometry, ParsedGerberGeometry::Region { .. }))
        .count();
    let actual_via_count = parsed
        .geometries
        .iter()
        .filter(|geometry| match geometry {
            ParsedGerberGeometry::Flash { aperture, position } => {
                expected_via_signatures.contains(&render_parsed_flash_geometry(aperture, position))
            }
            _ => false,
        })
        .count();

    Ok(NativeProjectGerberCopperComparisonView {
        action: "compare_gerber_copper_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        expected_pad_count: pads.len(),
        actual_pad_count,
        expected_track_count: tracks.len(),
        actual_track_count,
        expected_zone_count: zones.len(),
        actual_zone_count,
        expected_via_count: vias.len(),
        actual_via_count,
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    })
}

pub(crate) fn compare_native_project_gerber_soldermask_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberSoldermaskComparisonView> {
    let project = load_native_project(root)?;
    let (_mask_layer, source_copper_layer, pads) =
        resolve_native_project_soldermask_context(root, layer)?;
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber soldermask layer for semantic comparison")?;

    let expected_pad_signatures = pads
        .iter()
        .map(render_pad_flash_geometry)
        .collect::<BTreeSet<_>>();
    let expected_entries = gerber_soldermask_expected_entries(&pads);
    let actual_entries = gerber_soldermask_actual_entries(&parsed, &expected_pad_signatures);
    let (matched_count, missing_count, extra_count, matched, missing, extra) =
        compare_entry_views(expected_entries, actual_entries);

    let actual_pad_count = parsed
        .geometries
        .iter()
        .filter(|geometry| match geometry {
            ParsedGerberGeometry::Flash { aperture, position } => {
                expected_pad_signatures.contains(&render_parsed_flash_geometry(aperture, position))
            }
            _ => false,
        })
        .count();

    Ok(NativeProjectGerberSoldermaskComparisonView {
        action: "compare_gerber_soldermask_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        source_copper_layer,
        expected_pad_count: pads.len(),
        actual_pad_count,
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    })
}

pub(crate) fn compare_native_project_gerber_silkscreen_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberSilkscreenComparisonView> {
    let project = load_native_project(root)?;
    let component_text_count = count_native_component_silkscreen_texts(&project.board, layer);
    let component_line_count = count_native_component_silkscreen_lines(&project.board, layer);
    let component_arc_count = count_native_component_silkscreen_arcs(&project.board, layer);
    let component_circle_count = count_native_component_silkscreen_circles(&project.board, layer);
    let component_polygon_count = count_native_component_silkscreen_polygons(&project.board, layer);
    let component_polyline_count =
        count_native_component_silkscreen_polylines(&project.board, layer);
    let (_silk_layer, texts, component_strokes) =
        resolve_native_project_silkscreen_context(root, layer)?;
    let expected_gerber = render_rs274x_silkscreen_layer(layer, &texts, &component_strokes)
        .context("failed to render expected native board silkscreen layer as RS-274X")?;
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let expected_parsed = parse_rs274x_subset(&expected_gerber)
        .context("failed to parse expected Gerber silkscreen layer for semantic comparison")?;
    let actual_parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber silkscreen layer for semantic comparison")?;

    let expected_entries = gerber_silkscreen_expected_entries(&expected_parsed);
    let actual_entries = gerber_silkscreen_expected_entries(&actual_parsed);
    let (matched_count, missing_count, extra_count, matched, missing, extra) =
        compare_entry_views(expected_entries, actual_entries);

    Ok(NativeProjectGerberSilkscreenComparisonView {
        action: "compare_gerber_silkscreen_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        expected_text_count: texts.len().saturating_sub(component_text_count),
        expected_component_text_count: component_text_count,
        expected_component_stroke_count: component_line_count,
        expected_component_arc_count: component_arc_count,
        expected_component_circle_count: component_circle_count,
        expected_component_polygon_count: component_polygon_count,
        expected_component_polyline_count: component_polyline_count,
        actual_geometry_count: actual_parsed.geometries.len(),
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    })
}

pub(crate) fn compare_native_project_gerber_paste_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberPasteComparisonView> {
    let project = load_native_project(root)?;
    let (_paste_layer, source_copper_layer, pads) =
        resolve_native_project_paste_context(root, layer)?;
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber paste layer for semantic comparison")?;

    let expected_pad_signatures = pads
        .iter()
        .map(render_pad_flash_geometry)
        .collect::<BTreeSet<_>>();
    let expected_entries = gerber_soldermask_expected_entries(&pads);
    let actual_entries = gerber_soldermask_actual_entries(&parsed, &expected_pad_signatures);
    let (matched_count, missing_count, extra_count, matched, missing, extra) =
        compare_entry_views(expected_entries, actual_entries);

    let actual_pad_count = parsed
        .geometries
        .iter()
        .filter(|geometry| match geometry {
            ParsedGerberGeometry::Flash { aperture, position } => {
                expected_pad_signatures.contains(&render_parsed_flash_geometry(aperture, position))
            }
            _ => false,
        })
        .count();

    Ok(NativeProjectGerberPasteComparisonView {
        action: "compare_gerber_paste_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        source_copper_layer,
        expected_pad_count: pads.len(),
        actual_pad_count,
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    })
}
