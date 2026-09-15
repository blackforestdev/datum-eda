// commands/gerber/views_render.rs — text renderers for gerber export,
// validation, comparison, and plan views (moved from main.rs in Wave 2).

#[allow(unused_imports)]
use super::*;

pub(crate) fn render_native_project_gerber_outline_export_text(
    report: &NativeProjectGerberOutlineExportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("outline_vertex_count: {}", report.outline_vertex_count),
        format!("outline_closed: {}", report.outline_closed),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_gerber_copper_export_text(
    report: &NativeProjectGerberCopperExportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("pad_count: {}", report.pad_count),
        format!("track_count: {}", report.track_count),
        format!("zone_count: {}", report.zone_count),
        format!("via_count: {}", report.via_count),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_gerber_soldermask_export_text(
    report: &NativeProjectGerberSoldermaskExportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("source_copper_layer: {}", report.source_copper_layer),
        format!("pad_count: {}", report.pad_count),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_gerber_paste_export_text(
    report: &NativeProjectGerberPasteExportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("source_copper_layer: {}", report.source_copper_layer),
        format!("pad_count: {}", report.pad_count),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_gerber_outline_validation_text(
    report: &NativeProjectGerberOutlineValidationView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("matches_expected: {}", report.matches_expected),
        format!("expected_bytes: {}", report.expected_bytes),
        format!("actual_bytes: {}", report.actual_bytes),
        format!("outline_vertex_count: {}", report.outline_vertex_count),
        format!("outline_closed: {}", report.outline_closed),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_gerber_copper_validation_text(
    report: &NativeProjectGerberCopperValidationView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("matches_expected: {}", report.matches_expected),
        format!("expected_bytes: {}", report.expected_bytes),
        format!("actual_bytes: {}", report.actual_bytes),
        format!("pad_count: {}", report.pad_count),
        format!("track_count: {}", report.track_count),
        format!("zone_count: {}", report.zone_count),
        format!("via_count: {}", report.via_count),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_gerber_soldermask_validation_text(
    report: &NativeProjectGerberSoldermaskValidationView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("source_copper_layer: {}", report.source_copper_layer),
        format!("matches_expected: {}", report.matches_expected),
        format!("expected_bytes: {}", report.expected_bytes),
        format!("actual_bytes: {}", report.actual_bytes),
        format!("pad_count: {}", report.pad_count),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_gerber_paste_validation_text(
    report: &NativeProjectGerberPasteValidationView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("source_copper_layer: {}", report.source_copper_layer),
        format!("matches_expected: {}", report.matches_expected),
        format!("expected_bytes: {}", report.expected_bytes),
        format!("actual_bytes: {}", report.actual_bytes),
        format!("pad_count: {}", report.pad_count),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_gerber_outline_comparison_text(
    report: &NativeProjectGerberOutlineComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("expected_outline_count: {}", report.expected_outline_count),
        format!("actual_geometry_count: {}", report.actual_geometry_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
    ];
    append_gerber_geometry_entries(&mut lines, "matched", &report.matched);
    append_gerber_geometry_entries(&mut lines, "missing", &report.missing);
    append_gerber_geometry_entries(&mut lines, "extra", &report.extra);
    lines.join("\n")
}

pub(crate) fn render_native_project_gerber_copper_comparison_text(
    report: &NativeProjectGerberCopperComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("expected_pad_count: {}", report.expected_pad_count),
        format!("actual_pad_count: {}", report.actual_pad_count),
        format!("expected_track_count: {}", report.expected_track_count),
        format!("actual_track_count: {}", report.actual_track_count),
        format!("expected_zone_count: {}", report.expected_zone_count),
        format!("actual_zone_count: {}", report.actual_zone_count),
        format!("expected_via_count: {}", report.expected_via_count),
        format!("actual_via_count: {}", report.actual_via_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
    ];
    append_gerber_geometry_entries(&mut lines, "matched", &report.matched);
    append_gerber_geometry_entries(&mut lines, "missing", &report.missing);
    append_gerber_geometry_entries(&mut lines, "extra", &report.extra);
    lines.join("\n")
}

pub(crate) fn render_native_project_gerber_soldermask_comparison_text(
    report: &NativeProjectGerberSoldermaskComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("source_copper_layer: {}", report.source_copper_layer),
        format!("expected_pad_count: {}", report.expected_pad_count),
        format!("actual_pad_count: {}", report.actual_pad_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
    ];
    append_gerber_geometry_entries(&mut lines, "matched", &report.matched);
    append_gerber_geometry_entries(&mut lines, "missing", &report.missing);
    append_gerber_geometry_entries(&mut lines, "extra", &report.extra);
    lines.join("\n")
}

pub(crate) fn render_native_project_gerber_paste_comparison_text(
    report: &NativeProjectGerberPasteComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("source_copper_layer: {}", report.source_copper_layer),
        format!("expected_pad_count: {}", report.expected_pad_count),
        format!("actual_pad_count: {}", report.actual_pad_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
    ];
    append_gerber_geometry_entries(&mut lines, "matched", &report.matched);
    append_gerber_geometry_entries(&mut lines, "missing", &report.missing);
    append_gerber_geometry_entries(&mut lines, "extra", &report.extra);
    lines.join("\n")
}

pub(crate) fn append_gerber_geometry_entries(
    lines: &mut Vec<String>,
    label: &str,
    entries: &[NativeProjectGerberGeometryEntryView],
) {
    if entries.is_empty() {
        return;
    }
    lines.push(format!("{label}:"));
    for entry in entries {
        lines.push(format!(
            "- kind={} count={} geometry={}",
            entry.kind, entry.count, entry.geometry
        ));
    }
}

pub(crate) fn render_native_project_gerber_plan_text(
    report: &NativeProjectGerberPlanView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("prefix: {}", report.prefix),
        format!("outline_vertex_count: {}", report.outline_vertex_count),
        format!("outline_closed: {}", report.outline_closed),
        format!("copper_layers: {}", report.copper_layers),
        format!("soldermask_layers: {}", report.soldermask_layers),
        format!("silkscreen_layers: {}", report.silkscreen_layers),
        format!("paste_layers: {}", report.paste_layers),
        format!("mechanical_layers: {}", report.mechanical_layers),
    ];
    if !report.artifacts.is_empty() {
        lines.push("artifacts:".to_string());
        for artifact in &report.artifacts {
            let layer_suffix = match (artifact.layer_id, artifact.layer_name.as_ref()) {
                (Some(layer_id), Some(layer_name)) => format!(" layer={layer_id}:{layer_name}"),
                (Some(layer_id), None) => format!(" layer={layer_id}"),
                _ => String::new(),
            };
            lines.push(format!(
                "  {}:{}{}",
                artifact.kind, artifact.filename, layer_suffix
            ));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_gerber_plan_comparison_text(
    report: &NativeProjectGerberPlanComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("output_dir: {}", report.output_dir),
        format!("prefix: {}", report.prefix),
        format!("expected_count: {}", report.expected_count),
        format!("present_count: {}", report.present_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
    ];
    if !report.matched.is_empty() {
        lines.push("matched:".to_string());
        for file in &report.matched {
            lines.push(format!("  {file}"));
        }
    }
    if !report.missing.is_empty() {
        lines.push("missing:".to_string());
        for file in &report.missing {
            lines.push(format!("  {file}"));
        }
    }
    if !report.extra.is_empty() {
        lines.push("extra:".to_string());
        for file in &report.extra {
            lines.push(format!("  {file}"));
        }
    }
    lines.join("\n")
}
