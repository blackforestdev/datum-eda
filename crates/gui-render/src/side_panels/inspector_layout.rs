//! Inspector geometry retention; selection content remains live paint input.
use super::*;
use crate::shell_layout::cache::LayoutCache;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, PartialEq)]
struct OuterKey {
    rect: RectPx,
    desired_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DetailKey {
    rect: RectPx,
    action: bool,
    segment: bool,
    status: bool,
    row_height: f32,
}

impl DetailKey {
    fn from_state(state: &ReviewWorkspaceState, rect: RectPx) -> Self {
        Self {
            rect,
            action: state.selected_review_action().is_some(),
            segment: state.selected_segment_evidence().is_some(),
            status: state.last_command_status.is_some(),
            row_height: key_value_row_height(),
        }
    }
}

thread_local! {
    static OUTER: RefCell<LayoutCache<OuterKey, RightPanelLayout>> = RefCell::new(LayoutCache::default());
    static DETAIL: RefCell<LayoutCache<DetailKey, InspectorDetailLayout>> = RefCell::new(LayoutCache::default());
}

pub(crate) fn solve_right_panel_layout_with_taffy(
    state: &ReviewWorkspaceState,
    rect: RectPx,
) -> Option<RightPanelLayout> {
    let key = OuterKey {
        rect,
        desired_height: inspector_height_for_state(state),
    };
    OUTER.with(|cache| cache.borrow_mut().try_resolve(key, || solve_outer(key)))
}

pub(super) fn solve_inspector_detail_layout_with_taffy(
    state: &ReviewWorkspaceState,
    rect: RectPx,
) -> Option<InspectorDetailLayout> {
    let key = DetailKey::from_state(state, rect);
    DETAIL.with(|cache| cache.borrow_mut().try_resolve(key, || solve_detail(key)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RightPanelNode {
    Inspector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InspectorDetailNode {
    Contract,
    Net,
    Segment,
    Layer,
    LastStatus,
}

fn solve_outer(key: OuterKey) -> Option<RightPanelLayout> {
    record_solve(0);
    let right = key.rect;
    let card_x = right.x + UI_CARD_MARGIN;
    let card_y = right.y + UI_CARD_MARGIN;
    let card_width = (right.width - UI_CARD_MARGIN * 2.0).max(1.0);
    let content_height = (right.height - UI_CARD_MARGIN * 2.0).max(1.0);
    let inspector_height = content_height.max(key.desired_height.min(content_height));

    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let inspector = taffy
        .new_leaf(Style {
            size: Size {
                width: length(card_width),
                height: length(inspector_height),
            },
            ..Default::default()
        })
        .ok()?;
    let nodes = [(RightPanelNode::Inspector, inspector)];
    let children = nodes.iter().map(|(_, node)| *node).collect::<Vec<_>>();
    let root = taffy
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: length(card_width),
                    height: length(content_height),
                },
                ..Default::default()
            },
            &children,
        )
        .ok()?;
    taffy.compute_layout(root, Size::MAX_CONTENT).ok()?;
    let rect_for = |kind: RightPanelNode| -> Option<RectPx> {
        let node = nodes.iter().find(|(node_kind, _)| *node_kind == kind)?.1;
        let layout = taffy.layout(node).ok()?;
        Some(RectPx {
            x: card_x + layout.location.x,
            y: card_y + layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        })
    };
    Some(RightPanelLayout {
        inspector_rect: rect_for(RightPanelNode::Inspector)?,
    })
}

fn solve_detail(key: DetailKey) -> Option<InspectorDetailLayout> {
    record_solve(1);
    let inspector_rect = key.rect;
    let content_x = inspector_rect.x + UI_CARD_PADDING_X;
    // Detail rows sit below the title band (y+34..y+66) and the "ROUTE ACTION"
    // section-header strip (y+72..y+90).
    let content_y = inspector_rect.y + 96.0;
    let content_width = (inspector_rect.width - UI_CARD_PADDING_X * 2.0).max(1.0);
    let mut nodes = Vec::new();
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let row_height = key.row_height;
    let mut add_node = |kind: InspectorDetailNode| -> Option<()> {
        let node = taffy
            .new_leaf(Style {
                size: Size {
                    width: length(content_width),
                    height: length(row_height),
                },
                ..Default::default()
            })
            .ok()?;
        nodes.push((kind, node));
        Some(())
    };

    if key.action {
        add_node(InspectorDetailNode::Contract)?;
        add_node(InspectorDetailNode::Net)?;
        add_node(InspectorDetailNode::Segment)?;
    }
    if key.segment {
        add_node(InspectorDetailNode::Layer)?;
    }
    if key.status {
        add_node(InspectorDetailNode::LastStatus)?;
    }

    if nodes.is_empty() {
        return Some(InspectorDetailLayout {
            divider_y: None,
            contract: None,
            net: None,
            segment: None,
            layer: None,
            last_status: None,
        });
    }

    let children = nodes.iter().map(|(_, node)| *node).collect::<Vec<_>>();
    let root = taffy
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: length(content_width),
                    height: Dimension::AUTO,
                },
                ..Default::default()
            },
            &children,
        )
        .ok()?;
    taffy.compute_layout(root, Size::MAX_CONTENT).ok()?;

    let rect_for = |kind: InspectorDetailNode| -> Option<RectPx> {
        let node = nodes.iter().find(|(node_kind, _)| *node_kind == kind)?.1;
        let layout = taffy.layout(node).ok()?;
        Some(RectPx {
            x: content_x + layout.location.x,
            y: content_y + layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        })
    };

    Some(InspectorDetailLayout {
        divider_y: Some(inspector_rect.y + 76.0),
        contract: rect_for(InspectorDetailNode::Contract),
        net: rect_for(InspectorDetailNode::Net),
        segment: rect_for(InspectorDetailNode::Segment),
        layer: rect_for(InspectorDetailNode::Layer),
        last_status: rect_for(InspectorDetailNode::LastStatus),
    })
}
#[inline]
fn record_solve(_kind: usize) {
    #[cfg(test)]
    tests::SOLVES.with(|count| {
        let mut value = count.get();
        value[_kind] += 1;
        count.set(value);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    thread_local! { pub(super) static SOLVES: std::cell::Cell<[usize; 2]> = const { std::cell::Cell::new([0, 0]) }; }

    #[test]
    fn inspector_keys_preserve_fresh_geometry_across_profiles() {
        for rect in [
            RectPx {
                x: 900.0,
                y: 29.0,
                width: 296.0,
                height: 700.0,
            },
            RectPx {
                x: 1300.5,
                y: 43.5,
                width: 444.0,
                height: 500.5,
            },
            RectPx {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        ] {
            let mut outer = LayoutCache::default();
            for desired_height in [150.0, 330.0, 600.0] {
                let key = OuterKey {
                    rect,
                    desired_height,
                };
                let expected = solve_outer(key);
                for _ in 0..2 {
                    assert_eq!(outer.try_resolve(key, || solve_outer(key)), expected);
                }
            }
            let mut detail = LayoutCache::default();
            for flags in 0..8 {
                for row_height in [20.0, 25.5] {
                    let key = DetailKey {
                        rect,
                        action: flags & 1 != 0,
                        segment: flags & 2 != 0,
                        status: flags & 4 != 0,
                        row_height,
                    };
                    let expected = solve_detail(key);
                    for _ in 0..2 {
                        assert_eq!(detail.try_resolve(key, || solve_detail(key)), expected);
                    }
                }
            }
        }
    }

    #[test]
    fn actual_inspector_preparation_reuses_layout_and_updates_values() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let id = state
            .review
            .proposal_actions
            .first()
            .expect("existing review fixture")
            .action_id
            .clone();
        state.select_review_action(&id);
        assert!(state.selected_review_action().is_some());
        OUTER.with(|cache| *cache.borrow_mut() = LayoutCache::default());
        DETAIL.with(|cache| *cache.borrow_mut() = LayoutCache::default());
        SOLVES.with(|count| count.set([0, 0]));
        let retained = crate::RetainedScene::from_workspace(&state, 1280, 800);
        let camera = crate::CameraState::fit_to_bounds(&state.scene.bounds);
        for _ in 0..3 {
            let _ =
                crate::PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
        }
        assert_eq!(
            SOLVES.with(|count| count.get()),
            [1, 1],
            "warm Inspector frames must reuse both layouts"
        );
        state.review.proposal_actions[0].net_name = "CACHE_NET".to_owned();
        let prepared =
            crate::PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
        assert!(prepared.text_runs.iter().any(|run| run.text == "CACHE_NET"));
        assert_eq!(SOLVES.with(|count| count.get()), [1, 1]);
    }
}
