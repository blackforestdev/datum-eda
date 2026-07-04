use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::api::native_write::board_components::{
    BoardPackagePlacement, build_place_board_packages, derive_board_package_from_symbol_id,
};
use eda_engine::api::native_write::{PreparedWrite, WriteProvenance, commit_prepared};
use eda_engine::board::PlacedPackage;
use eda_engine::ir::geometry::Point;
use eda_engine::pool::{Footprint, Part};
use eda_engine::schematic::PlacedSymbol;
use eda_engine::substrate::{
    DesignModel, Proposal, ProposalCreateRequest, ProposalSource, ResolveDiagnostic,
    SourceShardTaxon, create_draft_proposal_from_batch,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    board_package_materialization_payload_for_component, build_native_project_schematic,
    load_native_project_with_resolved_board_and_model,
};

use crate::cli_commit_source;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardHandoffReport {
    pub(crate) contract: &'static str,
    pub(crate) applied: bool,
    pub(crate) proposed: bool,
    pub(crate) proposal_id: Option<Uuid>,
    pub(crate) proposal: Option<Proposal>,
    pub(crate) generated_count: usize,
    pub(crate) skipped_count: usize,
    pub(crate) unresolved_count: usize,
    pub(crate) generated_packages: Vec<NativeProjectBoardGeneratedPackage>,
    pub(crate) skipped_symbols: Vec<NativeProjectBoardSkippedSymbol>,
    pub(crate) unresolved_symbols: Vec<NativeProjectBoardUnresolvedSymbol>,
    pub(crate) relationship_diagnostics: Vec<ResolveDiagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardGeneratedPackage {
    pub(crate) symbol_uuid: Uuid,
    pub(crate) package_uuid: Uuid,
    pub(crate) part_uuid: Uuid,
    pub(crate) package_ref_uuid: Uuid,
    pub(crate) default_footprint_uuid: Option<Uuid>,
    pub(crate) reference: String,
    pub(crate) value: String,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
    pub(crate) layer: i32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardSkippedSymbol {
    pub(crate) symbol_uuid: Uuid,
    pub(crate) reference: String,
    pub(crate) part_uuid: Option<Uuid>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardUnresolvedSymbol {
    pub(crate) symbol_uuid: Uuid,
    pub(crate) reference: String,
    pub(crate) part_uuid: Option<Uuid>,
    pub(crate) reason: String,
}

pub(crate) fn generate_native_project_board_components(
    root: &Path,
    apply: bool,
    as_proposal: bool,
    proposal_id: Option<Uuid>,
    rationale: Option<String>,
    origin: Point,
    pitch_nm: i64,
    layer: i32,
) -> Result<NativeProjectBoardHandoffReport> {
    if apply && as_proposal {
        anyhow::bail!("--apply and --as-proposal are mutually exclusive");
    }
    let (project, mut model) = load_native_project_with_resolved_board_and_model(root)?;
    let schematic = build_native_project_schematic(&project)?;
    let existing_join_keys = existing_board_join_keys(&project.board.packages)?;
    let relationship_diagnostics = component_instance_diagnostics(&model.diagnostics);
    let mut generated = Vec::new();
    let mut skipped = Vec::new();
    let mut unresolved = Vec::new();

    let mut symbols = schematic
        .sheets
        .values()
        .flat_map(|sheet| sheet.symbols.values().cloned())
        .collect::<Vec<_>>();
    symbols.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.uuid.cmp(&b.uuid))
    });

    for symbol in symbols {
        let Some(part_uuid) = symbol.part else {
            skipped.push(NativeProjectBoardSkippedSymbol {
                symbol_uuid: symbol.uuid,
                reference: symbol.reference,
                part_uuid: None,
                reason: "schematic symbol has no bound part".to_string(),
            });
            continue;
        };
        let join_key = (symbol.reference.clone(), part_uuid);
        if existing_join_keys.contains(&join_key) {
            skipped.push(NativeProjectBoardSkippedSymbol {
                symbol_uuid: symbol.uuid,
                reference: symbol.reference,
                part_uuid: Some(part_uuid),
                reason: "matching board package already exists".to_string(),
            });
            continue;
        }
        let Some(part) = resolve_pool_part(&model, part_uuid)? else {
            unresolved.push(NativeProjectBoardUnresolvedSymbol {
                symbol_uuid: symbol.uuid,
                reference: symbol.reference,
                part_uuid: Some(part_uuid),
                reason: "bound part was not found in resolved project pools".to_string(),
            });
            continue;
        };
        let (package_ref_uuid, default_footprint_uuid) =
            package_ref_for_part(&model, &part)?.unwrap_or((part.package, None));
        let index = generated.len() as i64;
        let component = generated_component(
            &model.project.project_id,
            &symbol,
            part_uuid,
            package_ref_uuid,
            Point {
                x: origin.x + pitch_nm * index,
                y: origin.y,
            },
            layer,
        );
        generated.push(NativeProjectBoardGeneratedPackage {
            symbol_uuid: symbol.uuid,
            package_uuid: component.uuid,
            part_uuid,
            package_ref_uuid,
            default_footprint_uuid,
            reference: component.reference,
            value: component.value,
            x_nm: component.position.x,
            y_nm: component.position.y,
            layer: component.layer,
        });
    }

    let proposal = if (apply || as_proposal) && !generated.is_empty() {
        let prepared = generated_board_component_prepared_write(
            root,
            &model,
            &generated,
            "generate board components from schematic",
        )?;
        if apply {
            commit_prepared(&mut model, root, prepared)?;
            None
        } else {
            Some(create_draft_proposal_from_batch(
                &mut model,
                root,
                ProposalCreateRequest {
                    proposal_id,
                    batch: prepared.batch,
                    rationale: rationale.unwrap_or_else(|| {
                        "Review generated board components from schematic".to_string()
                    }),
                    source: ProposalSource::Cli,
                    checks_run: Vec::new(),
                    finding_fingerprints: Vec::new(),
                },
            )?)
        }
    } else {
        None
    };
    let created_proposal_id = proposal.as_ref().map(|proposal| proposal.proposal_id);

    Ok(NativeProjectBoardHandoffReport {
        contract: "native_project_board_handoff_v1",
        applied: apply,
        proposed: created_proposal_id.is_some(),
        proposal_id: created_proposal_id,
        proposal,
        generated_count: generated.len(),
        skipped_count: skipped.len(),
        unresolved_count: unresolved.len(),
        generated_packages: generated,
        skipped_symbols: skipped,
        unresolved_symbols: unresolved,
        relationship_diagnostics,
    })
}

fn generated_board_component_prepared_write(
    root: &Path,
    model: &DesignModel,
    generated: &[NativeProjectBoardGeneratedPackage],
    reason: &str,
) -> Result<PreparedWrite> {
    let placements = generated
        .iter()
        .map(|entry| {
            let component = PlacedPackage {
                uuid: entry.package_uuid,
                part: entry.part_uuid,
                package: entry.package_ref_uuid,
                reference: entry.reference.clone(),
                value: entry.value.clone(),
                position: Point {
                    x: entry.x_nm,
                    y: entry.y_nm,
                },
                rotation: 0,
                layer: entry.layer,
                locked: false,
            };
            let materialized =
                board_package_materialization_payload_for_component(root, &component)?;
            Ok(BoardPackagePlacement {
                package: component,
                materialized,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(build_place_board_packages(
        model,
        WriteProvenance::new("datum-eda-cli", cli_commit_source()?, reason),
        &placements,
    )?)
}

pub(crate) fn render_native_project_board_handoff_text(
    report: &NativeProjectBoardHandoffReport,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("contract: {}", report.contract));
    lines.push(format!("applied: {}", report.applied));
    lines.push(format!("proposed: {}", report.proposed));
    if let Some(proposal_id) = report.proposal_id {
        lines.push(format!("proposal: {proposal_id}"));
    }
    lines.push(format!("generated: {}", report.generated_count));
    lines.push(format!("skipped: {}", report.skipped_count));
    lines.push(format!("unresolved: {}", report.unresolved_count));
    for package in &report.generated_packages {
        lines.push(format!(
            "package {} {} part={} package={} at {},{} layer={}",
            package.package_uuid,
            package.reference,
            package.part_uuid,
            package.package_ref_uuid,
            package.x_nm,
            package.y_nm,
            package.layer
        ));
    }
    for diagnostic in &report.relationship_diagnostics {
        lines.push(format!(
            "relationship_diagnostic {}: {}",
            diagnostic.code, diagnostic.message
        ));
    }
    lines.join("\n")
}

fn existing_board_join_keys(
    packages: &BTreeMap<String, serde_json::Value>,
) -> Result<BTreeSet<(String, Uuid)>> {
    let mut keys = BTreeSet::new();
    for value in packages.values() {
        let component: PlacedPackage = serde_json::from_value(value.clone())
            .context("failed to parse native board package")?;
        keys.insert((component.reference, component.part));
    }
    Ok(keys)
}

fn component_instance_diagnostics(diagnostics: &[ResolveDiagnostic]) -> Vec<ResolveDiagnostic> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.starts_with("component_instance_"))
        .cloned()
        .collect()
}

fn generated_component(
    project_id: &Uuid,
    symbol: &PlacedSymbol,
    part_uuid: Uuid,
    package_ref_uuid: Uuid,
    position: Point,
    layer: i32,
) -> PlacedPackage {
    let uuid = derive_board_package_from_symbol_id(project_id, symbol.uuid);
    PlacedPackage {
        uuid,
        part: part_uuid,
        package: package_ref_uuid,
        reference: symbol.reference.clone(),
        value: symbol.value.clone(),
        position,
        rotation: 0,
        layer,
        locked: false,
    }
}

fn package_ref_for_part(model: &DesignModel, part: &Part) -> Result<Option<(Uuid, Option<Uuid>)>> {
    let Some(default_footprint_uuid) = part.default_footprint else {
        return Ok(Some((part.package, None)));
    };
    let Some(footprint) = resolve_pool_footprint(model, default_footprint_uuid)? else {
        return Ok(Some((part.package, None)));
    };
    Ok(Some((footprint.package, Some(default_footprint_uuid))))
}

fn resolve_pool_part(model: &DesignModel, part_uuid: Uuid) -> Result<Option<Part>> {
    let relative_path = pool_object_relative_path(model, SourceShardTaxon::PoolPart, part_uuid);
    let Some(relative_path) = relative_path else {
        return Ok(None);
    };
    let value = model
        .materialized_source_shard_value_by_relative_path(&relative_path)
        .with_context(|| format!("failed to materialize {relative_path}"))?;
    let part = serde_json::from_value(value)
        .with_context(|| format!("failed to parse materialized {relative_path}"))?;
    Ok(Some(part))
}

fn resolve_pool_footprint(model: &DesignModel, footprint_uuid: Uuid) -> Result<Option<Footprint>> {
    let relative_path =
        pool_object_relative_path(model, SourceShardTaxon::PoolFootprint, footprint_uuid);
    let Some(relative_path) = relative_path else {
        return Ok(None);
    };
    let value = model
        .materialized_source_shard_value_by_relative_path(&relative_path)
        .with_context(|| format!("failed to materialize {relative_path}"))?;
    let footprint = serde_json::from_value(value)
        .with_context(|| format!("failed to parse materialized {relative_path}"))?;
    Ok(Some(footprint))
}

fn pool_object_relative_path(
    model: &DesignModel,
    taxon: SourceShardTaxon,
    object_uuid: Uuid,
) -> Option<String> {
    let suffix = format!("{object_uuid}.json");
    model
        .source_shards
        .iter()
        .filter(|shard| shard.taxon == Some(taxon))
        .filter(|shard| shard.relative_path.ends_with(&suffix))
        .map(|shard| shard.relative_path.clone())
        .min()
}
