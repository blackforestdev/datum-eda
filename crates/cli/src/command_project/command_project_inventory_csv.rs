use std::path::Path;

use anyhow::{Context, Result, bail};
use uuid::Uuid;

use super::command_project_inventory::{NativeBomRow, NativePnpRow};
use super::command_project_support::{csv_escape, parse_csv_line};

pub(super) fn render_expected_native_project_bom_csv_rows(rows: &[NativeBomRow]) -> String {
    let mut csv = String::from(
        "component_instance_uuid,reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked\n",
    );
    for row in rows {
        let row = [
            csv_escape(
                &row.component_instance_uuid
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
            ),
            csv_escape(&row.reference),
            csv_escape(&row.value),
            csv_escape(&row.part_uuid),
            csv_escape(&row.package_uuid),
            row.layer.to_string(),
            row.x_nm.to_string(),
            row.y_nm.to_string(),
            row.rotation_deg.to_string(),
            row.locked.to_string(),
        ]
        .join(",");
        csv.push_str(&row);
        csv.push('\n');
    }
    csv
}

pub(super) fn render_expected_native_project_pnp_csv_rows(rows: &[NativePnpRow]) -> String {
    let mut csv = String::from(
        "component_instance_uuid,reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked\n",
    );
    for row in rows {
        let row = [
            csv_escape(
                &row.component_instance_uuid
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
            ),
            csv_escape(&row.reference),
            row.x_nm.to_string(),
            row.y_nm.to_string(),
            row.rotation_deg.to_string(),
            row.layer.to_string(),
            csv_escape(&row.side),
            csv_escape(&row.package_uuid),
            csv_escape(&row.part_uuid),
            csv_escape(&row.value),
            row.locked.to_string(),
        ]
        .join(",");
        csv.push_str(&row);
        csv.push('\n');
    }
    csv
}

pub(super) fn parse_bom_csv(path: &Path) -> Result<Vec<NativeBomRow>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut lines = contents.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing BOM CSV header in {}", path.display()))?;
    let has_component_instance = match header {
        "component_instance_uuid,reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked" => {
            true
        }
        "reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked" => false,
        _ => bail!("unexpected BOM CSV header in {}", path.display()),
    };

    let mut rows = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let fields = parse_csv_line(line)
            .with_context(|| format!("failed to parse BOM CSV row in {}", path.display()))?;
        let expected_len = if has_component_instance { 10 } else { 9 };
        if fields.len() != expected_len {
            bail!("unexpected BOM CSV column count in {}", path.display());
        }
        let offset = if has_component_instance { 1 } else { 0 };
        rows.push(NativeBomRow {
            component_instance_uuid: parse_optional_uuid(&fields[0..offset]),
            reference: fields[offset].clone(),
            value: fields[offset + 1].clone(),
            part_uuid: fields[offset + 2].clone(),
            package_uuid: fields[offset + 3].clone(),
            layer: fields[offset + 4].parse().context("invalid layer")?,
            x_nm: fields[offset + 5].parse().context("invalid x_nm")?,
            y_nm: fields[offset + 6].parse().context("invalid y_nm")?,
            rotation_deg: fields[offset + 7].parse().context("invalid rotation_deg")?,
            locked: fields[offset + 8].parse().context("invalid locked")?,
        });
    }
    Ok(rows)
}

pub(super) fn parse_pnp_csv(path: &Path) -> Result<Vec<NativePnpRow>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut lines = contents.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing PnP CSV header in {}", path.display()))?;
    let has_component_instance = match header {
        "component_instance_uuid,reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked" => {
            true
        }
        "reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked" => false,
        _ => bail!("unexpected PnP CSV header in {}", path.display()),
    };

    let mut rows = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let fields = parse_csv_line(line)
            .with_context(|| format!("failed to parse PnP CSV row in {}", path.display()))?;
        let expected_len = if has_component_instance { 11 } else { 10 };
        if fields.len() != expected_len {
            bail!("unexpected PnP CSV column count in {}", path.display());
        }
        let offset = if has_component_instance { 1 } else { 0 };
        rows.push(NativePnpRow {
            component_instance_uuid: parse_optional_uuid(&fields[0..offset]),
            reference: fields[offset].clone(),
            x_nm: fields[offset + 1].parse().context("invalid x_nm")?,
            y_nm: fields[offset + 2].parse().context("invalid y_nm")?,
            rotation_deg: fields[offset + 3].parse().context("invalid rotation_deg")?,
            layer: fields[offset + 4].parse().context("invalid layer")?,
            side: fields[offset + 5].clone(),
            package_uuid: fields[offset + 6].clone(),
            part_uuid: fields[offset + 7].clone(),
            value: fields[offset + 8].clone(),
            locked: fields[offset + 9].parse().context("invalid locked")?,
        });
    }
    Ok(rows)
}

fn parse_optional_uuid(fields: &[String]) -> Option<Uuid> {
    fields
        .first()
        .filter(|value| !value.is_empty())
        .and_then(|value| Uuid::parse_str(value).ok())
}
