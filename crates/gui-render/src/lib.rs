mod hit_target;
pub use hit_target::HitTarget;
mod geometry_output;
#[path = "render/glyph_mesh_lookup.rs"]
mod glyph_mesh_lookup;
#[path = "render/presentation_ranges.rs"]
mod presentation_ranges;
#[path = "render/source_types.rs"]
mod source_types;
#[path = "render/world_polygon.rs"]
mod world_polygon;
pub use datum_gui_viewport::CameraState;
use geometry_output::Output;
use source_types::*;
#[path = "render/world_primitives.rs"]
mod world_primitives;
use taffy::prelude::*;
mod bottom_dock;
#[cfg(feature = "visual")]
pub mod capture_resource;
mod datum_console;
mod design_tokens;
mod global_preferences_dialog;
mod global_preferences_primitives;
mod inspector_check_finding;
mod marking_menu;
mod menu_chrome;
mod new_project_dialog;
#[path = "render/resource_observation.rs"]
pub mod resource_observation;
mod revision_workspace;
mod side_panels;
mod terminal_clipboard_menu;
mod terminal_core_render;
mod terminal_render_cache;
mod text_gpu;
pub use text_gpu::lifetime::observation as gpu_allocation_observation;
pub use text_gpu::upload_totals::{AllocationUploadFrame, UploadFrame, UploadTotals};
pub use text_gpu::{ReleasedAllocations, RetirementReason};
mod text_layout;
pub use terminal_render_cache::TerminalRenderCache;
mod terminal_pane_render;
pub use terminal_pane_render::TerminalPaneRenderState;
#[cfg(test)]
#[path = "terminal_font_tests.rs"]
mod terminal_font_tests;
mod terminal_scene;
mod terminal_session_chrome;
mod terminal_tab_strip;
#[cfg(test)]
mod terminal_tab_strip_tests;
#[cfg(feature = "visual")]
mod visual;
#[cfg(feature = "visual")]
pub use visual::{design_artboards, visual_capture, visual_diff, visual_manifest, visual_runner};

include!("render/layout.rs");
include!("render/types.rs");
#[path = "render/draw_command.rs"]
mod draw_command;
use draw_command::RetainedDrawCommand;
#[path = "render/pane_chrome.rs"]
mod pane_chrome;
use pane_chrome::render_viewport_panes;
include!("render/scene.rs");
#[path = "render/retained.rs"]
mod retained;
#[path = "render/scene_retained_access.rs"]
mod scene_retained_access;
include!("render/overlay.rs");
mod dim_policy;
#[path = "render/native_selection_overlay.rs"]
mod native_selection_overlay;
#[path = "render/selection_stroke.rs"]
mod selection_stroke;
pub(crate) use dim_policy::*;
#[path = "render/stroke_policy.rs"]
mod stroke_policy;
pub(crate) use stroke_policy::*;

include!("render/draw_primitives.rs");
include!("render/pads_and_layers.rs");
include!("render/geometry.rs");
include!("render/gpu.rs");
#[path = "render/gpu_surface.rs"]
mod gpu_surface;
#[path = "render/gpu_surface_pass.rs"]
mod gpu_surface_pass;
#[path = "render/grid.rs"]
mod grid;
#[path = "render/surface_grid_pass.rs"]
mod surface_grid_pass;
pub use grid::resolve_surface_grid_lod;
#[path = "render/via.rs"]
mod via;
pub(crate) use via::push_via_primitive_world;
#[path = "render/gpu_data.rs"]
mod gpu_data;
pub use gpu_data::{DocumentGpuUsage, Vertex};
pub(crate) use gpu_data::{SceneUniform, ScreenUniform, quads_to_vertices};
#[path = "render/gpu_strokes.rs"]
mod gpu_strokes;
pub(crate) use gpu_strokes::*;
#[path = "render/render_helpers.rs"]
mod render_helpers;
pub(crate) use render_helpers::{
    draw_rich_text, draw_text, draw_text_clipped, key_value_row_height, suffix_id,
    text_row_height_for_size, trace_graphic_timing, trace_render_timing,
};
include!("render/test_support.rs");
#[cfg(test)]
mod global_preferences_dialog_tests;
#[cfg(test)]
mod layout_invariant_tests;
#[cfg(test)]
mod retained_draw_order_tests;
#[cfg(test)]
mod terminal_dock_contract_tests;

#[cfg(test)]
#[path = "render/board_text_mesh_tests.rs"]
mod board_text_mesh_tests;
#[cfg(test)]
#[path = "render/grid_tests.rs"]
mod grid_tests;
#[cfg(test)]
#[path = "render/tests.rs"]
mod renderer_tests;
#[cfg(test)]
mod revision_pane_tests;
