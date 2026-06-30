use super::RectPx;
use taffy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub(super) struct OutputsLaneLayout {
    pub(super) title: RectPx,
    pub(super) status: RectPx,
    pub(super) artifact_command: RectPx,
    pub(super) body: RectPx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OutputsBodySectionKind {
    FocusedArtifact,
    Checks,
    Actions,
    Panels,
    Plans,
    Jobs,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct OutputsBodySectionSpec {
    pub(super) kind: OutputsBodySectionKind,
    pub(super) height: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct OutputsBodySectionLayout {
    pub(super) kind: OutputsBodySectionKind,
    pub(super) rect: RectPx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputsLaneNode {
    Title,
    Status,
    ArtifactCommand,
    Body,
}

pub(super) fn solve_outputs_lane_layout_with_taffy(rect: RectPx) -> Option<OutputsLaneLayout> {
    let content_x = rect.x + 12.0;
    let content_y = rect.y + 12.0;
    let content_width = (rect.width - 24.0).max(1.0);
    let content_height = (rect.height - 30.0).max(1.0);
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let mut nodes = Vec::new();
    let mut add_node = |kind: OutputsLaneNode, height: f32, flex_grow: f32| -> Option<()> {
        let node = taffy
            .new_leaf(Style {
                size: Size {
                    width: length(content_width),
                    height: length(height),
                },
                flex_grow,
                ..Default::default()
            })
            .ok()?;
        nodes.push((kind, node));
        Some(())
    };

    add_node(OutputsLaneNode::Title, 16.0, 0.0)?;
    add_node(OutputsLaneNode::Status, 18.0, 0.0)?;
    add_node(OutputsLaneNode::ArtifactCommand, 18.0, 0.0)?;
    add_node(OutputsLaneNode::Body, 1.0, 1.0)?;
    drop(add_node);

    let children = nodes.iter().map(|(_, node)| *node).collect::<Vec<_>>();
    let root = taffy
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: length(content_width),
                    height: length(content_height),
                },
                ..Default::default()
            },
            &children,
        )
        .ok()?;
    taffy.compute_layout(root, Size::MAX_CONTENT).ok()?;

    let rect_for = |kind: OutputsLaneNode| -> Option<RectPx> {
        let node = nodes.iter().find(|(node_kind, _)| *node_kind == kind)?.1;
        let solved = taffy.layout(node).ok()?;
        Some(RectPx {
            x: content_x + solved.location.x,
            y: content_y + solved.location.y,
            width: solved.size.width,
            height: solved.size.height,
        })
    };

    Some(OutputsLaneLayout {
        title: rect_for(OutputsLaneNode::Title)?,
        status: rect_for(OutputsLaneNode::Status)?,
        artifact_command: rect_for(OutputsLaneNode::ArtifactCommand)?,
        body: rect_for(OutputsLaneNode::Body)?,
    })
}

pub(super) fn solve_outputs_body_sections_with_taffy(
    body: RectPx,
    specs: &[OutputsBodySectionSpec],
) -> Option<Vec<OutputsBodySectionLayout>> {
    let content_width = body.width.max(1.0);
    let content_height = body.height.max(1.0);
    let mut used_height = 0.0;
    let mut active_specs = Vec::new();
    for spec in specs.iter().copied() {
        let height = spec.height.max(0.0);
        if height <= 0.0 {
            continue;
        }
        if used_height + height > content_height {
            let remaining_height = content_height - used_height;
            if remaining_height > 0.0 {
                active_specs.push(OutputsBodySectionSpec {
                    height: remaining_height,
                    ..spec
                });
            }
            break;
        }
        used_height += height;
        active_specs.push(OutputsBodySectionSpec { height, ..spec });
    }

    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let mut nodes = Vec::new();
    for spec in active_specs {
        let node = taffy
            .new_leaf(Style {
                size: Size {
                    width: length(content_width),
                    height: length(spec.height),
                },
                ..Default::default()
            })
            .ok()?;
        nodes.push((spec.kind, node));
    }

    let children = nodes.iter().map(|(_, node)| *node).collect::<Vec<_>>();
    let root = taffy
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: Size {
                    width: length(content_width),
                    height: length(content_height),
                },
                ..Default::default()
            },
            &children,
        )
        .ok()?;
    taffy.compute_layout(root, Size::MAX_CONTENT).ok()?;

    nodes
        .into_iter()
        .map(|(kind, node)| {
            let solved = taffy.layout(node).ok()?;
            Some(OutputsBodySectionLayout {
                kind,
                rect: RectPx {
                    x: body.x + solved.location.x,
                    y: body.y + solved.location.y,
                    width: solved.size.width,
                    height: solved.size.height,
                },
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outputs_lane_header_and_body_are_solver_backed_and_ordered() {
        let rect = RectPx {
            x: 12.0,
            y: 44.0,
            width: 1000.0,
            height: 220.0,
        };
        let layout =
            solve_outputs_lane_layout_with_taffy(rect).expect("outputs lane layout should solve");

        assert!(layout.title.y < layout.status.y);
        assert!(layout.status.y < layout.artifact_command.y);
        assert!(layout.artifact_command.y < layout.body.y);
        assert!(layout.body.height > 100.0);
        assert!(layout.body.x >= rect.x);
        assert!(layout.body.x + layout.body.width <= rect.x + rect.width);
        assert!(layout.body.y + layout.body.height <= rect.y + rect.height);
    }

    #[test]
    fn outputs_body_sections_are_solver_backed_and_clipped_to_body() {
        let body = RectPx {
            x: 24.0,
            y: 80.0,
            width: 900.0,
            height: 120.0,
        };
        let sections = solve_outputs_body_sections_with_taffy(
            body,
            &[
                OutputsBodySectionSpec {
                    kind: OutputsBodySectionKind::FocusedArtifact,
                    height: 20.0,
                },
                OutputsBodySectionSpec {
                    kind: OutputsBodySectionKind::Checks,
                    height: 20.0,
                },
                OutputsBodySectionSpec {
                    kind: OutputsBodySectionKind::Actions,
                    height: 20.0,
                },
                OutputsBodySectionSpec {
                    kind: OutputsBodySectionKind::Panels,
                    height: 34.0,
                },
                OutputsBodySectionSpec {
                    kind: OutputsBodySectionKind::Plans,
                    height: 48.0,
                },
                OutputsBodySectionSpec {
                    kind: OutputsBodySectionKind::Jobs,
                    height: 80.0,
                },
            ],
        )
        .expect("body section layout should solve");

        assert_eq!(sections.len(), 5);
        assert_eq!(sections[0].kind, OutputsBodySectionKind::FocusedArtifact);
        assert_eq!(sections[1].kind, OutputsBodySectionKind::Checks);
        assert_eq!(sections[2].kind, OutputsBodySectionKind::Actions);
        assert_eq!(sections[3].kind, OutputsBodySectionKind::Panels);
        assert_eq!(sections[4].kind, OutputsBodySectionKind::Plans);
        assert!(sections[0].rect.y < sections[1].rect.y);
        for section in sections {
            assert!(section.rect.x >= body.x);
            assert!(section.rect.y >= body.y);
            assert!(section.rect.x + section.rect.width <= body.x + body.width);
            assert!(section.rect.y + section.rect.height <= body.y + body.height);
        }
    }
}
