// commands/project — project-scoped commands: project/rules bootstrap
// (roots), summary, validate, component instances, waivers, generic
// proposal lifecycle, relationships/variants, and journal undo/redo.
//
// Wave 2 move. Files came from two legacy hosts; the re-exports below
// reproduce exactly what those hosts exported for this family:
//   - command_project_surface.rs: the named lists for component_instances /
//     journal_mutation / proposals / relationships / summary / validate /
//     waivers.
//   - command_project_native_surface.rs: the roots::* glob.
// validate_materialized.rs stays a #[path] child of validate.rs (which also
// owns ../pool/validation.rs), and output_log.rs is deliberately NOT
// declared here: it remains a #[path] child of commands/gerber/plan.rs;
// only the file lives in this family directory.

#[allow(unused_imports)]
use super::*;

mod component_instances;
mod journal_mutation;
mod proposals;
mod relationships;
mod roots;
mod summary;
mod validate;
mod waivers;

pub(crate) use self::component_instances::{
    bind_native_project_component_instance, delete_native_project_component_instance,
    query_native_project_component_instances, set_native_project_component_instance,
};
pub(crate) use self::journal_mutation::{
    execute_native_project_journal_redo, execute_native_project_journal_undo,
};
pub(crate) use self::proposals::{
    BoardComponentReplacementPlanSelectionSpec, BoardComponentReplacementSpec,
    NativeProjectProposalCreateView, accept_and_apply_native_project_proposal,
    apply_native_project_proposal, create_native_project_proposal, defer_native_project_proposal,
    preview_native_project_proposal, propose_native_project_board_component_replacement,
    propose_native_project_board_component_replacement_plan,
    propose_native_project_board_component_replacements, query_native_project_proposals,
    review_native_project_proposal, show_native_project_proposal, validate_native_project_proposal,
    validate_proposal_in_model,
};
pub(crate) use self::relationships::{
    query_native_project_relationships, query_native_project_variants,
};
pub(crate) use self::roots::*;
pub(crate) use self::summary::query_native_project_summary;
pub(crate) use self::validate::validate_native_project;
pub(crate) use self::waivers::{accept_native_project_deviation, waive_native_project_finding};
