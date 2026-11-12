//! Immutable native check inputs, captured before producer acknowledgement.
//! All source-dependent evaluation finishes before acknowledgement; only the
//! owned report and canonical source basis cross that boundary.
use super::*;

/// Complete inputs retained before acknowledging a guarded ERC producer job.
pub(crate) struct CapturedNativeErcInputs {
    pub(crate) project: LoadedNativeProject,
    pub(crate) schematic: Schematic,
    pub(crate) nets: Vec<eda_engine::schematic::SchematicNetInfo>,
}

pub(crate) fn capture_native_erc_inputs(
    root: &Path,
    model: &DesignModel,
) -> Result<CapturedNativeErcInputs> {
    let mut values = BTreeMap::new();
    for shard in &model.source_shards {
        if matches!(
            shard.kind,
            SourceShardKind::ProjectManifest
                | SourceShardKind::SchematicRoot
                | SourceShardKind::SchematicSheet
                | SourceShardKind::SchematicDefinition
                | SourceShardKind::BoardRoot
                | SourceShardKind::RulesRoot
        ) {
            values.insert(
                shard.relative_path.clone(),
                model.capture_source_shard_value(shard.shard_id)?,
            );
        }
    }
    let read = |path: &str| {
        values
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("missing captured source {path}"))
    };
    let project = native_project_from_reader(root, |kind| {
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.kind == kind)
            .ok_or_else(|| anyhow::anyhow!("missing captured source kind {kind:?}"))?;
        read(&shard.relative_path)
    })?;
    let schematic = build_native_project_schematic_with_reader(&project, read)?;
    let nets = eda_engine::connectivity::native_schematic_net_info(model, &schematic)?;
    Ok(CapturedNativeErcInputs {
        project,
        schematic,
        nets,
    })
}

pub(crate) struct CapturedNativeDrcInputs {
    pub(crate) project: LoadedNativeProject,
    pub(crate) report: eda_engine::api::CheckReport,
}

pub(crate) fn capture_native_drc_inputs(
    root: &Path,
    model: &DesignModel,
    rules: &[eda_engine::rules::ast::RuleType],
) -> Result<CapturedNativeDrcInputs> {
    let project =
        native_project_from_reader(
            root,
            |kind| Ok(model.materialized_source_shard_value(kind)?),
        )?;
    let schematic = build_native_project_schematic_with_reader(&project, |path| {
        Ok(model.materialized_source_shard_value_by_relative_path(path)?)
    })?;
    let board = build_native_project_board_with_model(&project, Some(model))?;
    // Native DRC certifies electrical and process sources while evaluating.
    // Do that here, not after acknowledging an immutable producer basis.
    let report = crate::native_drc_report_from_board(&board, model, &schematic, rules);
    Ok(CapturedNativeDrcInputs { project, report })
}

pub(crate) enum CapturedNativeCheckInputs {
    Erc(CapturedNativeErcInputs),
    Drc(CapturedNativeDrcInputs),
}

pub(crate) fn capture_native_check_inputs(
    root: &Path,
    model: &DesignModel,
    profile: &str,
    drc_rules: &[eda_engine::rules::ast::RuleType],
) -> Result<CapturedNativeCheckInputs> {
    match profile {
        "erc" => Ok(CapturedNativeCheckInputs::Erc(capture_native_erc_inputs(
            root, model,
        )?)),
        "drc" => Ok(CapturedNativeCheckInputs::Drc(capture_native_drc_inputs(
            root, model, drc_rules,
        )?)),
        _ => anyhow::bail!("Unsupported guarded check profile {profile}"),
    }
}
