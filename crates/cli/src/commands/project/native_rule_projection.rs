//! Legacy Board rules and model-backed native rules have different readers.
//! Native policy stays in the captured DesignModel, including its u64 priority.
use super::*;

pub(super) fn legacy_board_rules(
    project: &LoadedNativeProject,
    model: Option<&DesignModel>,
) -> Result<Vec<Rule>> {
    let mut legacy = Vec::new();
    let mut native_basis_checked = false;
    for value in &project.rules.rules {
        let kind = value.get("rule_type").and_then(serde_json::Value::as_str);
        // `params` belongs to the native source schema; legacy Rule uses
        // `parameters`. The engine validator owns native kinds and parameters.
        let native = value.get("params").is_some();
        if !native {
            legacy.push(
                serde_json::from_value(value.clone())
                    .context("failed to parse legacy board rule")?,
            );
            continue;
        }

        eda_engine::substrate::validate_native_project_rule_payload(value)?;
        let model = model.ok_or_else(|| {
            anyhow::anyhow!("Authored native Rules require a captured native Project source")
        })?;
        if !native_basis_checked {
            let shard = model
                .source_shards
                .iter()
                .find(|shard| shard.kind == SourceShardKind::RulesRoot)
                .context("Captured native Project has no Rules source")?;
            let source = model.capture_source_shard_value(shard.shard_id)?;
            anyhow::ensure!(
                model.project.project_id == project.manifest.uuid
                    && source["uuid"] == serde_json::to_value(project.rules.uuid)?
                    && source["rules"] == serde_json::to_value(&project.rules.rules)?,
                "Native Rule projection differs from the captured Project source"
            );
            native_basis_checked = true;
        }
        // This is an evaluator boundary, not a conversion to legacy Rule(u32).
        // The native checker reads the exact validated source through `model`.
        // Other kinds need their own actual consumer before this reader admits
        // them; never erase a constraint merely because its AST differs.
        anyhow::ensure!(
            matches!(kind, Some("clearance_copper" | "track_width")),
            "Authored native Rule kind {} has no supported native Board evaluator",
            kind.unwrap_or("unknown")
        );
    }
    Ok(legacy)
}
