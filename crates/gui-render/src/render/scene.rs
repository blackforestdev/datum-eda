impl PreparedScene {
    pub fn from_workspace(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        camera: CameraState,
        retained_scene: &RetainedScene,
    ) -> Self {
        Self::from_workspace_for_surface(state, width, height, 1.0, camera, retained_scene)
    }

    pub fn from_workspace_for_surface(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
        camera: CameraState,
        retained_scene: &RetainedScene,
    ) -> Self {
        let scale = scale_factor.max(0.01);
        let layout = ShellLayout::for_surface(width, height, scale, dock_height_for_state(state));
        let mut panel_quads = Vec::new();
        let mut menu_overlay_quads = Vec::new();
        let mut menu_overlay_text_runs = Vec::new();
        let mut viewport_underlay_quads = Vec::new();
        let mut viewport_overlay_quads = Vec::new();
        let mut text_runs = Vec::new();
        let mut hit_regions = Vec::new();
        let scene_viewport = layout.scene_viewport(&state.ui.layout);
        // The board scene renders only when a Board leaf exists to host it (the
        // common Board|Schematic layout always has one; an all-Schematic layout does
        // not). Independent of focus, so the PCB persists in its pane while another
        // pane is focused.
        let board_scene_active = layout
            .viewport_panes(&state.ui.layout)
            .scene_leaf()
            .is_some();

        panel_quads.push(Quad::from_rect(layout.top_menu_bar, APP_BG));
        panel_quads.push(Quad::from_rect(layout.left_sidebar, APP_BG));
        panel_quads.push(Quad::from_rect(layout.right_sidebar, APP_BG));
        panel_quads.push(Quad::from_rect(layout.bottom_strip, APP_BG));
        panel_quads.push(Quad::from_rect(layout.status_bar, PANEL_BG));
        viewport_underlay_quads.push(Quad::from_rect(layout.viewport, VIEWPORT_BG));

        render_phase1_shell_chrome(state, &layout, &mut panel_quads, &mut text_runs);
        render_menu_bar(
            state,
            &layout,
            &mut panel_quads,
            &mut menu_overlay_quads,
            &mut menu_overlay_text_runs,
            &mut text_runs,
            &mut hit_regions,
        );
        render_side_panels(
            state,
            &layout,
            &mut panel_quads,
            &mut text_runs,
            &mut hit_regions,
        );
        render_bottom_tabs(
            state,
            &layout,
            &mut panel_quads,
            &mut text_runs,
            &mut hit_regions,
        );
        // Single-live-scene: the board scene (substrate + grid underlay, selection
        // overlay, and world PCB) renders into the BOARD leaf's rect whenever a
        // board leaf exists — independent of which pane is focused, so the PCB stays
        // visible in its pane while a Schematic pane is focused. Panes that are not
        // the board scene leaf show their own placeholder (render_viewport_panes).
        if board_scene_active {
            render_scene(
                state,
                &layout,
                scene_viewport,
                camera,
                &mut viewport_underlay_quads,
                &mut viewport_overlay_quads,
                &mut text_runs,
                &mut hit_regions,
            );
        }
        render_marking_menu(
            state,
            &layout,
            &mut panel_quads,
            &mut text_runs,
            &mut hit_regions,
        );
        if (scale - 1.0).abs() > f32::EPSILON {
            scale_text_run_sizes(&mut text_runs, scale);
            scale_text_run_sizes(&mut menu_overlay_text_runs, scale);
        }
        let panel_vertices = quads_to_vertices(&panel_quads);
        let menu_overlay_vertices = quads_to_vertices(&menu_overlay_quads);
        let viewport_underlay_vertices = quads_to_vertices(&viewport_underlay_quads);
        let viewport_overlay_vertices = quads_to_vertices(&viewport_overlay_quads);
        let visible_world_ranges = if board_scene_active {
            retained_scene.visible_world_ranges(state)
        } else {
            Vec::new()
        };

        Self {
            layout,
            hit_regions,
            scene_viewport,
            scene_bounds: state.scene.bounds.clone(),
            camera,
            panel_vertices,
            menu_overlay_vertices,
            menu_overlay_text_runs,
            viewport_underlay_vertices,
            viewport_overlay_vertices,
            visible_world_ranges,
            text_runs,
        }
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<&HitTarget> {
        self.hit_regions
            .iter()
            .rev()
            .find(|region| region.rect.contains(x, y))
            .map(|region| &region.target)
    }

    pub fn world_point_at_screen(&self, x: f32, y: f32) -> Option<PointNm> {
        if !self.scene_viewport.contains(x, y) {
            return None;
        }
        let board_field = inset_rect(self.scene_viewport, 10.0, 10.0, 10.0, 10.0);
        let projection = Projection::new(board_field, &self.scene_bounds, self.camera);
        Some(projection.screen_to_world(x, y))
    }

    fn panel_vertices(&self) -> &[Vertex] {
        &self.panel_vertices
    }

    /// Top-overlay quads for the open menu dropdown, composited AFTER the
    /// scissored viewport passes (see gpu.rs) so work-pane content cannot
    /// overpaint them. Empty when no menu is open. Menu-bar TITLES are NOT here —
    /// they live in `panel_vertices`; only the dropdown body/rows land in this
    /// sink.
    fn menu_overlay_vertices(&self) -> &[Vertex] {
        &self.menu_overlay_vertices
    }

    /// The open dropdown's OWN text (item labels, shortcuts, fallback-icon
    /// glyphs), rendered in a dedicated pass AFTER the dropdown card so it sits
    /// crisply on top of it while the main text pass (drawn before the card) is
    /// fully occluded by the card. Empty when no menu is open. Menu-bar TITLE text
    /// is NOT here — titles live in the bar and are never occluded, so they stay
    /// in the main `text_runs`.
    fn menu_overlay_text_runs(&self) -> &[TextRun] {
        &self.menu_overlay_text_runs
    }

    fn viewport_underlay_vertices(&self) -> &[Vertex] {
        &self.viewport_underlay_vertices
    }

    fn viewport_overlay_vertices(&self) -> &[Vertex] {
        &self.viewport_overlay_vertices
    }

    fn visible_world_ranges(&self) -> &[Range<u32>] {
        &self.visible_world_ranges
    }
}

thread_local! {
    /// Per-thread count of ACTUAL world-scene resolves — every time the retained
    /// world buffer is rebuilt from scratch (a cache MISS). A warm workspace pane
    /// op (focus-switch / split / close / zoom / preset) reuses the already-
    /// resolved retained scene and MUST NOT bump this: that is the P2.1b "clicking
    /// an adjacent viewport to make it live has no noticeable lag" latency gate
    /// (decision 021). Thread-local so parallel tests never perturb each other's
    /// baseline; the increment is a single Cell add per full resolve (resolves are
    /// rare), so it is always compiled, not test-gated.
    static RETAINED_RESOLVE_COUNT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Read the current thread's world-scene resolve counter (see
/// `RETAINED_RESOLVE_COUNT`). Intended for the pane-op latency assertion: warm a
/// scene, record this, run pane ops, and assert it is unchanged (zero re-resolve).
pub fn retained_scene_resolve_count() -> u64 {
    RETAINED_RESOLVE_COUNT.with(|count| count.get())
}

impl RetainedScene {
    pub fn from_workspace(state: &ReviewWorkspaceState, width: u32, height: u32) -> Self {
        Self::from_workspace_for_surface(state, width, height, 1.0)
    }

    pub fn from_workspace_for_surface(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Self {
        // This is the single world-scene resolve entry point; count the miss.
        // (`reference_projection` below is derived here and nowhere else, so a pane
        // op that reuses the retained scene provably never recomputes it.)
        RETAINED_RESOLVE_COUNT.with(|count| count.set(count.get() + 1));
        let started = std::time::Instant::now();
        let layout =
            ShellLayout::for_surface(width, height, scale_factor, dock_height_for_state(state));
        let scene_viewport = layout.scene_viewport(&state.ui.layout);
        let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
        let reference_projection = Projection::new(
            board_field,
            &state.scene.bounds,
            CameraState::fit_to_bounds(&state.scene.bounds),
        );
        let mut world_quads = Vec::new();
        let mut world_batches = Vec::new();
        let mut world_hit_regions = Vec::new();
        let geometry_started = std::time::Instant::now();
        push_retained_scene_geometry(&mut world_quads, &state.scene, &reference_projection, state);
        if !world_quads.is_empty() {
            world_batches.push(RetainedWorldBatch {
                layer_id: None,
                start: 0,
                len: (world_quads.len() * 6) as u32,
            });
        }
        let board_graphics_started = std::time::Instant::now();
        let board_graphics_before = world_quads.len();
        push_retained_board_text_geometry_batches(
            &mut world_quads,
            &mut world_batches,
            &state.scene,
            &reference_projection,
            state,
        );
        push_retained_board_graphic_batches(
            &mut world_quads,
            &mut world_batches,
            &state.scene,
            &reference_projection,
            state,
        );
        trace_render_timing(format!(
            "retained text+board_graphics batches={}ms/{}q",
            board_graphics_started.elapsed().as_millis(),
            world_quads.len().saturating_sub(board_graphics_before)
        ));
        let geometry_elapsed = geometry_started.elapsed();
        let hits_started = std::time::Instant::now();
        push_retained_world_hit_regions(&mut world_hit_regions, &state.scene, state);
        let hits_elapsed = hits_started.elapsed();
        let vertex_started = std::time::Instant::now();
        let world_vertices = quads_to_vertices(&world_quads);
        let vertex_elapsed = vertex_started.elapsed();
        trace_render_timing(format!(
            "retained total={}ms geometry={}ms hits={}ms vertices={}ms quads={} vertices={} hit_regions={}",
            started.elapsed().as_millis(),
            geometry_elapsed.as_millis(),
            hits_elapsed.as_millis(),
            vertex_elapsed.as_millis(),
            world_quads.len(),
            world_vertices.len(),
            world_hit_regions.len()
        ));
        Self {
            world_vertices,
            world_batches,
            world_hit_regions,
        }
    }

    pub fn world_vertices(&self) -> &[Vertex] {
        &self.world_vertices
    }

    fn visible_world_ranges(&self, state: &ReviewWorkspaceState) -> Vec<Range<u32>> {
        // Note: whether the board world renders AT ALL (i.e. a board leaf exists to
        // host it) is gated by the caller in `from_workspace_for_surface`; here we
        // only filter which layer batches are visible. The scene is scissored to the
        // BOARD leaf's rect, so the PCB stays in its pane independent of focus.
        if !authored_visible(state) {
            return Vec::new();
        }
        self.world_batches
            .iter()
            .filter(|batch| {
                batch
                    .layer_id
                    .as_deref()
                    .is_none_or(|layer_id| layer_visible(state, layer_id))
            })
            .map(|batch| batch.start..batch.start + batch.len)
            .collect()
    }

    pub fn hit_test_authored_world(
        &self,
        point: PointNm,
        state: &ReviewWorkspaceState,
    ) -> Option<&HitTarget> {
        // Board hit-testing is scoped by scene-rect containment upstream
        // (`world_point_at_screen` returns None outside the board leaf's rect), so a
        // click in the Schematic pane never reaches board geometry — and a click in
        // the board pane DOES hit it even while another pane is focused (view/inspect
        // the board while working elsewhere).
        if !authored_visible(state) {
            return None;
        }
        self.world_hit_regions
            .iter()
            .rev()
            .find(|region| {
                region
                    .layer_id
                    .as_deref()
                    .is_none_or(|layer_id| layer_visible(state, layer_id))
                    && region.shape.contains(point)
            })
            .map(|region| &region.target)
    }
}

impl WorldHitShape {
    fn contains(&self, point: PointNm) -> bool {
        match self {
            Self::Rect(rect) => point_in_rect(point, *rect),
            Self::Polyline {
                path,
                half_width_nm,
            } => polyline_contains_world_point(path, point, *half_width_nm),
            Self::Polygon(path) => point_in_polygon_world(path, point),
            Self::Circle { center, radius_nm } => {
                let dx = point.x as f32 - center.x as f32;
                let dy = point.y as f32 - center.y as f32;
                dx * dx + dy * dy <= radius_nm * radius_nm
            }
        }
    }
}

fn dock_height_for_state(state: &ReviewWorkspaceState) -> Option<u32> {
    if state.ui.active_dock_tab.is_some() {
        Some(state.ui.dock_height_px)
    } else {
        None
    }
}

fn render_phase1_shell_chrome(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    // Menu bar carries only a bottom hairline (Design Book .menubar
    // border-bottom), never a boxed 4-sided outline.
    panel_quads.push(Quad::from_rect(
        RectPx {
            x: layout.top_menu_bar.x,
            y: layout.top_menu_bar.y + layout.top_menu_bar.height - 1.0,
            width: layout.top_menu_bar.width,
            height: 1.0,
        },
        PANEL_CARD_BORDER,
    ));
    // Brand wordmark: three runs on one baseline — "Datum" / accent middot /
    // "EDA" — advancing x by each measured run width so the middot is truly
    // colored and kerned, not a full "Datum EDA" string.
    let brand_size = 14.0;
    let brand_y = layout.top_menu_bar.y + design_tokens::spacing::SP_03;
    let mut brand_x = layout.top_menu_bar.x + design_tokens::spacing::SP_04;
    for (run, color) in [
        ("Datum", TEXT_PRIMARY),
        ("\u{00B7}", TEXT_ACCENT),
        ("EDA", TEXT_PRIMARY),
    ] {
        draw_text(
            run,
            brand_x,
            brand_y,
            brand_size,
            color,
            TextFace::UiStrong,
            text_runs,
        );
        brand_x += estimated_text_run_width_px(run, brand_size, TextFace::UiStrong) - 16.0;
    }
    // Rev pill: "{project} · rev {short-revision}" in a SURFACE_01 quad with a
    // BORDER_SUBTLE border, right-aligned to the menubar right edge.
    let short_rev: String = state.scene.source_revision.chars().take(6).collect();
    let rev_label = if short_rev.is_empty() {
        truncate_text(&state.scene.project_name, 30)
    } else {
        format!("{} \u{00B7} rev {}", truncate_text(&state.scene.project_name, 24), short_rev)
    };
    let rev_text_w =
        estimated_text_run_width_px(&rev_label, design_tokens::typography::DATA_SIZE, TextFace::Mono)
            - 16.0;
    let pill_pad_x = design_tokens::spacing::SP_03;
    let pill_pad_y = design_tokens::spacing::SP_02;
    let pill_h = design_tokens::typography::DATA_SIZE + pill_pad_y * 2.0;
    let pill_w = rev_text_w + pill_pad_x * 2.0;
    let pill_x = (layout.top_menu_bar.x + layout.top_menu_bar.width
        - design_tokens::spacing::SP_03
        - pill_w)
        .max(layout.top_menu_bar.x);
    let pill_y = layout.top_menu_bar.y + (layout.top_menu_bar.height - pill_h) * 0.5;
    let pill_rect = RectPx {
        x: pill_x,
        y: pill_y,
        width: pill_w,
        height: pill_h,
    };
    panel_quads.push(Quad::from_rect(pill_rect, PANEL_BG));
    push_rect_border(panel_quads, pill_rect, PANEL_CARD_BORDER, 1.0);
    draw_text(
        &rev_label,
        pill_x + pill_pad_x,
        pill_y + pill_pad_y,
        design_tokens::typography::DATA_SIZE,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );

    render_viewport_panes(layout, &state.ui.layout, panel_quads, text_runs);
    render_status_bar(state, layout, panel_quads, text_runs);
}

/// Segmented status bar (Design Book .status): labelled key/value segments with
/// full-height dividers, a flex gap, and a right-aligned build/version run. The
/// focus value reads accent; a DRC segment reads STATUS_WARN and is hidden at
/// zero findings.
fn render_status_bar(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let sb = layout.status_bar;
    // Single top-edge hairline (no boxed 4-side border).
    panel_quads.push(Quad::from_rect(
        RectPx {
            x: sb.x,
            y: sb.y,
            width: sb.width,
            height: 1.0,
        },
        PANEL_CARD_BORDER,
    ));
    let text_y = sb.y + design_tokens::spacing::SP_02 + 1.0;
    let lab_size = design_tokens::typography::CAPTION_SIZE;
    let val_size = design_tokens::typography::DATA_SIZE;
    let gap = design_tokens::spacing::SP_02 + 2.0;
    let seg_pad = design_tokens::spacing::SP_04;
    let text_w = |text: &str, size: f32| estimated_text_run_width_px(text, size, TextFace::Mono) - 16.0;
    let divider = |panel_quads: &mut Vec<Quad>, x: f32| {
        panel_quads.push(Quad::from_rect(
            RectPx {
                x,
                y: sb.y,
                width: 1.0,
                height: sb.height,
            },
            PANEL_CARD_BORDER,
        ));
    };

    let sel = match &state.selection {
        SelectionTarget::None => "none".to_string(),
        SelectionTarget::ReviewAction(id)
        | SelectionTarget::AuthoredObject(id)
        | SelectionTarget::CheckFinding(id) => truncate_text(suffix_id(id), 8),
    };
    let tool = workspace_tool_label(state.tool);
    let layers = state.scene.layers.len().to_string();
    // Reflect the actually-focused document, not a hardcoded value — focusing the
    // Schematic pane must read "Schematic" here (context-follows-focus).
    let focus_label = match state.ui.layout.focused_content() {
        datum_gui_protocol::PaneContent::Board => "Board",
        datum_gui_protocol::PaneContent::Schematic => "Schematic",
    };
    let left: [(&str, &str, [f32; 3]); 4] = [
        ("focus", focus_label, TEXT_ACCENT),
        ("Tool", tool, TEXT_SECONDARY),
        ("Sel", sel.as_str(), TEXT_SECONDARY),
        ("Layers", layers.as_str(), TEXT_SECONDARY),
    ];
    let mut x = sb.x + seg_pad;
    for (i, (label, value, color)) in left.iter().enumerate() {
        if i > 0 {
            divider(panel_quads, x - seg_pad * 0.5);
        }
        draw_text(label, x, text_y, lab_size, TEXT_MUTED, TextFace::Mono, text_runs);
        let lw = text_w(label, lab_size) + gap;
        draw_text(value, x + lw, text_y, val_size, *color, TextFace::Mono, text_runs);
        x += lw + text_w(value, val_size) + seg_pad;
    }

    // Right cluster (right-to-left): version, rev, DRC.
    let version = "Datum EDA \u{2014} design pass";
    let mut rx = sb.x + sb.width - 13.0 - text_w(version, val_size);
    draw_text(version, rx, text_y, val_size, TEXT_MUTED, TextFace::Mono, text_runs);

    let short_rev: String = state.scene.source_revision.chars().take(6).collect();
    if !short_rev.is_empty() {
        let lw = text_w("rev", lab_size) + gap;
        rx -= seg_pad + lw + text_w(&short_rev, val_size);
        divider(panel_quads, rx - seg_pad * 0.5);
        draw_text("rev", rx, text_y, lab_size, TEXT_MUTED, TextFace::Mono, text_runs);
        draw_text(&short_rev, rx + lw, text_y, val_size, TEXT_SECONDARY, TextFace::Mono, text_runs);
    }

    let findings = state.supervision.checks.finding_count;
    if findings > 0 {
        let drc = format!("DRC {}", findings);
        rx -= seg_pad + text_w(&drc, val_size);
        divider(panel_quads, rx - seg_pad * 0.5);
        draw_text(
            &drc,
            rx,
            text_y,
            val_size,
            design_tokens::chrome::STATUS_WARN,
            TextFace::Mono,
            text_runs,
        );
    }
}

/// Render the workspace pane tiling described by the `WorkspaceLayout` tree:
/// walk its leaf set (generalized to N leaves, nested H/V splits, and zoom) and
/// draw one header per leaf, each divider, and a per-content canvas. Every leaf
/// carries real chrome (header band, title, tool cluster, focus differentiation);
/// the focused leaf gets the accent frame + focus dot. The BOARD scene leaf's
/// canvas is the world scene (the single retained world buffer, scissored to its
/// `scene_viewport`) — it renders the PCB regardless of focus, so the board stays
/// visible while another pane is focused. Every other pane is a labeled
/// placeholder under the single-live-scene model: a Schematic leaf shows
/// "Schematic (coming)", and any ADDITIONAL Board leaf (not the scene leaf) shows
/// "Inactive \u{00B7} click to focus". No world geometry is emitted for
/// placeholders; idle real-content snapshots are deferred to the P2.2 multi-scene
/// GPU pass.
fn render_viewport_panes(
    layout: &ShellLayout,
    workspace: &datum_gui_protocol::WorkspaceLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let panes = layout.viewport_panes(workspace);
    // The leaf that hosts the live board scene (focus-independent). Its canvas gets
    // the substrate + world PCB; any other board leaf is an inactive placeholder.
    let scene_leaf_id = panes.scene_leaf_id();
    // Tool cluster of the layout pane (the active tool is the first entry). A
    // non-focused pane renders the same cluster dimmed to read as available-but-
    // -inactive rather than owned.
    const PANE_TOOLS: [&str; 5] = ["S", "M", "R", "V", "Z"];
    // Focus is the single source of truth: it drives the per-pane header chrome
    // here and (context-follows-focus) which document the side panels read.
    for leaf in &panes.panes {
        let focused = leaf.id == panes.focused;
        let is_scene_leaf = Some(leaf.id) == scene_leaf_id;
        let title = match leaf.content {
            datum_gui_protocol::PaneContent::Board => "Board \u{00B7} Layout",
            datum_gui_protocol::PaneContent::Schematic => "Schematic \u{00B7} Sheet 1",
        };
        // Board scene leaf: the world scene renders into `scene_viewport`, which is
        // inset 16px inside the pane frame (layout.rs). Paint the whole pane canvas
        // with the board substrate FIRST — before the header + pink focus frame — so
        // that inset margin is a shader quad matching the scene field. Without it the
        // margin fell through to the render-pass CLEAR color; the surface is sRGB and
        // the clear is gamma-encoded while shader quads are not, so the cleared
        // margin resolved ~3x brighter and read as a spurious grey border around the
        // board. Drawing it before the header/frame keeps the pink focus frame and
        // header on top; the scene underlay/world paint over the interior. Done for
        // the board scene leaf whether or not it is focused, so the PCB pane never
        // reverts to a placeholder when focus moves away.
        if is_scene_leaf {
            let pane = &leaf.rect;
            let canvas = RectPx {
                x: pane.frame.x,
                y: pane.header.y + pane.header.height,
                width: pane.frame.width,
                height: (pane.frame.height - pane.header.height).max(0.0),
            };
            panel_quads.push(Quad::from_rect(
                canvas,
                board_surface_color(BoardSurfaceRole::InnerField),
            ));
        }
        render_pane_header(
            &leaf.rect,
            title,
            &PANE_TOOLS,
            focused,
            panel_quads,
            text_runs,
        );
        match leaf.content {
            datum_gui_protocol::PaneContent::Schematic => {
                render_pane_placeholder(&leaf.rect, "Schematic (coming)", panel_quads, text_runs);
            }
            // A Board leaf that is NOT the live scene leaf (e.g. a second board pane
            // in a split) cannot render live under single-live-scene; label it so it
            // reads as intentional, not blank. Idle real-content snapshots land with
            // the P2.2 multi-scene pass.
            datum_gui_protocol::PaneContent::Board if !is_scene_leaf => {
                render_pane_placeholder(
                    &leaf.rect,
                    "Inactive \u{00B7} click to focus",
                    panel_quads,
                    text_runs,
                );
            }
            // Board scene leaf canvas is painted above (before the header/frame);
            // the world PCB renders into it.
            datum_gui_protocol::PaneContent::Board => {}
        }
    }
    // Divider gutters between split siblings. They never overlap a pane frame
    // (each sits in the reserved gutter span), so painting them after the panes
    // is order-safe.
    for divider in &panes.dividers {
        panel_quads.push(Quad::from_rect(*divider, PANEL_CARD_BORDER));
    }
}

/// Paint a non-live pane's placeholder canvas: a VIEWPORT_BG fill beneath the
/// header with a centered `caption`. The world underlay pass is scissored to the
/// single FOCUSED Board leaf's scene, so any other pane (a Schematic leaf, or a
/// non-focused Board leaf under the single-live-scene model) never receives that
/// underlay; we paint its canvas explicitly as a panel quad here so it reads as a
/// real (empty) canvas awaiting content, not bare chrome. No world geometry is
/// emitted (labeled placeholder only).
fn render_pane_placeholder(
    pane: &PaneRect,
    caption: &str,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let canvas = RectPx {
        x: pane.frame.x,
        y: pane.header.y + pane.header.height,
        width: pane.frame.width,
        height: (pane.frame.height - pane.header.height).max(0.0),
    };
    panel_quads.push(Quad::from_rect(canvas, VIEWPORT_BG));
    let cap_size = design_tokens::typography::DATA_SIZE;
    let cap_w = estimated_text_run_width_px(caption, cap_size, TextFace::Mono) - 16.0;
    let cap_x = pane.scene.x + (pane.scene.width - cap_w) * 0.5;
    let cap_y = pane.scene.y + (pane.scene.height - cap_size) * 0.5;
    draw_text(
        caption,
        cap_x,
        cap_y,
        cap_size,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
}

/// Render one pane's header chrome. Focused = today's lightened header (#16181f)
/// + active tool cluster + accent focus dot + inset ACCENT pane frame. Unfocused
/// = muted (recessed) header fill, dimmed tools, no accent frame, no focus dot —
/// so a glance distinguishes the focused document that owns the Inspector/Layers
/// (context-follows-focus) from the passive one.
fn render_pane_header(
    pane: &PaneRect,
    title: &str,
    tools: &[&str],
    focused: bool,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let header = pane.header;
    // Focused pane: lightened header background (~#16181f). Unfocused: a darker,
    // recessed fill (BG_BASE) so it reads as the passive document.
    const PANE_HEADER_FOCUS: [f32; 3] = [0x16 as f32 / 255.0, 0x18 as f32 / 255.0, 0x1F as f32 / 255.0];
    let header_fill = if focused {
        PANE_HEADER_FOCUS
    } else {
        design_tokens::chrome::BG_BASE
    };
    panel_quads.push(Quad::from_rect(header, header_fill));
    // Bottom hairline only (no boxed outline), on both panes.
    panel_quads.push(Quad::from_rect(
        RectPx {
            x: header.x,
            y: header.y + header.height - 1.0,
            width: header.width,
            height: 1.0,
        },
        PANEL_CARD_BORDER,
    ));
    // Small pane glyph, then the pane title with a middot separator. The glyph
    // reads accent when focused, muted-border when passive.
    let glyph = RectPx {
        x: header.x + design_tokens::spacing::SP_04,
        y: header.y + 11.0,
        width: 11.0,
        height: 9.0,
    };
    panel_quads.push(Quad::from_rect(
        glyph,
        if focused {
            REVIEW_ROW_ACTIVE_BG
        } else {
            design_tokens::chrome::SURFACE_02
        },
    ));
    push_rect_border(
        panel_quads,
        glyph,
        if focused { TEXT_ACCENT } else { PANEL_CARD_BORDER },
        1.0,
    );
    let title_x = glyph.x + glyph.width + design_tokens::spacing::SP_03;
    draw_text(
        title,
        title_x,
        header.y + design_tokens::spacing::SP_03,
        design_tokens::typography::DATA_SIZE,
        if focused { TEXT_PRIMARY } else { TEXT_MUTED },
        TextFace::Mono,
        text_runs,
    );
    // Tool cluster after the measured pane-title width. Buttons render on the
    // interactive SURFACE_02; the focused pane's active tool gets the accent
    // tint + accent border. On the unfocused pane the whole cluster is dimmed
    // (recessed fill, subtle border, muted label) to read as passive.
    // (Letter placeholders are an accepted Phase-1 interim: the chrome path
    // only emits axis-aligned quads, so the five vector tool icons and true
    // 5px corner rounding are a tracked fidelity gap.)
    let title_w = estimated_text_run_width_px(
        title,
        design_tokens::typography::DATA_SIZE,
        TextFace::Mono,
    ) - 16.0;
    let mut x = title_x + title_w + design_tokens::spacing::SP_04;
    for (i, label) in tools.iter().enumerate() {
        // The active tool is the first entry, and only on the focused pane.
        let active = focused && i == 0;
        let rect = RectPx {
            x,
            y: header.y + 3.0,
            width: 25.0,
            height: 25.0,
        };
        let fill = if active {
            REVIEW_ROW_ACTIVE_BG
        } else if focused {
            design_tokens::chrome::SURFACE_02
        } else {
            // Passive pane: recessed fill so tools read as dimmed, not clickable.
            design_tokens::chrome::BG_BASE
        };
        panel_quads.push(Quad::from_rect(rect, fill));
        push_rect_border(
            panel_quads,
            rect,
            if active { TEXT_ACCENT } else { PANEL_CARD_BORDER },
            1.0,
        );
        draw_text(
            label,
            rect.x + 8.0,
            rect.y + 7.0,
            design_tokens::typography::CAPTION_SIZE,
            if active { TEXT_ACCENT } else { TEXT_MUTED },
            if active {
                TextFace::UiMedium
            } else {
                TextFace::Mono
            },
            text_runs,
        );
        x += 27.0;
    }
    // Focused pane only: right-aligned accent focus dot (this pane owns the
    // tools + Inspector) and a ~1.5px inset ACCENT frame around the whole pane.
    // The unfocused pane emits neither, so focus is unambiguous.
    if focused {
        let dot = RectPx {
            x: header.x + header.width - design_tokens::spacing::SP_04 - 7.0,
            y: header.y + (header.height - 7.0) * 0.5,
            width: 7.0,
            height: 7.0,
        };
        let dot_points = ellipse_points(
            (dot.x + dot.width * 0.5, dot.y + dot.height * 0.5),
            dot.width,
            dot.height,
            0.0,
            20,
        );
        push_convex_polygon_fill(panel_quads, &dot_points, TEXT_ACCENT);
        let pane_frame = inset_rect(pane.frame, 1.0, 1.0, 1.0, 1.0);
        push_rect_border(panel_quads, pane_frame, TEXT_ACCENT, 1.5);
    }
}

fn render_scene(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    scene_viewport: RectPx,
    camera: CameraState,
    viewport_underlay_quads: &mut Vec<Quad>,
    viewport_overlay_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    push_scene_underlay(
        viewport_underlay_quads,
        &state.scene,
        scene_viewport,
        camera,
    );
    push_scene_overlay_and_hits(
        viewport_overlay_quads,
        &state.scene,
        scene_viewport,
        camera,
        state,
        text_runs,
        hit_regions,
    );
    // (Removed the redundant canvas scene title that collided with the pane
    // header at the viewport top — the pane header "Board / Layout" and the
    // Project panel already name the document, matching the prototype.)
    // (Removed the "ACTIVE <action-id> / NET <name>" review-HUD overlay that
    // bled over the canvas top-left — internal selection state, not designed
    // board-pane chrome. The selection is reflected in the Inspector + status bar.)
    // (Removed the "F FIT / REVIEW NAV / CLICK SELECT / SCROLL ZOOM / ESC CLEAR"
    // keyboard-hint overlay that overflowed across the canvas top — not part of the
    // designed board pane; shortcuts belong in a proper help surface, not a HUD.)
    // (Removed the in-canvas TOOL/ZOOM/SEL status strip and the command-status
    // overlay that painted a PANEL_BG band across the bottom of the canvas.
    // These readouts belong in the global status bar (see M7), not floating on
    // the board field — the canvas stays a clean board surface.)
    let _ = layout;
}


fn push_scene_underlay(
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    scene_viewport: RectPx,
    camera: CameraState,
) {
    // One uniform board substrate fills the ENTIRE viewport. Previously this was a
    // two-tone step — an outer CANVAS band around a 10px-inset InnerField
    // rectangle — whose boundary was only ever masked by the decorative gold edge
    // stroke. With that stroke removed (Bug A), the bare color step read as a
    // spurious grey border, so the field is now a single flat substrate.
    let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
    let projection = Projection::new(board_field, &scene.bounds, camera);
    out.push(Quad::from_rect(
        scene_viewport,
        board_surface_color(BoardSurfaceRole::InnerField),
    ));
    // No decorative board-edge stroke here: the only board outline is the REAL
    // projected Edge.Cuts, drawn from `scene.outline` in the retained world pass
    // (`push_retained_board_graphic_batches`). A fixed viewport-inset frame here
    // was not the true board bounds and read as spurious chrome. The board is still
    // projected into the 10px-inset `board_field` so it keeps a small margin.
    push_scene_grid(out, &projection);
}

fn authored_visible(state: &ReviewWorkspaceState) -> bool {
    state.ui.filters.show_authored
}

fn proposed_visible(state: &ReviewWorkspaceState) -> bool {
    state.ui.filters.show_proposed
}

fn unrouted_visible(state: &ReviewWorkspaceState) -> bool {
    state.ui.filters.show_unrouted
}

fn layer_visible(state: &ReviewWorkspaceState, layer_id: &str) -> bool {
    state
        .ui
        .filters
        .layer_visibility
        .get(layer_id)
        .copied()
        .unwrap_or(true)
}

fn via_visible(state: &ReviewWorkspaceState, start_layer_id: &str, end_layer_id: &str) -> bool {
    layer_visible(state, start_layer_id) || layer_visible(state, end_layer_id)
}

fn pad_copper_layer_ids<'a>(pad: &'a datum_gui_protocol::PadPrimitive) -> Vec<&'a str> {
    if pad.copper_layer_ids.is_empty() {
        vec![pad.layer_id.as_str()]
    } else {
        pad.copper_layer_ids.iter().map(String::as_str).collect()
    }
}

fn pad_visible_on_any_copper_layer(
    state: &ReviewWorkspaceState,
    pad: &datum_gui_protocol::PadPrimitive,
) -> bool {
    pad_copper_layer_ids(pad)
        .into_iter()
        .any(|layer_id| layer_visible(state, layer_id))
}

fn dim_unrelated_active(state: &ReviewWorkspaceState) -> bool {
    if !state.ui.filters.dim_unrelated {
        return false;
    }
    has_review_focus(state) || !matches!(state.selection, SelectionTarget::None)
}

fn is_hovered(state: &ReviewWorkspaceState, object_id: &str) -> bool {
    if !matches!(state.selection, SelectionTarget::None) {
        return false;
    }
    state
        .ui
        .hovered_object_id
        .as_deref()
        .is_some_and(|id| id == object_id)
}

fn unrouted_matches_active_action(
    unrouted: &UnroutedPrimitive,
    state: &ReviewWorkspaceState,
) -> bool {
    state
        .selected_review_action()
        .is_some_and(|action| action.net_uuid == unrouted.net_uuid)
}

fn unrouted_base_color(scene: &BoardReviewSceneV1, unrouted: &UnroutedPrimitive) -> [f32; 3] {
    scene
        .net_display
        .iter()
        .find(|entry| entry.net_uuid == unrouted.net_uuid)
        .map(|entry| entry.airwire_color_rgb)
        .unwrap_or(UNROUTED_BASE)
}

fn selected_component_uuid<'a>(
    scene: &'a BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) -> Option<&'a str> {
    let SelectionTarget::AuthoredObject(object_id) = &state.selection else {
        return None;
    };
    scene.components.iter().find_map(|component| {
        ((&component.object_id == object_id)
            || (format!("component:{}", component.component_uuid) == *object_id))
            .then_some(component.component_uuid.as_str())
    })
}

fn component_is_selection_related(
    component_uuid: &str,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) -> bool {
    selected_component_uuid(scene, state).is_some_and(|selected| selected == component_uuid)
}

fn component_is_selection_active(
    component_uuid: &str,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) -> bool {
    component_is_selection_related(component_uuid, scene, state)
}

fn proposal_preview_affected_ids(state: &ReviewWorkspaceState) -> Vec<&str> {
    state
        .production
        .proposals
        .iter()
        .filter_map(|proposal| proposal.preview.as_ref())
        .flat_map(|preview| preview.affected_objects.iter().map(String::as_str))
        .collect()
}

fn source_object_matches_preview(
    affected_ids: &[&str],
    object_id: &str,
    source_object_uuid: &str,
) -> bool {
    affected_ids
        .iter()
        .any(|affected| *affected == object_id || *affected == source_object_uuid)
}

fn component_matches_preview(
    component_uuid: &str,
    scene: &BoardReviewSceneV1,
    affected_ids: &[&str],
) -> bool {
    scene.components.iter().any(|component| {
        component.component_uuid == component_uuid
            && source_object_matches_preview(
                affected_ids,
                &component.object_id,
                &component.source_object_uuid,
            )
    })
}

fn component_object_id_for_uuid<'a>(
    scene: &'a BoardReviewSceneV1,
    component_uuid: &str,
) -> Option<&'a str> {
    scene.components.iter().find_map(|component| {
        (component.component_uuid == component_uuid).then_some(component.object_id.as_str())
    })
}

