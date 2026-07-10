// Panels are flush stacked bodies that span the sidebar edge-to-edge (Design
// Book panel language), not inset floating cards — so the shared column margin
// is zero and separation reads through surface + a single hairline divider.
const UI_CARD_MARGIN: f32 = 0.0;
const UI_CARD_PADDING_X: f32 = design_tokens::spacing::SP_04;
const UI_CARD_TITLE_Y: f32 = design_tokens::spacing::SP_04;
const UI_CARD_DIVIDER_Y: f32 = design_tokens::spacing::SP_07 - design_tokens::spacing::SP_02;
const UI_CARD_CONTENT_TOP: f32 = design_tokens::spacing::SP_07 + design_tokens::spacing::SP_01;
const UI_CARD_CONTENT_BOTTOM: f32 = design_tokens::typography::BODY_LINE;
const UI_ROW_PROJECT_TITLE: f32 = design_tokens::typography::BODY_LINE;
const UI_ROW_BOARD_SUBTITLE: f32 = design_tokens::typography::HEADER_LINE;
const UI_ROW_NET: f32 = design_tokens::typography::BODY_LINE;
const UI_ROW_SOURCE_LABEL: f32 = design_tokens::typography::CAPTION_LINE;
const UI_ROW_SOURCE_ATTENTION: f32 = design_tokens::typography::CAPTION_LINE;
const UI_ROW_BUTTON: f32 = design_tokens::spacing::SP_06 - design_tokens::spacing::SP_02;
const UI_ROW_TOOL_LABEL: f32 = design_tokens::typography::CAPTION_LINE;
const UI_ROW_TOOL_GRID: f32 = design_tokens::spacing::SP_08 + design_tokens::spacing::SP_02;
const UI_ROW_NOTICE: f32 = design_tokens::typography::CAPTION_LINE;
const UI_STACK_GAP_SMALL: f32 = design_tokens::spacing::SP_03;
const UI_STACK_GAP_MEDIUM: f32 = design_tokens::spacing::SP_04;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectPx {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RectPx {
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    fn scale_by(self, scale: f32) -> Self {
        Self {
            x: self.x * scale,
            y: self.y * scale,
            width: self.width * scale,
            height: self.height * scale,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraState {
    pub center_x_nm: f32,
    pub center_y_nm: f32,
    pub zoom: f32,
}

impl CameraState {
    pub fn fit_to_bounds(bounds: &datum_gui_protocol::SceneBounds) -> Self {
        Self {
            center_x_nm: ((bounds.min_x + bounds.max_x) as f32) * 0.5,
            center_y_nm: ((bounds.min_y + bounds.max_y) as f32) * 0.5,
            zoom: 1.0,
        }
    }

    pub fn pan_pixels(
        &mut self,
        viewport: RectPx,
        bounds: &datum_gui_protocol::SceneBounds,
        delta_x_px: f32,
        delta_y_px: f32,
    ) {
        let projection = Projection::new(viewport, bounds, *self);
        self.center_x_nm -= delta_x_px / projection.scale;
        self.center_y_nm -= delta_y_px / projection.scale;
    }

    pub fn zoom_about_screen_point(
        &mut self,
        viewport: RectPx,
        bounds: &datum_gui_protocol::SceneBounds,
        screen_x: f32,
        screen_y: f32,
        zoom_delta: f32,
    ) {
        let before = Projection::new(viewport, bounds, *self).screen_to_world(screen_x, screen_y);
        self.zoom = (self.zoom * zoom_delta).clamp(0.35, 8.0);
        let after = Projection::new(viewport, bounds, *self).screen_to_world(screen_x, screen_y);
        self.center_x_nm += before.x as f32 - after.x as f32;
        self.center_y_nm += before.y as f32 - after.y as f32;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShellLayout {
    pub top_menu_bar: RectPx,
    pub viewport: RectPx,
    pub left_sidebar: RectPx,
    pub right_sidebar: RectPx,
    pub bottom_strip: RectPx,
    pub status_bar: RectPx,
}

/// One split pane inside the central viewport: its whole `frame`, the 31px
/// `header` band at the top of the frame, and the inset `scene` canvas beneath
/// the header. A (document, view) pair from the Workspace & Mode model renders
/// into a `PaneRect` (docs/gui/DATUM_GUI_DESIGN_SPEC.md → Workspace & Mode
/// Model). Derived purely from an already-solved (and already scaled) frame so
/// hi-DPI `scale_by` needs no pane awareness.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaneRect {
    pub frame: RectPx,
    pub header: RectPx,
    pub scene: RectPx,
}

impl PaneRect {
    // Header band height and canvas insets match the pre-split single-pane
    // `scene_viewport` (16px side/bottom gutter, 42px top clearing the 31px
    // header). Fixed px in the same space the caller solved in — no scale math.
    const HEADER_H: f32 = 31.0;

    fn from_frame(frame: RectPx) -> Self {
        let header = RectPx {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: Self::HEADER_H.min(frame.height),
        };
        let scene = inset_rect(frame, 16.0, 42.0, 16.0, 16.0);
        Self {
            frame,
            header,
            scene,
        }
    }
}

/// One leaf pane placed in screen space by the tile walk: its stable `PaneId`,
/// the `(document, view)` content it shows, and its solved `PaneRect`
/// (frame/header/scene). Derived purely as a post-solve over the central
/// `viewport`, so neither the Taffy solver nor `scale_by` becomes pane-aware.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeafPane {
    pub id: datum_gui_protocol::PaneId,
    pub content: datum_gui_protocol::PaneContent,
    pub rect: PaneRect,
}

/// The central viewport tiled per the `WorkspaceLayout` tree: the in-order set
/// of leaf `panes`, the `dividers` (one hairline gutter per Split, between its
/// two children), and which leaf `focused` owns the accent chrome + (via
/// context-follows-focus) the Inspector/Layers side panels
/// (docs/gui/DATUM_GUI_DESIGN_SPEC.md → Workspace & Mode Model). Generalizes the
/// former fixed two-pane slice to N leaves, nested H/V splits, and zoom, still as
/// a pure post-split derived AFTER the taffy/fallback solve and AFTER `scale_by`.
///
/// The single-live-scene architecture is preserved: `focused_scene()` names the
/// one canvas the retained world buffer, gpu scissor/uniform, and hit-testing all
/// follow. Non-focused Board leaves render as today (no per-leaf snapshot yet);
/// Schematic leaves show the "Schematic (coming)" placeholder.
/// A split's divider gutter plus everything a divider-drag resize needs
/// (decision 021): the hairline `rect` painted between the two children, the full
/// `split_frame` the new ratio is measured within, the split `orientation`, and
/// the `path` from the tree root to the controlling `Split` (so the runtime maps a
/// grabbed gutter to `WorkspaceLayout::set_ratio_at_path`). Purely derived layout
/// geometry — consumer view state, never journaled.
#[derive(Debug, Clone, PartialEq)]
pub struct PaneDivider {
    pub rect: RectPx,
    pub split_frame: RectPx,
    pub orientation: datum_gui_protocol::SplitOrientation,
    pub path: Vec<datum_gui_protocol::SplitChild>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewportPanes {
    pub panes: Vec<LeafPane>,
    pub dividers: Vec<PaneDivider>,
    pub focused: datum_gui_protocol::PaneId,
}

impl ViewportPanes {
    /// The focused leaf pane (falls back to the first leaf if `focused` names no
    /// present leaf — e.g. a stale id; the tree always has at least one leaf).
    pub fn focused_pane(&self) -> &LeafPane {
        self.panes
            .iter()
            .find(|p| p.id == self.focused)
            .unwrap_or(&self.panes[0])
    }

    /// The content of the focused leaf. This is the seam context-follows-focus
    /// reads — the side panels reflect the focused pane's document.
    pub fn focused_document(&self) -> datum_gui_protocol::PaneContent {
        self.focused_pane().content
    }

    /// The focused leaf's scene canvas rect — the single live scene viewport.
    pub fn focused_scene(&self) -> RectPx {
        self.focused_pane().rect.scene
    }

    /// The leaf that renders the single live BOARD scene: the focused leaf when it
    /// is a Board, otherwise the first Board leaf in walk order. `None` when no
    /// leaf shows Board content. The board world buffer, gpu scissor/uniform, and
    /// board hit-testing all follow THIS leaf — so the PCB stays put in its own
    /// pane and does NOT vanish (or migrate) when focus moves to a Schematic pane.
    /// Preferring the focused leaf when it is a Board keeps a split Board|Board
    /// pairing rendering into whichever board the user is working, while the common
    /// Board|Schematic layout always renders the PCB in its board pane regardless
    /// of focus. (Idle snapshots for additional board leaves are the deferred P2.2
    /// multi-scene pass.)
    pub fn scene_leaf(&self) -> Option<&LeafPane> {
        let focused = self.focused_pane();
        if focused.content == datum_gui_protocol::PaneContent::Board {
            return Some(focused);
        }
        self.panes
            .iter()
            .find(|p| p.content == datum_gui_protocol::PaneContent::Board)
    }

    /// The `PaneId` of the leaf that renders the live board scene, if any.
    pub fn scene_leaf_id(&self) -> Option<datum_gui_protocol::PaneId> {
        self.scene_leaf().map(|p| p.id)
    }

    /// The leaf pane whose whole `frame` (header + canvas) contains screen point
    /// `(x, y)`, if any. This is the click-to-focus hit map (decision 021): a
    /// pointer press in a non-focused pane's frame swaps focus to it. Returns
    /// `None` for points outside every pane (sidebars, dock, menu bar), so the
    /// caller falls through to its normal (focused-pane) behavior. Purely a
    /// screen-space rect test over the already-solved tile frames — no model
    /// mutation; pane focus is consumer view state, never journaled.
    pub fn leaf_at(&self, x: f32, y: f32) -> Option<datum_gui_protocol::PaneId> {
        self.panes
            .iter()
            .find(|pane| pane.rect.frame.contains(x, y))
            .map(|pane| pane.id)
    }

    /// The split divider whose grab-widened gutter contains screen point `(x, y)`,
    /// if any — backs divider-drag resize (decision 021). The 1px hairline is
    /// widened by a small grab margin so the gutter is easy to grab with a pointer.
    /// Dividers never overlap, so at most one matches; returns the first in tree
    /// order. Pure screen-space test — no mutation; layout is consumer view state.
    pub fn divider_at(&self, x: f32, y: f32) -> Option<&PaneDivider> {
        const GRAB_MARGIN: f32 = 4.0;
        self.dividers.iter().find(|d| {
            x >= d.rect.x - GRAB_MARGIN
                && x <= d.rect.x + d.rect.width + GRAB_MARGIN
                && y >= d.rect.y - GRAB_MARGIN
                && y <= d.rect.y + d.rect.height + GRAB_MARGIN
        })
    }
}

/// Hairline divider gutter width between two split siblings (same 1px as the
/// former fixed two-pane divider).
const PANE_DIVIDER_W: f32 = 1.0;

/// Recursively tile `node` into `frame`, pushing one `LeafPane` per leaf and one
/// divider per Split. A `Vertical` split places its children side-by-side (left
/// `first` | right `second`) with a vertical divider; a `Horizontal` split stacks
/// them (top `first` / bottom `second`) with a horizontal divider. Each Split's
/// `ratio` governs the `first` child's share of the available (post-divider) span.
fn tile_pane_node(
    node: &datum_gui_protocol::PaneNode,
    frame: RectPx,
    panes: &mut Vec<LeafPane>,
    dividers: &mut Vec<PaneDivider>,
    path: &mut Vec<datum_gui_protocol::SplitChild>,
) {
    use datum_gui_protocol::{PaneNode, SplitChild, SplitOrientation};
    match node {
        PaneNode::Leaf { id, content } => {
            panes.push(LeafPane {
                id: *id,
                content: *content,
                rect: PaneRect::from_frame(frame),
            });
        }
        PaneNode::Split {
            orientation,
            ratio,
            first,
            second,
        } => match orientation {
            SplitOrientation::Vertical => {
                let divider_w = PANE_DIVIDER_W.min(frame.width);
                let first_w = ((frame.width - divider_w) * *ratio).max(0.0);
                let first_frame = RectPx {
                    x: frame.x,
                    y: frame.y,
                    width: first_w,
                    height: frame.height,
                };
                let divider = RectPx {
                    x: frame.x + first_w,
                    y: frame.y,
                    width: divider_w,
                    height: frame.height,
                };
                let second_x = frame.x + first_w + divider_w;
                let second_frame = RectPx {
                    x: second_x,
                    y: frame.y,
                    width: (frame.x + frame.width - second_x).max(0.0),
                    height: frame.height,
                };
                path.push(SplitChild::First);
                tile_pane_node(first, first_frame, panes, dividers, path);
                path.pop();
                dividers.push(PaneDivider {
                    rect: divider,
                    split_frame: frame,
                    orientation: *orientation,
                    path: path.clone(),
                });
                path.push(SplitChild::Second);
                tile_pane_node(second, second_frame, panes, dividers, path);
                path.pop();
            }
            SplitOrientation::Horizontal => {
                let divider_h = PANE_DIVIDER_W.min(frame.height);
                let first_h = ((frame.height - divider_h) * *ratio).max(0.0);
                let first_frame = RectPx {
                    x: frame.x,
                    y: frame.y,
                    width: frame.width,
                    height: first_h,
                };
                let divider = RectPx {
                    x: frame.x,
                    y: frame.y + first_h,
                    width: frame.width,
                    height: divider_h,
                };
                let second_y = frame.y + first_h + divider_h;
                let second_frame = RectPx {
                    x: frame.x,
                    y: second_y,
                    width: frame.width,
                    height: (frame.y + frame.height - second_y).max(0.0),
                };
                path.push(SplitChild::First);
                tile_pane_node(first, first_frame, panes, dividers, path);
                path.pop();
                dividers.push(PaneDivider {
                    rect: divider,
                    split_frame: frame,
                    orientation: *orientation,
                    path: path.clone(),
                });
                path.push(SplitChild::Second);
                tile_pane_node(second, second_frame, panes, dividers, path);
                path.pop();
            }
        },
    }
}

/// The content of the leaf named `id`, if present in `node`'s subtree.
fn leaf_pane_content(
    node: &datum_gui_protocol::PaneNode,
    id: datum_gui_protocol::PaneId,
) -> Option<datum_gui_protocol::PaneContent> {
    use datum_gui_protocol::PaneNode;
    match node {
        PaneNode::Leaf {
            id: leaf_id,
            content,
        } => (*leaf_id == id).then_some(*content),
        PaneNode::Split { first, second, .. } => {
            leaf_pane_content(first, id).or_else(|| leaf_pane_content(second, id))
        }
    }
}

impl ShellLayout {
    pub fn for_surface(
        physical_width: u32,
        physical_height: u32,
        scale_factor: f32,
        dock_height_px: Option<u32>,
    ) -> Self {
        let scale = scale_factor.max(0.01);
        let logical_width = ((physical_width as f32) / scale).round().max(1.0) as u32;
        let logical_height = ((physical_height as f32) / scale).round().max(1.0) as u32;
        Self::for_window(logical_width, logical_height, dock_height_px).scale_by(scale)
    }

    pub fn for_window(width: u32, height: u32, dock_height_px: Option<u32>) -> Self {
        let width = width as f32;
        let height = height as f32;
        let menu_height = design_tokens::spacing::SP_07 + 1.0;
        let status_height = design_tokens::spacing::SP_06 + design_tokens::spacing::SP_01;
        let left_width = 224.0_f32.min(width * 0.3);
        let right_width = 296.0_f32.min(width * 0.35);
        let bottom_height = match dock_height_px {
            Some(h) => (h as f32).clamp(design_tokens::spacing::SP_07, height * 0.6),
            None => design_tokens::spacing::SP_07.min(height * 0.25),
        };
        if let Some(layout) = solve_shell_layout_with_taffy(
            width,
            height,
            menu_height,
            left_width,
            right_width,
            bottom_height,
            status_height,
        ) {
            return layout;
        }
        // Taffy is the adopted shell solver; keep a manual fallback so a
        // malformed runtime input cannot prevent the GUI from opening.
        Self {
            top_menu_bar: RectPx {
                x: 0.0,
                y: 0.0,
                width,
                height: menu_height,
            },
            left_sidebar: RectPx {
                x: 0.0,
                y: menu_height,
                width: left_width,
                height: (height - menu_height - bottom_height - status_height).max(0.0),
            },
            viewport: RectPx {
                x: left_width,
                y: menu_height,
                width: (width - left_width - right_width).max(0.0),
                height: (height - menu_height - bottom_height - status_height).max(0.0),
            },
            right_sidebar: RectPx {
                x: (width - right_width).max(0.0),
                y: menu_height,
                width: right_width,
                height: (height - menu_height - bottom_height - status_height).max(0.0),
            },
            bottom_strip: RectPx {
                x: 0.0,
                y: height - bottom_height - status_height,
                width,
                height: bottom_height,
            },
            status_bar: RectPx {
                x: 0.0,
                y: height - status_height,
                width,
                height: status_height,
            },
        }
    }

    /// Tile the resolved central `viewport` per the `WorkspaceLayout` tree into
    /// the set of leaf panes plus divider gutters. A pure post-split derived
    /// AFTER the taffy/fallback solve and AFTER `scale_by`, so the world scene,
    /// gpu scissor, and hit-testing all follow the FOCUSED leaf with no further
    /// edits. If `layout.zoomed == Some(id)`, that leaf fills the whole viewport
    /// and no others/dividers are emitted (transient maximize; the tree is
    /// untouched). The DEFAULT tree (vertical Board|Schematic at 0.5, Board
    /// focused) reproduces the former fixed two-pane split pixel-for-pixel.
    pub fn viewport_panes(&self, layout: &datum_gui_protocol::WorkspaceLayout) -> ViewportPanes {
        let mut panes = Vec::new();
        let mut dividers = Vec::new();
        if let Some(zoomed) = layout.zoomed {
            // Maximize: the zoomed leaf fills the viewport; no siblings, no
            // dividers. The tree is never mutated — this is transient view state.
            let content =
                leaf_pane_content(&layout.root, zoomed).unwrap_or(datum_gui_protocol::PaneContent::Board);
            panes.push(LeafPane {
                id: zoomed,
                content,
                rect: PaneRect::from_frame(self.viewport),
            });
        } else {
            tile_pane_node(
                &layout.root,
                self.viewport,
                &mut panes,
                &mut dividers,
                &mut Vec::new(),
            );
        }
        ViewportPanes {
            panes,
            dividers,
            focused: layout.focused,
        }
    }

    pub fn scene_viewport(&self, layout: &datum_gui_protocol::WorkspaceLayout) -> RectPx {
        // The world board scene renders into the BOARD leaf's canvas — the one that
        // owns the live PCB — NOT merely whichever leaf is focused. Returning that
        // scene rect means RetainedScene's reference_projection, gpu.rs
        // scissor/uniform, and `world_point_at_screen` all follow the board pane, so
        // the PCB stays visible in its pane while another pane (e.g. Schematic) is
        // focused. Falls back to the focused rect only when no board leaf exists
        // (nothing renders there — the board scene is gated off in that case).
        let panes = self.viewport_panes(layout);
        panes
            .scene_leaf()
            .map(|leaf| leaf.rect.scene)
            .unwrap_or_else(|| panes.focused_scene())
    }

    fn scale_by(self, scale: f32) -> Self {
        Self {
            top_menu_bar: self.top_menu_bar.scale_by(scale),
            viewport: self.viewport.scale_by(scale),
            left_sidebar: self.left_sidebar.scale_by(scale),
            right_sidebar: self.right_sidebar.scale_by(scale),
            bottom_strip: self.bottom_strip.scale_by(scale),
            status_bar: self.status_bar.scale_by(scale),
        }
    }
}

fn solve_shell_layout_with_taffy(
    width: f32,
    height: f32,
    menu_height: f32,
    left_width: f32,
    right_width: f32,
    bottom_height: f32,
    status_height: f32,
) -> Option<ShellLayout> {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let top_menu_bar = taffy
        .new_leaf(Style {
            grid_row: line(1),
            grid_column: span(3),
            ..Default::default()
        })
        .ok()?;
    let left_sidebar = taffy
        .new_leaf(Style {
            grid_row: line(2),
            grid_column: line(1),
            ..Default::default()
        })
        .ok()?;
    let viewport = taffy
        .new_leaf(Style {
            grid_row: line(2),
            grid_column: line(2),
            ..Default::default()
        })
        .ok()?;
    let right_sidebar = taffy
        .new_leaf(Style {
            grid_row: line(2),
            grid_column: line(3),
            ..Default::default()
        })
        .ok()?;
    let bottom_strip = taffy
        .new_leaf(Style {
            grid_row: line(3),
            grid_column: span(3),
            ..Default::default()
        })
        .ok()?;
    let status_bar = taffy
        .new_leaf(Style {
            grid_row: line(4),
            grid_column: span(3),
            ..Default::default()
        })
        .ok()?;
    let root = taffy
        .new_with_children(
            Style {
                display: Display::Grid,
                size: Size {
                    width: length(width),
                    height: length(height),
                },
                grid_template_columns: vec![length(left_width), fr(1.0), length(right_width)],
                grid_template_rows: vec![
                    length(menu_height),
                    fr(1.0),
                    length(bottom_height),
                    length(status_height),
                ],
                ..Default::default()
            },
            &[
                top_menu_bar,
                left_sidebar,
                viewport,
                right_sidebar,
                bottom_strip,
                status_bar,
            ],
        )
        .ok()?;
    taffy
        .compute_layout(
            root,
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
        )
        .ok()?;

    let rect_for = |tree: &TaffyTree<()>, node| -> Option<RectPx> {
        let layout = tree.layout(node).ok()?;
        Some(RectPx {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        })
    };
    Some(ShellLayout {
        top_menu_bar: rect_for(&taffy, top_menu_bar)?,
        left_sidebar: rect_for(&taffy, left_sidebar)?,
        viewport: rect_for(&taffy, viewport)?,
        right_sidebar: rect_for(&taffy, right_sidebar)?,
        bottom_strip: rect_for(&taffy, bottom_strip)?,
        status_bar: rect_for(&taffy, status_bar)?,
    })
}

