//! Runtime construction, native GPU setup and initial product-state ownership.
use super::*;

impl Runtime {
    pub(super) async fn new(
        window: std::sync::Arc<Window>,
        launch_state: LaunchState,
        scale_factor_override: Option<f32>,
        wake: winit::event_loop::EventLoopProxy<()>,
    ) -> Result<Self> {
        let runtime_started = std::time::Instant::now();
        let LaunchState {
            reuse_resize_buffers,
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
            PaneContent::Library | PaneContent::Revision(_) => camera,
        };
        let native_edit = runtime_native_edit::EditWork::new(wake.clone());
        let native_preparation =
            if datum_gui_protocol::gui_menu_model::native_project_edit_available(&state) {
                state.backing.as_ref().and_then(|backing| {
                    native_edit.prepare_read(backing.request.project_root.clone())
                })
            } else {
                None
            };
        let wgpu_started = std::time::Instant::now();
        let (instance, surface, adapter, device, queue) =
            native_gpu::create(window.clone()).await?;
        trace_startup_timing(format!(
            "wgpu init {}ms",
            wgpu_started.elapsed().as_millis()
        ));
        let size = window.inner_size();
        let scale_factor = scale_factor_override.unwrap_or_else(|| window.scale_factor() as f32);
        let caps = surface.get_capabilities(&adapter);
        let mut config = gui_runtime_support::surface_configuration(&caps, size, None);
        let native_content =
            native_content_probe::Probe::from_environment(&caps, &mut config, wake.clone())?;
        let msaa_samples = select_msaa_samples(&adapter, config.format);
        append_gui_diagnostic_line(format!(
            "initial surface configuration requested {}x{} format={:?} present={:?} msaa={}",
            config.width, config.height, config.format, config.present_mode, msaa_samples
        ));
        // Initial configuration joins the same admission path as later frames.
        let device_health = native_device_recovery::DeviceHealth::observe(&device, wake.clone());
        let renderer_started = std::time::Instant::now();
        append_gui_diagnostic_line("renderer init begin");
        let mut renderer = Renderer::new(&device, &queue, config.format, msaa_samples)?;
        pollster::block_on(renderer.admit_damage_restoration(&device, &adapter));
        let measurements =
            native_gpu_measurements::Host::new(&mut renderer, &device, &queue, &window, None)?;
        append_gui_diagnostic_line("renderer init end");
        trace_startup_timing(format!(
            "renderer init {}ms",
            renderer_started.elapsed().as_millis()
        ));
        let mut surface_transaction = SurfaceTransaction::new(&window, &device_health);
        if reuse_resize_buffers {
            surface_transaction.enable_resize_buffer_reuse();
        }
        let mut runtime = Self {
            native_content,
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
            selection_read_cache: Default::default(),
            selection_run: runtime_selection_run::RunWork::new(wake.clone()),
            native_edit,
            project_open: project_open::OpenWork::new(wake.clone()),
            folder_browser: folder_browser::FolderBrowser::new(wake.clone()),
            native_selection: None,
            selection_gesture: None,
            selection_evaluation: None,
            selection_release_suppressed: false,
            selection_tick: None,
            render_sources: Default::default(),
            camera,
            pane_cameras: PaneCameras::new(initial_focus, initial_content, initial_pane_camera),
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
            menu_keys: Default::default(),
            presented_hits: Default::default(),
            presented_console_layout: None,
            terminal_accessibility:
                terminal_accessibility_bridge::LinuxTerminalAccessibilityBridge::default(),
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
        // Finish the startup-only read before selection admission: an independent
        // reader must not encounter our companion's transient writer exclusion.
        // No surface frame/input dispatch has begun; edit-time checks remain fresh.
        if let Some(preparation) = native_preparation {
            let _ = preparation.join();
        }
        // Prepare the validated source basis before the first interactive frame.
        // Subsequent actions still compare canonical inputs through the resolver cache.
        if let Err(error) = runtime.ensure_selection_authority() {
            append_gui_diagnostic_line(format!("initial selection source unavailable: {error}"));
        }
        trace_startup_timing(format!(
            "runtime total {}ms",
            runtime_started.elapsed().as_millis()
        ));
        Ok(runtime)
    }
}
