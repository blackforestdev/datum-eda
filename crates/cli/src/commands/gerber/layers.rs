use crate::*;
use eda_engine::substrate::DesignModel;
use std::collections::BTreeSet;
#[path = "copper_projection.rs"]
mod command_project_gerber_copper_projection;
#[path = "../../command_project_zone_fill_projection.rs"]
mod command_project_zone_fill_projection;
use command_project_gerber_copper_projection::render_native_project_gerber_copper_projection;
use command_project_zone_fill_projection::zone_fill_copper_projection_zones;
struct NativeCopperLayerContext {
    pads: Vec<PlacedPad>,
    tracks: Vec<Track>,
    zones: Vec<Zone>,
    unfilled_zone_count: usize,
    unfilled_zone_ids: Vec<String>,
    vias: Vec<Via>,
}
fn resolve_native_project_copper_layer_context(
    project: &LoadedNativeProject,
    model: Option<&DesignModel>,
    layer: i32,
) -> Result<NativeCopperLayerContext> {
    let mut pads = project
        .board
        .pads
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board pad"))
        .collect::<Result<Vec<PlacedPad>>>()?;
    for (component_key, component_pads) in &project.board.component_pads {
        let component_uuid = Uuid::parse_str(component_key).with_context(|| {
            format!(
                "failed to parse component UUID in {}",
                project.board_path.display()
            )
        })?;
        pads.extend(component_pads.iter().filter_map(|pad| {
            pad.shape.map(|shape| PlacedPad {
                uuid: pad.uuid,
                package: component_uuid,
                name: pad.name.clone(),
                net: None,
                position: Point {
                    x: pad.position.x,
                    y: pad.position.y,
                },
                layer: pad.layer,
                copper_layers: vec![pad.layer],
                shape,
                diameter: pad.diameter_nm,
                width: pad.width_nm,
                height: pad.height_nm,
                drill: pad.drill_nm.unwrap_or(0),
                rotation: 0,
                mask_layers: Vec::new(),
                paste_layers: Vec::new(),
                solder_mask_margin_nm: 0,
                solder_paste_margin_nm: 0,
                solder_paste_margin_ratio_ppm: 0,
                roundrect_rratio_ppm: 250_000,
            })
        }));
    }
    pads.retain(|pad| pad.layer == layer);
    pads.sort_by(|a, b| {
        a.layer
            .cmp(&b.layer)
            .then_with(|| a.package.cmp(&b.package))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    let mut tracks = project
        .board
        .tracks
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board track"))
        .collect::<Result<Vec<Track>>>()?;
    tracks.retain(|track| track.layer == layer);
    tracks.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    let mut authored_zones = project
        .board
        .zones
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board zone"))
        .collect::<Result<Vec<Zone>>>()?;
    authored_zones.retain(|zone| zone.layer == layer);
    authored_zones.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    let (zones, unfilled_zone_ids) = match model {
        Some(model) => zone_fill_copper_projection_zones(&authored_zones, &model.zone_fills),
        None => (
            Vec::new(),
            authored_zones
                .iter()
                .map(|zone| zone.uuid.to_string())
                .collect::<Vec<_>>(),
        ),
    };
    let unfilled_zone_count = unfilled_zone_ids.len();
    let mut vias = project
        .board
        .vias
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board via"))
        .collect::<Result<Vec<Via>>>()?;
    vias.retain(|via| {
        layer >= via.from_layer.min(via.to_layer) && layer <= via.from_layer.max(via.to_layer)
    });
    vias.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(NativeCopperLayerContext {
        pads,
        tracks,
        zones,
        unfilled_zone_count,
        unfilled_zone_ids,
        vias,
    })
}

fn resolve_native_project_paste_context(
    project: &LoadedNativeProject,
    layer: i32,
) -> Result<(StackupLayer, i32, Vec<PlacedPad>)> {
    let stackup = project
        .board
        .stackup
        .layers
        .iter()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board stackup layer"))
        .collect::<Result<Vec<StackupLayer>>>()?;
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

    let pads =
        resolve_native_project_copper_layer_context(project, None, associated_copper_layer)?.pads;
    Ok((paste_layer, associated_copper_layer, pads))
}
pub(crate) fn export_native_project_gerber_outline(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectGerberOutlineExportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let outline = native_outline_to_polygon(&project.board.outline);
    let gerber = render_rs274x_outline_default(&outline)
        .context("failed to render native board outline as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberOutlineExportView {
        action: "export_gerber_outline".to_string(),
        production_classification: "manual_debug_export".to_string(),
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
    let projection = render_native_project_gerber_copper_projection(root, layer)?;
    std::fs::write(output_path, &projection.gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberCopperExportView {
        action: "export_gerber_copper_layer".to_string(),
        production_classification: "manual_debug_export".to_string(),
        project_root: projection.project_root,
        board_path: projection.board_path,
        gerber_path: output_path.display().to_string(),
        layer: projection.layer,
        pad_count: projection.context.pads.len(),
        track_count: projection.context.tracks.len(),
        zone_count: projection.context.zones.len(),
        unfilled_zone_count: projection.context.unfilled_zone_count,
        unfilled_zone_ids: projection.context.unfilled_zone_ids,
        via_count: projection.context.vias.len(),
        production_projection: projection.production_projection,
    })
}
pub(crate) fn export_native_project_gerber_soldermask_layer(
    root: &Path,
    layer: i32,
    output_path: &Path,
) -> Result<NativeProjectGerberSoldermaskExportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (_mask_layer, source_copper_layer) =
        resolve_native_project_soldermask_context(&project, layer)?;
    let pads =
        resolve_native_project_copper_layer_context(&project, None, source_copper_layer)?.pads;
    let gerber = render_rs274x_soldermask_layer(layer, &pads)
        .context("failed to render native board soldermask layer as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberSoldermaskExportView {
        action: "export_gerber_soldermask_layer".to_string(),
        production_classification: "manual_debug_export".to_string(),
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
    let project = load_native_project_with_resolved_board(root)?;
    let component_text_count = count_native_component_silkscreen_texts(&project.board, layer);
    let component_line_count = count_native_component_silkscreen_lines(&project.board, layer);
    let component_arc_count = count_native_component_silkscreen_arcs(&project.board, layer);
    let component_circle_count = count_native_component_silkscreen_circles(&project.board, layer);
    let component_polygon_count = count_native_component_silkscreen_polygons(&project.board, layer);
    let component_polyline_count =
        count_native_component_silkscreen_polylines(&project.board, layer);
    let (_silk_layer, texts, component_strokes) =
        resolve_native_project_silkscreen_context(&project, layer)?;
    let gerber = render_rs274x_silkscreen_layer(layer, &texts, &component_strokes)
        .context("failed to render native board silkscreen layer as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberSilkscreenExportView {
        action: "export_gerber_silkscreen_layer".to_string(),
        production_classification: "manual_debug_export".to_string(),
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
    let project = load_native_project_with_resolved_board(root)?;
    let (_paste_layer, source_copper_layer, pads) =
        resolve_native_project_paste_context(&project, layer)?;
    let gerber = render_rs274x_paste_layer(layer, &pads)
        .context("failed to render native board paste layer as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberPasteExportView {
        action: "export_gerber_paste_layer".to_string(),
        production_classification: "manual_debug_export".to_string(),
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
    let project = load_native_project_with_resolved_board(root)?;
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
    let projection = render_native_project_gerber_copper_projection(root, layer)?;
    let actual = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;

    Ok(NativeProjectGerberCopperValidationView {
        action: "validate_gerber_copper_layer".to_string(),
        project_root: projection.project_root,
        board_path: projection.board_path,
        gerber_path: gerber_path.display().to_string(),
        layer: projection.layer,
        matches_expected: actual == projection.gerber,
        expected_bytes: projection.gerber.len(),
        actual_bytes: actual.len(),
        pad_count: projection.context.pads.len(),
        track_count: projection.context.tracks.len(),
        zone_count: projection.context.zones.len(),
        unfilled_zone_count: projection.context.unfilled_zone_count,
        unfilled_zone_ids: projection.context.unfilled_zone_ids,
        via_count: projection.context.vias.len(),
        production_projection: projection.production_projection,
    })
}

pub(crate) fn validate_native_project_gerber_soldermask_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberSoldermaskValidationView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (_mask_layer, source_copper_layer) =
        resolve_native_project_soldermask_context(&project, layer)?;
    let pads =
        resolve_native_project_copper_layer_context(&project, None, source_copper_layer)?.pads;
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
    let project = load_native_project_with_resolved_board(root)?;
    let component_text_count = count_native_component_silkscreen_texts(&project.board, layer);
    let component_line_count = count_native_component_silkscreen_lines(&project.board, layer);
    let component_arc_count = count_native_component_silkscreen_arcs(&project.board, layer);
    let component_circle_count = count_native_component_silkscreen_circles(&project.board, layer);
    let component_polygon_count = count_native_component_silkscreen_polygons(&project.board, layer);
    let component_polyline_count =
        count_native_component_silkscreen_polylines(&project.board, layer);
    let (_silk_layer, texts, component_strokes) =
        resolve_native_project_silkscreen_context(&project, layer)?;
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
    let project = load_native_project_with_resolved_board(root)?;
    let (_paste_layer, source_copper_layer, pads) =
        resolve_native_project_paste_context(&project, layer)?;
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
    let project = load_native_project_with_resolved_board(root)?;
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
    let (project, model) = load_native_project_with_resolved_board_and_model(root)?;
    let context = resolve_native_project_copper_layer_context(&project, Some(&model), layer)?;
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber copper layer for semantic comparison")?;

    let expected_pad_signatures = context
        .pads
        .iter()
        .map(render_pad_flash_geometry)
        .collect::<BTreeSet<_>>();
    let expected_via_signatures = context
        .vias
        .iter()
        .map(|via| render_circular_flash_geometry(via.diameter, &via.position))
        .collect::<BTreeSet<_>>();
    let expected_entries = gerber_copper_expected_entries(
        &context.pads,
        &context.tracks,
        &context.zones,
        &context.vias,
    );
    let actual_entries =
        gerber_copper_actual_entries(&parsed, &expected_pad_signatures, &expected_via_signatures);
    let actual_pad_count = actual_entries
        .iter()
        .filter(|((kind, _), _)| kind == "pad")
        .map(|(_, count)| *count)
        .sum();
    let (matched_count, missing_count, extra_count, matched, missing, extra) =
        compare_entry_views(expected_entries, actual_entries);
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
        expected_pad_count: context.pads.len(),
        actual_pad_count,
        expected_track_count: context.tracks.len(),
        actual_track_count,
        expected_zone_count: context.zones.len(),
        actual_zone_count,
        unfilled_zone_count: context.unfilled_zone_count,
        unfilled_zone_ids: context.unfilled_zone_ids,
        expected_via_count: context.vias.len(),
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
    let project = load_native_project_with_resolved_board(root)?;
    let (_mask_layer, source_copper_layer) =
        resolve_native_project_soldermask_context(&project, layer)?;
    let pads =
        resolve_native_project_copper_layer_context(&project, None, source_copper_layer)?.pads;
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
    let project = load_native_project_with_resolved_board(root)?;
    let component_text_count = count_native_component_silkscreen_texts(&project.board, layer);
    let component_line_count = count_native_component_silkscreen_lines(&project.board, layer);
    let component_arc_count = count_native_component_silkscreen_arcs(&project.board, layer);
    let component_circle_count = count_native_component_silkscreen_circles(&project.board, layer);
    let component_polygon_count = count_native_component_silkscreen_polygons(&project.board, layer);
    let component_polyline_count =
        count_native_component_silkscreen_polylines(&project.board, layer);
    let (_silk_layer, texts, component_strokes) =
        resolve_native_project_silkscreen_context(&project, layer)?;
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
    let project = load_native_project_with_resolved_board(root)?;
    let (_paste_layer, source_copper_layer, pads) =
        resolve_native_project_paste_context(&project, layer)?;
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
