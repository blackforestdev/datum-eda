use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};

use super::{
    NativeProjectGerberGeometryEntryView, NativeProjectGerberInspectionView, ParsedGerber,
    ParsedGerberGeometry, parse_rs274x_subset, render_parsed_flash_geometry,
    render_region_geometry, render_stroke_geometry,
};

pub(crate) fn inspect_gerber(gerber_path: &Path) -> Result<NativeProjectGerberInspectionView> {
    let contents = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let parsed = parse_rs274x_subset(&contents)
        .context("failed to parse Gerber inspection input as supported RS-274X subset")?;
    let stroke_count = parsed
        .geometries
        .iter()
        .filter(|geometry| matches!(geometry, ParsedGerberGeometry::Stroke { .. }))
        .count();
    let flash_count = parsed
        .geometries
        .iter()
        .filter(|geometry| matches!(geometry, ParsedGerberGeometry::Flash { .. }))
        .count();
    let region_count = parsed
        .geometries
        .iter()
        .filter(|geometry| matches!(geometry, ParsedGerberGeometry::Region { .. }))
        .count();
    Ok(NativeProjectGerberInspectionView {
        action: "inspect_gerber".to_string(),
        gerber_path: gerber_path.display().to_string(),
        geometry_count: parsed.geometries.len(),
        stroke_count,
        flash_count,
        region_count,
        entries: gerber_inspection_entries(&parsed),
    })
}

fn gerber_inspection_entries(gerber: &ParsedGerber) -> Vec<NativeProjectGerberGeometryEntryView> {
    let mut entries = BTreeMap::<(String, String), usize>::new();
    for geometry in &gerber.geometries {
        let (kind, signature) = match geometry {
            ParsedGerberGeometry::Stroke {
                aperture_diameter_nm,
                points,
            } => (
                "stroke".to_string(),
                render_stroke_geometry(*aperture_diameter_nm, points),
            ),
            ParsedGerberGeometry::Flash { aperture, position } => (
                "flash".to_string(),
                render_parsed_flash_geometry(aperture, position),
            ),
            ParsedGerberGeometry::Region { points } => {
                ("region".to_string(), render_region_geometry(points))
            }
        };
        *entries.entry((kind, signature)).or_insert(0) += 1;
    }
    entries
        .into_iter()
        .map(
            |((kind, geometry), count)| NativeProjectGerberGeometryEntryView {
                kind,
                geometry,
                count,
            },
        )
        .collect()
}
