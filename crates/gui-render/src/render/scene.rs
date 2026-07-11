// Per-pane coordinate + hit resolution (UVT-004 keystone) is extracted into a
// real child module — a descendant of this crate-root scope, so it still reaches
// the private `PreparedScene`/`RetainedScene` fields and the private
// `WorldHitShape`/`WorldHitRegion` types. Declared here (not in lib.rs) so lib.rs
// stays untouched and this file carries the extraction. `#[path]` resolves beside
// this physical file (`src/render/`), not the include! host.
#[path = "coordinate_hit.rs"]
mod coordinate_hit;

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
        // P2.2a: describe the companion schematic pass. It is active only when the
        // layout has a Schematic pane AND the workspace carries a projected
        // schematic scene. The camera seeded here is fit-to-schematic-bounds — the
        // INITIAL framing; P2.2d makes the focused schematic pane interactive by
        // overriding this via `set_schematic_camera` with the pane's warm camera
        // (the gui-app render/capture path). Left as fit, this is byte-identical to
        // the pre-P2.2d static default (goldens/tests take this path unchanged).
        let (schematic_scene_viewport, schematic_bounds, schematic_camera) =
            match state.schematic_scene.as_ref() {
                Some(schematic_scene) => (
                    layout.schematic_scene_viewport(&state.ui.layout),
                    schematic_scene.bounds.clone(),
                    CameraState::fit_to_bounds(&schematic_scene.bounds),
                ),
                None => {
                    // Inert placeholder: with no schematic viewport the second pass
                    // is gated off in gpu.rs, so these values are never consumed.
                    let inert = datum_gui_protocol::SceneBounds {
                        min_x: 0,
                        min_y: 0,
                        max_x: 1,
                        max_y: 1,
                    };
                    let camera = CameraState::fit_to_bounds(&inert);
                    (None, inert, camera)
                }
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
            schematic_scene_viewport,
            schematic_bounds,
            schematic_camera,
        }
    }

    /// Override the companion schematic pass camera (P2.2d — interactive schematic
    /// camera). Construction seeds `schematic_camera` with fit-to-schematic-bounds
    /// (the INITIAL framing, byte-identical to the pre-P2.2d static fit); the
    /// gui-app supplies the schematic pane's WARM camera here so the owner's
    /// pan/zoom on the focused schematic pane persists frame-to-frame, exactly like
    /// the board camera. A no-op when the layout has no schematic pass (the field
    /// is never read — see `schematic_scene_viewport`). Only the gui-app render/
    /// capture path calls this; the golden/test path keeps the fit default.
    pub fn set_schematic_camera(&mut self, camera: CameraState) {
        self.schematic_camera = camera;
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<&HitTarget> {
        self.hit_regions
            .iter()
            .rev()
            .find(|region| region.rect.contains(x, y))
            .map(|region| &region.target)
    }

    // `world_point_at_screen` (per-pane screen->world resolve, UVT-004) lives in
    // the `coordinate_hit` include-module alongside the world hit-test.

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

    /// Build the STATIC companion schematic world buffer for the P2.2a
    /// multi-scene GPU pass. This mirrors `from_workspace_for_surface` exactly —
    /// the world geometry pipeline is coordinate-agnostic, so it is reused
    /// verbatim — but projects `state.schematic_scene` into the SCHEMATIC pane's
    /// rect with its own fit-to-schematic-bounds reference projection. Returns
    /// `None` (second pass gated off) when the workspace has no companion
    /// schematic scene or the layout has no Schematic pane. Symbols currently
    /// project as bare boxes (the projector discards labels/pins) — expected this
    /// slice; projection fidelity is P2.2b. No hit regions are emitted: pane B is
    /// non-interactive this slice.
    ///
    /// This is a strictly ADDITIVE resolve; it deliberately does NOT bump
    /// `RETAINED_RESOLVE_COUNT` (that counter gates BOARD pane-op latency and must
    /// stay board-scoped).
    pub fn from_workspace_schematic_for_surface(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Option<Self> {
        let schematic_scene = state.schematic_scene.as_ref()?;
        let layout =
            ShellLayout::for_surface(width, height, scale_factor, dock_height_for_state(state));
        let scene_viewport = layout.schematic_scene_viewport(&state.ui.layout)?;
        let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
        let reference_projection = Projection::new(
            board_field,
            &schematic_scene.bounds,
            CameraState::fit_to_bounds(&schematic_scene.bounds),
        );
        let mut world_quads = Vec::new();
        let mut world_batches = Vec::new();
        // Slice S1b: the schematic grid is NO LONGER baked here. It used to be pushed
        // FIRST as world-nm lines so scene geometry painted over it, but world-baked
        // lines are re-scaled by the live schematic camera and thicken on zoom-in.
        // The grid now draws as an IMMEDIATE screen-space pass (shared `GridEngine`,
        // `ScreenConstant` weight) in gpu.rs, scissored to the schematic pane — so
        // this retained WORLD buffer holds only real geometry (wires/symbols/text).
        push_retained_scene_geometry(
            &mut world_quads,
            schematic_scene,
            &reference_projection,
            state,
        );
        if !world_quads.is_empty() {
            world_batches.push(RetainedWorldBatch {
                layer_id: None,
                start: 0,
                len: (world_quads.len() * 6) as u32,
            });
        }
        push_retained_board_text_geometry_batches(
            &mut world_quads,
            &mut world_batches,
            schematic_scene,
            &reference_projection,
            state,
        );
        push_retained_board_graphic_batches(
            &mut world_quads,
            &mut world_batches,
            schematic_scene,
            &reference_projection,
            state,
        );
        let world_vertices = quads_to_vertices(&world_quads);
        // S3 / UVT-004: the schematic pane gets real hit regions for the first
        // time — placed symbols emit selectable world hit shapes tagged with their
        // stable projected identity, mirroring how the board tags authored objects.
        // Wires/labels stay non-interactive this slice (symbols are the S5/S7
        // selection/menu target); the world geometry above is unchanged.
        let mut world_hit_regions = Vec::new();
        coordinate_hit::push_schematic_symbol_hit_regions(&mut world_hit_regions, schematic_scene);
        Some(Self {
            world_vertices,
            world_batches,
            world_hit_regions,
        })
    }

    pub fn world_vertices(&self) -> &[Vertex] {
        &self.world_vertices
    }

    /// Every batch's vertex range, unfiltered — the draw list for the static
    /// companion schematic pass (P2.2a). The schematic's layers are not the
    /// board's layer-toggle set, so all of its geometry renders; `state`-filtered
    /// `visible_world_ranges` is the BOARD path.
    pub fn all_world_ranges(&self) -> Vec<Range<u32>> {
        self.world_batches
            .iter()
            .map(|batch| batch.start..batch.start + batch.len)
            .collect()
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

    // `hit_test_authored_world` (board) and `hit_test_world` (schematic,
    // unfiltered) live in the `coordinate_hit` include-module, sharing one scan
    // core so the board path stays byte-identical while the schematic surface
    // gets a filter-free twin.
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

    render_viewport_panes(
        layout,
        &state.ui.layout,
        state.schematic_scene.is_some(),
        panel_quads,
        text_runs,
    );
    render_status_bar(state, layout, panel_quads, text_runs);
}

// Workspace pane-chrome rendering (viewport panes, per-pane headers, and
// non-live placeholders) lives in the `pane_chrome` submodule; entry point
// `render_viewport_panes` is imported at the crate root.

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

// Render helper threads many quad/text-run/hit-region sinks.
#[allow(clippy::too_many_arguments)]
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

fn pad_copper_layer_ids(pad: &datum_gui_protocol::PadPrimitive) -> Vec<&str> {
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

