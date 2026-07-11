use super::{
    BoardSurfaceRole, PANEL_CARD_BORDER, PaneRect, Quad, REVIEW_ROW_ACTIVE_BG, RectPx, ShellLayout,
    TEXT_ACCENT, TEXT_MUTED, TEXT_PRIMARY, TextFace, TextRun, VIEWPORT_BG, board_surface_color,
    design_tokens, draw_text, draw_text_clipped, ellipse_points, estimated_text_run_width_px,
    inset_rect, push_convex_polygon_fill, push_rect_border,
};

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
pub(super) fn render_viewport_panes(
    layout: &ShellLayout,
    workspace: &datum_gui_protocol::WorkspaceLayout,
    has_schematic_scene: bool,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let panes = layout.viewport_panes(workspace);
    // The leaf that hosts the live board scene (focus-independent). Its canvas gets
    // the substrate + world PCB; any other board leaf is an inactive placeholder.
    let scene_leaf_id = panes.scene_leaf_id();
    // P2.2a: the FIRST Schematic leaf (walk order) hosts the live companion
    // schematic scene — its viewport matches `schematic_scene_viewport`'s
    // first-match `.find`. Claim it once so any additional Schematic leaf keeps
    // the placeholder under the two-scene (board+schematic) bound of this slice.
    let mut schematic_live_claimed = false;
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
            // Live companion schematic (P2.2a): the first Schematic leaf, when the
            // workspace carries a projected schematic scene, gets its pane canvas
            // painted with the substrate underlay (mirroring the board scene leaf's
            // canvas fill) and NO placeholder caption — the world geometry is drawn
            // by the additive second GPU pass from the schematic RetainedScene. Any
            // further Schematic leaf, or an all-schematic workspace with no scene,
            // falls through to the placeholder below.
            datum_gui_protocol::PaneContent::Schematic
                if has_schematic_scene && !schematic_live_claimed =>
            {
                schematic_live_claimed = true;
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
        panel_quads.push(Quad::from_rect(divider.rect, PANEL_CARD_BORDER));
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
///   = muted (recessed) header fill, dimmed tools, no accent frame, no focus dot —
///   so a glance distinguishes the focused document that owns the Inspector/Layers
///   (context-follows-focus) from the passive one.
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
    // Right edge available for header content. Reserve the focus-dot zone on the
    // focused pane so nothing overlaps it. A pane shrunk by a divider-drag resize
    // must CLIP its header content here instead of spilling the title/tools into
    // the adjacent (enlarged) pane — the panel/text passes are not scissored
    // per-pane, so overflow would otherwise float over the neighbor's header.
    let content_right = header.x + header.width
        - if focused {
            design_tokens::spacing::SP_04 + 7.0 + design_tokens::spacing::SP_02
        } else {
            design_tokens::spacing::SP_02
        };
    let title_x = glyph.x + glyph.width + design_tokens::spacing::SP_03;
    draw_text_clipped(
        title,
        title_x,
        header.y + design_tokens::spacing::SP_03,
        design_tokens::typography::DATA_SIZE,
        if focused { TEXT_PRIMARY } else { TEXT_MUTED },
        TextFace::Mono,
        RectPx {
            x: title_x,
            y: header.y,
            width: (content_right - title_x).max(0.0),
            height: header.height,
        },
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
        // Cull tool buttons that would overflow the pane (after a resize shrank
        // it): x only grows, so once one does not fit, none after it will. This
        // stops the tool cluster bleeding into the adjacent pane's header.
        if rect.x + rect.width > content_right {
            break;
        }
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
