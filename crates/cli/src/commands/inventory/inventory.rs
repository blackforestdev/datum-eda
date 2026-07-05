use crate::*;
use std::collections::BTreeMap;
use std::path::Path;

use crate::{
    NativeProjectBomInspectionRowView, NativeProjectBomInspectionView,
    NativeProjectBomValidationView, NativeProjectPnpInspectionRowView,
    NativeProjectPnpInspectionView, NativeProjectPnpValidationView,
};
use anyhow::{Context, Result, bail};
use eda_engine::board::PlacedPackage;
use eda_engine::substrate::{DerivedVariantPopulation, DesignModel, PanelProjection};
use uuid::Uuid;

#[path = "rows.rs"]
mod rows;

use super::csv::{
    parse_bom_csv, parse_pnp_csv, render_expected_native_project_bom_csv_rows,
    render_expected_native_project_pnp_csv_rows,
};
use crate::{
    NativeProjectBomComparisonView, NativeProjectBomDriftView, NativeProjectBomExportView,
    NativeProjectPnpComparisonView, NativeProjectPnpDriftView, NativeProjectPnpExportView,
    load_native_project_with_resolved_board, load_native_project_with_resolved_board_and_model,
};
use rows::{component_instances_by_package, component_to_bom_row, component_to_pnp_row};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NativeBomRow {
    pub(super) component_instance_uuid: Option<Uuid>,
    pub(super) component_instance_role: Option<String>,
    pub(super) component_instance_label: Option<String>,
    pub(super) reference: String,
    pub(super) value: String,
    pub(super) part_uuid: String,
    pub(super) package_uuid: String,
    pub(super) layer: i32,
    pub(super) x_nm: i64,
    pub(super) y_nm: i64,
    pub(super) rotation_deg: i32,
    pub(super) locked: bool,
}

impl NativeBomRow {
    fn identity_key(&self) -> String {
        self.component_instance_uuid
            .map(|id| id.to_string())
            .unwrap_or_else(|| self.package_uuid.clone())
    }

    fn diff_fields(&self, other: &Self) -> Vec<String> {
        let mut fields = Vec::new();
        if self.reference != other.reference {
            fields.push("reference".to_string());
        }
        if self.component_instance_role != other.component_instance_role {
            fields.push("component_instance_role".to_string());
        }
        if self.component_instance_label != other.component_instance_label {
            fields.push("component_instance_label".to_string());
        }
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

trait NativeVariantRow {
    fn component_instance_uuid(&self) -> Option<Uuid>;
    fn package_uuid(&self) -> &str;
}

impl NativeVariantRow for NativeBomRow {
    fn component_instance_uuid(&self) -> Option<Uuid> {
        self.component_instance_uuid
    }

    fn package_uuid(&self) -> &str {
        &self.package_uuid
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NativePnpRow {
    pub(super) component_instance_uuid: Option<Uuid>,
    pub(super) component_instance_role: Option<String>,
    pub(super) component_instance_label: Option<String>,
    pub(super) reference: String,
    pub(super) x_nm: i64,
    pub(super) y_nm: i64,
    pub(super) rotation_deg: i32,
    pub(super) layer: i32,
    pub(super) side: String,
    pub(super) package_uuid: String,
    pub(super) part_uuid: String,
    pub(super) value: String,
    pub(super) locked: bool,
}

impl NativePnpRow {
    fn identity_key(&self) -> String {
        self.component_instance_uuid
            .map(|id| id.to_string())
            .unwrap_or_else(|| self.package_uuid.clone())
    }

    fn diff_fields(&self, other: &Self) -> Vec<String> {
        let mut fields = Vec::new();
        if self.reference != other.reference {
            fields.push("reference".to_string());
        }
        if self.component_instance_role != other.component_instance_role {
            fields.push("component_instance_role".to_string());
        }
        if self.component_instance_label != other.component_instance_label {
            fields.push("component_instance_label".to_string());
        }
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

impl NativeVariantRow for NativePnpRow {
    fn component_instance_uuid(&self) -> Option<Uuid> {
        self.component_instance_uuid
    }

    fn package_uuid(&self) -> &str {
        &self.package_uuid
    }
}

pub(crate) fn export_native_project_bom(
    root: &Path,
    output_path: &Path,
    variant: Option<Uuid>,
) -> Result<NativeProjectBomExportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let rows = native_inventory_bom_rows(root, variant)?;
    let csv = render_expected_native_project_bom_csv_rows(&rows);
    std::fs::write(output_path, csv)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectBomExportView {
        action: "export_bom".to_string(),
        production_classification: "manual_debug_export".to_string(),
        project_root: project.root.display().to_string(),
        bom_path: output_path.display().to_string(),
        rows: rows.len(),
    })
}

pub(crate) fn validate_native_project_bom(
    root: &Path,
    bom_path: &Path,
    variant: Option<Uuid>,
) -> Result<NativeProjectBomValidationView> {
    let project = load_native_project_with_resolved_board(root)?;
    let rows = native_inventory_bom_rows(root, variant)?;
    let expected = render_expected_native_project_bom_csv_rows(&rows);
    let actual = std::fs::read_to_string(bom_path)
        .with_context(|| format!("failed to read {}", bom_path.display()))?;
    Ok(NativeProjectBomValidationView {
        action: "validate_bom".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        bom_path: bom_path.display().to_string(),
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        matches_expected: actual == expected,
    })
}

pub(crate) fn compare_native_project_bom(
    root: &Path,
    bom_path: &Path,
    variant: Option<Uuid>,
) -> Result<NativeProjectBomComparisonView> {
    let project = load_native_project_with_resolved_board(root)?;
    let expected = native_inventory_bom_rows(root, variant)?;
    let actual = parse_bom_csv(bom_path)?;

    let (expected_by_identity, expected_duplicates) = bom_rows_by_identity(&expected);
    let (actual_by_identity, actual_duplicates) = bom_rows_by_identity(&actual);

    let matched = expected_by_identity
        .iter()
        .filter_map(|(identity, expected)| {
            actual_by_identity
                .get(identity)
                .filter(|actual| *actual == expected)
                .map(|_| identity.clone())
        })
        .collect::<Vec<_>>();
    let missing = expected_by_identity
        .keys()
        .filter(|identity| !actual_by_identity.contains_key(*identity))
        .cloned()
        .collect::<Vec<_>>();
    let mut extra = actual_by_identity
        .keys()
        .filter(|identity| !expected_by_identity.contains_key(*identity))
        .cloned()
        .collect::<Vec<_>>();
    extra.extend(actual_duplicates);
    let mut missing = missing;
    missing.extend(expected_duplicates);
    let drift = expected_by_identity
        .iter()
        .filter_map(|(identity, expected)| {
            actual_by_identity.get(identity).and_then(|actual| {
                let fields = expected.diff_fields(actual);
                if fields.is_empty() {
                    None
                } else {
                    Some(NativeProjectBomDriftView {
                        component_instance_uuid: expected
                            .component_instance_uuid
                            .map(|id| id.to_string()),
                        reference: expected.reference.clone(),
                        identity: identity.clone(),
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

pub(crate) fn inspect_native_project_bom(
    bom_path: &Path,
) -> Result<NativeProjectBomInspectionView> {
    let rows = parse_bom_csv(bom_path)?
        .into_iter()
        .map(|row| NativeProjectBomInspectionRowView {
            component_instance_uuid: row.component_instance_uuid.map(|id| id.to_string()),
            component_instance_role: row.component_instance_role,
            component_instance_label: row.component_instance_label,
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
    variant: Option<Uuid>,
) -> Result<NativeProjectPnpExportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let rows = native_inventory_pnp_rows(root, variant)?;
    write_native_project_pnp_rows(output_path, &rows)?;
    Ok(NativeProjectPnpExportView {
        action: "export_pnp".to_string(),
        production_classification: "manual_debug_export".to_string(),
        project_root: project.root.display().to_string(),
        pnp_path: output_path.display().to_string(),
        rows: rows.len(),
    })
}

pub(crate) fn export_native_project_panel_pnp(
    root: &Path,
    output_path: &Path,
    variant: Option<Uuid>,
    panel_projection: &PanelProjection,
) -> Result<NativeProjectPnpExportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let rows = native_inventory_panel_pnp_rows(root, variant, panel_projection)?;
    write_native_project_pnp_rows(output_path, &rows)?;
    Ok(NativeProjectPnpExportView {
        action: "export_pnp".to_string(),
        production_classification: "panel_projection_export".to_string(),
        project_root: project.root.display().to_string(),
        pnp_path: output_path.display().to_string(),
        rows: rows.len(),
    })
}

pub(crate) fn render_expected_native_project_panel_pnp_csv(
    root: &Path,
    variant: Option<Uuid>,
    panel_projection: &PanelProjection,
) -> Result<String> {
    let rows = native_inventory_panel_pnp_rows(root, variant, panel_projection)?;
    Ok(render_expected_native_project_pnp_csv_rows(&rows))
}

fn write_native_project_pnp_rows(output_path: &Path, rows: &[NativePnpRow]) -> Result<()> {
    let csv = render_expected_native_project_pnp_csv_rows(rows);
    std::fs::write(output_path, csv)
        .with_context(|| format!("failed to write {}", output_path.display()))
}

pub(crate) fn validate_native_project_pnp(
    root: &Path,
    pnp_path: &Path,
    variant: Option<Uuid>,
) -> Result<NativeProjectPnpValidationView> {
    let project = load_native_project_with_resolved_board(root)?;
    let rows = native_inventory_pnp_rows(root, variant)?;
    let expected = render_expected_native_project_pnp_csv_rows(&rows);
    let actual = std::fs::read_to_string(pnp_path)
        .with_context(|| format!("failed to read {}", pnp_path.display()))?;
    Ok(NativeProjectPnpValidationView {
        action: "validate_pnp".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        pnp_path: pnp_path.display().to_string(),
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        matches_expected: actual == expected,
    })
}

pub(crate) fn compare_native_project_pnp(
    root: &Path,
    pnp_path: &Path,
    variant: Option<Uuid>,
) -> Result<NativeProjectPnpComparisonView> {
    let project = load_native_project_with_resolved_board(root)?;
    let expected = native_inventory_pnp_rows(root, variant)?;
    let actual = parse_pnp_csv(pnp_path)?;

    let (expected_by_identity, expected_duplicates) = pnp_rows_by_identity(&expected);
    let (actual_by_identity, actual_duplicates) = pnp_rows_by_identity(&actual);

    let matched = expected_by_identity
        .iter()
        .filter_map(|(identity, expected)| {
            actual_by_identity
                .get(identity)
                .filter(|actual| *actual == expected)
                .map(|_| identity.clone())
        })
        .collect::<Vec<_>>();
    let missing = expected_by_identity
        .keys()
        .filter(|identity| !actual_by_identity.contains_key(*identity))
        .cloned()
        .collect::<Vec<_>>();
    let mut extra = actual_by_identity
        .keys()
        .filter(|identity| !expected_by_identity.contains_key(*identity))
        .cloned()
        .collect::<Vec<_>>();
    extra.extend(actual_duplicates);
    let mut missing = missing;
    missing.extend(expected_duplicates);
    let drift = expected_by_identity
        .iter()
        .filter_map(|(identity, expected)| {
            actual_by_identity.get(identity).and_then(|actual| {
                let fields = expected.diff_fields(actual);
                if fields.is_empty() {
                    None
                } else {
                    Some(NativeProjectPnpDriftView {
                        component_instance_uuid: expected
                            .component_instance_uuid
                            .map(|id| id.to_string()),
                        reference: expected.reference.clone(),
                        identity: identity.clone(),
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

pub(crate) fn inspect_native_project_pnp(
    pnp_path: &Path,
) -> Result<NativeProjectPnpInspectionView> {
    let rows = parse_pnp_csv(pnp_path)?
        .into_iter()
        .map(|row| NativeProjectPnpInspectionRowView {
            component_instance_uuid: row.component_instance_uuid.map(|id| id.to_string()),
            component_instance_role: row.component_instance_role,
            component_instance_label: row.component_instance_label,
            reference: row.reference,
            x_nm: row.x_nm,
            y_nm: row.y_nm,
            rotation_deg: row.rotation_deg,
            layer: row.layer,
            side: row.side,
            package_uuid: row.package_uuid,
            part_uuid: row.part_uuid,
            value: row.value,
            locked: row.locked,
        })
        .collect::<Vec<_>>();
    Ok(NativeProjectPnpInspectionView {
        action: "inspect_pnp".to_string(),
        pnp_path: pnp_path.display().to_string(),
        row_count: rows.len(),
        rows,
    })
}

fn native_inventory_bom_rows(root: &Path, variant: Option<Uuid>) -> Result<Vec<NativeBomRow>> {
    let (project, model) = load_native_project_with_resolved_board_and_model(root)?;
    let component_instances_by_package = component_instances_by_package(&model);
    let variant_population = variant_population(&model, variant)?;
    Ok(resolved_board_components(&project.board.packages)?
        .into_iter()
        .filter_map(|component| {
            let component_instance_uuid = component_instances_by_package
                .get(&component.uuid)
                .map(|entry| entry.0);
            let role = component_instances_by_package
                .get(&component.uuid)
                .and_then(|entry| entry.1.as_ref());
            let row = component_to_bom_row(component, component_instance_uuid, role);
            fitted_for_variant(&row, variant_population).then_some(row)
        })
        .collect())
}

fn native_inventory_pnp_rows(root: &Path, variant: Option<Uuid>) -> Result<Vec<NativePnpRow>> {
    let (project, model) = load_native_project_with_resolved_board_and_model(root)?;
    let component_instances_by_package = component_instances_by_package(&model);
    let variant_population = variant_population(&model, variant)?;
    Ok(resolved_board_components(&project.board.packages)?
        .into_iter()
        .filter_map(|component| {
            let component_instance_uuid = component_instances_by_package
                .get(&component.uuid)
                .map(|entry| entry.0);
            let role = component_instances_by_package
                .get(&component.uuid)
                .and_then(|entry| entry.1.as_ref());
            let row = component_to_pnp_row(component, component_instance_uuid, role);
            fitted_for_variant(&row, variant_population).then_some(row)
        })
        .collect())
}

fn native_inventory_panel_pnp_rows(
    root: &Path,
    variant: Option<Uuid>,
    panel_projection: &PanelProjection,
) -> Result<Vec<NativePnpRow>> {
    let (project, model) = load_native_project_with_resolved_board_and_model(root)?;
    let component_instances_by_package = component_instances_by_package(&model);
    let variant_population = variant_population(&model, variant)?;
    let base_rows = resolved_board_components(&project.board.packages)?
        .into_iter()
        .filter_map(|component| {
            let component_instance_uuid = component_instances_by_package
                .get(&component.uuid)
                .map(|entry| entry.0);
            let role = component_instances_by_package
                .get(&component.uuid)
                .and_then(|entry| entry.1.as_ref());
            let row = component_to_pnp_row(component, component_instance_uuid, role);
            fitted_for_variant(&row, variant_population).then_some(row)
        })
        .collect::<Vec<_>>();
    let instances = panel_projection
        .board_instances
        .iter()
        .filter(|instance| instance.board == project.board.uuid)
        .collect::<Vec<_>>();
    if instances.is_empty() {
        bail!(
            "panel projection {} does not reference board {}",
            panel_projection.id,
            project.board.uuid
        );
    }
    let mut rows = Vec::new();
    for instance in instances {
        if instance.rotation_deg != 0 {
            bail!(
                "panel projection {} has board instance rotation {}; panel PnP export currently supports translation-only instances",
                panel_projection.id,
                instance.rotation_deg
            );
        }
        for row in &base_rows {
            let mut row = row.clone();
            row.x_nm += instance.x_nm;
            row.y_nm += instance.y_nm;
            rows.push(row);
        }
    }
    Ok(rows)
}

fn variant_population(
    model: &DesignModel,
    variant: Option<Uuid>,
) -> Result<Option<&BTreeMap<Uuid, DerivedVariantPopulation>>> {
    match variant {
        Some(variant_id) => model
            .variant_populations
            .get(&variant_id)
            .map(Some)
            .ok_or_else(|| anyhow::anyhow!("variant {} was not found", variant_id)),
        None => Ok(None),
    }
}

fn fitted_for_variant(
    row: &impl NativeVariantRow,
    population: Option<&BTreeMap<Uuid, DerivedVariantPopulation>>,
) -> bool {
    let Some(population) = population else {
        return true;
    };
    if let Some(component_instance_uuid) = row.component_instance_uuid() {
        return population
            .get(&component_instance_uuid)
            .copied()
            .unwrap_or(DerivedVariantPopulation::Applicable)
            == DerivedVariantPopulation::Applicable;
    }
    row.package_uuid()
        .parse::<Uuid>()
        .ok()
        .and_then(|id| population.get(&id).copied())
        .unwrap_or(DerivedVariantPopulation::Applicable)
        == DerivedVariantPopulation::Applicable
}

fn resolved_board_components(
    packages: &BTreeMap<String, serde_json::Value>,
) -> Result<Vec<PlacedPackage>> {
    let mut components = packages
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board component"))
        .collect::<Result<Vec<PlacedPackage>>>()?;
    components.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(components)
}

fn bom_rows_by_identity(rows: &[NativeBomRow]) -> (BTreeMap<String, NativeBomRow>, Vec<String>) {
    let mut by_identity = BTreeMap::new();
    let mut duplicates = Vec::new();
    for row in rows {
        let identity = row.identity_key();
        if by_identity.contains_key(&identity) {
            duplicates.push(identity);
        } else {
            by_identity.insert(identity, row.clone());
        }
    }
    (by_identity, duplicates)
}

fn pnp_rows_by_identity(rows: &[NativePnpRow]) -> (BTreeMap<String, NativePnpRow>, Vec<String>) {
    let mut by_identity = BTreeMap::new();
    let mut duplicates = Vec::new();
    for row in rows {
        let identity = row.identity_key();
        if by_identity.contains_key(&identity) {
            duplicates.push(identity);
        } else {
            by_identity.insert(identity, row.clone());
        }
    }
    (by_identity, duplicates)
}

// Phase 5: exec-layer dissolution — variant run() impls (the former
// command_exec destructure-and-forward glue, now inherent methods on the
// clap args structs).

impl ExportBomArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, out, variant } = self;
        let report = export_native_project_bom(&path, &out, variant)?;
        let output = render_report(format, &report, render_native_project_bom_export_text);
        Ok((output, 0))
    }
}

impl CompareBomArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, bom, variant } = self;
        let report = compare_native_project_bom(&path, &bom, variant)?;
        let output = render_report(format, &report, render_native_project_bom_comparison_text);
        Ok((output, 0))
    }
}

impl ValidateBomArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, bom, variant } = self;
        let report = validate_native_project_bom(&path, &bom, variant)?;
        let output = render_report(format, &report, render_native_project_bom_validation_text);
        let exit_code = if report.matches_expected { 0 } else { 1 };
        Ok((output, exit_code))
    }
}

impl InspectBomArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path } = self;
        let report = inspect_native_project_bom(&path)?;
        let output = render_report(format, &report, render_native_project_bom_inspection_text);
        Ok((output, 0))
    }
}

impl ExportPnpArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, out, variant } = self;
        let report = export_native_project_pnp(&path, &out, variant)?;
        let output = render_report(format, &report, render_native_project_pnp_export_text);
        Ok((output, 0))
    }
}

impl ComparePnpArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, pnp, variant } = self;
        let report = compare_native_project_pnp(&path, &pnp, variant)?;
        let output = render_report(format, &report, render_native_project_pnp_comparison_text);
        Ok((output, 0))
    }
}

impl ValidatePnpArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, pnp, variant } = self;
        let report = validate_native_project_pnp(&path, &pnp, variant)?;
        let output = render_report(format, &report, render_native_project_pnp_validation_text);
        let exit_code = if report.matches_expected { 0 } else { 1 };
        Ok((output, exit_code))
    }
}

impl InspectPnpArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path } = self;
        let report = inspect_native_project_pnp(&path)?;
        let output = render_report(format, &report, render_native_project_pnp_inspection_text);
        Ok((output, 0))
    }
}
