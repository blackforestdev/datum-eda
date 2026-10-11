use super::*;

#[test]
fn parent_action_chrome_remains_reachable_and_clipped_beside_each_subject() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let action = state.review.proposal_actions[0].action_id.clone();
    for selection in [
        SelectionTarget::None,
        SelectionTarget::ReviewAction(action.clone()),
    ] {
        state.selection = selection;
        for (width, height) in [(296.0, 672.0), (90.0, 180.0)] {
            let outer = RectPx {
                x: 400.0,
                y: 33.0,
                width,
                height,
            };
            let (inspector, lane) = split(&state, outer);
            let lane = lane.unwrap();
            assert!(inspector.height > 0.0 && inspector.y + inspector.height == lane.y);
            let (mut quads, mut text, mut hits) = (Vec::new(), Vec::new(), Vec::new());
            render(&state, lane, &mut quads, &mut text, &mut hits);
            assert!(
                hits.iter()
                    .any(|hit| hit.target == HitTarget::ReviewAction(action.clone()))
            );
            assert!(
                hits.iter()
                    .all(|hit| hit.rect.intersect(lane) == Some(hit.rect))
            );
            assert!(text.iter().any(|run| run.text == "REVIEW ACTIONS"));
            assert!(
                quads
                    .iter()
                    .all(|quad| quad.points.iter().all(|&(x, y)| lane.contains(x, y)))
            );
        }
    }
    state.review.proposal_actions.clear();
    let rect = RectPx {
        x: 0.0,
        y: 0.0,
        width: 296.0,
        height: 600.0,
    };
    assert_eq!(split(&state, rect), (rect, None));
}
