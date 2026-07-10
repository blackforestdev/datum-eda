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

/// The central viewport tiled into two side-by-side panes (Board pane A, left |
/// Schematic pane B, right) with a hairline `divider` gutter between them. This
/// is the Phase-2 split-view first slice: a real two-pane LAYOUT derived from
/// the resolved `viewport` as a pure post-split, so neither the Taffy solver nor
/// `scale_by` becomes pane-aware.
/// Which pane currently owns focus. Focus is the single source of truth for both
/// the per-pane header chrome (accent frame + focus dot + active tools) and — via
/// context-follows-focus — which document the Inspector/Layers side panels read
/// (docs/gui/DATUM_GUI_DESIGN_SPEC.md → Workspace & Mode Model). This slice pins
/// focus to pane A (the Board document); focus-switch input and the toggle for
/// the unfocused document are deferred to a later slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
    /// Pane A — the Board · Layout document.
    Board,
    /// Pane B — the Schematic · Sheet document (placeholder this slice).
    Schematic,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportPanes {
    pub pane_a: PaneRect,
    pub pane_b: PaneRect,
    pub divider: RectPx,
}

impl ViewportPanes {
    /// The focused pane this slice: pane A (Board). This is the seam
    /// context-follows-focus reads — the side panels reflect the focused pane's
    /// document. Making it a single accessor (not a scattered literal) keeps the
    /// mechanism real while the focus-switch toggle is deferred.
    pub fn focused_document(&self) -> FocusedPane {
        FocusedPane::Board
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

    /// Split the resolved central `viewport` into two panes (Board A | Schematic
    /// B) with a hairline divider gutter. A pure post-split derived AFTER the
    /// taffy/fallback solve and AFTER `scale_by`, so the world scene, gpu
    /// scissor, and hit-testing all follow pane A with no further edits. The
    /// ~50/50 ratio is a fidelity detail flagged for owner-approval against
    /// board-editor.html (deferred), not fixed here.
    pub fn viewport_panes(&self) -> ViewportPanes {
        let v = self.viewport;
        const DIVIDER_W: f32 = 1.0;
        // Guard against zero/negative pane widths at small widths: never let the
        // divider overflow a narrow viewport.
        let divider_w = DIVIDER_W.min(v.width);
        let pane_a_w = ((v.width - divider_w) * 0.5).max(0.0);
        let pane_a_frame = RectPx {
            x: v.x,
            y: v.y,
            width: pane_a_w,
            height: v.height,
        };
        let divider = RectPx {
            x: v.x + pane_a_w,
            y: v.y,
            width: divider_w,
            height: v.height,
        };
        let pane_b_x = v.x + pane_a_w + divider_w;
        let pane_b_frame = RectPx {
            x: pane_b_x,
            y: v.y,
            width: (v.x + v.width - pane_b_x).max(0.0),
            height: v.height,
        };
        ViewportPanes {
            pane_a: PaneRect::from_frame(pane_a_frame),
            pane_b: PaneRect::from_frame(pane_b_frame),
            divider,
        }
    }

    pub fn scene_viewport(&self) -> RectPx {
        // The world board scene renders into pane A's canvas (the focused
        // left-half Board pane). Returning pane A's scene here means
        // RetainedScene's reference_projection, gpu.rs scissor/uniform, and
        // `world_point_at_screen` all follow pane A with no further change.
        self.viewport_panes().pane_a.scene
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

