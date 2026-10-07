//! Existing CLI forwarding for generic library Proposals.
use super::*;
impl ProposalCreatePoolLibraryObjectArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            pool,
            kind,
            object,
            from_json,
            proposal,
            rationale,
        } = self;
        Ok((
            render_output(
                format,
                &propose_create_native_project_pool_library_object(
                    &path,
                    &pool,
                    &kind,
                    object,
                    &from_json,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        ))
    }
}
