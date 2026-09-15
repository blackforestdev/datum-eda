use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Result, anyhow};
use serde::Serialize;

use crate::{
    ParsedGerberAperture, ParsedGerberGeometry, inspect_excellon_drill, inspect_gerber,
    parse_rs274x_subset,
};

pub(crate) const MAX_PREVIEW_PRIMITIVES: usize = 64;
const MAX_PREVIEW_POINTS_PER_PRIMITIVE: usize = 64;
const MAX_PREVIEW_CSV_ROWS: usize = 6;
const MAX_PREVIEW_CSV_COLUMNS: usize = 8;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectArtifactPreviewPrimitive {
    pub(crate) kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) aperture_diameter_nm: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) aperture_width_nm: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) aperture_height_nm: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) diameter_mm: Option<String>,
    pub(crate) points: Vec<NativeProjectArtifactPreviewPoint>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectArtifactPreviewPoint {
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
struct CsvArtifactPreview {
    family: &'static str,
    header: String,
    row_count: usize,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

pub(crate) fn inspect_artifact_preview_file(
    path: &Path,
    bytes: &[u8],
) -> Result<(
    String,
    bool,
    serde_json::Value,
    Vec<NativeProjectArtifactPreviewPrimitive>,
)> {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if matches!(extension.as_str(), "gbr" | "ger" | "gtl") {
        let gerber = std::str::from_utf8(bytes)
            .map_err(|error| anyhow!("failed to decode Gerber preview as UTF-8: {error}"))?;
        let parsed = parse_rs274x_subset(gerber)?;
        return Ok((
            "gerber_rs274x".to_string(),
            true,
            serde_json::to_value(inspect_gerber(path)?)?,
            gerber_preview_primitives(&parsed.geometries),
        ));
    }
    if extension == "drl" {
        let inspection = inspect_excellon_drill(path)?;
        return Ok((
            "excellon_drill".to_string(),
            true,
            serde_json::to_value(inspection)?,
            excellon_preview_primitives(bytes)?,
        ));
    }
    if extension == "csv" {
        return Ok(csv_artifact_preview(&file_name, bytes));
    }
    Ok((
        "unsupported".to_string(),
        false,
        serde_json::Value::Null,
        Vec::new(),
    ))
}

fn csv_artifact_preview(
    file_name: &str,
    bytes: &[u8],
) -> (
    String,
    bool,
    serde_json::Value,
    Vec<NativeProjectArtifactPreviewPrimitive>,
) {
    let text = String::from_utf8_lossy(bytes);
    let mut lines = text.lines();
    let header = lines.next().unwrap_or_default().to_string();
    let rows = lines
        .filter(|line| !line.trim().is_empty())
        .map(csv_preview_cells)
        .collect::<Vec<_>>();
    let row_count = rows.len();
    let columns = csv_preview_cells(&header);
    let rows = rows.into_iter().take(MAX_PREVIEW_CSV_ROWS).collect();
    let family = if file_name.contains("bom") {
        "bom"
    } else if file_name.contains("pnp") || file_name.contains("pick") {
        "pick_and_place"
    } else if file_name.contains("drill") {
        "drill_table"
    } else {
        "csv"
    };
    (
        format!("{family}_csv"),
        true,
        serde_json::to_value(CsvArtifactPreview {
            family,
            header,
            row_count,
            columns,
            rows,
        })
        .expect("CSV preview serialization must succeed"),
        Vec::new(),
    )
}

fn csv_preview_cells(line: &str) -> Vec<String> {
    line.split(',')
        .take(MAX_PREVIEW_CSV_COLUMNS)
        .map(|cell| cell.trim().trim_matches('"').to_string())
        .collect()
}

fn gerber_preview_primitives(
    geometries: &[ParsedGerberGeometry],
) -> Vec<NativeProjectArtifactPreviewPrimitive> {
    geometries
        .iter()
        .take(MAX_PREVIEW_PRIMITIVES)
        .map(|geometry| match geometry {
            ParsedGerberGeometry::Stroke {
                aperture_diameter_nm,
                points,
            } => NativeProjectArtifactPreviewPrimitive {
                kind: "stroke",
                aperture_diameter_nm: Some(*aperture_diameter_nm),
                aperture_width_nm: None,
                aperture_height_nm: None,
                tool: None,
                diameter_mm: None,
                points: preview_points(points),
            },
            ParsedGerberGeometry::Flash { aperture, position } => {
                let (diameter, width, height) = match aperture {
                    ParsedGerberAperture::Circle { diameter_nm } => {
                        (Some(*diameter_nm), None, None)
                    }
                    ParsedGerberAperture::Rect {
                        width_nm,
                        height_nm,
                    } => (None, Some(*width_nm), Some(*height_nm)),
                };
                NativeProjectArtifactPreviewPrimitive {
                    kind: "flash",
                    aperture_diameter_nm: diameter,
                    aperture_width_nm: width,
                    aperture_height_nm: height,
                    tool: None,
                    diameter_mm: None,
                    points: vec![NativeProjectArtifactPreviewPoint {
                        x_nm: position.x,
                        y_nm: position.y,
                    }],
                }
            }
            ParsedGerberGeometry::Region { points } => NativeProjectArtifactPreviewPrimitive {
                kind: "region",
                aperture_diameter_nm: None,
                aperture_width_nm: None,
                aperture_height_nm: None,
                tool: None,
                diameter_mm: None,
                points: preview_points(points),
            },
        })
        .collect()
}

fn excellon_preview_primitives(bytes: &[u8]) -> Result<Vec<NativeProjectArtifactPreviewPrimitive>> {
    let contents = std::str::from_utf8(bytes)
        .map_err(|error| anyhow!("failed to decode Excellon preview as UTF-8: {error}"))?;
    let mut tools = BTreeMap::<String, String>::new();
    let mut current_tool = None::<String>;
    let mut primitives = Vec::new();
    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line == "M48" || line == "%" || line == "M30" || line == "METRIC,TZ" {
            continue;
        }
        if let Some(rest) = line.strip_prefix('T') {
            if let Some((tool_digits, diameter)) = rest.split_once('C') {
                tools.insert(format!("T{tool_digits}"), diameter.to_string());
                continue;
            }
            current_tool = Some(format!("T{rest}"));
            continue;
        }
        if line.starts_with('X') {
            let Some(tool) = current_tool.clone() else {
                continue;
            };
            let Some(diameter_mm) = tools.get(&tool).cloned() else {
                continue;
            };
            let (x_nm, y_nm) = parse_excellon_hit_coordinates(line)?;
            primitives.push((tool, diameter_mm, x_nm, y_nm));
        }
    }
    Ok(primitives
        .into_iter()
        .take(MAX_PREVIEW_PRIMITIVES)
        .map(
            |(tool, diameter_mm, x_nm, y_nm)| NativeProjectArtifactPreviewPrimitive {
                kind: "drill_hit",
                aperture_diameter_nm: None,
                aperture_width_nm: None,
                aperture_height_nm: None,
                tool: Some(tool),
                diameter_mm: Some(diameter_mm),
                points: vec![NativeProjectArtifactPreviewPoint { x_nm, y_nm }],
            },
        )
        .collect())
}

fn parse_excellon_hit_coordinates(line: &str) -> Result<(i64, i64)> {
    let rest = line
        .strip_prefix('X')
        .ok_or_else(|| anyhow!("Excellon hit must start with X"))?;
    let (x, y) = rest
        .split_once('Y')
        .ok_or_else(|| anyhow!("Excellon hit must include Y coordinate"))?;
    Ok((parse_decimal_mm_to_nm(x)?, parse_decimal_mm_to_nm(y)?))
}

fn parse_decimal_mm_to_nm(value: &str) -> Result<i64> {
    let sign = if value.starts_with('-') { -1 } else { 1 };
    let value = value.trim_start_matches('-');
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    let whole_nm = whole
        .parse::<i64>()
        .map_err(|error| anyhow!("invalid Excellon whole millimeter coordinate: {error}"))?
        * 1_000_000;
    let mut fraction = fraction.to_string();
    if fraction.len() > 6 {
        fraction.truncate(6);
    }
    while fraction.len() < 6 {
        fraction.push('0');
    }
    let fraction_nm = fraction
        .parse::<i64>()
        .map_err(|error| anyhow!("invalid Excellon fractional millimeter coordinate: {error}"))?;
    Ok(sign * (whole_nm + fraction_nm))
}

fn preview_points(
    points: &[eda_engine::ir::geometry::Point],
) -> Vec<NativeProjectArtifactPreviewPoint> {
    points
        .iter()
        .take(MAX_PREVIEW_POINTS_PER_PRIMITIVE)
        .map(|point| NativeProjectArtifactPreviewPoint {
            x_nm: point.x,
            y_nm: point.y,
        })
        .collect()
}
