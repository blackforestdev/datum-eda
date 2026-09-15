use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::substrate::{ArtifactProductionProjection, DesignModel, PanelProjection};
use uuid::Uuid;

use super::{compute_source_hash_bytes, load_native_project_with_resolved_board};

pub(crate) fn panelize_and_rewrite_rs274x_gerber_file(
    root: &Path,
    output_path: &Path,
    panel_projection: &PanelProjection,
) -> Result<String> {
    let board_gerber = std::fs::read_to_string(output_path)
        .with_context(|| format!("failed to read {}", output_path.display()))?;
    let panel_gerber = panelize_rs274x_gerber(root, &board_gerber, panel_projection)?;
    std::fs::write(output_path, &panel_gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(panel_gerber)
}

pub(crate) fn panelize_rs274x_gerber(
    root: &Path,
    base: &str,
    panel_projection: &PanelProjection,
) -> Result<String> {
    let project = load_native_project_with_resolved_board(root)?;
    let instances = panel_gerber_instances(&project.board.uuid, panel_projection)?;
    let (header, body) = split_rs274x_body(base)?;
    let mut panel = header.to_string();
    for instance in instances {
        panel.push_str(&translate_rs274x_body(body, instance.x_nm, instance.y_nm)?);
    }
    panel.push_str("M02*\n");
    Ok(panel)
}

pub(crate) fn panel_gerber_production_projection(
    model: &DesignModel,
    bytes: &[u8],
) -> ArtifactProductionProjection {
    ArtifactProductionProjection {
        projection_kind: "panel_gerber_copper_layer".to_string(),
        projection_contract: "datum.production_projection.panel_gerber_copper_layer.v1".to_string(),
        model_revision: model.model_revision.clone(),
        byte_count: bytes.len(),
        sha256: compute_source_hash_bytes(bytes),
    }
}

fn panel_gerber_instances<'a>(
    board: &Uuid,
    panel_projection: &'a PanelProjection,
) -> Result<Vec<&'a eda_engine::substrate::PanelBoardInstance>> {
    let instances = panel_projection
        .board_instances
        .iter()
        .filter(|instance| &instance.board == board)
        .collect::<Vec<_>>();
    if instances.is_empty() {
        anyhow::bail!(
            "panel projection {} does not reference board {}",
            panel_projection.id,
            board
        );
    }
    for instance in &instances {
        if instance.rotation_deg != 0 {
            anyhow::bail!(
                "panel projection {} has board instance rotation {}; panel Gerber export currently supports translation-only instances",
                panel_projection.id,
                instance.rotation_deg
            );
        }
    }
    Ok(instances)
}

fn split_rs274x_body(gerber: &str) -> Result<(&str, &str)> {
    let Some(end_index) = gerber.rfind("M02*") else {
        anyhow::bail!("RS-274X Gerber missing M02 terminator");
    };
    let without_end = &gerber[..end_index];
    let mut cursor = 0usize;
    for line in without_end.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with('D') || trimmed.starts_with('X') || trimmed == "G36*" {
            return Ok((&without_end[..cursor], &without_end[cursor..]));
        }
        cursor += line.len();
    }
    Ok((without_end, ""))
}

fn translate_rs274x_body(body: &str, dx_nm: i64, dy_nm: i64) -> Result<String> {
    let mut out = String::new();
    for line in body.lines() {
        out.push_str(&translate_rs274x_line(line, dx_nm, dy_nm)?);
        out.push('\n');
    }
    Ok(out)
}

fn translate_rs274x_line(line: &str, dx_nm: i64, dy_nm: i64) -> Result<String> {
    let Some(x_start) = line.find('X') else {
        return Ok(line.to_string());
    };
    let x_digits_start = x_start + 1;
    let x_digits_end = coordinate_token_end(line, x_digits_start);
    let Some(y_start_rel) = line[x_digits_end..].find('Y') else {
        return Ok(line.to_string());
    };
    let y_start = x_digits_end + y_start_rel;
    let y_digits_start = y_start + 1;
    let y_digits_end = coordinate_token_end(line, y_digits_start);
    let x = parse_rs274x_coord(&line[x_digits_start..x_digits_end])? + dx_nm;
    let y = parse_rs274x_coord(&line[y_digits_start..y_digits_end])? + dy_nm;
    Ok(format!(
        "{}X{}Y{}{}",
        &line[..x_start],
        x,
        y,
        &line[y_digits_end..]
    ))
}

fn coordinate_token_end(line: &str, start: usize) -> usize {
    line[start..]
        .find(|ch: char| !(ch == '-' || ch.is_ascii_digit()))
        .map(|offset| start + offset)
        .unwrap_or(line.len())
}

fn parse_rs274x_coord(value: &str) -> Result<i64> {
    value
        .parse::<i64>()
        .with_context(|| format!("invalid RS-274X coordinate `{value}`"))
}
