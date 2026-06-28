use std::path::Path;

use crate::{OutputFormat, render_output};
use anyhow::{Result, anyhow};
use eda_engine::api::CheckReport;
use eda_engine::substrate::*;
use serde::Serialize;
use uuid::Uuid;

#[path = "command_project_artifact_checks.rs"]
mod command_project_artifact_checks;
#[path = "command_project_check_finding_identity.rs"]
mod command_project_check_finding_identity;
#[path = "command_project_check_proposal_refs.rs"]
mod command_project_check_proposal_refs;
#[path = "command_project_check_run_history.rs"]
mod command_project_check_run_history;
#[path = "command_project_check_run_view.rs"]
mod command_project_check_run_view;
#[path = "command_project_check_targets.rs"]
mod command_project_check_targets;

use self::command_project_artifact_checks::*;
pub(crate) use self::command_project_check_run_history::query_native_project_check_run_list;
pub(crate) use self::command_project_check_run_view::{
    NativeProjectCheckFindingView, NativeProjectCheckProposalCommandTemplates,
    NativeProjectCheckProposalLinkView, NativeProjectCheckRunView, append_finding_values,
    apply_accepted_deviations, apply_fingerprint_waivers, check_profile_drc_rules,
    check_profile_includes_artifacts, check_profile_includes_erc,
    check_profile_includes_relationships, check_profile_includes_zone_fills,
    check_run_coverage_for_profile, filter_check_run_findings_for_profile,
    native_check_run_to_substrate, profile_basis_for_check_run,
    query_native_project_check_profiles, resolve_native_project_check_profile,
    summarize_check_run_findings,
};
use super::command_project_schematic_queries::query_native_project_check_with_inputs;
use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectResolveDebugView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) project_name: String,
    pub(crate) model_revision: String,
    pub(crate) source_shards: Vec<NativeProjectResolveDebugShardView>,
    pub(crate) object_count: usize,
    pub(crate) component_instance_count: usize,
    pub(crate) relationship_count: usize,
    pub(crate) relationship_status_count: usize,
    pub(crate) variant_count: usize,
    pub(crate) variant_population_count: usize,
    pub(crate) zone_fill_count: usize,
    pub(crate) output_job_count: usize,
    pub(crate) check_run_count: usize,
    pub(crate) proposal_count: usize,
    pub(crate) import_map_count: usize,
    pub(crate) diagnostics: Vec<NativeProjectResolveDebugDiagnosticView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectResolveDebugShardView {
    pub(crate) kind: String,
    pub(crate) taxon: Option<String>,
    pub(crate) shard_id: String,
    pub(crate) path: String,
    pub(crate) authority: String,
    pub(crate) dirty_state: String,
    pub(crate) schema_version: Option<u64>,
    pub(crate) content_hash: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectResolveDebugDiagnosticView {
    pub(crate) code: String,
    pub(crate) message: String,
    pub(crate) path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectCheckRunRecordView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) check_run: CheckRun,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectOperationBatchCommitDebugView {
    pub(crate) contract: &'static str,
    pub(crate) mode: &'static str,
    pub(crate) status: &'static str,
    pub(crate) project_id: String,
    pub(crate) before_model_revision: String,
    pub(crate) after_model_revision: String,
    pub(crate) journal_len: usize,
    pub(crate) transaction: eda_engine::substrate::TransactionRecord,
    pub(crate) write_boundary: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectJournalListView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) journal_path: &'static str,
    pub(crate) cursor_path: &'static str,
    pub(crate) count: usize,
    pub(crate) cursor_index: usize,
    pub(crate) can_undo: bool,
    pub(crate) can_redo: bool,
    pub(crate) transactions: Vec<NativeProjectJournalSummaryView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectJournalSummaryView {
    pub(crate) transaction_id: Uuid,
    pub(crate) batch_id: Uuid,
    pub(crate) before_model_revision: String,
    pub(crate) after_model_revision: String,
    pub(crate) actor: String,
    pub(crate) source: eda_engine::substrate::CommitSource,
    pub(crate) reason: String,
    pub(crate) created: usize,
    pub(crate) modified: usize,
    pub(crate) deleted: usize,
    pub(crate) operations: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectJournalRecordView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) journal_path: &'static str,
    pub(crate) index: usize,
    pub(crate) transaction: eda_engine::substrate::TransactionRecord,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectJournalMutationView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) status: &'static str,
    pub(crate) project_id: String,
    pub(crate) before_model_revision: String,
    pub(crate) after_model_revision: String,
    pub(crate) journal_path: &'static str,
    pub(crate) cursor_path: &'static str,
    pub(crate) journal_len: usize,
    pub(crate) cursor_before: usize,
    pub(crate) cursor_after: usize,
    pub(crate) guard: NativeProjectJournalMutationGuardView,
    pub(crate) can_undo: bool,
    pub(crate) can_redo: bool,
    pub(crate) transaction: eda_engine::substrate::TransactionRecord,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectJournalMutationGuardView {
    pub(crate) checked: bool,
    pub(crate) current_model_revision: String,
    pub(crate) expected_model_revision: Option<String>,
    pub(crate) current_tip_transaction: Option<Uuid>,
    pub(crate) expected_tip_transaction: Option<Uuid>,
}

pub(crate) fn inspect_native_project(root: &Path) -> Result<NativeProjectInspectReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let pool_refs = collect_native_project_pool_ref_views(&project);

    Ok(NativeProjectInspectReportView {
        project_root: project.root.display().to_string(),
        project_name: project.manifest.name.clone(),
        schema_version: project.manifest.schema_version,
        project_uuid: project.manifest.uuid.to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
        board_uuid: project.board.uuid.to_string(),
        pools: project.manifest.pools.len(),
        pool_refs,
        schematic_path: project.schematic_path.display().to_string(),
        board_path: project.board_path.display().to_string(),
        rules_path: project.rules_path.display().to_string(),
        sheet_count: project.schematic.sheets.len(),
        sheet_definition_count: project.schematic.definitions.len(),
        sheet_instance_count: project.schematic.instances.len(),
        variant_count: project.schematic.variants.len(),
        board_package_count: project.board.packages.len(),
        board_components_with_persisted_silkscreen: project
            .board
            .packages
            .keys()
            .filter(|key| component_has_persisted_silkscreen(&project, key))
            .count(),
        board_components_with_persisted_mechanical: project
            .board
            .packages
            .keys()
            .filter(|key| component_has_persisted_mechanical(&project, key))
            .count(),
        board_components_with_persisted_pads: project
            .board
            .packages
            .keys()
            .filter(|key| component_package_pad_count(&project, key) > 0)
            .count(),
        board_components_with_persisted_models_3d: project
            .board
            .packages
            .keys()
            .filter(|key| component_model_count(&project, key) > 0)
            .count(),
        board_pad_count: project.board.pads.len(),
        board_net_count: project.board.nets.len(),
        board_track_count: project.board.tracks.len(),
        board_via_count: project.board.vias.len(),
        board_zone_count: project.board.zones.len(),
        persisted_component_silkscreen_texts: project
            .board
            .component_silkscreen_texts
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_silkscreen_lines: project
            .board
            .component_silkscreen
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_silkscreen_arcs: project
            .board
            .component_silkscreen_arcs
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_silkscreen_circles: project
            .board
            .component_silkscreen_circles
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_silkscreen_polygons: project
            .board
            .component_silkscreen_polygons
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_silkscreen_polylines: project
            .board
            .component_silkscreen_polylines
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_mechanical_texts: project
            .board
            .component_mechanical_texts
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_mechanical_lines: project
            .board
            .component_mechanical_lines
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_mechanical_arcs: project
            .board
            .component_mechanical_arcs
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_mechanical_circles: project
            .board
            .component_mechanical_circles
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_mechanical_polygons: project
            .board
            .component_mechanical_polygons
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_mechanical_polylines: project
            .board
            .component_mechanical_polylines
            .values()
            .map(Vec::len)
            .sum(),
        persisted_component_pads: project.board.component_pads.values().map(Vec::len).sum(),
        persisted_component_models_3d: project
            .board
            .component_models_3d
            .values()
            .map(Vec::len)
            .sum(),
        rule_count: project.rules.rules.len(),
    })
}

pub(crate) fn query_native_project_resolve_debug(
    root: &Path,
) -> Result<NativeProjectResolveDebugView> {
    let model = ProjectResolver::new(root).resolve()?;
    Ok(resolve_debug_view(model))
}

pub(crate) fn execute_native_project_resolve_debug_query(
    format: &OutputFormat,
    root: &Path,
    commit_batch: Option<&Path>,
    apply: bool,
) -> Result<(String, i32)> {
    if let Some(batch_path) = commit_batch {
        let report = query_native_project_operation_batch_commit_debug(root, batch_path, apply)?;
        return Ok((render_output(format, &report), 0));
    }
    let report = query_native_project_resolve_debug(root)?;
    Ok((render_output(format, &report), 0))
}

pub(crate) fn query_native_project_operation_batch_commit_debug(
    root: &Path,
    batch_path: &Path,
    apply: bool,
) -> Result<NativeProjectOperationBatchCommitDebugView> {
    let batch: OperationBatch = serde_json::from_slice(&std::fs::read(batch_path)?)?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let project_id = model.project.project_id.to_string();
    let CommitReport {
        transaction,
        journal_len,
    } = if apply {
        model.commit_journaled(root, batch)?
    } else {
        model.commit(batch)?
    };
    let write_boundary = if apply
        && transaction
            .operations
            .iter()
            .any(eda_engine::substrate::Operation::writes_project_shard)
    {
        "journal_and_project_shards_written"
    } else if apply {
        "journal_only_no_project_shards_written"
    } else {
        "in_memory_only_no_project_shards_written"
    };
    Ok(NativeProjectOperationBatchCommitDebugView {
        contract: "operation_batch_commit_debug_v1",
        mode: if apply { "journal_apply" } else { "dry_run" },
        status: "accepted",
        project_id,
        before_model_revision: transaction.before_model_revision.0.clone(),
        after_model_revision: transaction.after_model_revision.0.clone(),
        journal_len,
        transaction,
        write_boundary,
    })
}

pub(crate) fn query_native_project_journal_list(
    root: &Path,
) -> Result<NativeProjectJournalListView> {
    let model = ProjectResolver::new(root).resolve()?;
    Ok(journal_list_view(model))
}

pub(crate) fn query_native_project_journal_show(
    root: &Path,
    transaction_id: Uuid,
) -> Result<NativeProjectJournalRecordView> {
    let model = ProjectResolver::new(root).resolve()?;
    let project_id = model.project.project_id.to_string();
    let model_revision = model.model_revision.0.clone();
    let (index, transaction) = model
        .journal
        .into_iter()
        .enumerate()
        .find(|(_, transaction)| transaction.transaction_id == transaction_id)
        .ok_or_else(|| anyhow!("transaction {transaction_id} not found in project journal"))?;

    Ok(NativeProjectJournalRecordView {
        contract: "project_transaction_journal_record_v1",
        project_id,
        model_revision,
        journal_path: eda_engine::substrate::JOURNAL_RELATIVE_PATH,
        index,
        transaction,
    })
}

pub(crate) fn query_native_project_check_run(root: &Path) -> Result<NativeProjectCheckRunView> {
    query_native_project_check_run_with_profile_and_persistence(root, None, false)
}

pub(crate) fn query_native_project_check_run_with_profile(
    root: &Path,
    profile: Option<&str>,
) -> Result<NativeProjectCheckRunView> {
    query_native_project_check_run_with_profile_and_persistence(root, profile, false)
}

pub(crate) fn run_native_project_check_with_profile(
    root: &Path,
    profile: Option<&str>,
) -> Result<NativeProjectCheckRunView> {
    query_native_project_check_run_with_profile_and_persistence(root, profile, true)
}

fn query_native_project_check_run_with_profile_and_persistence(
    root: &Path,
    profile: Option<&str>,
    persist: bool,
) -> Result<NativeProjectCheckRunView> {
    let profile_id = resolve_native_project_check_profile(profile)?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let project = load_native_project_with_resolved_board(root)?;
    let report = query_native_project_check_with_inputs(
        root,
        check_profile_includes_relationships(profile_id),
        check_profile_includes_erc(profile_id),
        check_profile_drc_rules(profile_id),
    )?;
    let mut findings = check_run_findings(
        &model.project.project_id,
        &model.model_revision.0,
        &report,
        &model.artifact_metadata,
        &model.zone_fills,
        check_profile_includes_artifacts(profile_id),
        check_profile_includes_zone_fills(profile_id),
    )?;
    filter_check_run_findings_for_profile(profile_id, &mut findings);
    let waivers = project
        .schematic
        .waivers
        .iter()
        .filter_map(|value| serde_json::from_value(value.clone()).ok())
        .collect::<Vec<_>>();
    apply_fingerprint_waivers(&mut findings, &waivers);
    let deviations = project
        .schematic
        .deviations
        .iter()
        .filter_map(|value| serde_json::from_value(value.clone()).ok())
        .collect::<Vec<_>>();
    apply_accepted_deviations(&mut findings, &deviations);
    let proposal_links =
        command_project_check_proposal_refs::apply_proposal_links(root, &mut findings, &model);
    let proposal_refs = proposal_links
        .iter()
        .map(|link| link.proposal_id.clone())
        .collect::<Vec<_>>();
    let summary = summarize_check_run_findings(&findings);
    let profile_basis = profile_basis_for_check_run(profile_id);
    let coverage = check_run_coverage_for_profile(profile_id);
    let run_material = format!(
        "datum-eda:check-run:{}:{}:{}",
        model.project.project_id, model.model_revision.0, profile_id
    );
    let view = NativeProjectCheckRunView {
        contract: "check_run_v1",
        persisted: persist,
        check_run_id: Uuid::new_v5(&model.project.project_id, run_material.as_bytes()),
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0.clone(),
        profile_id,
        status: summary.status,
        summary,
        finding_count: findings.len(),
        findings,
        proposal_refs,
        proposal_links,
        profile_basis,
        coverage,
        raw_report: report,
    };
    if persist {
        commit_check_run_evidence(root, &mut model, &view)?;
    }
    Ok(view)
}

fn commit_check_run_evidence(
    root: &Path,
    model: &mut DesignModel,
    view: &NativeProjectCheckRunView,
) -> Result<()> {
    let check_run = native_check_run_to_substrate(&model.project.project_id, view)?;
    if model.check_runs.contains_key(&check_run.check_run_id) {
        return Ok(());
    }
    let previous_check_run = model
        .check_runs
        .get(&check_run.check_run_id)
        .map(|run| serde_json::to_value(run).expect("check run serialization must succeed"));
    model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: format!("record {} check run evidence", view.profile_id),
            },
            operations: vec![Operation::SetCheckRun {
                check_run_id: check_run.check_run_id,
                previous_check_run,
                check_run: serde_json::to_value(&check_run)
                    .expect("check run serialization must succeed"),
            }],
        },
    )?;
    Ok(())
}

pub(crate) fn query_native_project_check_run_show(
    root: &Path,
    check_run_id: Uuid,
) -> Result<NativeProjectCheckRunRecordView> {
    let model = ProjectResolver::new(root).resolve()?;
    let mut check_run = model
        .check_runs
        .get(&check_run_id)
        .cloned()
        .ok_or_else(|| anyhow!("check run not found: {check_run_id}"))?;
    command_project_check_proposal_refs::apply_proposal_links_to_persisted_check_run(
        root,
        &mut check_run,
        &model,
    );
    Ok(NativeProjectCheckRunRecordView {
        contract: "check_run_record_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        check_run,
    })
}

fn resolve_debug_view(model: DesignModel) -> NativeProjectResolveDebugView {
    NativeProjectResolveDebugView {
        contract: "project_resolver_debug_v1",
        project_id: model.project.project_id.to_string(),
        project_name: model.project.name,
        model_revision: model.model_revision.0,
        source_shards: model
            .source_shards
            .into_iter()
            .map(|shard| NativeProjectResolveDebugShardView {
                kind: format!("{:?}", shard.kind),
                taxon: shard.taxon.map(|taxon| format!("{taxon:?}")),
                shard_id: shard.shard_id.to_string(),
                path: shard.relative_path,
                authority: format!("{:?}", shard.authority),
                dirty_state: format!("{:?}", shard.dirty_state),
                schema_version: shard.schema_version,
                content_hash: shard.content_hash,
            })
            .collect(),
        object_count: model.objects.len(),
        component_instance_count: model
            .component_instances
            .values()
            .filter(|instance| instance.authority == ComponentInstanceAuthority::Authored)
            .count(),
        relationship_count: model.relationships.len(),
        relationship_status_count: model.relationship_statuses.len(),
        variant_count: model.variants.len(),
        variant_population_count: model.variant_populations.len(),
        zone_fill_count: model.zone_fills.len(),
        output_job_count: model.output_jobs.len(),
        check_run_count: model.check_runs.len(),
        proposal_count: model.proposals.len(),
        import_map_count: model.import_map.len(),
        diagnostics: model
            .diagnostics
            .into_iter()
            .map(|diagnostic| NativeProjectResolveDebugDiagnosticView {
                code: diagnostic.code,
                message: diagnostic.message,
                path: diagnostic.path.map(|path| path.display().to_string()),
            })
            .collect(),
    }
}

fn check_run_findings(
    project_id: &Uuid,
    model_revision: &str,
    report: &CheckReport,
    artifact_metadata: &std::collections::BTreeMap<Uuid, ArtifactMetadata>,
    zone_fills: &std::collections::BTreeMap<Uuid, ZoneFill>,
    include_artifacts: bool,
    include_zone_fills: bool,
) -> Result<Vec<NativeProjectCheckFindingView>> {
    let report_value = serde_json::to_value(report)?;
    let mut findings = Vec::new();
    for source in ["diagnostic", "erc", "drc"] {
        append_finding_values(
            project_id,
            model_revision,
            source,
            &report_value,
            &mut findings,
        )?;
    }
    if include_artifacts {
        append_artifact_finding_values(
            project_id,
            model_revision,
            artifact_metadata,
            &mut findings,
        )?;
    }
    if include_zone_fills {
        append_zone_fill_finding_values(project_id, model_revision, zone_fills, &mut findings)?;
    }
    Ok(findings)
}

fn journal_list_view(model: DesignModel) -> NativeProjectJournalListView {
    let count = model.journal.len();
    let cursor_index = model.journal_cursor.applied_transaction_count;
    let (can_undo, can_redo) = journal_tip_availability(&model);
    NativeProjectJournalListView {
        contract: "project_transaction_journal_list_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        journal_path: eda_engine::substrate::JOURNAL_RELATIVE_PATH,
        cursor_path: ".datum/journal/cursor.json",
        count,
        cursor_index,
        can_undo,
        can_redo,
        transactions: model
            .journal
            .into_iter()
            .map(|transaction| NativeProjectJournalSummaryView {
                transaction_id: transaction.transaction_id,
                batch_id: transaction.batch_id,
                before_model_revision: transaction.before_model_revision.0,
                after_model_revision: transaction.after_model_revision.0,
                actor: transaction.provenance.actor,
                source: transaction.provenance.source,
                reason: transaction.provenance.reason,
                created: transaction.diff.created.len(),
                modified: transaction.diff.modified.len(),
                deleted: transaction.diff.deleted.len(),
                operations: transaction.operations.len(),
            })
            .collect(),
    }
}

pub(super) fn journal_tip_availability(model: &DesignModel) -> (bool, bool) {
    let Some(transaction) = model.journal.last() else {
        return (false, false);
    };
    let has_inverse = !transaction.inverse_operations.is_empty();
    (
        has_inverse && transaction.transaction_kind != eda_engine::substrate::TransactionKind::Undo,
        has_inverse && transaction.transaction_kind == eda_engine::substrate::TransactionKind::Undo,
    )
}
