use super::*;

pub(super) fn parse_window_size(value: &str) -> Result<(u32, u32)> {
    let (width, height) = value
        .split_once('x')
        .ok_or_else(|| anyhow::anyhow!("window size must use <width>x<height>"))?;
    let width = width
        .parse::<u32>()
        .with_context(|| format!("parse window width from {value:?}"))?;
    let height = height
        .parse::<u32>()
        .with_context(|| format!("parse window height from {value:?}"))?;
    if width == 0 || height == 0 {
        anyhow::bail!("window size dimensions must be non-zero");
    }
    Ok((width, height))
}
#[derive(Debug, Clone, Parser)]
#[command(name = "datum-gui", about = "Datum EDA Phase 1 read-only board GUI")]
pub(super) struct GuiArgs {
    #[arg(long = "demo-known-good", default_value_t = false)]
    pub(super) demo_known_good: bool,
    #[arg(
        long = "board",
        help = "Open a KiCad .kicad_pcb board file directly",
        conflicts_with = "schematic_file"
    )]
    pub(super) board_file: Option<PathBuf>,
    #[arg(
        long = "schematic",
        help = "Open a KiCad .kicad_sch schematic file directly",
        conflicts_with = "artifact_path",
        conflicts_with = "demo_known_good"
    )]
    pub(super) schematic_file: Option<PathBuf>,
    #[arg(long = "artifact")]
    pub(super) artifact_path: Option<PathBuf>,
    #[arg(long = "project-root")]
    pub(super) project_root: Option<PathBuf>,
    #[arg(
        long = "select",
        help = "Preset selection to a component by reference designator (e.g. R1) or object_id"
    )]
    pub(super) select: Option<String>,
    #[arg(long = "net")]
    pub(super) net_uuid: Option<String>,
    #[arg(long = "from-anchor")]
    pub(super) from_anchor_pad_uuid: Option<String>,
    #[arg(long = "to-anchor")]
    pub(super) to_anchor_pad_uuid: Option<String>,
    #[arg(long = "profile")]
    pub(super) profile: Option<String>,
    #[arg(long = "visual-test", default_value_t = false)]
    pub(super) visual_test: bool,
    /// Capture/test affordance (decision 021): seed the workspace pane tree
    /// (`state.ui.layout`) to a named shape at boot so each pane state can be
    /// screenshotted for owner validation and later golden coverage. Omitting it
    /// (the default) leaves the default Board|Schematic layout untouched, so the
    /// parity gate stays pixel-identical. Accepts:
    /// `single` | `board-schematic` | `horizontal-split` | `zoom`.
    #[arg(long = "initial-layout")]
    pub(super) initial_layout: Option<String>,
    /// Capture/test affordance: open a named top menu dropdown at boot (sets
    /// `state.ui.active_menu`), so a dropdown-open state can be screenshotted for
    /// owner validation. Applied in both the windowed and offscreen visual-test
    /// paths. Absent (the default) leaves every menu closed, so the parity gate
    /// stays pixel-identical. The menu name matches the menubar title (e.g.
    /// `File`, `View`); an unknown name simply renders no open dropdown.
    #[arg(long = "open-menu")]
    pub(super) open_menu: Option<String>,
    /// Capture/test affordance: focus the first leaf showing the named content
    /// (`board` | `schematic`) at boot, so the "view one pane while another is
    /// focused" states can be screenshotted for owner validation. Applied in both
    /// the windowed and offscreen visual-test paths. Absent (the default) leaves the
    /// default Board focus, so the parity gate stays pixel-identical. An unknown or
    /// absent-content name is a no-op. Pane focus is consumer view state, never
    /// journaled.
    #[arg(long = "focus-pane")]
    pub(super) focus_pane: Option<String>,
    #[arg(long = "window-size", default_value = "1280x768")]
    pub(super) window_size: String,
    #[arg(long = "screenshot-out")]
    pub(super) screenshot_out: Option<PathBuf>,
    #[arg(long = "exit-after-screenshot", default_value_t = false)]
    pub(super) exit_after_screenshot: bool,
    #[arg(long = "window-visual-test", default_value_t = false, hide = true)]
    pub(super) window_visual_test: bool,
    #[arg(long = "visual-scale-factor", hide = true)]
    pub(super) visual_scale_factor: Option<f32>,
    #[arg(long = "interaction-smoke", default_value_t = false, hide = true)]
    pub(super) interaction_smoke: bool,
    #[arg(long = "resize-torture-smoke", default_value_t = false, hide = true)]
    pub(super) resize_torture_smoke: bool,
    #[arg(long = "kwin-lifecycle-smoke", default_value_t = false, hide = true)]
    pub(super) kwin_lifecycle_smoke: bool,
}
pub(super) struct LaunchState {
    pub(super) request: LiveReviewRequest,
    pub(super) state: datum_gui_protocol::ReviewWorkspaceState,
    pub(super) camera: CameraState,
    pub(super) terminal_launch_context: TerminalLaunchContext,
    pub(super) terminal_sessions: TerminalSessionRegistry,
    pub(super) workspace_include_review: bool,
}
impl GuiArgs {
    pub(super) fn visual_window_size(&self) -> Result<(u32, u32)> {
        parse_window_size(&self.window_size)
    }

    pub(super) fn validate_visual_args(&self) -> Result<()> {
        if !self.visual_test {
            return Ok(());
        }
        if self.screenshot_out.is_none() {
            anyhow::bail!("--visual-test requires --screenshot-out");
        }
        if let Some(scale_factor) = self.visual_scale_factor
            && (!scale_factor.is_finite() || scale_factor <= 0.0) {
                anyhow::bail!("--visual-scale-factor must be a positive finite number");
            }
        self.visual_window_size()?;
        Ok(())
    }

    /// Apply the `--initial-layout` capture affordance (decision 021) to a freshly
    /// loaded workspace pane tree. A no-op when the flag is absent (default), so
    /// nothing changes without it — the parity gate stays pixel-identical. An
    /// unrecognized value is a no-op (leaves the default layout) rather than a
    /// hard failure, since this is a test/capture affordance, not product input.
    /// Pane layout is consumer/workspace view state, never journaled.
    pub(super) fn apply_initial_layout(&self, layout: &mut datum_gui_protocol::WorkspaceLayout) {
        use datum_gui_protocol::{SplitOrientation, WorkspaceLayout, WorkspacePreset};
        let Some(spec) = self.initial_layout.as_deref() else {
            return;
        };
        match spec {
            "single" => layout.apply_preset(WorkspacePreset::Single),
            "board-schematic" => layout.apply_preset(WorkspacePreset::BoardSchematic),
            "horizontal-split" => {
                // A clean top/bottom split: one Board leaf split horizontally.
                *layout = WorkspaceLayout::single();
                layout.split_focused(SplitOrientation::Horizontal);
            }
            "zoom" => {
                // The default two-pane layout with the focused leaf maximized.
                layout.apply_preset(WorkspacePreset::BoardSchematic);
                layout.toggle_zoom();
            }
            _ => {}
        }
    }

    /// Apply the `--focus-pane` capture affordance: focus the first leaf showing
    /// the requested content (`board` | `schematic`). A no-op when the flag is
    /// absent, unrecognized, or no such leaf exists — so the default Board focus is
    /// untouched and the parity gate stays pixel-identical. Pane focus is
    /// consumer/workspace view state, never journaled.
    pub(super) fn apply_focus_pane(&self, layout: &mut datum_gui_protocol::WorkspaceLayout) {
        use datum_gui_protocol::PaneContent;
        let want = match self.focus_pane.as_deref() {
            Some("board") => PaneContent::Board,
            Some("schematic") => PaneContent::Schematic,
            _ => return,
        };
        for id in layout.leaves() {
            let mut probe = layout.clone();
            probe.focused = id;
            if probe.focused_content() == want {
                layout.focused = id;
                return;
            }
        }
    }

    pub(super) fn wants_plain_project_board_view(&self) -> bool {
        (self.project_root.is_some() || self.board_file.is_some() || self.schematic_file.is_some())
            && self.artifact_path.is_none()
            && self.net_uuid.is_none()
            && self.from_anchor_pad_uuid.is_none()
            && self.to_anchor_pad_uuid.is_none()
    }

    pub(super) fn resolve_request(&self) -> Result<LiveReviewRequest> {
        if self.demo_known_good {
            return ensure_known_good_demo_request();
        }
        if let Some(board_file) = &self.board_file {
            return materialize_kicad_board_request(board_file, self.project_root.clone());
        }
        if let Some(schematic_file) = &self.schematic_file {
            let source = schematic_file.canonicalize().with_context(|| {
                format!(
                    "failed to resolve KiCad schematic {}",
                    schematic_file.display()
                )
            })?;
            return Ok(LiveReviewRequest {
                project_root: self
                    .project_root
                    .clone()
                    .or_else(|| source.parent().map(PathBuf::from))
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "--project-root is required when --schematic has no parent directory"
                        )
                    })?,
                board_file: None,
                artifact_path: None,
                net_uuid: None,
                from_anchor_pad_uuid: None,
                to_anchor_pad_uuid: None,
                profile: None,
            });
        }
        if let Some(artifact_path) = &self.artifact_path {
            let project_root = self
                .project_root
                .clone()
                .or_else(|| artifact_path.parent().map(PathBuf::from))
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "--project-root is required when --artifact has no parent directory"
                    )
                })?;
            return Ok(LiveReviewRequest {
                project_root,
                board_file: None,
                artifact_path: Some(artifact_path.clone()),
                net_uuid: None,
                from_anchor_pad_uuid: None,
                to_anchor_pad_uuid: None,
                profile: self.profile.clone(),
            });
        }
        Ok(LiveReviewRequest {
            project_root: self.project_root.clone().ok_or_else(|| {
                anyhow::anyhow!(
                    "--project-root, --board, --schematic, or --demo-known-good is required"
                )
            })?,
            board_file: None,
            artifact_path: None,
            net_uuid: self.net_uuid.clone(),
            from_anchor_pad_uuid: self.from_anchor_pad_uuid.clone(),
            to_anchor_pad_uuid: self.to_anchor_pad_uuid.clone(),
            profile: self.profile.clone(),
        })
    }

    pub(super) fn load_launch_state(&self) -> Result<LaunchState> {
        let request_started = std::time::Instant::now();
        append_gui_diagnostic_line("request resolve begin");
        let request = self
            .resolve_request()
            .context("resolve GUI launch review context")?;
        append_gui_diagnostic_line(format!(
            "request resolve end project_root={} board_file={:?}",
            request.project_root.display(),
            request.board_file
        ));
        trace_startup_timing(format!(
            "request resolve {}ms",
            request_started.elapsed().as_millis()
        ));

        let workspace_started = std::time::Instant::now();
        append_gui_diagnostic_line("workspace load begin");
        let workspace_include_review = !self.wants_plain_project_board_view();
        let mut state = if let Some(schematic_file) = &self.schematic_file {
            load_kicad_schematic_workspace_state(schematic_file)
                .context("load schematic workspace state")?
        } else if self.wants_plain_project_board_view() {
            load_board_editor_workspace_state(&request)
                .context("load board editor workspace state")?
        } else {
            load_live_workspace_state(&request).context("load live M7 review workspace state")?
        };
        append_gui_diagnostic_line("workspace load end");
        trace_startup_timing(format!(
            "workspace load {}ms",
            workspace_started.elapsed().as_millis()
        ));

        // Preset a component selection when requested. `--select` accepts a human-
        // stable reference designator (e.g. R1) resolved against the loaded scene,
        // or a raw object_id. This is additive over the plain board view (which
        // loads with `selection = None`) and reuses the existing selection API;
        // an unknown selector leaves the inspector empty rather than crashing.
        if let Some(sel) = &self.select {
            let object_id = state
                .scene
                .components
                .iter()
                .find(|c| c.reference == *sel)
                .map(|c| c.object_id.clone())
                .unwrap_or_else(|| sel.clone());
            state.select_authored_object(&object_id);
        }

        // Capture/test affordance: seed the pane tree if --initial-layout was set
        // (a no-op otherwise, so the default boot layout is untouched).
        self.apply_initial_layout(&mut state.ui.layout);
        // Capture/test affordance: focus a named pane (a no-op otherwise).
        self.apply_focus_pane(&mut state.ui.layout);

        // Capture/test affordance: open a named menu dropdown at boot if
        // --open-menu was set (a no-op otherwise, so parity stays identical).
        // This LaunchState builder is shared by the windowed and offscreen
        // visual-test paths, so setting it here covers both.
        if let Some(menu) = &self.open_menu {
            state.ui.active_menu = Some(menu.clone());
        }

        let camera_started = std::time::Instant::now();
        append_gui_diagnostic_line("camera fit begin");
        let camera = CameraState::fit_to_bounds(&state.scene.bounds);
        append_gui_diagnostic_line("camera fit end");
        trace_startup_timing(format!(
            "camera fit {}ms",
            camera_started.elapsed().as_millis()
        ));

        let terminal_launch_context =
            terminal_launch_context_from_state(&request.project_root, &state);
        let terminal_started = std::time::Instant::now();
        append_gui_diagnostic_line("terminal spawn begin");
        let mut terminal_sessions = TerminalSessionRegistry::spawn(&terminal_launch_context)
            .context("spawn integrated terminal lane")?;
        terminal_sessions.sync_lane_tabs(&mut state.ui.terminal);
        append_gui_diagnostic_line("terminal spawn end");
        trace_startup_timing(format!(
            "terminal spawn {}ms",
            terminal_started.elapsed().as_millis()
        ));

        Ok(LaunchState {
            request,
            state,
            camera,
            terminal_launch_context,
            terminal_sessions,
            workspace_include_review,
        })
    }
}

#[cfg(test)]
mod initial_layout_tests {
    use super::*;
    use datum_gui_protocol::{PaneNode, SplitOrientation, WorkspaceLayout};

    fn args_with(spec: Option<&str>) -> GuiArgs {
        let mut argv = vec!["datum-gui".to_string()];
        if let Some(s) = spec {
            argv.push("--initial-layout".to_string());
            argv.push(s.to_string());
        }
        GuiArgs::parse_from(argv)
    }

    #[test]
    fn absent_flag_leaves_default_layout() {
        let args = args_with(None);
        let mut layout = WorkspaceLayout::default();
        let before = layout.clone();
        args.apply_initial_layout(&mut layout);
        assert_eq!(
            layout, before,
            "default layout must be untouched without --initial-layout"
        );
    }

    #[test]
    fn focus_pane_flag_parses_and_focuses_named_content() {
        use datum_gui_protocol::{PaneContent, WorkspaceLayout};
        // Absent: no focus change requested.
        let args = GuiArgs::parse_from(["datum-gui".to_string()]);
        assert_eq!(args.focus_pane, None);
        // Present: focuses the first leaf with the requested content.
        let args = GuiArgs::parse_from([
            "datum-gui".to_string(),
            "--focus-pane".to_string(),
            "schematic".to_string(),
        ]);
        assert_eq!(args.focus_pane.as_deref(), Some("schematic"));
        let mut layout = WorkspaceLayout::default();
        assert_eq!(layout.focused_content(), PaneContent::Board);
        args.apply_focus_pane(&mut layout);
        assert_eq!(
            layout.focused_content(),
            PaneContent::Schematic,
            "--focus-pane schematic must focus the schematic leaf"
        );
    }

    #[test]
    fn open_menu_flag_parses_and_defaults_absent() {
        // Absent (default): no menu requested, so parity capture is untouched.
        let args = GuiArgs::parse_from(["datum-gui".to_string()]);
        assert_eq!(args.open_menu, None);
        // Present: the named menu is captured for the boot-time open-dropdown state.
        let args = GuiArgs::parse_from([
            "datum-gui".to_string(),
            "--open-menu".to_string(),
            "View".to_string(),
        ]);
        assert_eq!(args.open_menu.as_deref(), Some("View"));
    }

    #[test]
    fn single_preset_collapses_to_one_leaf() {
        let args = args_with(Some("single"));
        let mut layout = WorkspaceLayout::default();
        args.apply_initial_layout(&mut layout);
        assert_eq!(layout.leaves().len(), 1);
        assert!(layout.zoomed.is_none());
    }

    #[test]
    fn board_schematic_preset_is_two_leaves() {
        let args = args_with(Some("board-schematic"));
        let mut layout = WorkspaceLayout::single();
        args.apply_initial_layout(&mut layout);
        assert_eq!(layout.leaves().len(), 2);
    }

    #[test]
    fn horizontal_split_is_two_horizontally_stacked_leaves() {
        let args = args_with(Some("horizontal-split"));
        let mut layout = WorkspaceLayout::default();
        args.apply_initial_layout(&mut layout);
        assert_eq!(layout.leaves().len(), 2);
        assert!(matches!(
            layout.root,
            PaneNode::Split {
                orientation: SplitOrientation::Horizontal,
                ..
            }
        ));
    }

    #[test]
    fn zoom_maximizes_the_focused_leaf() {
        let args = args_with(Some("zoom"));
        let mut layout = WorkspaceLayout::default();
        args.apply_initial_layout(&mut layout);
        assert_eq!(layout.zoomed, Some(layout.focused));
        // Zoom is transient: the underlying tree still has both leaves.
        assert_eq!(layout.leaves().len(), 2);
    }

    #[test]
    fn unknown_value_is_a_no_op() {
        let args = args_with(Some("bogus"));
        let mut layout = WorkspaceLayout::default();
        let before = layout.clone();
        args.apply_initial_layout(&mut layout);
        assert_eq!(layout, before);
    }
}
