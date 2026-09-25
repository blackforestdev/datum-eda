//! Retained Layers row geometry, independent of painted labels and scroll offset.
use super::*;
use std::{cell::RefCell, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq)]
struct LayoutKey {
    rect: RectPx,
    layer_count: usize,
}

impl LayoutKey {
    fn new(filters_rect: RectPx, total_layers: usize) -> Self {
        // Layers/review-filter rows sit on a 25px rhythm (Design Book --row) so the
        // 13px name + 13px swatch center comfortably.
        let row_h = 25.0_f32;
        let summary_height = 62.0;
        let fixed_height =
            4.0 * row_h + UI_STACK_GAP_MEDIUM + summary_height + UI_CARD_CONTENT_BOTTOM;
        let available_layer_height =
            (filters_rect.height - UI_CARD_CONTENT_TOP - fixed_height).max(row_h);
        let max_layer_rows = (available_layer_height / row_h).floor().max(1.0) as usize;
        let layer_count = total_layers.min(max_layer_rows);

        Self {
            rect: filters_rect,
            layer_count,
        }
    }
}

type LayoutCache = crate::shell_layout::cache::LayoutCache<LayoutKey, Arc<FiltersPanelLayout>>;
thread_local! { static CACHE: RefCell<LayoutCache> = RefCell::new(LayoutCache::default()); }

pub(super) fn solve_filters_panel_layout_with_taffy(
    state: &ReviewWorkspaceState,
    filters_rect: RectPx,
) -> Option<Arc<FiltersPanelLayout>> {
    let key = LayoutKey::new(filters_rect, state.scene.layers.len());
    CACHE.with(|cache| {
        cache
            .borrow_mut()
            .try_resolve(key, || solve(key).map(Arc::new))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FiltersPanelNode {
    Authored,
    Proposed,
    Unrouted,
    DimUnrelated,
    Layer(usize),
    ActiveSummary,
    LayersSummary,
    FocusSummary,
    OutputsSummary,
    Gap,
}

fn solve(key: LayoutKey) -> Option<FiltersPanelLayout> {
    record_solve();
    let filters_rect = key.rect;
    let layer_count = key.layer_count;
    let row_h = 25.0_f32;
    let content_x = filters_rect.x + UI_CARD_PADDING_X;
    let content_y = filters_rect.y + UI_CARD_CONTENT_TOP;
    let content_width = (filters_rect.width - UI_CARD_PADDING_X * 2.0).max(1.0);

    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let mut nodes = Vec::new();
    let mut add_node = |kind: FiltersPanelNode, height: f32| -> Option<()> {
        let node = taffy
            .new_leaf(Style {
                size: Size {
                    width: length(content_width),
                    height: length(height),
                },
                ..Default::default()
            })
            .ok()?;
        nodes.push((kind, node));
        Some(())
    };

    add_node(FiltersPanelNode::Authored, row_h)?;
    add_node(FiltersPanelNode::Proposed, row_h)?;
    add_node(FiltersPanelNode::Unrouted, row_h)?;
    add_node(FiltersPanelNode::DimUnrelated, row_h)?;
    for index in 0..layer_count {
        add_node(FiltersPanelNode::Layer(index), row_h)?;
    }
    add_node(FiltersPanelNode::Gap, UI_STACK_GAP_MEDIUM)?;
    add_node(FiltersPanelNode::ActiveSummary, 18.0)?;
    add_node(FiltersPanelNode::LayersSummary, 16.0)?;
    add_node(FiltersPanelNode::FocusSummary, 16.0)?;
    add_node(FiltersPanelNode::OutputsSummary, 16.0)?;

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

    let rect_for = |kind: FiltersPanelNode| -> Option<RectPx> {
        let node = nodes.iter().find(|(node_kind, _)| *node_kind == kind)?.1;
        let layout = taffy.layout(node).ok()?;
        Some(RectPx {
            x: content_x + layout.location.x,
            y: content_y + layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        })
    };
    let mut layer_rows = Vec::with_capacity(layer_count);
    for index in 0..layer_count {
        layer_rows.push(rect_for(FiltersPanelNode::Layer(index))?);
    }

    Some(FiltersPanelLayout {
        authored: rect_for(FiltersPanelNode::Authored)?,
        proposed: rect_for(FiltersPanelNode::Proposed)?,
        unrouted: rect_for(FiltersPanelNode::Unrouted)?,
        dim_unrelated: rect_for(FiltersPanelNode::DimUnrelated)?,
        layer_rows,
        active_summary: rect_for(FiltersPanelNode::ActiveSummary),
        layers_summary: rect_for(FiltersPanelNode::LayersSummary)?,
        focus_summary: rect_for(FiltersPanelNode::FocusSummary)?,
        outputs_summary: rect_for(FiltersPanelNode::OutputsSummary)?,
    })
}

#[inline]
fn record_solve() {
    #[cfg(test)]
    tests::SOLVES.with(|count| count.set(count.get() + 1));
}

#[cfg(test)]
mod tests {
    use super::*;
    thread_local! { pub(super) static SOLVES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

    #[test]
    fn cached_geometry_matches_fresh_and_reuses_row_storage() {
        for height in [1.0, 300.0, 600.0, 900.5] {
            for width in [1.0, 224.0, 336.5, 448.0] {
                let rect = RectPx {
                    x: 0.5,
                    y: 350.5,
                    width,
                    height,
                };
                for total in [0, 1, 8, 24, 100] {
                    let key = LayoutKey::new(rect, total);
                    let expected = solve(key).unwrap();
                    let mut cache = LayoutCache::default();
                    let first = cache.try_resolve(key, || solve(key).map(Arc::new)).unwrap();
                    let second = cache
                        .try_resolve(key, || panic!("warm geometry must not solve"))
                        .unwrap();
                    assert_eq!(*first, expected);
                    assert!(
                        Arc::ptr_eq(&first, &second),
                        "warm lookup must not clone row storage"
                    );
                }
            }
        }
    }

    #[test]
    fn actual_frame_reuses_layer_geometry_but_updates_labels() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        CACHE.with(|cache| *cache.borrow_mut() = LayoutCache::default());
        SOLVES.with(|count| count.set(0));
        let retained = crate::RetainedScene::from_workspace(&state, 1280, 800);
        let camera = crate::CameraState::fit_to_bounds(&state.scene.bounds);
        let first =
            crate::PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
        let layers = |scene: &crate::PreparedScene| {
            scene
                .hit_regions
                .iter()
                .filter_map(|hit| {
                    if let crate::HitTarget::ToggleLayer(id) = &hit.target {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };
        let initial = layers(&first);
        assert!(!initial.is_empty());
        state.scene.layers[0].name = "Renamed layer".to_owned();
        let next =
            crate::PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
        assert_eq!(
            initial,
            layers(&next),
            "label changes preserve hit identity"
        );
        assert!(next.text_runs.iter().any(|run| run.text == "Renamed layer"));
        assert_eq!(
            SOLVES.with(|count| count.get()),
            1,
            "warm Layers frames must reuse geometry"
        );
    }
}
