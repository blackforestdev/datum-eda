use datum_gui_protocol::{
    BoardReviewSceneV1, PointNm, ProposalOverlayPrimitive, ReviewActionRow, ReviewWorkspaceState,
    SelectionTarget,
};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use wgpu::util::DeviceExt;

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
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShellLayout {
    pub viewport: RectPx,
    pub left_sidebar: RectPx,
    pub right_sidebar: RectPx,
    pub bottom_strip: RectPx,
}

impl ShellLayout {
    pub fn for_window(width: u32, height: u32) -> Self {
        let width = width as f32;
        let height = height as f32;
        let left_width = 280.0_f32.min(width * 0.3);
        let right_width = 340.0_f32.min(width * 0.35);
        let bottom_height = 44.0_f32.min(height * 0.25);
        Self {
            left_sidebar: RectPx {
                x: 0.0,
                y: 0.0,
                width: left_width,
                height: height - bottom_height,
            },
            viewport: RectPx {
                x: left_width,
                y: 0.0,
                width: (width - left_width - right_width).max(0.0),
                height: height - bottom_height,
            },
            right_sidebar: RectPx {
                x: (width - right_width).max(0.0),
                y: 0.0,
                width: right_width,
                height: height - bottom_height,
            },
            bottom_strip: RectPx {
                x: 0.0,
                y: height - bottom_height,
                width,
                height: bottom_height,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitTarget {
    ReviewAction(String),
    AuthoredObject(String),
    TerminalTab,
    AssistantTab,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HitRegion {
    pub target: HitTarget,
    pub rect: RectPx,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedScene {
    pub layout: ShellLayout,
    pub hit_regions: Vec<HitRegion>,
    panel_quads: Vec<Quad>,
    viewport_quads: Vec<Quad>,
    text_runs: Vec<TextRun>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Quad {
    points: [(f32, f32); 4],
    color: [f32; 3],
}

impl Quad {
    fn from_rect(rect: RectPx, color: [f32; 3]) -> Self {
        Self {
            points: [
                (rect.x, rect.y),
                (rect.x + rect.width, rect.y),
                (rect.x + rect.width, rect.y + rect.height),
                (rect.x, rect.y + rect.height),
            ],
            color,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextFace {
    Ui,
    Mono,
}

#[derive(Debug, Clone, PartialEq)]
struct TextRun {
    text: String,
    x: f32,
    y: f32,
    size: f32,
    color: [f32; 3],
    face: TextFace,
}

const APP_BG: [f32; 3] = [0.07, 0.08, 0.09];
const PANEL_BG: [f32; 3] = [0.11, 0.12, 0.14];
const PANEL_CARD_BG: [f32; 3] = [0.14, 0.15, 0.18];
const PANEL_CARD_BORDER: [f32; 3] = [0.20, 0.22, 0.26];
const VIEWPORT_BG: [f32; 3] = [0.09, 0.10, 0.11];
const VIEWPORT_FRAME: [f32; 3] = [0.17, 0.19, 0.22];
const BOARD_OUTER_FIELD: [f32; 3] = [0.14, 0.15, 0.17];
const BOARD_INNER_FIELD: [f32; 3] = [0.32, 0.34, 0.38];
const BOARD_GRID_MAJOR: [f32; 3] = [0.40, 0.42, 0.46];
const BOARD_GRID_MINOR: [f32; 3] = [0.35, 0.37, 0.40];
const BOARD_EDGE: [f32; 3] = [0.87, 0.89, 0.92];
const BOARD_EDGE_GLOW: [f32; 3] = [0.57, 0.60, 0.65];
const TEXT_PRIMARY: [f32; 3] = [0.92, 0.93, 0.95];
const TEXT_SECONDARY: [f32; 3] = [0.62, 0.66, 0.72];
const TEXT_MUTED: [f32; 3] = [0.48, 0.52, 0.58];
const TEXT_ACCENT: [f32; 3] = [0.96, 0.78, 0.41];
const COMPONENT_BODY: [f32; 3] = [0.37, 0.39, 0.43];
const COMPONENT_BODY_RELATED: [f32; 3] = [0.42, 0.45, 0.50];
const COMPONENT_BODY_SELECTED: [f32; 3] = [0.48, 0.52, 0.58];
const COMPONENT_HEADER: [f32; 3] = [0.26, 0.28, 0.31];
const COMPONENT_OUTLINE: [f32; 3] = [0.78, 0.80, 0.84];
const PAD_COPPER: [f32; 3] = [0.86, 0.74, 0.53];
const PAD_COPPER_RELATED: [f32; 3] = [0.88, 0.85, 0.68];
const PAD_COPPER_SELECTED: [f32; 3] = [0.94, 0.95, 0.97];
const PAD_CORE: [f32; 3] = [0.27, 0.22, 0.17];
const AUTHOR_BASE: [f32; 3] = [0.42, 0.49, 0.57];
const AUTHOR_RELATED: [f32; 3] = [0.77, 0.83, 0.90];
const AUTHOR_SELECTED: [f32; 3] = [0.88, 0.92, 0.98];
const PROPOSAL_BASE: [f32; 3] = [0.95, 0.67, 0.26];
const PROPOSAL_FOCUS: [f32; 3] = [1.00, 0.84, 0.47];
const PROPOSAL_UNDERLAY: [f32; 3] = [0.52, 0.34, 0.11];
const PROPOSAL_OUTER: [f32; 3] = [0.74, 0.52, 0.21];
const PROPOSAL_CENTERLINE: [f32; 3] = [1.00, 0.95, 0.82];
const PROPOSAL_ANCHOR_RING: [f32; 3] = [0.99, 0.88, 0.63];
const PROPOSAL_ANCHOR_CORE: [f32; 3] = [0.39, 0.28, 0.13];
const PROPOSAL_SEGMENTED: [f32; 3] = [1.00, 0.90, 0.66];
const DIAGNOSTIC_BASE: [f32; 3] = [0.48, 0.78, 0.82];
const DIAGNOSTIC_FOCUS: [f32; 3] = [0.72, 0.93, 0.97];
const DIAGNOSTIC_UNDERLAY: [f32; 3] = [0.18, 0.32, 0.35];

impl PreparedScene {
    pub fn from_workspace(state: &ReviewWorkspaceState, width: u32, height: u32) -> Self {
        let layout = ShellLayout::for_window(width, height);
        let mut panel_quads = Vec::new();
        let mut viewport_quads = Vec::new();
        let mut text_runs = Vec::new();
        let mut hit_regions = Vec::new();

        panel_quads.push(Quad::from_rect(layout.left_sidebar, PANEL_BG));
        panel_quads.push(Quad::from_rect(layout.right_sidebar, PANEL_BG));
        panel_quads.push(Quad::from_rect(layout.bottom_strip, PANEL_BG));
        viewport_quads.push(Quad::from_rect(layout.viewport, VIEWPORT_BG));

        render_side_panels(
            state,
            &layout,
            &mut panel_quads,
            &mut text_runs,
            &mut hit_regions,
        );
        render_bottom_tabs(&layout, &mut panel_quads, &mut text_runs, &mut hit_regions);
        render_scene(
            state,
            &layout,
            &mut viewport_quads,
            &mut text_runs,
            &mut hit_regions,
        );

        Self {
            layout,
            hit_regions,
            panel_quads,
            viewport_quads,
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

    fn vertices(&self) -> Vec<Vertex> {
        let mut out = Vec::new();
        for quad in self.panel_quads.iter().chain(self.viewport_quads.iter()) {
            quad_to_vertices(&mut out, *quad);
        }
        out
    }
}

fn render_side_panels(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let left = layout.left_sidebar;
    let right = layout.right_sidebar;

    let project_rect = RectPx {
        x: left.x + 14.0,
        y: left.y + 14.0,
        width: left.width - 28.0,
        height: 124.0,
    };
    let filters_rect = RectPx {
        x: left.x + 14.0,
        y: left.y + 150.0,
        width: left.width - 28.0,
        height: 154.0,
    };
    let inspector_rect = RectPx {
        x: right.x + 14.0,
        y: right.y + 14.0,
        width: right.width - 28.0,
        height: 150.0,
    };
    let review_rect = RectPx {
        x: right.x + 14.0,
        y: right.y + 176.0,
        width: right.width - 28.0,
        height: right.height - 190.0,
    };

    for (rect, title) in [
        (project_rect, "PROJECT"),
        (filters_rect, "FILTERS"),
        (inspector_rect, "INSPECTOR"),
        (review_rect, "REVIEW"),
    ] {
        panel_quads.push(Quad::from_rect(rect, PANEL_CARD_BG));
        push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);
        draw_text(
            title,
            rect.x + 12.0,
            rect.y + 12.0,
            12.0,
            TEXT_SECONDARY,
            TextFace::Ui,
            text_runs,
        );
        push_section_divider(
            panel_quads,
            rect.x + 12.0,
            rect.y + 28.0,
            rect.width - 24.0,
            PANEL_CARD_BORDER,
        );
    }

    draw_text(
        &truncate_text(&state.scene.project_name.to_uppercase(), 22),
        project_rect.x + 12.0,
        project_rect.y + 34.0,
        16.0,
        TEXT_PRIMARY,
        TextFace::Ui,
        text_runs,
    );
    draw_text(
        &format!(
            "BOARD {}",
            truncate_text(&state.scene.board_name.to_uppercase(), 18)
        ),
        project_rect.x + 12.0,
        project_rect.y + 54.0,
        12.5,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    if let Some(action) = state.selected_review_action() {
        draw_text(
            &format!("NET {}", truncate_text(&action.net_name.to_uppercase(), 18)),
            project_rect.x + 12.0,
            project_rect.y + 74.0,
            13.0,
            TEXT_ACCENT,
            TextFace::Ui,
            text_runs,
        );
    }
    push_boolean_row(
        filters_rect.x + 12.0,
        filters_rect.y + 36.0,
        "AUTHORED",
        true,
        text_runs,
    );
    push_boolean_row(
        filters_rect.x + 12.0,
        filters_rect.y + 56.0,
        "PROPOSED",
        true,
        text_runs,
    );
    push_boolean_row(
        filters_rect.x + 12.0,
        filters_rect.y + 76.0,
        "DIM UNRELATED",
        true,
        text_runs,
    );
    if let Some(action) = state.selected_review_action() {
        draw_text(
            &format!(
                "ACTIVE {}",
                truncate_text(&suffix_id(&action.action_id).to_uppercase(), 14)
            ),
            filters_rect.x + 12.0,
            filters_rect.y + 108.0,
            11.0,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
    }

    draw_text(
        "SELECTION",
        inspector_rect.x + 12.0,
        inspector_rect.y + 34.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    match &state.selection {
        SelectionTarget::ReviewAction(action_id) => {
            draw_text(
                &format!(
                    "ACTION {}",
                    truncate_text(&suffix_id(action_id).to_uppercase(), 14)
                ),
                inspector_rect.x + 12.0,
                inspector_rect.y + 54.0,
                15.0,
                TEXT_PRIMARY,
                TextFace::Mono,
                text_runs,
            );
        }
        SelectionTarget::AuthoredObject(object_id) => {
            draw_text(
                &format!(
                    "OBJECT {}",
                    truncate_text(&suffix_id(object_id).to_uppercase(), 14)
                ),
                inspector_rect.x + 12.0,
                inspector_rect.y + 54.0,
                15.0,
                TEXT_PRIMARY,
                TextFace::Mono,
                text_runs,
            );
        }
        SelectionTarget::None => draw_text(
            "NONE",
            inspector_rect.x + 12.0,
            inspector_rect.y + 54.0,
            15.0,
            TEXT_MUTED,
            TextFace::Ui,
            text_runs,
        ),
    }
    if let Some(action) = state.selected_review_action() {
        push_section_divider(
            panel_quads,
            inspector_rect.x + 12.0,
            inspector_rect.y + 76.0,
            inspector_rect.width - 24.0,
            [0.23, 0.25, 0.29],
        );
        push_key_value(
            inspector_rect.x + 12.0,
            inspector_rect.y + 84.0,
            "CONTRACT",
            &truncate_text(&action.contract.to_uppercase(), 18),
            text_runs,
            TextFace::Mono,
        );
        push_key_value(
            inspector_rect.x + 12.0,
            inspector_rect.y + 104.0,
            "NET",
            &truncate_text(&action.net_name.to_uppercase(), 16),
            text_runs,
            TextFace::Ui,
        );
        push_key_value(
            inspector_rect.x + 12.0,
            inspector_rect.y + 124.0,
            "SEGMENT",
            &format!(
                "{} OF {}",
                action.selected_path_segment_index + 1,
                action.selected_path_segment_count
            ),
            text_runs,
            TextFace::Mono,
        );
    }
    if let Some(segment) = state.selected_segment_evidence() {
        push_key_value(
            inspector_rect.x + 12.0,
            inspector_rect.y + 144.0,
            "LAYER",
            &segment.layer.to_string(),
            text_runs,
            TextFace::Mono,
        );
    }

    draw_text(
        &format!(
            "SOURCE {}",
            truncate_text(&state.review.review_source.to_uppercase(), 20)
        ),
        review_rect.x + 12.0,
        review_rect.y + 34.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
    draw_text(
        &format!("{} ACTIONS", state.review.proposal_actions.len()),
        review_rect.x + 12.0,
        review_rect.y + 54.0,
        15.0,
        TEXT_PRIMARY,
        TextFace::Ui,
        text_runs,
    );
    push_section_divider(
        panel_quads,
        review_rect.x + 12.0,
        review_rect.y + 72.0,
        review_rect.width - 24.0,
        [0.23, 0.25, 0.29],
    );

    let rows: Vec<ReviewActionRow> = state.review_rows();
    let mut row_y = review_rect.y + 82.0;
    for row in rows {
        let selected = row.action_id == state.active_review_target_id;
        let row_rect = RectPx {
            x: review_rect.x + 8.0,
            y: row_y,
            width: review_rect.width - 16.0,
            height: 42.0,
        };
        panel_quads.push(Quad::from_rect(
            row_rect,
            if selected {
                [0.28, 0.20, 0.11]
            } else {
                [0.17, 0.18, 0.21]
            },
        ));
        push_rect_border(
            panel_quads,
            row_rect,
            if selected {
                PROPOSAL_BASE
            } else {
                PANEL_CARD_BORDER
            },
            1.0,
        );
        draw_text(
            &truncate_text(&row.title, 22),
            row_rect.x + 10.0,
            row_rect.y + 9.0,
            13.5,
            if selected { TEXT_ACCENT } else { TEXT_PRIMARY },
            TextFace::Ui,
            text_runs,
        );
        draw_text(
            &truncate_text(&row.subtitle, 24),
            row_rect.x + 10.0,
            row_rect.y + 25.0,
            11.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        hit_regions.push(HitRegion {
            target: HitTarget::ReviewAction(row.action_id),
            rect: row_rect,
        });
        row_y += 44.0;
    }
}

fn render_bottom_tabs(
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let terminal_rect = RectPx {
        x: layout.bottom_strip.x + 12.0,
        y: layout.bottom_strip.y + 8.0,
        width: 120.0,
        height: layout.bottom_strip.height - 16.0,
    };
    let assistant_rect = RectPx {
        x: layout.bottom_strip.x + 140.0,
        y: layout.bottom_strip.y + 8.0,
        width: 120.0,
        height: layout.bottom_strip.height - 16.0,
    };
    for (rect, label, target) in [
        (terminal_rect, "TERMINAL", HitTarget::TerminalTab),
        (assistant_rect, "ASSISTANT", HitTarget::AssistantTab),
    ] {
        panel_quads.push(Quad::from_rect(rect, [0.15, 0.16, 0.19]));
        push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);
        draw_text(
            label,
            rect.x + 12.0,
            rect.y + 10.0,
            12.0,
            TEXT_SECONDARY,
            TextFace::Ui,
            text_runs,
        );
        hit_regions.push(HitRegion { target, rect });
    }
}

fn render_scene(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    viewport_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    push_scene_geometry(
        viewport_quads,
        &state.scene,
        layout.viewport,
        state,
        text_runs,
        hit_regions,
    );
    draw_text(
        &truncate_text(&state.scene.board_name.to_uppercase(), 28),
        layout.viewport.x + 16.0,
        layout.viewport.y + 16.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    if let Some(action) = state.selected_review_action() {
        draw_text(
            &format!(
                "ACTIVE {}",
                truncate_text(&suffix_id(&action.action_id).to_uppercase(), 16)
            ),
            layout.viewport.x + 16.0,
            layout.viewport.y + 32.0,
            13.0,
            TEXT_ACCENT,
            TextFace::Mono,
            text_runs,
        );
        draw_text(
            &format!("NET {}", truncate_text(&action.net_name.to_uppercase(), 20)),
            layout.viewport.x + 16.0,
            layout.viewport.y + 50.0,
            10.5,
            TEXT_MUTED,
            TextFace::Ui,
            text_runs,
        );
    }
}

fn push_scene_geometry(
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    viewport: RectPx,
    state: &ReviewWorkspaceState,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let scene_viewport = inset_rect(viewport, 16.0, 76.0, 16.0, 16.0);
    out.push(Quad::from_rect(scene_viewport, BOARD_OUTER_FIELD));
    push_rect_border(out, scene_viewport, VIEWPORT_FRAME, 1.0);
    let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
    out.push(Quad::from_rect(board_field, BOARD_INNER_FIELD));
    push_rect_border(out, board_field, [0.46, 0.49, 0.53], 1.0);
    push_scene_grid(out, board_field);
    for outline in &scene.outline {
        push_polyline_segments(
            out,
            &outline.path,
            board_field,
            &scene.bounds,
            BOARD_EDGE_GLOW,
            4.2,
        );
        push_polyline_segments(
            out,
            &outline.path,
            board_field,
            &scene.bounds,
            BOARD_EDGE,
            1.6,
        );
    }
    for zone in &scene.zones {
        if zone.polygon.len() >= 4 {
            push_polygon_fill(
                out,
                &zone.polygon,
                board_field,
                &scene.bounds,
                [0.10, 0.16, 0.13],
            );
            push_polyline_segments(
                out,
                &close_path(&zone.polygon),
                board_field,
                &scene.bounds,
                [0.22, 0.34, 0.28],
                1.5,
            );
        }
    }
    for component in &scene.components {
        let selected = matches!(state.selection, SelectionTarget::AuthoredObject(ref id) if id == &component.object_id);
        let related = component_overlaps_active_action(component, state);
        let px = push_component_primitive(
            out,
            component,
            board_field,
            &scene.bounds,
            selected,
            related,
        );
        hit_regions.push(HitRegion {
            target: HitTarget::AuthoredObject(component.object_id.clone()),
            rect: px,
        });
        let (label_x, label_y) = project_point(component.position, board_field, &scene.bounds);
        draw_text(
            &truncate_text(&component.reference.to_uppercase(), 6),
            label_x - 9.0,
            (label_y - 4.0).max(board_field.y + 6.0),
            9.0,
            if selected {
                PAD_COPPER_SELECTED
            } else if related {
                PAD_COPPER_RELATED
            } else {
                [0.80, 0.82, 0.86]
            },
            TextFace::Mono,
            text_runs,
        );
    }
    for pad in &scene.pads {
        let active = matches!(state.selection, SelectionTarget::AuthoredObject(ref id) if id == &pad.object_id);
        let related = pad_matches_active_action(pad, state);
        let px = push_pad_primitive(
            out,
            pad,
            board_field,
            &scene.bounds,
            if active {
                PAD_COPPER_SELECTED
            } else if related {
                PAD_COPPER_RELATED
            } else {
                PAD_COPPER
            },
        );
        hit_regions.push(HitRegion {
            target: HitTarget::AuthoredObject(pad.object_id.clone()),
            rect: px,
        });
    }
    for track in &scene.tracks {
        let related = track_matches_active_action(track, state);
        let selected = matches!(state.selection, SelectionTarget::AuthoredObject(ref id) if id == &track.object_id);
        let color = if selected {
            AUTHOR_SELECTED
        } else if related {
            AUTHOR_RELATED
        } else {
            authored_track_color(&track.layer_id)
        };
        let rects = push_polyline_segments(
            out,
            &track.path,
            board_field,
            &scene.bounds,
            color,
            if selected {
                4.0
            } else if related {
                3.0
            } else {
                2.0
            },
        );
        for rect in rects {
            hit_regions.push(HitRegion {
                target: HitTarget::AuthoredObject(track.object_id.clone()),
                rect,
            });
        }
    }
    for via in &scene.vias {
        let px = push_via_primitive(
            out,
            via,
            board_field,
            &scene.bounds,
            matches!(state.selection, SelectionTarget::AuthoredObject(ref id) if id == &via.object_id),
        );
        hit_regions.push(HitRegion {
            target: HitTarget::AuthoredObject(via.object_id.clone()),
            rect: px,
        });
    }
    for overlay in &scene.proposal_overlay_primitives {
        let selected = overlay.proposal_action_id == state.active_review_target_id;
        let color = match overlay.render_role.as_str() {
            "proposed_focus" if selected => PROPOSAL_FOCUS,
            "proposed_overlay" if selected => PROPOSAL_FOCUS,
            "proposed_overlay" => PROPOSAL_BASE,
            "authored_related" => AUTHOR_RELATED,
            _ => PROPOSAL_BASE,
        };
        let rects = push_overlay(out, overlay, board_field, &scene.bounds, color, selected);
        for rect in rects {
            hit_regions.push(HitRegion {
                target: HitTarget::ReviewAction(overlay.proposal_action_id.clone()),
                rect,
            });
        }
    }
    let active_evidence_key = state
        .selected_review_action()
        .map(|action| format!("segment:{}", action.selected_path_segment_index));
    for review in &scene.review_primitives {
        let active = review.evidence_key.as_ref() == active_evidence_key.as_ref();
        push_dashed_polyline_segments(
            out,
            &review.path,
            board_field,
            &scene.bounds,
            DIAGNOSTIC_UNDERLAY,
            if active { 2.1 } else { 1.6 },
            10.0,
            6.0,
        );
        push_dashed_polyline_segments(
            out,
            &review.path,
            board_field,
            &scene.bounds,
            if active {
                DIAGNOSTIC_FOCUS
            } else {
                DIAGNOSTIC_BASE
            },
            if active { 1.2 } else { 0.9 },
            10.0,
            6.0,
        );
        push_points(
            out,
            &review.path,
            board_field,
            &scene.bounds,
            if active {
                DIAGNOSTIC_FOCUS
            } else {
                DIAGNOSTIC_BASE
            },
            if active { 4.0 } else { 3.0 },
        );
    }
}

fn pad_matches_active_action(
    pad: &datum_gui_protocol::PadPrimitive,
    state: &ReviewWorkspaceState,
) -> bool {
    let Some(action) = state.selected_review_action() else {
        return false;
    };
    pad.pad_uuid == action.from_anchor_pad_uuid || pad.pad_uuid == action.to_anchor_pad_uuid
}

fn track_matches_active_action(
    track: &datum_gui_protocol::TrackPrimitive,
    state: &ReviewWorkspaceState,
) -> bool {
    let Some(action) = state.selected_review_action() else {
        return false;
    };
    let Some(net_uuid) = &track.net_uuid else {
        return false;
    };
    net_uuid == &action.net_uuid
}

fn component_overlaps_active_action(
    component: &datum_gui_protocol::ComponentBounds,
    state: &ReviewWorkspaceState,
) -> bool {
    let Some(action) = state.selected_review_action() else {
        return false;
    };
    point_in_rect(action.from, component.bounds) || point_in_rect(action.to, component.bounds)
}

fn point_in_rect(point: PointNm, rect: datum_gui_protocol::RectNm) -> bool {
    point.x >= rect.min_x && point.x <= rect.max_x && point.y >= rect.min_y && point.y <= rect.max_y
}

fn push_overlay(
    out: &mut Vec<Quad>,
    overlay: &ProposalOverlayPrimitive,
    viewport: RectPx,
    bounds: &datum_gui_protocol::SceneBounds,
    color: [f32; 3],
    selected: bool,
) -> Vec<RectPx> {
    match overlay.primitive_kind.as_str() {
        "anchor_marker" => {
            let outer_size = if selected { 18.0 } else { 14.0 };
            let ring_size = if selected { 12.0 } else { 9.0 };
            let core_size = if selected { 6.0 } else { 4.0 };
            let mut rects = push_points(
                out,
                &overlay.path,
                viewport,
                bounds,
                if selected {
                    PROPOSAL_UNDERLAY
                } else {
                    [0.30, 0.22, 0.12]
                },
                outer_size,
            );
            rects.extend(push_points(
                out,
                &overlay.path,
                viewport,
                bounds,
                if selected {
                    PROPOSAL_FOCUS
                } else {
                    PROPOSAL_ANCHOR_RING
                },
                ring_size,
            ));
            rects.extend(push_points(
                out,
                &overlay.path,
                viewport,
                bounds,
                PROPOSAL_ANCHOR_CORE,
                core_size,
            ));
            rects
        }
        _ => {
            let mut hit_rects = push_polyline_segments(
                out,
                &overlay.path,
                viewport,
                bounds,
                if selected {
                    PROPOSAL_UNDERLAY
                } else {
                    [0.24, 0.18, 0.10]
                },
                if selected { 8.0 } else { 6.0 },
            );
            if selected {
                hit_rects.extend(push_polyline_endcaps(
                    out,
                    &overlay.path,
                    viewport,
                    bounds,
                    PROPOSAL_OUTER,
                    8.0,
                    18.0,
                ));
            }
            hit_rects.extend(push_polyline_segments(
                out,
                &overlay.path,
                viewport,
                bounds,
                if selected { PROPOSAL_OUTER } else { color },
                if selected { 4.6 } else { 2.6 },
            ));
            hit_rects.extend(push_polyline_segments(
                out,
                &overlay.path,
                viewport,
                bounds,
                if selected {
                    PROPOSAL_FOCUS
                } else {
                    PROPOSAL_BASE
                },
                if selected { 2.4 } else { 1.5 },
            ));
            if selected {
                hit_rects.extend(push_dashed_polyline_segments(
                    out,
                    &overlay.path,
                    viewport,
                    bounds,
                    PROPOSAL_SEGMENTED,
                    1.4,
                    20.0,
                    10.0,
                ));
                hit_rects.extend(push_polyline_endcaps(
                    out,
                    &overlay.path,
                    viewport,
                    bounds,
                    PROPOSAL_FOCUS,
                    3.0,
                    14.0,
                ));
                hit_rects.extend(push_polyline_segments(
                    out,
                    &overlay.path,
                    viewport,
                    bounds,
                    PROPOSAL_CENTERLINE,
                    0.6,
                ));
            }
            hit_rects
        }
    }
}

fn push_scene_grid(out: &mut Vec<Quad>, viewport: RectPx) {
    let cols = 8;
    let rows = 6;
    for idx in 1..cols {
        let x = viewport.x + viewport.width * idx as f32 / cols as f32;
        out.push(Quad::from_rect(
            RectPx {
                x,
                y: viewport.y,
                width: 1.0,
                height: viewport.height,
            },
            if idx % 2 == 0 {
                BOARD_GRID_MAJOR
            } else {
                BOARD_GRID_MINOR
            },
        ));
    }
    for idx in 1..rows {
        let y = viewport.y + viewport.height * idx as f32 / rows as f32;
        out.push(Quad::from_rect(
            RectPx {
                x: viewport.x,
                y,
                width: viewport.width,
                height: 1.0,
            },
            if idx % 2 == 0 {
                BOARD_GRID_MAJOR
            } else {
                BOARD_GRID_MINOR
            },
        ));
    }
}

fn push_polygon_fill(
    out: &mut Vec<Quad>,
    polygon: &[PointNm],
    viewport: RectPx,
    bounds: &datum_gui_protocol::SceneBounds,
    color: [f32; 3],
) {
    if polygon.is_empty() {
        return;
    }
    let rect = RectPx {
        x: polygon
            .iter()
            .map(|point| project_point(*point, viewport, bounds).0)
            .fold(f32::MAX, f32::min),
        y: polygon
            .iter()
            .map(|point| project_point(*point, viewport, bounds).1)
            .fold(f32::MAX, f32::min),
        width: polygon
            .iter()
            .map(|point| project_point(*point, viewport, bounds).0)
            .fold(f32::MIN, f32::max)
            - polygon
                .iter()
                .map(|point| project_point(*point, viewport, bounds).0)
                .fold(f32::MAX, f32::min),
        height: polygon
            .iter()
            .map(|point| project_point(*point, viewport, bounds).1)
            .fold(f32::MIN, f32::max)
            - polygon
                .iter()
                .map(|point| project_point(*point, viewport, bounds).1)
                .fold(f32::MAX, f32::min),
    };
    out.push(Quad::from_rect(rect, color));
}

fn push_component_primitive(
    out: &mut Vec<Quad>,
    component: &datum_gui_protocol::ComponentBounds,
    viewport: RectPx,
    bounds: &datum_gui_protocol::SceneBounds,
    selected: bool,
    related: bool,
) -> RectPx {
    let body = push_world_rect(
        out,
        component.bounds,
        viewport,
        bounds,
        if selected {
            COMPONENT_BODY_SELECTED
        } else if related {
            COMPONENT_BODY_RELATED
        } else {
            COMPONENT_BODY
        },
    );
    let header_h = body.height.clamp(6.0, 12.0);
    let header = RectPx {
        x: body.x + 1.0,
        y: body.y + 1.0,
        width: (body.width - 2.0).max(1.0),
        height: (header_h - 1.0).max(1.0),
    };
    out.push(Quad::from_rect(header, COMPONENT_HEADER));
    let inner = inset_rect(body, 2.0, header_h + 1.0, 2.0, 2.0);
    if inner.width > 2.0 && inner.height > 2.0 {
        out.push(Quad::from_rect(inner, [0.30, 0.32, 0.36]));
    }
    let pin1 = RectPx {
        x: body.x + 4.0,
        y: body.y + 4.0,
        width: 3.0,
        height: 3.0,
    };
    out.push(Quad::from_rect(
        pin1,
        if selected || related {
            PAD_COPPER_RELATED
        } else {
            PAD_COPPER
        },
    ));
    push_rect_border(
        out,
        body,
        if selected {
            AUTHOR_SELECTED
        } else if related {
            AUTHOR_RELATED
        } else {
            COMPONENT_OUTLINE
        },
        1.0,
    );
    body
}

fn push_pad_primitive(
    out: &mut Vec<Quad>,
    pad: &datum_gui_protocol::PadPrimitive,
    viewport: RectPx,
    bounds: &datum_gui_protocol::SceneBounds,
    outer_color: [f32; 3],
) -> RectPx {
    let px = push_world_rect(out, pad.bounds, viewport, bounds, outer_color);
    let copper = inset_rect(px, 1.0, 1.0, 1.0, 1.0);
    if copper.width > 1.0 && copper.height > 1.0 {
        out.push(Quad::from_rect(copper, outer_color));
    }
    if pad.shape_kind == "rect" {
        let core = inset_rect(px, 3.0, 3.0, 3.0, 3.0);
        if core.width > 1.0 && core.height > 1.0 {
            out.push(Quad::from_rect(core, PAD_CORE));
        }
    } else {
        let vertical = inset_rect(px, px.width * 0.32, 2.0, px.width * 0.32, 2.0);
        let horizontal = inset_rect(px, 2.0, px.height * 0.32, 2.0, px.height * 0.32);
        if vertical.width > 1.0 && vertical.height > 1.0 {
            out.push(Quad::from_rect(vertical, PAD_CORE));
        }
        if horizontal.width > 1.0 && horizontal.height > 1.0 {
            out.push(Quad::from_rect(horizontal, PAD_CORE));
        }
    }
    push_rect_border(out, px, [0.97, 0.95, 0.88], 1.0);
    px
}

fn push_via_primitive(
    out: &mut Vec<Quad>,
    via: &datum_gui_protocol::ViaPrimitive,
    viewport: RectPx,
    bounds: &datum_gui_protocol::SceneBounds,
    selected: bool,
) -> RectPx {
    let size = 9.0;
    let rect = push_point_square(
        out,
        via.position,
        viewport,
        bounds,
        size,
        if selected {
            AUTHOR_SELECTED
        } else {
            [0.72, 0.77, 0.84]
        },
    );
    let inner = inset_rect(rect, 2.0, 2.0, 2.0, 2.0);
    out.push(Quad::from_rect(inner, [0.13, 0.14, 0.16]));
    push_rect_border(
        out,
        rect,
        if selected {
            AUTHOR_RELATED
        } else {
            [0.95, 0.80, 0.42]
        },
        1.0,
    );
    rect
}

fn authored_track_color(layer_id: &str) -> [f32; 3] {
    match layer_id {
        "L1" => [0.39, 0.61, 0.78],
        "L3" => [0.30, 0.70, 0.56],
        _ => AUTHOR_BASE,
    }
}

fn push_points(
    out: &mut Vec<Quad>,
    points: &[PointNm],
    viewport: RectPx,
    bounds: &datum_gui_protocol::SceneBounds,
    color: [f32; 3],
    size_px: f32,
) -> Vec<RectPx> {
    let mut rects = Vec::new();
    for point in points {
        rects.push(push_point_square(
            out, *point, viewport, bounds, size_px, color,
        ));
    }
    rects
}

fn push_dashed_polyline_segments(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    viewport: RectPx,
    bounds: &datum_gui_protocol::SceneBounds,
    color: [f32; 3],
    thickness_px: f32,
    dash_px: f32,
    gap_px: f32,
) -> Vec<RectPx> {
    let mut rects = Vec::new();
    for segment in path.windows(2) {
        let a = project_point(segment[0], viewport, bounds);
        let b = project_point(segment[1], viewport, bounds);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let ux = dx / len;
        let uy = dy / len;
        let step = (dash_px + gap_px).max(1.0);
        let mut start = 0.0;
        while start < len {
            let end = (start + dash_px).min(len);
            if end > start {
                let start_point = (a.0 + ux * start, a.1 + uy * start);
                let end_point = (a.0 + ux * end, a.1 + uy * end);
                let seg_dx = end_point.0 - start_point.0;
                let seg_dy = end_point.1 - start_point.1;
                let seg_len = (seg_dx * seg_dx + seg_dy * seg_dy).sqrt().max(1.0);
                let nx = -seg_dy / seg_len * thickness_px * 0.5;
                let ny = seg_dx / seg_len * thickness_px * 0.5;
                let quad = [
                    (start_point.0 + nx, start_point.1 + ny),
                    (end_point.0 + nx, end_point.1 + ny),
                    (end_point.0 - nx, end_point.1 - ny),
                    (start_point.0 - nx, start_point.1 - ny),
                ];
                rects.push(bounds_from_projected_points(&quad));
                push_projected_quad(out, &quad, color);
            }
            start += step;
        }
    }
    rects
}

fn push_polyline_endcaps(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    viewport: RectPx,
    bounds: &datum_gui_protocol::SceneBounds,
    color: [f32; 3],
    thickness_px: f32,
    cap_length_px: f32,
) -> Vec<RectPx> {
    let mut rects = Vec::new();
    if path.len() < 2 {
        return rects;
    }

    let first_a = project_point(path[0], viewport, bounds);
    let first_b = project_point(path[1], viewport, bounds);
    if let Some(quad) = projected_cap_quad(first_a, first_b, thickness_px, cap_length_px) {
        rects.push(bounds_from_projected_points(&quad));
        push_projected_quad(out, &quad, color);
    }

    let last_a = project_point(path[path.len() - 1], viewport, bounds);
    let last_b = project_point(path[path.len() - 2], viewport, bounds);
    if let Some(quad) = projected_cap_quad(last_a, last_b, thickness_px, cap_length_px) {
        rects.push(bounds_from_projected_points(&quad));
        push_projected_quad(out, &quad, color);
    }

    rects
}

fn push_polyline_segments(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    viewport: RectPx,
    bounds: &datum_gui_protocol::SceneBounds,
    color: [f32; 3],
    thickness_px: f32,
) -> Vec<RectPx> {
    let mut rects = Vec::new();
    for segment in path.windows(2) {
        let a = project_point(segment[0], viewport, bounds);
        let b = project_point(segment[1], viewport, bounds);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let nx = -dy / len * thickness_px * 0.5;
        let ny = dx / len * thickness_px * 0.5;
        let quad = [
            (a.0 + nx, a.1 + ny),
            (b.0 + nx, b.1 + ny),
            (b.0 - nx, b.1 - ny),
            (a.0 - nx, a.1 - ny),
        ];
        let rect = bounds_from_projected_points(&quad);
        rects.push(rect);
        push_projected_quad(out, &quad, color);
    }
    rects
}

fn projected_cap_quad(
    start: (f32, f32),
    toward: (f32, f32),
    thickness_px: f32,
    cap_length_px: f32,
) -> Option<[(f32, f32); 4]> {
    let dx = toward.0 - start.0;
    let dy = toward.1 - start.1;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 0.01 {
        return None;
    }
    let ux = dx / len;
    let uy = dy / len;
    let end = (
        start.0 + ux * cap_length_px.min(len),
        start.1 + uy * cap_length_px.min(len),
    );
    let nx = -uy * thickness_px * 0.5;
    let ny = ux * thickness_px * 0.5;
    Some([
        (start.0 + nx, start.1 + ny),
        (end.0 + nx, end.1 + ny),
        (end.0 - nx, end.1 - ny),
        (start.0 - nx, start.1 - ny),
    ])
}

fn close_path(points: &[PointNm]) -> Vec<PointNm> {
    let mut out = points.to_vec();
    if let (Some(first), Some(last)) = (out.first().copied(), out.last().copied())
        && first != last
    {
        out.push(first);
    }
    out
}

fn push_world_rect(
    out: &mut Vec<Quad>,
    rect: datum_gui_protocol::RectNm,
    viewport: RectPx,
    bounds: &datum_gui_protocol::SceneBounds,
    color: [f32; 3],
) -> RectPx {
    let (x0, y0) = project_point(
        PointNm {
            x: rect.min_x,
            y: rect.min_y,
        },
        viewport,
        bounds,
    );
    let (x1, y1) = project_point(
        PointNm {
            x: rect.max_x,
            y: rect.max_y,
        },
        viewport,
        bounds,
    );
    let px = RectPx {
        x: x0,
        y: y0,
        width: (x1 - x0).max(1.0),
        height: (y1 - y0).max(1.0),
    };
    out.push(Quad::from_rect(px, color));
    px
}

fn push_point_square(
    out: &mut Vec<Quad>,
    point: PointNm,
    viewport: RectPx,
    bounds: &datum_gui_protocol::SceneBounds,
    size_px: f32,
    color: [f32; 3],
) -> RectPx {
    let (x, y) = project_point(point, viewport, bounds);
    let rect = RectPx {
        x: x - size_px * 0.5,
        y: y - size_px * 0.5,
        width: size_px.max(1.0),
        height: size_px.max(1.0),
    };
    out.push(Quad::from_rect(rect, color));
    rect
}

fn project_point(
    point: PointNm,
    viewport: RectPx,
    bounds: &datum_gui_protocol::SceneBounds,
) -> (f32, f32) {
    let x_scale = viewport.width / (bounds.max_x - bounds.min_x).max(1) as f32;
    let y_scale = viewport.height / (bounds.max_y - bounds.min_y).max(1) as f32;
    (
        viewport.x + (point.x - bounds.min_x) as f32 * x_scale,
        viewport.y + (point.y - bounds.min_y) as f32 * y_scale,
    )
}

fn push_projected_quad(out: &mut Vec<Quad>, quad: &[(f32, f32); 4], color: [f32; 3]) {
    out.push(Quad {
        points: *quad,
        color,
    });
}

fn bounds_from_projected_points(points: &[(f32, f32); 4]) -> RectPx {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for (x, y) in points {
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(*x);
        max_y = max_y.max(*y);
    }
    RectPx {
        x: min_x,
        y: min_y,
        width: (max_x - min_x).max(1.0),
        height: (max_y - min_y).max(1.0),
    }
}

fn inset_rect(rect: RectPx, left: f32, top: f32, right: f32, bottom: f32) -> RectPx {
    RectPx {
        x: rect.x + left,
        y: rect.y + top,
        width: (rect.width - left - right).max(1.0),
        height: (rect.height - top - bottom).max(1.0),
    }
}

fn push_rect_border(out: &mut Vec<Quad>, rect: RectPx, color: [f32; 3], thickness: f32) {
    out.push(Quad::from_rect(
        RectPx {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: thickness,
        },
        color,
    ));
    out.push(Quad::from_rect(
        RectPx {
            x: rect.x,
            y: rect.y + rect.height - thickness,
            width: rect.width,
            height: thickness,
        },
        color,
    ));
    out.push(Quad::from_rect(
        RectPx {
            x: rect.x,
            y: rect.y,
            width: thickness,
            height: rect.height,
        },
        color,
    ));
    out.push(Quad::from_rect(
        RectPx {
            x: rect.x + rect.width - thickness,
            y: rect.y,
            width: thickness,
            height: rect.height,
        },
        color,
    ));
}

fn push_section_divider(out: &mut Vec<Quad>, x: f32, y: f32, width: f32, color: [f32; 3]) {
    out.push(Quad::from_rect(
        RectPx {
            x,
            y,
            width,
            height: 1.0,
        },
        color,
    ));
}

fn push_boolean_row(x: f32, y: f32, label: &str, enabled: bool, text_runs: &mut Vec<TextRun>) {
    draw_text(label, x, y, 13.0, TEXT_SECONDARY, TextFace::Ui, text_runs);
    draw_text(
        if enabled { "ON" } else { "OFF" },
        x + 132.0,
        y,
        13.0,
        if enabled { TEXT_PRIMARY } else { TEXT_MUTED },
        TextFace::Ui,
        text_runs,
    );
}

fn push_key_value(
    x: f32,
    y: f32,
    key: &str,
    value: &str,
    text_runs: &mut Vec<TextRun>,
    value_face: TextFace,
) {
    draw_text(key, x, y, 11.5, TEXT_MUTED, TextFace::Ui, text_runs);
    draw_text(
        value,
        x + 74.0,
        y,
        12.5,
        TEXT_PRIMARY,
        value_face,
        text_runs,
    );
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    if max_chars <= 3 {
        return text.chars().take(max_chars).collect();
    }
    let keep = max_chars - 3;
    let front = keep / 2;
    let back = keep - front;
    let head: String = text.chars().take(front).collect();
    let tail: String = text
        .chars()
        .rev()
        .take(back)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{head}...{tail}")
}

fn prepare_text_buffers(
    font_system: &mut FontSystem,
    text_runs: &[TextRun],
    width: f32,
    height: f32,
) -> Vec<Buffer> {
    let mut buffers = Vec::with_capacity(text_runs.len());
    for run in text_runs {
        let mut buffer = Buffer::new(font_system, Metrics::new(run.size, run.size * 1.22));
        buffer.set_size(font_system, Some(width), Some(height));
        let attrs = text_attrs(run.face);
        buffer.set_text(font_system, &run.text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(font_system, false);
        buffers.push(buffer);
    }
    buffers
}

fn text_attrs(face: TextFace) -> Attrs<'static> {
    match face {
        TextFace::Ui => Attrs::new().family(Family::SansSerif),
        TextFace::Mono => Attrs::new().family(Family::Monospace),
    }
}

fn text_color(color: [f32; 3]) -> Color {
    Color::rgb(
        (color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

fn quad_to_vertices(out: &mut Vec<Vertex>, quad: Quad) {
    let [a, b, c, d] = quad.points;
    out.extend_from_slice(&[
        Vertex {
            pos: [a.0, a.1],
            color: quad.color,
        },
        Vertex {
            pos: [b.0, b.1],
            color: quad.color,
        },
        Vertex {
            pos: [c.0, c.1],
            color: quad.color,
        },
        Vertex {
            pos: [a.0, a.1],
            color: quad.color,
        },
        Vertex {
            pos: [c.0, c.1],
            color: quad.color,
        },
        Vertex {
            pos: [d.0, d.1],
            color: quad.color,
        },
    ]);
}

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("datum-gui-render-shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
struct VsIn {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    out.position = vec4<f32>(in.pos, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#
                .into(),
            ),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("datum-gui-render-pipeline-layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("datum-gui-render-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        Self {
            pipeline,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        prepared: &PreparedScene,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let vertices: Vec<Vertex> = prepared
            .vertices()
            .into_iter()
            .map(|vertex| Vertex {
                pos: [
                    (vertex.pos[0] / width as f32) * 2.0 - 1.0,
                    1.0 - (vertex.pos[1] / height as f32) * 2.0,
                ],
                color: vertex.color,
            })
            .collect();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("datum-gui-render-vertex-buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("datum-gui-render-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("datum-gui-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: APP_BG[0] as f64,
                            g: APP_BG[1] as f64,
                            b: APP_BG[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..vertices.len() as u32, 0..1);
        }
        self.viewport.update(queue, Resolution { width, height });
        let text_buffers = prepare_text_buffers(
            &mut self.font_system,
            &prepared.text_runs,
            width as f32,
            height as f32,
        );
        let text_areas: Vec<TextArea<'_>> = text_buffers
            .iter()
            .zip(prepared.text_runs.iter())
            .map(|(buffer, run)| TextArea {
                buffer,
                left: run.x,
                top: run.y,
                scale: 1.0,
                bounds: TextBounds::default(),
                default_color: text_color(run.color),
                custom_glyphs: &[],
            })
            .collect();
        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .map_err(|error| anyhow::anyhow!("prepare GUI text: {error}"))?;
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("datum-gui-text-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            self.text_renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .map_err(|error| anyhow::anyhow!("render GUI text: {error}"))?;
        }
        queue.submit([encoder.finish()]);
        Ok(())
    }
}

fn suffix_id(id: &str) -> &str {
    id.rsplit(':').next().unwrap_or(id)
}

fn draw_text(
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    color: [f32; 3],
    face: TextFace,
    out: &mut Vec<TextRun>,
) {
    out.push(TextRun {
        text: text.to_string(),
        x,
        y,
        size,
        color,
        face,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_layout_reserves_bottom_dock_and_viewport() {
        let layout = ShellLayout::for_window(1280, 800);
        assert!(layout.viewport.width > 0.0);
        assert_eq!(layout.bottom_strip.height, 44.0);
        assert!(layout.left_sidebar.width > 0.0);
        assert!(layout.right_sidebar.width > 0.0);
    }

    #[test]
    fn prepared_scene_preserves_viewport_dominance() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let prepared = PreparedScene::from_workspace(&state, 1280, 800);
        assert!(prepared.layout.viewport.width > prepared.layout.left_sidebar.width);
        assert!(prepared.layout.viewport.width > prepared.layout.right_sidebar.width / 2.0);
    }

    #[test]
    fn hit_regions_include_review_rows_and_overlay_targets() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let prepared = PreparedScene::from_workspace(&state, 1280, 800);
        assert!(prepared.hit_regions.iter().any(
            |region| matches!(region.target, HitTarget::ReviewAction(ref id) if id == "action-1")
        ));
    }

    #[test]
    fn hit_testing_prefers_overlay_over_underlying_authored_geometry() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let prepared = PreparedScene::from_workspace(&state, 1280, 800);
        let overlay_rect = prepared
            .hit_regions
            .iter()
            .rev()
            .find_map(|region| match &region.target {
                HitTarget::ReviewAction(id) if id == "action-1" => Some(region.rect),
                _ => None,
            })
            .expect("action overlay hit region should exist");
        let hit = prepared
            .hit_test(
                overlay_rect.x + overlay_rect.width / 2.0,
                overlay_rect.y + overlay_rect.height / 2.0,
            )
            .expect("topmost hit should exist");
        assert_eq!(hit, &HitTarget::ReviewAction("action-1".to_string()));
    }
}
