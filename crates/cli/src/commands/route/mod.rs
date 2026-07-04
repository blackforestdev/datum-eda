// commands/route/ — the route command family: route proposals, apply,
// corridor, preflight, path-candidate queries, and their views.
//
// Wave 2 move. Files came from three legacy hosts; the re-exports below
// reproduce exactly what those hosts exported for this family:
//   - command_project_route_surface.rs: glob re-exports of apply / proposal /
//     proposal_substrate.
//   - command_project_native_surface.rs: the named query/render lists for
//     corridor, preflight, and the path-candidate strategies
//     (path_candidate_orthogonal_graph_spine stays module-private, as before).
//   - main.rs: glob re-export of main_route_proposal.rs (now views.rs).

mod apply;
mod corridor;
mod path_candidate;
mod path_candidate_authored_copper_graph;
mod path_candidate_authored_copper_graph_explain;
mod path_candidate_authored_copper_plus_one_gap;
mod path_candidate_authored_copper_plus_one_gap_explain;
mod path_candidate_authored_via_chain;
mod path_candidate_authored_via_chain_explain;
mod path_candidate_explain;
mod path_candidate_five_via;
mod path_candidate_five_via_explain;
mod path_candidate_four_via;
mod path_candidate_four_via_explain;
mod path_candidate_orthogonal_dogleg;
mod path_candidate_orthogonal_dogleg_explain;
mod path_candidate_orthogonal_graph;
mod path_candidate_orthogonal_graph_explain;
mod path_candidate_orthogonal_graph_five_via;
mod path_candidate_orthogonal_graph_five_via_explain;
mod path_candidate_orthogonal_graph_four_via;
mod path_candidate_orthogonal_graph_four_via_explain;
mod path_candidate_orthogonal_graph_six_via;
mod path_candidate_orthogonal_graph_six_via_explain;
mod path_candidate_orthogonal_graph_spine;
mod path_candidate_orthogonal_graph_three_via;
mod path_candidate_orthogonal_graph_three_via_explain;
mod path_candidate_orthogonal_graph_two_via;
mod path_candidate_orthogonal_graph_two_via_explain;
mod path_candidate_orthogonal_graph_via;
mod path_candidate_orthogonal_graph_via_explain;
mod path_candidate_orthogonal_two_bend;
mod path_candidate_orthogonal_two_bend_explain;
mod path_candidate_six_via;
mod path_candidate_six_via_explain;
mod path_candidate_three_via;
mod path_candidate_three_via_explain;
mod path_candidate_two_via;
mod path_candidate_two_via_explain;
mod path_candidate_via;
mod path_candidate_via_explain;
mod preflight;
mod proposal;
mod proposal_substrate;
mod views;

pub(crate) use self::apply::*;
pub(crate) use self::corridor::{
    query_native_project_route_corridor, render_native_project_route_corridor_text,
};
pub(crate) use self::path_candidate::{
    query_native_project_route_path_candidate, render_native_project_route_path_candidate_text,
};
pub(crate) use self::path_candidate_authored_copper_graph::{
    query_native_project_route_path_candidate_authored_copper_graph,
    render_native_project_route_path_candidate_authored_copper_graph_text,
};
pub(crate) use self::path_candidate_authored_copper_graph_explain::{
    query_native_project_route_path_candidate_authored_copper_graph_explain,
    render_native_project_route_path_candidate_authored_copper_graph_explain_text,
};
pub(crate) use self::path_candidate_authored_copper_plus_one_gap::{
    query_native_project_route_path_candidate_authored_copper_plus_one_gap,
    render_native_project_route_path_candidate_authored_copper_plus_one_gap_text,
};
pub(crate) use self::path_candidate_authored_copper_plus_one_gap_explain::{
    query_native_project_route_path_candidate_authored_copper_plus_one_gap_explain,
    render_native_project_route_path_candidate_authored_copper_plus_one_gap_explain_text,
};
pub(crate) use self::path_candidate_authored_via_chain::{
    query_native_project_route_path_candidate_authored_via_chain,
    render_native_project_route_path_candidate_authored_via_chain_text,
};
pub(crate) use self::path_candidate_authored_via_chain_explain::{
    query_native_project_route_path_candidate_authored_via_chain_explain,
    render_native_project_route_path_candidate_authored_via_chain_explain_text,
};
pub(crate) use self::path_candidate_explain::{
    query_native_project_route_path_candidate_explain,
    render_native_project_route_path_candidate_explain_text,
};
pub(crate) use self::path_candidate_five_via::{
    query_native_project_route_path_candidate_five_via,
    render_native_project_route_path_candidate_five_via_text,
};
pub(crate) use self::path_candidate_five_via_explain::{
    query_native_project_route_path_candidate_five_via_explain,
    render_native_project_route_path_candidate_five_via_explain_text,
};
pub(crate) use self::path_candidate_four_via::{
    query_native_project_route_path_candidate_four_via,
    render_native_project_route_path_candidate_four_via_text,
};
pub(crate) use self::path_candidate_four_via_explain::{
    query_native_project_route_path_candidate_four_via_explain,
    render_native_project_route_path_candidate_four_via_explain_text,
};
pub(crate) use self::path_candidate_orthogonal_dogleg::{
    query_native_project_route_path_candidate_orthogonal_dogleg,
    render_native_project_route_path_candidate_orthogonal_dogleg_text,
};
pub(crate) use self::path_candidate_orthogonal_dogleg_explain::{
    query_native_project_route_path_candidate_orthogonal_dogleg_explain,
    render_native_project_route_path_candidate_orthogonal_dogleg_explain_text,
};
pub(crate) use self::path_candidate_orthogonal_graph::{
    query_native_project_route_path_candidate_orthogonal_graph,
    render_native_project_route_path_candidate_orthogonal_graph_text,
};
pub(crate) use self::path_candidate_orthogonal_graph_explain::{
    query_native_project_route_path_candidate_orthogonal_graph_explain,
    render_native_project_route_path_candidate_orthogonal_graph_explain_text,
};
pub(crate) use self::path_candidate_orthogonal_graph_five_via::{
    query_native_project_route_path_candidate_orthogonal_graph_five_via,
    render_native_project_route_path_candidate_orthogonal_graph_five_via_text,
};
pub(crate) use self::path_candidate_orthogonal_graph_five_via_explain::{
    query_native_project_route_path_candidate_orthogonal_graph_five_via_explain,
    render_native_project_route_path_candidate_orthogonal_graph_five_via_explain_text,
};
pub(crate) use self::path_candidate_orthogonal_graph_four_via::{
    query_native_project_route_path_candidate_orthogonal_graph_four_via,
    render_native_project_route_path_candidate_orthogonal_graph_four_via_text,
};
pub(crate) use self::path_candidate_orthogonal_graph_four_via_explain::{
    query_native_project_route_path_candidate_orthogonal_graph_four_via_explain,
    render_native_project_route_path_candidate_orthogonal_graph_four_via_explain_text,
};
pub(crate) use self::path_candidate_orthogonal_graph_six_via::{
    query_native_project_route_path_candidate_orthogonal_graph_six_via,
    render_native_project_route_path_candidate_orthogonal_graph_six_via_text,
};
pub(crate) use self::path_candidate_orthogonal_graph_six_via_explain::{
    query_native_project_route_path_candidate_orthogonal_graph_six_via_explain,
    render_native_project_route_path_candidate_orthogonal_graph_six_via_explain_text,
};
pub(crate) use self::path_candidate_orthogonal_graph_three_via::{
    query_native_project_route_path_candidate_orthogonal_graph_three_via,
    render_native_project_route_path_candidate_orthogonal_graph_three_via_text,
};
pub(crate) use self::path_candidate_orthogonal_graph_three_via_explain::{
    query_native_project_route_path_candidate_orthogonal_graph_three_via_explain,
    render_native_project_route_path_candidate_orthogonal_graph_three_via_explain_text,
};
pub(crate) use self::path_candidate_orthogonal_graph_two_via::{
    query_native_project_route_path_candidate_orthogonal_graph_two_via,
    render_native_project_route_path_candidate_orthogonal_graph_two_via_text,
};
pub(crate) use self::path_candidate_orthogonal_graph_two_via_explain::{
    query_native_project_route_path_candidate_orthogonal_graph_two_via_explain,
    render_native_project_route_path_candidate_orthogonal_graph_two_via_explain_text,
};
pub(crate) use self::path_candidate_orthogonal_graph_via::{
    query_native_project_route_path_candidate_orthogonal_graph_via,
    render_native_project_route_path_candidate_orthogonal_graph_via_text,
};
pub(crate) use self::path_candidate_orthogonal_graph_via_explain::{
    query_native_project_route_path_candidate_orthogonal_graph_via_explain,
    render_native_project_route_path_candidate_orthogonal_graph_via_explain_text,
};
pub(crate) use self::path_candidate_orthogonal_two_bend::{
    query_native_project_route_path_candidate_orthogonal_two_bend,
    render_native_project_route_path_candidate_orthogonal_two_bend_text,
};
pub(crate) use self::path_candidate_orthogonal_two_bend_explain::{
    query_native_project_route_path_candidate_orthogonal_two_bend_explain,
    render_native_project_route_path_candidate_orthogonal_two_bend_explain_text,
};
pub(crate) use self::path_candidate_six_via::{
    query_native_project_route_path_candidate_six_via,
    render_native_project_route_path_candidate_six_via_text,
};
pub(crate) use self::path_candidate_six_via_explain::{
    query_native_project_route_path_candidate_six_via_explain,
    render_native_project_route_path_candidate_six_via_explain_text,
};
pub(crate) use self::path_candidate_three_via::{
    query_native_project_route_path_candidate_three_via,
    render_native_project_route_path_candidate_three_via_text,
};
pub(crate) use self::path_candidate_three_via_explain::{
    query_native_project_route_path_candidate_three_via_explain,
    render_native_project_route_path_candidate_three_via_explain_text,
};
pub(crate) use self::path_candidate_two_via::{
    query_native_project_route_path_candidate_two_via,
    render_native_project_route_path_candidate_two_via_text,
};
pub(crate) use self::path_candidate_two_via_explain::{
    query_native_project_route_path_candidate_two_via_explain,
    render_native_project_route_path_candidate_two_via_explain_text,
};
pub(crate) use self::path_candidate_via::{
    query_native_project_route_path_candidate_via,
    render_native_project_route_path_candidate_via_text,
};
pub(crate) use self::path_candidate_via_explain::{
    query_native_project_route_path_candidate_via_explain,
    render_native_project_route_path_candidate_via_explain_text,
};
pub(crate) use self::preflight::{
    query_native_project_route_preflight, render_native_project_route_preflight_text,
};
pub(crate) use self::proposal::*;
pub(crate) use self::proposal_substrate::*;
pub(crate) use self::views::*;
