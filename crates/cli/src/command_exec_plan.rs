use super::*;

pub(super) fn execute_plan_command(
    format: &OutputFormat,
    action: PlanCommands,
) -> Result<(String, i32)> {
    match action {
        PlanCommands::ExportScopedReplacementManifest {
            path,
            out,
            policy,
            ref_prefix,
            value,
            package_uuid,
            part_uuid,
            exclude_component,
            override_component,
            libraries,
        } => {
            let policy = match policy {
                ReplacementPolicyArg::Package => ComponentReplacementPolicy::BestCompatiblePackage,
                ReplacementPolicyArg::Part => ComponentReplacementPolicy::BestCompatiblePart,
            };
            let overrides = override_component
                .iter()
                .map(|value| parse_scoped_replacement_override_arg(value))
                .collect::<Result<Vec<_>>>()?;
            let plan = query_scoped_component_replacement_plan(
                &path,
                ScopedComponentReplacementPolicyInput {
                    scope: ComponentReplacementScope {
                        reference_prefix: ref_prefix,
                        value_equals: value,
                        current_package_uuid: package_uuid,
                        current_part_uuid: part_uuid,
                    },
                    policy,
                },
                ScopedComponentReplacementPlanEdit {
                    exclude_component_uuids: exclude_component,
                    overrides,
                },
                &libraries,
            )?;
            let manifest = scoped_replacement_manifest_from_parts(&path, &libraries, plan)?;
            let payload =
                serde_json::to_string_pretty(&manifest).expect("manifest serialization must succeed");
            std::fs::write(&out, payload)
                .with_context(|| format!("failed to write manifest {}", out.display()))?;
            let output = match format {
                OutputFormat::Text => render_scoped_replacement_manifest_export_text(
                    &out,
                    &manifest.kind,
                    manifest.version,
                    manifest.plan.replacements.len(),
                ),
                OutputFormat::Json => render_output(
                    format,
                    &serde_json::json!({
                        "path": out.display().to_string(),
                        "kind": manifest.kind,
                        "version": manifest.version,
                        "replacements": manifest.plan.replacements.len(),
                    }),
                ),
            };
            Ok((output, 0))
        }
        PlanCommands::InspectScopedReplacementManifest { path } => {
            let inspection = inspect_scoped_replacement_manifest(&path)?;
            let output = match format {
                OutputFormat::Text => render_scoped_replacement_manifest_inspection_text(&inspection),
                OutputFormat::Json => render_output(format, &inspection),
            };
            Ok((output, 0))
        }
        PlanCommands::InspectScopedReplacementManifestArtifact { path } => {
            let inspection = inspect_scoped_replacement_manifest_artifact(&path)?;
            let output = match format {
                OutputFormat::Text => {
                    render_scoped_replacement_manifest_artifact_inspection_text(&inspection)
                }
                OutputFormat::Json => render_output(format, &inspection),
            };
            Ok((output, 0))
        }
        PlanCommands::ValidateScopedReplacementManifest { paths } => {
            let summary = validate_scoped_replacement_manifest_inputs_batch(&paths)?;
            let output = match format {
                OutputFormat::Text => render_scoped_replacement_manifest_validation_text(&summary),
                OutputFormat::Json => render_output(format, &summary),
            };
            let exit_code = if summary.manifests_failing == 0 { 0 } else { 1 };
            Ok((output, exit_code))
        }
        PlanCommands::ValidateScopedReplacementManifestArtifact { path } => {
            let report = validate_scoped_replacement_manifest_artifact(&path)?;
            let output = match format {
                OutputFormat::Text => {
                    render_scoped_replacement_manifest_artifact_validation_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            let exit_code = if report.matches_expected { 0 } else { 1 };
            Ok((output, exit_code))
        }
        PlanCommands::CompareScopedReplacementManifestArtifact { path, artifact } => {
            let report = compare_scoped_replacement_manifest_artifact(&path, &artifact)?;
            let output = match format {
                OutputFormat::Text => {
                    render_scoped_replacement_manifest_artifact_comparison_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            let exit_code = if report.matches_artifact { 0 } else { 1 };
            Ok((output, exit_code))
        }
        PlanCommands::UpgradeScopedReplacementManifest {
            path,
            out,
            in_place,
        } => {
            let output_path = match (out, in_place) {
                (Some(out), false) => out,
                (None, true) => path.clone(),
                (Some(_), true) => {
                    bail!(
                        "plan upgrade-scoped-replacement-manifest accepts either --out or --in-place, not both"
                    );
                }
                (None, false) => {
                    bail!(
                        "plan upgrade-scoped-replacement-manifest requires either --out <path> or --in-place"
                    );
                }
            };
            let report = upgrade_scoped_replacement_manifest(&path, &output_path)?;
            let output = match format {
                OutputFormat::Text => render_scoped_replacement_manifest_upgrade_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
    }
}
