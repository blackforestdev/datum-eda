//! Runtime construction, native GPU setup and initial product-state ownership.
use super::*;

impl Runtime {
    pub(super) async fn new(
        window: &'static Window,
        launch_state: LaunchState,
        scale_factor_override: Option<f32>,
        wake: winit::event_loop::EventLoopProxy<()>,
    ) -> Result<Self> {
        let runtime_started = std::time::Instant::now();
        let LaunchState {
            request: _request,
            mut state,
            camera,
            terminal_launch_context,
            terminal_profiles,
            terminal_sessions,
            workspace_include_review,
            open_global_preferences,
            open_new_project,
            open_project_preferences,
        } = launch_state;
        // The initially-focused leaf seeds its warm camera from the launch fit.
        let initial_focus = state.ui.layout.focused;
        let initial_application_focus = if open_global_preferences || open_new_project {
            ApplicationFocus::Overlay
        } else {
            ApplicationFocus::Editor(initial_focus)
        };
        keyboard_focus::initialize_application_focus(&mut state, initial_application_focus);
        let mut global_preferences =
            global_preferences_runtime::GlobalPreferencesCoordinator::from_platform()?;
        global_preferences.publish_projection(&mut state.ui);
        if open_global_preferences {
            state.ui.global_preferences.reset_transient_view();
            state.ui.global_preferences.open = true;
        }
        let initial_content = state.ui.layout.focused_content();
        let initial_pane_camera = match initial_content {
            PaneContent::Board => camera,
            PaneContent::Schematic => state
                .schematic_scene
                .as_ref()
                .map(|scene| CameraState::fit_to_bounds(&scene.bounds))
                .unwrap_or(camera),
            PaneContent::Revision(_) => camera,
        };
        let wgpu_started = std::time::Instant::now();
        let (instance, surface, adapter, device, queue) = native_gpu::create(window).await?;
        trace_startup_timing(format!(
            "wgpu init {}ms",
            wgpu_started.elapsed().as_millis()
        ));
        let size = window.inner_size();
        let scale_factor = scale_factor_override.unwrap_or_else(|| window.scale_factor() as f32);
        let caps = surface.get_capabilities(&adapter);
        let config = gui_runtime_support::surface_configuration(&caps, size, None);
        let msaa_samples = select_msaa_samples(&adapter, config.format);
        append_gui_diagnostic_line(format!(
            "initial surface configuration requested {}x{} format={:?} present={:?} msaa={}",
            config.width, config.height, config.format, config.present_mode, msaa_samples
        ));
        // Initial configuration joins the same admission path as later frames.
        let device_health = native_device_recovery::DeviceHealth::observe(&device, wake);
        let renderer_started = std::time::Instant::now();
        append_gui_diagnostic_line("renderer init begin");
        let mut renderer = Renderer::new(&device, &queue, config.format, msaa_samples);
        let measurements =
            native_gpu_measurements::Host::new(&mut renderer, &device, &queue, window, None)?;
        append_gui_diagnostic_line("renderer init end");
        trace_startup_timing(format!(
            "renderer init {}ms",
            renderer_started.elapsed().as_millis()
        ));
        let surface_transaction = SurfaceTransaction::new(window, &device_health);
        let mut runtime = Self {
            window,
            instance,
            adapter,
            surface,
            device,
            device_health,
            queue,
            surface_transaction,
            config,
            scale_factor,
            renderer,
            measurements,
            session: LiveDesignSession::new(state),
            camera,
            pane_cameras: PaneCameras::new(initial_focus, initial_content, initial_pane_camera),
            pane_grid_lod: pane_grid_lod::PaneGridLod::default(),
            last_cursor_pos: None,
            pan_gesture: PanGestureState::default(),
            dock_drag_active: false,
            terminal_tab_drag: None,
            terminal_tab_drag_release_suppressed: false,
            terminal_split_drag: None,
            terminal_text_selection_drag: None,
            divider_drag: None,
            terminal_mouse_button: None,
            modifiers: ModifiersState::empty(),
            retained_scene: None,
            retained_scene_cache: Vec::new(),
            prepared_scene: None,
            presented_hits: Default::default(),
            presented_console_layout: None,
            terminal_render_cache: TerminalRenderCache::new(),
            terminal_accessibility:
                terminal_accessibility_bridge::LinuxTerminalAccessibilityBridge::default(),
            schematic_retained_scene: None,
            scene_dirty: true,
            terminal_sessions,
            terminal_launch_context,
            terminal_profiles,
            workspace_include_review,
            terminal_production_refresh_pending: false,
            terminal_workspace_refresh_pending: false,
            terminal_production_refresh_due: None,
            terminal_production_refresh_attempts: 0,
            clipboard: Clipboard::new().ok(),
            pending_terminal_clipboard_write: None,
            terminal_notification_bridge:
                runtime_terminal_notifications::TerminalNotificationBridge::from_environment(),
            window_focused: true,
            application_shutdown_started: None,
            application_shutdown_blocked: false,
            global_preferences_raise_requested: open_global_preferences,
            global_preferences,
            project_preferences_raise_requested: false,
            project_preferences: project_preferences_runtime::ProjectPreferencesCoordinator::new(),
        };
        runtime.sync_terminal_tabs();
        runtime.resize_terminal_to_dock();
        if open_project_preferences {
            runtime.open_project_preferences();
        }
        if open_new_project {
            runtime.open_new_project();
        }
        trace_startup_timing(format!(
            "runtime total {}ms",
            runtime_started.elapsed().as_millis()
        ));
        Ok(runtime)
    }
}
