use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::board::Via;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NativeProjectDrillCsvRow {
    pub(super) net_uuid: Uuid,
    pub(super) x_nm: i64,
    pub(super) y_nm: i64,
    pub(super) drill_nm: i64,
    pub(super) diameter_nm: i64,
    pub(super) from_layer: i32,
    pub(super) to_layer: i32,
}

pub(super) fn csv_drill_row_from_via(via: Via) -> (Uuid, NativeProjectDrillCsvRow) {
    (
        via.uuid,
        NativeProjectDrillCsvRow {
            net_uuid: via.net,
            x_nm: via.position.x,
            y_nm: via.position.y,
            drill_nm: via.drill,
            diameter_nm: via.diameter,
            from_layer: via.from_layer,
            to_layer: via.to_layer,
        },
    )
}

pub(super) fn parse_native_project_drill_csv(
    drill_path: &Path,
) -> Result<BTreeMap<Uuid, NativeProjectDrillCsvRow>> {
    Ok(parse_native_project_drill_csv_rows(drill_path)?
        .into_iter()
        .collect())
}

pub(super) fn parse_native_project_drill_csv_rows(
    drill_path: &Path,
) -> Result<Vec<(Uuid, NativeProjectDrillCsvRow)>> {
    let contents = std::fs::read_to_string(drill_path)
        .with_context(|| format!("failed to read {}", drill_path.display()))?;
    let mut lines = contents.lines();
    let header = lines.next().unwrap_or_default();
    if header != "via_uuid,net_uuid,x_nm,y_nm,drill_nm,diameter_nm,from_layer,to_layer" {
        bail!("unexpected drill CSV header in {}", drill_path.display());
    }
    let mut rows = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let columns = line.split(',').collect::<Vec<_>>();
        if columns.len() != 8 {
            bail!(
                "unexpected drill CSV column count on line {} in {}",
                index + 2,
                drill_path.display()
            );
        }
        rows.push((
            parse_uuid(columns[0], "via_uuid", index, drill_path)?,
            row_from_columns(&columns, index, drill_path)?,
        ));
    }
    Ok(rows)
}

fn row_from_columns(
    columns: &[&str],
    index: usize,
    drill_path: &Path,
) -> Result<NativeProjectDrillCsvRow> {
    Ok(NativeProjectDrillCsvRow {
        net_uuid: parse_uuid(columns[1], "net_uuid", index, drill_path)?,
        x_nm: parse_i64(columns[2], "x_nm", index, drill_path)?,
        y_nm: parse_i64(columns[3], "y_nm", index, drill_path)?,
        drill_nm: parse_i64(columns[4], "drill_nm", index, drill_path)?,
        diameter_nm: parse_i64(columns[5], "diameter_nm", index, drill_path)?,
        from_layer: parse_i32(columns[6], "from_layer", index, drill_path)?,
        to_layer: parse_i32(columns[7], "to_layer", index, drill_path)?,
    })
}

fn parse_uuid(value: &str, label: &str, index: usize, drill_path: &Path) -> Result<Uuid> {
    Uuid::parse_str(value).with_context(|| {
        format!(
            "invalid {label} on line {} in {}",
            index + 2,
            drill_path.display()
        )
    })
}

fn parse_i64(value: &str, label: &str, index: usize, drill_path: &Path) -> Result<i64> {
    value.parse().with_context(|| {
        format!(
            "invalid {label} on line {} in {}",
            index + 2,
            drill_path.display()
        )
    })
}

fn parse_i32(value: &str, label: &str, index: usize, drill_path: &Path) -> Result<i32> {
    value.parse().with_context(|| {
        format!(
            "invalid {label} on line {} in {}",
            index + 2,
            drill_path.display()
        )
    })
}
