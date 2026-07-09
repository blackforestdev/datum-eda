//! Profile-ordered, first-win route-proposal candidate selection.
//!
//! Moved from `crates/cli/src/command_project_route_proposal.rs` (family F).
//! The selector walks the profile's deterministic candidate order, builds
//! each candidate's proposal actions through the supplied builder, validates
//! them, and keeps the first candidate that produced a valid action set.
//! Statuses, the selection rule, and per-candidate skip/error messages are
//! byte-for-byte the historical CLI output (they land in exported strategy
//! artifacts and locked CLI tests).
//!
//! The action builder is a callback so callers control how per-candidate
//! build failures surface: the CLI resolves the project per candidate build,
//! preserving the historical behavior that a project-load failure is
//! reported as each candidate's message instead of failing the selection.

use crate::board::RoutePathCandidateAuthoredCopperGraphPolicy;

use super::{
    RouteProposalAction, RouteProposalCandidate, RouteProposalProfile, candidate_name,
    candidate_policy_name, candidate_spec_name, profile_name, validate_route_proposal_actions,
};

/// One attempted candidate in selection order.
#[derive(Debug, Clone)]
pub struct RouteProposalSelectionCandidate {
    pub candidate: String,
    pub policy: Option<String>,
    pub selected: bool,
    pub contract: Option<String>,
    pub actions: Option<usize>,
    pub selected_path_bend_count: Option<usize>,
    pub selected_path_point_count: Option<usize>,
    pub selected_path_segment_count: Option<usize>,
    pub message: Option<String>,
}

/// The full selection result: status, rule, winner facts, and the attempted
/// candidate ledger, plus the winning candidate spec for follow-up builds.
#[derive(Debug, Clone)]
pub struct RouteProposalSelectionOutcome {
    pub status: String,
    pub selection_rule: String,
    pub attempted_candidates: usize,
    pub selected_spec: Option<RouteProposalCandidate>,
    pub selected_candidate: Option<String>,
    pub selected_policy: Option<String>,
    pub selected_contract: Option<String>,
    pub selected_actions: Option<usize>,
    pub selected_path_bend_count: Option<usize>,
    pub selected_path_point_count: Option<usize>,
    pub selected_path_segment_count: Option<usize>,
    pub candidates: Vec<RouteProposalSelectionCandidate>,
}

/// The deterministic candidate order for `profile`.
pub fn route_proposal_selection_specs(
    profile: RouteProposalProfile,
) -> Vec<RouteProposalCandidate> {
    match profile {
        RouteProposalProfile::Default => default_route_proposal_selection_specs(),
        RouteProposalProfile::AuthoredCopperPriority => {
            authored_copper_priority_route_proposal_selection_specs()
        }
    }
}

fn default_route_proposal_selection_specs() -> Vec<RouteProposalCandidate> {
    vec![
        RouteProposalCandidate::RoutePathCandidate,
        RouteProposalCandidate::RoutePathCandidateOrthogonalDogleg,
        RouteProposalCandidate::RoutePathCandidateOrthogonalTwoBend,
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraph,
        RouteProposalCandidate::AuthoredCopperPlusOneGap,
        RouteProposalCandidate::RoutePathCandidateVia,
        RouteProposalCandidate::RoutePathCandidateTwoVia,
        RouteProposalCandidate::RoutePathCandidateThreeVia,
        RouteProposalCandidate::RoutePathCandidateFourVia,
        RouteProposalCandidate::RoutePathCandidateFiveVia,
        RouteProposalCandidate::RoutePathCandidateSixVia,
        RouteProposalCandidate::RoutePathCandidateAuthoredViaChain,
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphVia,
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphTwoVia,
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphThreeVia,
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphFourVia,
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphFiveVia,
        RouteProposalCandidate::RoutePathCandidateOrthogonalGraphSixVia,
    ]
}

fn authored_copper_priority_route_proposal_selection_specs() -> Vec<RouteProposalCandidate> {
    let mut specs = vec![
        RouteProposalCandidate::AuthoredCopperGraph(
            RoutePathCandidateAuthoredCopperGraphPolicy::Plain,
        ),
        RouteProposalCandidate::AuthoredCopperGraph(
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware,
        ),
        RouteProposalCandidate::AuthoredCopperGraph(
            RoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware,
        ),
        RouteProposalCandidate::AuthoredCopperGraph(
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware,
        ),
        RouteProposalCandidate::AuthoredCopperGraph(
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware,
        ),
        RouteProposalCandidate::AuthoredCopperGraph(
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware,
        ),
    ];
    specs.extend(default_route_proposal_selection_specs());
    specs
}

/// Run first-win selection over the profile's candidate order.
///
/// `build_actions` builds one candidate's proposal actions; an `Err` is
/// recorded as that candidate's message and selection continues. Actions
/// that fail [`validate_route_proposal_actions`], or a candidate that
/// produces an empty action set, fail the whole selection — exactly the
/// historical CLI control flow.
pub fn run_route_proposal_selection(
    profile: RouteProposalProfile,
    mut build_actions: impl FnMut(&RouteProposalCandidate) -> Result<Vec<RouteProposalAction>, String>,
) -> Result<RouteProposalSelectionOutcome, String> {
    let specs = route_proposal_selection_specs(profile);
    let selection_rule = format!(
        "profile {} selects the first successful candidate in this deterministic order: {}",
        profile_name(profile),
        specs
            .iter()
            .map(candidate_spec_name)
            .collect::<Vec<_>>()
            .join(" > ")
    );
    let mut selected_candidate: Option<(usize, RouteProposalCandidate)> = None;
    let mut candidates = Vec::with_capacity(specs.len());

    for (index, spec) in specs.iter().copied().enumerate() {
        match build_actions(&spec) {
            Ok(actions) => {
                validate_route_proposal_actions(&actions)?;
                let first_action = actions.first().ok_or_else(|| {
                    format!(
                        "route proposal selector candidate {} produced no actions",
                        candidate_spec_name(&spec)
                    )
                })?;
                let message = if let Some((winner_index, winner)) = selected_candidate {
                    Some(format!(
                        "available but skipped because earlier candidate {} won at selection index {}",
                        candidate_spec_name(&winner),
                        winner_index
                    ))
                } else {
                    selected_candidate = Some((index, spec));
                    None
                };
                candidates.push(RouteProposalSelectionCandidate {
                    candidate: candidate_name(spec).to_string(),
                    policy: candidate_policy_name(spec),
                    selected: selected_candidate == Some((index, spec)),
                    contract: Some(first_action.contract.clone()),
                    actions: Some(actions.len()),
                    selected_path_bend_count: Some(first_action.selected_path_bend_count),
                    selected_path_point_count: Some(first_action.selected_path_point_count),
                    selected_path_segment_count: Some(first_action.selected_path_segment_count),
                    message,
                });
            }
            Err(error) => {
                candidates.push(RouteProposalSelectionCandidate {
                    candidate: candidate_name(spec).to_string(),
                    policy: candidate_policy_name(spec),
                    selected: false,
                    contract: None,
                    actions: None,
                    selected_path_bend_count: None,
                    selected_path_point_count: None,
                    selected_path_segment_count: None,
                    message: Some(error),
                });
            }
        }
    }

    let selected_view = selected_candidate.and_then(|(winner_index, winner_spec)| {
        candidates
            .get(winner_index)
            .map(|candidate| (winner_spec, candidate.clone()))
    });

    let selected_spec = selected_view.as_ref().map(|(spec, _)| *spec);

    Ok(RouteProposalSelectionOutcome {
        status: if selected_view.is_some() {
            "deterministic_route_proposal_selected".to_string()
        } else {
            "no_route_proposal_under_current_authored_constraints".to_string()
        },
        selection_rule,
        attempted_candidates: candidates.len(),
        selected_spec,
        selected_candidate: selected_view
            .as_ref()
            .map(|(spec, _)| candidate_name(*spec).to_string()),
        selected_policy: selected_view
            .as_ref()
            .and_then(|(spec, _)| candidate_policy_name(*spec)),
        selected_contract: selected_view
            .as_ref()
            .and_then(|(_, candidate)| candidate.contract.clone()),
        selected_actions: selected_view
            .as_ref()
            .and_then(|(_, candidate)| candidate.actions),
        selected_path_bend_count: selected_view
            .as_ref()
            .and_then(|(_, candidate)| candidate.selected_path_bend_count),
        selected_path_point_count: selected_view
            .as_ref()
            .and_then(|(_, candidate)| candidate.selected_path_point_count),
        selected_path_segment_count: selected_view
            .as_ref()
            .and_then(|(_, candidate)| candidate.selected_path_segment_count),
        candidates,
    })
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::ir::geometry::Point;

    use super::*;

    fn stub_action(contract: &str) -> RouteProposalAction {
        RouteProposalAction {
            action_id: "id".to_string(),
            proposal_action: "draw_track".to_string(),
            reason: "unit_test".to_string(),
            contract: contract.to_string(),
            net_uuid: Uuid::nil(),
            net_name: "SIG".to_string(),
            from_anchor_pad_uuid: Uuid::nil(),
            to_anchor_pad_uuid: Uuid::nil(),
            layer: 1,
            width_nm: 200_000,
            from: Point { x: 0, y: 0 },
            to: Point { x: 1, y: 0 },
            reused_via_uuid: None,
            reused_via_uuids: Vec::new(),
            reused_object_kind: None,
            reused_object_uuid: None,
            reused_object_from_layer: None,
            reused_object_to_layer: None,
            selected_path_bend_count: 0,
            selected_path_point_count: 2,
            selected_path_segment_index: 0,
            selected_path_segment_count: 1,
            selected_path_layer_segment_index: None,
            selected_path_layer_segment_count: None,
            selected_path_layer_segment_bend_count: None,
            selected_path_layer_segment_point_count: None,
        }
    }

    #[test]
    fn default_profile_preserves_the_accepted_candidate_order() {
        let specs = route_proposal_selection_specs(RouteProposalProfile::Default);
        assert_eq!(specs.len(), 18);
        assert_eq!(specs[0], RouteProposalCandidate::RoutePathCandidate);
        assert_eq!(specs[4], RouteProposalCandidate::AuthoredCopperPlusOneGap);
        assert_eq!(
            specs[17],
            RouteProposalCandidate::RoutePathCandidateOrthogonalGraphSixVia
        );
    }

    #[test]
    fn authored_copper_priority_prepends_the_policy_family() {
        let specs = route_proposal_selection_specs(RouteProposalProfile::AuthoredCopperPriority);
        assert_eq!(specs.len(), 24);
        assert_eq!(
            specs[0],
            RouteProposalCandidate::AuthoredCopperGraph(
                RoutePathCandidateAuthoredCopperGraphPolicy::Plain
            )
        );
        assert_eq!(
            specs[6..],
            route_proposal_selection_specs(RouteProposalProfile::Default)[..]
        );
    }

    #[test]
    fn first_successful_candidate_wins_and_later_successes_are_skipped() {
        let outcome =
            run_route_proposal_selection(RouteProposalProfile::Default, |spec| match spec {
                RouteProposalCandidate::RoutePathCandidate => Err("no path".to_string()),
                RouteProposalCandidate::RoutePathCandidateOrthogonalDogleg
                | RouteProposalCandidate::RoutePathCandidateOrthogonalTwoBend => {
                    Ok(vec![stub_action("m5_test_contract_v1")])
                }
                _ => Err("no path".to_string()),
            })
            .expect("selection should run");

        assert_eq!(outcome.status, "deterministic_route_proposal_selected");
        assert_eq!(
            outcome.selected_spec,
            Some(RouteProposalCandidate::RoutePathCandidateOrthogonalDogleg)
        );
        assert_eq!(
            outcome.selected_candidate.as_deref(),
            Some("route-path-candidate-orthogonal-dogleg")
        );
        assert_eq!(
            outcome.selected_contract.as_deref(),
            Some("m5_test_contract_v1")
        );
        assert_eq!(outcome.attempted_candidates, 18);
        assert_eq!(outcome.candidates[0].message.as_deref(), Some("no path"));
        assert!(outcome.candidates[1].selected);
        assert_eq!(
            outcome.candidates[2].message.as_deref(),
            Some(
                "available but skipped because earlier candidate route-path-candidate-orthogonal-dogleg won at selection index 1"
            )
        );
    }

    #[test]
    fn no_successful_candidate_reports_the_no_proposal_status() {
        let outcome =
            run_route_proposal_selection(
                RouteProposalProfile::Default,
                |_| Err("boom".to_string()),
            )
            .expect("selection should run");
        assert_eq!(
            outcome.status,
            "no_route_proposal_under_current_authored_constraints"
        );
        assert_eq!(outcome.selected_spec, None);
        assert!(
            outcome
                .candidates
                .iter()
                .all(|candidate| !candidate.selected)
        );
    }

    #[test]
    fn empty_action_set_fails_the_whole_selection() {
        let error =
            run_route_proposal_selection(RouteProposalProfile::Default, |spec| match spec {
                RouteProposalCandidate::RoutePathCandidate => Ok(Vec::new()),
                _ => Err("no path".to_string()),
            })
            .expect_err("empty action set should fail");
        assert_eq!(
            error,
            "route proposal selector candidate route-path-candidate produced no actions"
        );
    }
}
