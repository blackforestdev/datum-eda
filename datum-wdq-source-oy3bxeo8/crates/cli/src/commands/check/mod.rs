// commands/check — native check runs, check-run views/history, finding
// identity, proposal links, release check gate, and project inspect /
// resolve-debug / journal queries.
//
// Wave 2 move. native_inspect.rs keeps its #[path] child modules
// (finding_identity.rs, proposal_refs.rs, run_history.rs, run_view.rs,
// targets.rs here, plus ../artifacts/checks.rs), exactly as before the move.
// gate.rs is deliberately NOT declared here: it remains a #[path] child of
// its four consumers (commands/{artifacts,gerber,manufacturing,output_jobs}),
// exactly as it was as command_project_check_gate.rs.
//
// The named list below reproduces what command_project_surface.rs exported
// for this family, plus three items whose consumers moved to sibling
// families in this wave (NativeProjectCheckFindingView for standards,
// the journal-mutation views and journal_tip_availability for
// commands/project/journal_mutation.rs).

#[allow(unused_imports)]
use super::*;

mod native_inspect;

pub(crate) use self::native_inspect::{
    NativeProjectCheckFindingView, NativeProjectCheckRunView,
    NativeProjectJournalMutationGuardView, NativeProjectJournalMutationView,
    execute_native_project_resolve_debug_query, inspect_native_project, journal_tip_availability,
    query_native_project_check_profiles, query_native_project_check_run,
    query_native_project_check_run_show, query_native_project_check_run_with_profile,
    query_native_project_journal_list, query_native_project_journal_show,
    run_native_project_check_with_profile,
};
