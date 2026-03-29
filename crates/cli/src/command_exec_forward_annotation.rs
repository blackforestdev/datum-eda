use super::*;

pub(super) fn execute_forward_annotation_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::ApplyForwardAnnotationAction(ProjectApplyForwardAnnotationActionArgs {
            path,
            action_id,
            package_uuid,
            part_uuid,
            x_nm,
            y_nm,
            layer,
        }) => {
            let report = apply_native_project_forward_annotation_action(
                &path,
                &action_id,
                package_uuid,
                part_uuid,
                x_nm,
                y_nm,
                layer,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_forward_annotation_apply_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ApplyForwardAnnotationReviewed(ProjectApplyForwardAnnotationReviewedArgs { path }) => {
            let report = apply_native_project_forward_annotation_reviewed(&path)?;
            let output = match format {
                OutputFormat::Text => render_native_forward_annotation_batch_apply_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportForwardAnnotationProposal(ProjectExportForwardAnnotationProposalArgs { path, out }) => {
            let report = export_native_project_forward_annotation_proposal(&path, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_forward_annotation_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ExportForwardAnnotationProposalSelection(ProjectExportForwardAnnotationProposalSelectionArgs {
            path,
            action_ids,
            out,
        }) => {
            let report = export_native_project_forward_annotation_proposal_selection(
                &path,
                &action_ids,
                &out,
            )?;
            let output = match format {
                OutputFormat::Text => render_native_forward_annotation_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::SelectForwardAnnotationProposalArtifact(ProjectSelectForwardAnnotationProposalArtifactArgs {
            artifact,
            action_ids,
            out,
        }) => {
            let report = select_forward_annotation_proposal_artifact(&artifact, &action_ids, &out)?;
            let output = match format {
                OutputFormat::Text => render_native_forward_annotation_export_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::InspectForwardAnnotationProposalArtifact(ProjectInspectForwardAnnotationProposalArtifactArgs { path }) => {
            let report = inspect_forward_annotation_proposal_artifact(&path)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_forward_annotation_artifact_inspection_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ValidateForwardAnnotationProposalArtifact(ProjectValidateForwardAnnotationProposalArtifactArgs { path }) => {
            let report = validate_forward_annotation_proposal_artifact(&path)?;
            let exit_code = if report.matches_expected { 0 } else { 1 };
            let output = match format {
                OutputFormat::Text => {
                    render_native_forward_annotation_artifact_validation_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, exit_code))
        }
        ProjectCommands::CompareForwardAnnotationProposalArtifact(ProjectCompareForwardAnnotationProposalArtifactArgs { path, artifact }) => {
            let report = compare_forward_annotation_proposal_artifact(&path, &artifact)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_forward_annotation_artifact_comparison_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::FilterForwardAnnotationProposalArtifact(ProjectFilterForwardAnnotationProposalArtifactArgs {
            path,
            artifact,
            out,
        }) => {
            let report = filter_forward_annotation_proposal_artifact(&path, &artifact, &out)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_forward_annotation_artifact_filter_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::PlanForwardAnnotationProposalArtifactApply(ProjectPlanForwardAnnotationProposalArtifactApplyArgs { path, artifact }) => {
            let report = plan_forward_annotation_proposal_artifact_apply(&path, &artifact)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_forward_annotation_artifact_apply_plan_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ApplyForwardAnnotationProposalArtifact(ProjectApplyForwardAnnotationProposalArtifactArgs { path, artifact }) => {
            let report = apply_forward_annotation_proposal_artifact(&path, &artifact)?;
            let output = match format {
                OutputFormat::Text => render_native_forward_annotation_artifact_apply_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ImportForwardAnnotationArtifactReview(ProjectImportForwardAnnotationArtifactReviewArgs { path, artifact }) => {
            let report = import_forward_annotation_artifact_review(&path, &artifact)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_forward_annotation_artifact_review_import_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ReplaceForwardAnnotationArtifactReview(ProjectReplaceForwardAnnotationArtifactReviewArgs { path, artifact }) => {
            let report = replace_forward_annotation_artifact_review(&path, &artifact)?;
            let output = match format {
                OutputFormat::Text => {
                    render_native_forward_annotation_artifact_review_replace_text(&report)
                }
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::DeferForwardAnnotationAction(ProjectDeferForwardAnnotationActionArgs { path, action_id }) => {
            let report =
                record_native_project_forward_annotation_review(&path, &action_id, "deferred")?;
            let output = match format {
                OutputFormat::Text => render_native_forward_annotation_review_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::RejectForwardAnnotationAction(ProjectRejectForwardAnnotationActionArgs { path, action_id }) => {
            let report =
                record_native_project_forward_annotation_review(&path, &action_id, "rejected")?;
            let output = match format {
                OutputFormat::Text => render_native_forward_annotation_review_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        ProjectCommands::ClearForwardAnnotationActionReview(ProjectClearForwardAnnotationActionReviewArgs { path, action_id }) => {
            let report = clear_native_project_forward_annotation_review(&path, &action_id)?;
            let output = match format {
                OutputFormat::Text => render_native_forward_annotation_review_report_text(&report),
                OutputFormat::Json => render_output(format, &report),
            };
            Ok((output, 0))
        }
        _ => unreachable!("non-forward-annotation command passed to dispatcher"),
    }
}
