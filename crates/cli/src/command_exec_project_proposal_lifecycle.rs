use super::*;

pub(crate) fn execute_project_proposal_lifecycle_command(
    format: &OutputFormat,
    command: ProjectCommands,
) -> Result<(String, i32)> {
    match command {
        ProjectCommands::ReviewProposal(ProjectReviewProposalArgs {
            path,
            proposal,
            status,
        }) => Ok((
            render_output(
                format,
                &review_native_project_proposal(
                    &path,
                    proposal,
                    match status {
                        ProposalReviewStatusArg::Accepted => {
                            eda_engine::substrate::ProposalStatus::Accepted
                        }
                        ProposalReviewStatusArg::Deferred => {
                            eda_engine::substrate::ProposalStatus::Deferred
                        }
                        ProposalReviewStatusArg::Rejected => {
                            eda_engine::substrate::ProposalStatus::Rejected
                        }
                    },
                )?,
            ),
            0,
        )),
        ProjectCommands::ShowProposal(ProjectShowProposalArgs { path, proposal }) => Ok((
            render_output(format, &show_native_project_proposal(&path, proposal)?),
            0,
        )),
        ProjectCommands::ValidateProposal(ProjectValidateProposalArgs { path, proposal }) => Ok((
            render_output(format, &validate_native_project_proposal(&path, proposal)?),
            0,
        )),
        ProjectCommands::DeferProposal(ProjectDeferProposalArgs { path, proposal }) => Ok((
            render_output(format, &defer_native_project_proposal(&path, proposal)?),
            0,
        )),
        ProjectCommands::ApplyProposal(ProjectApplyProposalArgs { path, proposal }) => Ok((
            render_output(format, &apply_native_project_proposal(&path, proposal)?),
            0,
        )),
        _ => unreachable!("unsupported project proposal lifecycle command"),
    }
}
