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
#[command(name = "datum-gui", about = "Datum M7 route-proposal review spike")]
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
        if let Some(scale_factor) = self.visual_scale_factor {
            if !scale_factor.is_finite() || scale_factor <= 0.0 {
                anyhow::bail!("--visual-scale-factor must be a positive finite number");
            }
        }
        self.visual_window_size()?;
        Ok(())
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
