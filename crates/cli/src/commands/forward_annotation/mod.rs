// commands/forward_annotation — forward-annotation proposals, artifacts,
// review state, and substrate apply paths, plus their CLI views.
//
// Wave 2 move. Files came from two legacy hosts; the re-exports below
// reproduce exactly what those hosts exported for this family:
//   - command_project_forward_annotation_surface.rs: the named lists for
//     apply_review / artifact / artifact_review / proposal.
//   - main.rs: glob re-exports of main_forward_annotation.rs (now
//     handlers.rs) and its _views / _audit_views / _reports satellites.
// review_state.rs, substrate.rs, and artifact_compare.rs remain #[path]
// children of their consuming modules, exactly as before the move.

#[allow(unused_imports)]
use super::*;

mod apply_review;
mod artifact;
mod artifact_review;
mod audit_views;
mod handlers;
mod proposal;
mod reports;
mod views;

pub(crate) use self::apply_review::{
    apply_native_project_forward_annotation_action,
    apply_native_project_forward_annotation_reviewed,
    clear_native_project_forward_annotation_review, query_native_project_forward_annotation_review,
    record_native_project_forward_annotation_review,
};
pub(crate) use self::artifact::{
    apply_forward_annotation_proposal_artifact, compare_forward_annotation_proposal_artifact,
    export_native_project_forward_annotation_proposal,
    export_native_project_forward_annotation_proposal_selection,
    filter_forward_annotation_proposal_artifact, inspect_forward_annotation_proposal_artifact,
    plan_forward_annotation_proposal_artifact_apply, select_forward_annotation_proposal_artifact,
    validate_forward_annotation_proposal_artifact,
};
pub(crate) use self::artifact_review::{
    import_forward_annotation_artifact_review, replace_forward_annotation_artifact_review,
};
pub(crate) use self::audit_views::*;
pub(crate) use self::handlers::*;
pub(crate) use self::proposal::{
    query_native_project_forward_annotation_audit, query_native_project_forward_annotation_proposal,
};
pub(crate) use self::reports::*;
pub(crate) use self::views::*;
