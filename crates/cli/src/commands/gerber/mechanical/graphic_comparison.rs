//! Preserve historical classification while including exact independent graphics.
use super::*;
pub(crate) fn compare_native_project_gerber_mechanical_layer(
    root: &std::path::Path,
    layer: i32,
    gerber_path: &std::path::Path,
) -> Result<NativeProjectGerberMechanicalComparisonView> {
    let project = load_native_project_with_resolved_board(root)?;
    let dimension_count = count_native_board_dimensions(&project.board, layer);
    let board_text_count = count_native_board_texts(&project.board, layer);
    let component_text_count = project
        .board
        .component_mechanical_texts
        .values()
        .map(|entries| entries.iter().filter(|entry| entry.layer == layer).count())
        .sum::<usize>();
    let component_polygon_count = count_native_component_mechanical_polygons(&project.board, layer);
    let component_stroke_count = count_native_component_mechanical_lines(&project.board, layer);
    let component_polyline_count =
        count_native_component_mechanical_polylines(&project.board, layer);
    let component_circle_count = count_native_component_mechanical_circles(&project.board, layer);
    let component_arc_count = count_native_component_mechanical_arcs(&project.board, layer);
    let (
        _mechanical_layer,
        polygons,
        component_polygons,
        component_strokes,
        component_polyline_strokes,
        component_circle_strokes,
        component_arc_strokes,
        dimension_strokes,
        board_text_strokes,
        component_text_strokes,
    ) = resolve_native_project_mechanical_context(&project, layer)?;
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber mechanical layer for semantic comparison")?;

    let expected_component_polygon_signatures = component_polygons
        .iter()
        .map(render_polygon_region_geometry)
        .collect::<std::collections::BTreeSet<_>>();
    let expected_component_signatures = component_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_component_polyline_signatures = component_polyline_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_component_circle_signatures = component_circle_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_component_arc_signatures = component_arc_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_dimension_signatures = dimension_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_board_text_signatures = board_text_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_component_text_signatures = component_text_strokes
        .iter()
        .map(|stroke| render_stroke_geometry(stroke.width_nm, &[stroke.from, stroke.to]))
        .collect::<std::collections::BTreeSet<_>>();
    let expected_entries = gerber_mechanical_expected_entries(
        &polygons,
        &component_polygons,
        &component_strokes,
        &component_polyline_strokes,
        &component_circle_strokes,
        &component_arc_strokes,
        &dimension_strokes,
        &board_text_strokes,
        &component_text_strokes,
    );
    let actual_entries = gerber_mechanical_actual_entries(
        &parsed,
        &expected_component_polygon_signatures,
        &expected_component_signatures,
        &expected_component_polyline_signatures,
        &expected_component_circle_signatures,
        &expected_component_arc_signatures,
        &expected_dimension_signatures,
        &expected_board_text_signatures,
        &expected_component_text_signatures,
    );
    let (expected_entries, actual_entries) =
        if project.board.graphics.values().any(|g| g.layer == layer) {
            let all_polygons: Vec<_> = polygons
                .iter()
                .chain(component_polygons.iter())
                .cloned()
                .collect();
            let all_strokes: Vec<_> = component_strokes
                .iter()
                .chain(component_polyline_strokes.iter())
                .chain(component_circle_strokes.iter())
                .chain(component_arc_strokes.iter())
                .chain(dimension_strokes.iter())
                .chain(board_text_strokes.iter())
                .chain(component_text_strokes.iter())
                .copied()
                .collect();
            let base = render_rs274x_mechanical_layer(layer, &all_polygons, &all_strokes)?;
            let expected = eda_engine::export::append_rs274x_board_graphics(
                &base,
                layer,
                &project.board.graphics.values().cloned().collect::<Vec<_>>(),
            )?;
            let expected = parse_rs274x_subset(&expected)?;
            (
                super::super::semantics::gerber_silkscreen_expected_entries(&expected),
                super::super::semantics::gerber_silkscreen_expected_entries(&parsed),
            )
        } else {
            (expected_entries, actual_entries)
        };
    let (matched_count, missing_count, extra_count, matched, missing, extra) =
        compare_entry_views(expected_entries, actual_entries);

    Ok(NativeProjectGerberMechanicalComparisonView {
        action: "compare_gerber_mechanical_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        board_graphic_count: project
            .board
            .graphics
            .values()
            .filter(|g| g.layer == layer)
            .count(),
        expected_keepout_count: polygons.len(),
        expected_dimension_count: dimension_count,
        expected_board_text_count: board_text_count,
        expected_component_text_count: component_text_count,
        expected_component_polygon_count: component_polygon_count,
        expected_component_stroke_count: component_stroke_count,
        expected_component_polyline_count: component_polyline_count,
        expected_component_circle_count: component_circle_count,
        expected_component_arc_count: component_arc_count,
        actual_geometry_count: parsed.geometries.len(),
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    })
}
