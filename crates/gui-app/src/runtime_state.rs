//! Live runtime state; GPU replacement never reconstructs the session or terminal registry.
use super::*;

pub(super) struct Runtime {
    pub(super) instance: wgpu::Instance,
    pub(super) adapter: wgpu::Adapter,
    pub(super) surface: wgpu::Surface<'static>,
    pub(super) device: wgpu::Device,
    pub(super) device_health: native_device_recovery::DeviceHealth,
    pub(super) queue: wgpu::Queue,
    pub(super) config: wgpu::SurfaceConfiguration,
    // Drop renderer references before recording host attachment retirement.
    pub(super) renderer: Renderer,
    pub(super) surface_transaction: SurfaceTransaction,
    pub(super) scale_factor: f32,
    pub(super) measurements: native_gpu_measurements::Host,
    pub(super) session: LiveDesignSession,
    /// Camera for the renderer's live board leaf. Pointer and focused commands
    /// reach it only when their typed pane route names that leaf; schematic and
    /// additional-pane cameras remain independently warm in `pane_cameras`.
    pub(super) camera: CameraState,
    /// Warm per-leaf view cameras keyed by `PaneId` (decision 021, P2.1b).
    pub(super) pane_cameras: PaneCameras,
    pub(super) pane_grid_lod: pane_grid_lod::PaneGridLod,
    pub(super) last_cursor_pos: Option<(f32, f32)>,
    pub(super) pan_gesture: PanGestureState,
    pub(super) dock_drag_active: bool,
    pub(super) terminal_tab_drag: Option<terminal_tab_drag::TerminalTabDrag>,
    pub(super) terminal_tab_drag_release_suppressed: bool,
    pub(super) terminal_split_drag: Option<TerminalSplitDividerDrag>,
    pub(super) terminal_text_selection_drag:
        Option<runtime_terminal_pointer::TerminalSelectionPoint>,
    /// In-progress split divider-drag resize (decision 021), or `None`. Consumer
    /// view state; never journaled.
    pub(super) divider_drag: Option<DividerDrag>,
    pub(super) terminal_mouse_button: Option<MouseButton>,
    pub(super) modifiers: ModifiersState,
    pub(super) retained_scene: Option<RetainedScene>,
    pub(super) retained_scene_cache: RetainedSceneHistory,
    pub(super) prepared_scene: Option<PreparedScene>,
    pub(super) presented_hits: gui_runtime_support::presented_hit_regions::PresentedHitRegions,
    /// Console input/lifetime queries must not prepare a dirty workspace frame.
    pub(super) presented_console_layout: Option<datum_gui_render::ConsoleOverlayLayout>,
    pub(super) terminal_render_cache: TerminalRenderCache,
    pub(super) terminal_accessibility:
        terminal_accessibility_bridge::LinuxTerminalAccessibilityBridge,
    // Lazily retained schematic world geometry; camera-independent geometry
    // survives frame invalidation and eligible surface-size changes.
    pub(super) schematic_retained_scene: Option<RetainedScene>,
    pub(super) scene_dirty: bool,
    pub(super) terminal_sessions: TerminalSessionRegistry,
    pub(super) terminal_launch_context: TerminalLaunchContext,
    pub(super) terminal_profiles: terminal_profile::TerminalProfileCatalog,
    pub(super) workspace_include_review: bool,
    pub(super) terminal_production_refresh_pending: bool,
    pub(super) terminal_workspace_refresh_pending: bool,
    pub(super) terminal_production_refresh_due: Option<std::time::Instant>,
    pub(super) terminal_production_refresh_attempts: u8,
    pub(super) clipboard: Option<Clipboard>,
    pub(super) pending_terminal_clipboard_write:
        Option<runtime_terminal_clipboard::PendingTerminalClipboardWrite>,
    pub(super) terminal_notification_bridge:
        runtime_terminal_notifications::TerminalNotificationBridge,
    pub(super) window_focused: bool,
    pub(super) application_shutdown_started: Option<std::time::Instant>,
    pub(super) application_shutdown_blocked: bool,
    pub(super) global_preferences_raise_requested: bool,
    pub(super) global_preferences: global_preferences_runtime::GlobalPreferencesCoordinator,
    pub(super) project_preferences_raise_requested: bool,
    pub(super) project_preferences: project_preferences_runtime::ProjectPreferencesCoordinator,
    // Rust drops fields in declaration order. Keep the native display alive
    // until every GPU handle (including unused backend instances) is destroyed.
    pub(super) window: std::sync::Arc<Window>,
}
