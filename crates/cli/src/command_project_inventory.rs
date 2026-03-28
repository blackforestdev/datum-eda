use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use crate::{NativeProjectBomInspectionRowView, NativeProjectBomInspectionView};
use eda_engine::board::PlacedPackage;

use super::{
    NativeProjectBomComparisonView, NativeProjectBomDriftView, NativeProjectBomExportView,
    NativeProjectPnpComparisonView, NativeProjectPnpDriftView, NativeProjectPnpExportView,
    csv_escape, load_native_project, parse_csv_line, query_native_project_board_components,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeBomRow {
    reference: String,
    value: String,
    part_uuid: String,
    package_uuid: String,
    layer: i32,
    x_nm: i64,
    y_nm: i64,
    rotation_deg: i32,
    locked: bool,
}

impl NativeBomRow {
    fn diff_fields(&self, other: &Self) -> Vec<String> {
        let mut fields = Vec::new();
        if self.value != other.value {
            fields.push("value".to_string());
        }
        if self.part_uuid != other.part_uuid {
            fields.push("part_uuid".to_string());
        }
        if self.package_uuid != other.package_uuid {
            fields.push("package_uuid".to_string());
        }
        if self.layer != other.layer {
            fields.push("layer".to_string());
        }
        if self.x_nm != other.x_nm || self.y_nm != other.y_nm {
            fields.push("position".to_string());
        }
        if self.rotation_deg != other.rotation_deg {
            fields.push("rotation_deg".to_string());
        }
        if self.locked != other.locked {
            fields.push("locked".to_string());
        }
        fields
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativePnpRow {
    reference: String,
    x_nm: i64,
    y_nm: i64,
    rotation_deg: i32,
    layer: i32,
    side: String,
    package_uuid: String,
    part_uuid: String,
    value: String,
    locked: bool,
}

impl NativePnpRow {
    fn diff_fields(&self, other: &Self) -> Vec<String> {
        let mut fields = Vec::new();
        if self.x_nm != other.x_nm || self.y_nm != other.y_nm {
            fields.push("position".to_string());
        }
        if self.rotation_deg != other.rotation_deg {
            fields.push("rotation_deg".to_string());
        }
        if self.layer != other.layer {
            fields.push("layer".to_string());
        }
        if self.side != other.side {
            fields.push("side".to_string());
        }
        if self.package_uuid != other.package_uuid {
            fields.push("package_uuid".to_string());
        }
        if self.part_uuid != other.part_uuid {
            fields.push("part_uuid".to_string());
        }
        if self.value != other.value {
            fields.push("value".to_string());
        }
        if self.locked != other.locked {
            fields.push("locked".to_string());
        }
        fields
    }
}

pub(crate) fn export_native_project_bom(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectBomExportView> {
    let project = load_native_project(root)?;
    let components = query_native_project_board_components(root)?;
    let mut csv = String::from(
        "reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked\n",
    );
    for component in &components {
        let row = [
            csv_escape(&component.reference),
            csv_escape(&component.value),
            csv_escape(&component.part.to_string()),
            csv_escape(&component.package.to_string()),
            component.layer.to_string(),
            component.position.x.to_string(),
            component.position.y.to_string(),
            component.rotation.to_string(),
            component.locked.to_string(),
        ]
        .join(",");
        csv.push_str(&row);
        csv.push('\n');
    }
    std::fs::write(output_path, csv)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectBomExportView {
        action: "export_bom".to_string(),
        project_root: project.root.display().to_string(),
        bom_path: output_path.display().to_string(),
        rows: components.len(),
    })
}

pub(crate) fn compare_native_project_bom(
    root: &Path,
    bom_path: &Path,
) -> Result<NativeProjectBomComparisonView> {
    let project = load_native_project(root)?;
    let expected = query_native_project_board_components(root)?
        .into_iter()
        .map(component_to_bom_row)
        .collect::<Vec<_>>();
    let actual = parse_bom_csv(bom_path)?;

    let expected_by_reference = expected
        .iter()
        .map(|row| (row.reference.clone(), row.clone()))
        .collect::<BTreeMap<_, _>>();
    let actual_by_reference = actual
        .iter()
        .map(|row| (row.reference.clone(), row.clone()))
        .collect::<BTreeMap<_, _>>();

    let matched = expected_by_reference
        .iter()
        .filter_map(|(reference, expected)| {
            actual_by_reference
                .get(reference)
                .filter(|actual| *actual == expected)
                .map(|_| reference.clone())
        })
        .collect::<Vec<_>>();
    let missing = expected_by_reference
        .keys()
        .filter(|reference| !actual_by_reference.contains_key(*reference))
        .cloned()
        .collect::<Vec<_>>();
    let extra = actual_by_reference
        .keys()
        .filter(|reference| !expected_by_reference.contains_key(*reference))
        .cloned()
        .collect::<Vec<_>>();
    let drift = expected_by_reference
        .iter()
        .filter_map(|(reference, expected)| {
            actual_by_reference.get(reference).and_then(|actual| {
                let fields = expected.diff_fields(actual);
                if fields.is_empty() {
                    None
                } else {
                    Some(NativeProjectBomDriftView {
                        reference: reference.clone(),
                        fields,
                    })
                }
            })
        })
        .collect::<Vec<_>>();

    Ok(NativeProjectBomComparisonView {
        action: "compare_bom".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        bom_path: bom_path.display().to_string(),
        expected_count: expected.len(),
        actual_count: actual.len(),
        matched_count: matched.len(),
        missing_count: missing.len(),
        extra_count: extra.len(),
        drift_count: drift.len(),
        matched,
        missing,
        extra,
        drift,
    })
}

pub(crate) fn inspect_native_project_bom(bom_path: &Path) -> Result<NativeProjectBomInspectionView> {
    let rows = parse_bom_csv(bom_path)?
        .into_iter()
        .map(|row| NativeProjectBomInspectionRowView {
            reference: row.reference,
            value: row.value,
            part_uuid: row.part_uuid,
            package_uuid: row.package_uuid,
            layer: row.layer,
            x_nm: row.x_nm,
            y_nm: row.y_nm,
            rotation_deg: row.rotation_deg,
            locked: row.locked,
        })
        .collect::<Vec<_>>();
    Ok(NativeProjectBomInspectionView {
        action: "inspect_bom".to_string(),
        bom_path: bom_path.display().to_string(),
        row_count: rows.len(),
        rows,
    })
}

pub(crate) fn export_native_project_pnp(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectPnpExportView> {
    let project = load_native_project(root)?;
    let components = query_native_project_board_components(root)?;
    let mut csv = String::from(
        "reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked\n",
    );
    for component in &components {
        let side = if component.layer <= 16 { "top" } else { "bottom" };
        let row = [
            csv_escape(&component.reference),
            component.position.x.to_string(),
            component.position.y.to_string(),
            component.rotation.to_string(),
            component.layer.to_string(),
            side.to_string(),
            csv_escape(&component.package.to_string()),
            csv_escape(&component.part.to_string()),
            csv_escape(&component.value),
            component.locked.to_string(),
        ]
        .join(",");
        csv.push_str(&row);
        csv.push('\n');
    }
    std::fs::write(output_path, csv)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectPnpExportView {
        action: "export_pnp".to_string(),
        project_root: project.root.display().to_string(),
        pnp_path: output_path.display().to_string(),
        rows: components.len(),
    })
}

pub(crate) fn compare_native_project_pnp(
    root: &Path,
    pnp_path: &Path,
) -> Result<NativeProjectPnpComparisonView> {
    let project = load_native_project(root)?;
    let expected = query_native_project_board_components(root)?
        .into_iter()
        .map(component_to_pnp_row)
        .collect::<Vec<_>>();
    let actual = parse_pnp_csv(pnp_path)?;

    let expected_by_reference = expected
        .iter()
        .map(|row| (row.reference.clone(), row.clone()))
        .collect::<BTreeMap<_, _>>();
    let actual_by_reference = actual
        .iter()
        .map(|row| (row.reference.clone(), row.clone()))
        .collect::<BTreeMap<_, _>>();

    let matched = expected_by_reference
        .iter()
        .filter_map(|(reference, expected)| {
            actual_by_reference
                .get(reference)
                .filter(|actual| *actual == expected)
                .map(|_| reference.clone())
        })
        .collect::<Vec<_>>();
    let missing = expected_by_reference
        .keys()
        .filter(|reference| !actual_by_reference.contains_key(*reference))
        .cloned()
        .collect::<Vec<_>>();
    let extra = actual_by_reference
        .keys()
        .filter(|reference| !expected_by_reference.contains_key(*reference))
        .cloned()
        .collect::<Vec<_>>();
    let drift = expected_by_reference
        .iter()
        .filter_map(|(reference, expected)| {
            actual_by_reference.get(reference).and_then(|actual| {
                let fields = expected.diff_fields(actual);
                if fields.is_empty() {
                    None
                } else {
                    Some(NativeProjectPnpDriftView {
                        reference: reference.clone(),
                        fields,
                    })
                }
            })
        })
        .collect::<Vec<_>>();

    Ok(NativeProjectPnpComparisonView {
        action: "compare_pnp".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        pnp_path: pnp_path.display().to_string(),
        expected_count: expected.len(),
        actual_count: actual.len(),
        matched_count: matched.len(),
        missing_count: missing.len(),
        extra_count: extra.len(),
        drift_count: drift.len(),
        matched,
        missing,
        extra,
        drift,
    })
}

fn component_to_bom_row(component: PlacedPackage) -> NativeBomRow {
    NativeBomRow {
        reference: component.reference,
        value: component.value,
        part_uuid: component.part.to_string(),
        package_uuid: component.package.to_string(),
        layer: component.layer,
        x_nm: component.position.x,
        y_nm: component.position.y,
        rotation_deg: component.rotation,
        locked: component.locked,
    }
}

fn component_to_pnp_row(component: PlacedPackage) -> NativePnpRow {
    NativePnpRow {
        reference: component.reference,
        x_nm: component.position.x,
        y_nm: component.position.y,
        rotation_deg: component.rotation,
        layer: component.layer,
        side: if component.layer <= 16 {
            "top".to_string()
        } else {
            "bottom".to_string()
        },
        package_uuid: component.package.to_string(),
        part_uuid: component.part.to_string(),
        value: component.value,
        locked: component.locked,
    }
}

fn parse_bom_csv(path: &Path) -> Result<Vec<NativeBomRow>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut lines = contents.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing BOM CSV header in {}", path.display()))?;
    if header != "reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked" {
        bail!("unexpected BOM CSV header in {}", path.display());
    }

    let mut rows = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let fields = parse_csv_line(line)
            .with_context(|| format!("failed to parse BOM CSV row in {}", path.display()))?;
        if fields.len() != 9 {
            bail!("unexpected BOM CSV column count in {}", path.display());
        }
        rows.push(NativeBomRow {
            reference: fields[0].clone(),
            value: fields[1].clone(),
            part_uuid: fields[2].clone(),
            package_uuid: fields[3].clone(),
            layer: fields[4].parse().context("invalid layer")?,
            x_nm: fields[5].parse().context("invalid x_nm")?,
            y_nm: fields[6].parse().context("invalid y_nm")?,
            rotation_deg: fields[7].parse().context("invalid rotation_deg")?,
            locked: fields[8].parse().context("invalid locked")?,
        });
    }
    Ok(rows)
}

fn parse_pnp_csv(path: &Path) -> Result<Vec<NativePnpRow>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut lines = contents.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing PnP CSV header in {}", path.display()))?;
    if header != "reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked" {
        bail!("unexpected PnP CSV header in {}", path.display());
    }

    let mut rows = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let fields = parse_csv_line(line)
            .with_context(|| format!("failed to parse PnP CSV row in {}", path.display()))?;
        if fields.len() != 10 {
            bail!("unexpected PnP CSV column count in {}", path.display());
        }
        rows.push(NativePnpRow {
            reference: fields[0].clone(),
            x_nm: fields[1].parse().context("invalid x_nm")?,
            y_nm: fields[2].parse().context("invalid y_nm")?,
            rotation_deg: fields[3].parse().context("invalid rotation_deg")?,
            layer: fields[4].parse().context("invalid layer")?,
            side: fields[5].clone(),
            package_uuid: fields[6].clone(),
            part_uuid: fields[7].clone(),
            value: fields[8].clone(),
            locked: fields[9].parse().context("invalid locked")?,
        });
    }
    Ok(rows)
}
