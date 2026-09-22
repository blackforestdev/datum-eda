#[path = "prepared_scene_access.rs"]
mod prepared_scene_access;
#[path = "scene_console.rs"]
mod scene_console;
#[path = "schematic_retained.rs"]
mod schematic_retained;

// Per-pane coordinate + hit resolution stays a child module with private scene access.
#[path = "coordinate_hit.rs"]
mod coordinate_hit;
pub use coordinate_hit::resolve_pane_hover;

// S4 interaction overlays and status stay child modules with private geometry access.
mod hit_clipping;
#[path = "interaction_overlay.rs"]
mod interaction_overlay;
#[path = "status_bar.rs"]
mod status_bar;

#[path = "frame_preparation.rs"]
mod frame_preparation;

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

fn dock_height_for_state(state: &ReviewWorkspaceState) -> Option<u32> {
    if state.ui.active_dock_tab.is_some() {
        Some(state.ui.effective_dock_height_px())
    } else {
        None
    }
}

fn render_phase1_shell_chrome(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut global_preferences_primitives::ControlPainter<'_>,
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
        if state.ui.global_preferences.high_contrast_noncolor {
            TEXT_PRIMARY
        } else {
            PANEL_CARD_BORDER
        },
    ));
    if state.ui.global_preferences.high_contrast_noncolor {
        push_rect_border(
            panel_quads,
            RectPx {
                x: layout.top_menu_bar.x,
                y: layout.top_menu_bar.y,
                width: layout.top_menu_bar.width,
                height: layout.status_bar.y + layout.status_bar.height,
            },
            TEXT_PRIMARY,
            2.0,
        );
        draw_text(
            "[HC] HIGH CONTRAST + NON-COLOR CUES",
            layout.top_menu_bar.x + layout.top_menu_bar.width - 430.0,
            layout.status_bar.y + design_tokens::spacing::SP_02,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_PRIMARY,
            TextFace::Mono,
            text_runs,
        );
    }
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
        format!(
            "{} \u{00B7} rev {}",
            truncate_text(&state.scene.project_name, 24),
            short_rev
        )
    };
    let rev_text_w = estimated_text_run_width_px(
        &rev_label,
        design_tokens::typography::DATA_SIZE,
        TextFace::Mono,
    ) - 16.0;
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
    status_bar::render_status_bar(state, layout, panel_quads, text_runs);
}

// Workspace pane-chrome rendering (viewport panes, per-pane headers, and
// non-live placeholders) lives in the `pane_chrome` submodule; entry point
// `render_viewport_panes` is imported at the crate root.

// Render helper threads many quad/text-run/hit-region sinks.
#[allow(clippy::too_many_arguments)]
fn render_scene(
    state: &ReviewWorkspaceState,
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
    let scene_hit_start = hit_regions.len();
    push_scene_overlay_and_hits(
        viewport_overlay_quads,
        &state.scene,
        scene_viewport,
        camera,
        state,
        text_runs,
        hit_regions,
    );
    hit_clipping::clip_new_hit_regions(hit_regions, scene_hit_start, scene_viewport);
    // The pane header and Project panel already name the document.
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
    state.ui.hovered_object.as_ref().is_some_and(|hover| {
        hover.surface == datum_gui_protocol::PaneContent::Board && hover.object_id == object_id
    })
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

#[path = "selection_relation.rs"]
mod selection_relation;
use selection_relation::{component_is_selection_active, component_is_selection_related};

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
