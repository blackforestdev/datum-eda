use datum_gui_protocol::{
    Affine2DFixedPrimitive, BoardGraphicPrimitive, BoardReviewSceneV1, BoardTextGeometryPrimitive,
    BoardTextPrimitive, ComponentGraphicPrimitive, ComponentTextPrimitive, GlyphMeshAssetPrimitive,
    GlyphMeshHandlePrimitive, PointNm, ProposalOverlayPrimitive, ReviewWorkspaceState,
    SelectionTarget, UnroutedPrimitive, WorkspaceTool,
};
use eda_engine::board::BoardText;
use eda_engine::export::render_silkscreen_text_strokes;
use eda_engine::ir::geometry::{LayerId, Point};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Weight,
};
use std::collections::BTreeMap;
use std::ops::Range;
use taffy::prelude::*;
use uuid::Uuid;
use wgpu::util::DeviceExt;

pub use datum_gui_viewport::CameraState;

mod bottom_dock;
#[cfg(feature = "visual")]
pub mod design_artboards;
mod design_tokens;
mod inspector_check_finding;
mod marking_menu;
mod menu_chrome;
mod side_panels;
#[cfg(feature = "visual")]
pub mod visual_capture;
use bottom_dock::render_bottom_tabs;
use marking_menu::render_marking_menu;
use menu_chrome::render_menu_bar;
use side_panels::render_side_panels;
#[cfg(feature = "visual")]
pub mod visual_diff;
#[cfg(feature = "visual")]
pub mod visual_manifest;
#[cfg(feature = "visual")]
pub mod visual_runner;

include!("render/layout.rs");
include!("render/types.rs");
#[path = "render/pane_chrome.rs"]
mod pane_chrome;
use pane_chrome::render_viewport_panes;
include!("render/scene.rs");
#[path = "render/scene_retained_access.rs"]
mod scene_retained_access;
include!("render/retained.rs");
include!("render/overlay.rs");
mod dim_policy;
pub(crate) use dim_policy::*;
#[path = "render/stroke_policy.rs"]
mod stroke_policy;
pub(crate) use stroke_policy::*;

include!("render/draw_primitives.rs");
include!("render/pads_and_layers.rs");
include!("render/geometry.rs");
include!("render/gpu.rs");
#[path = "render/gpu_surface_pass.rs"]
mod gpu_surface_pass;
#[path = "render/surface_grid_pass.rs"]
mod surface_grid_pass;
#[path = "render/gpu_surface.rs"]
mod gpu_surface;
#[path = "render/grid.rs"]
mod grid;
pub use grid::resolve_surface_grid_lod;
pub(crate) use grid::{push_scene_grid, push_schematic_grid};
#[path = "render/via.rs"]
mod via;
pub(crate) use via::push_via_primitive_world;
#[path = "render/gpu_data.rs"]
mod gpu_data;
pub use gpu_data::Vertex;
pub(crate) use gpu_data::{SceneUniform, ScreenUniform, quads_to_vertices};
#[path = "render/gpu_strokes.rs"]
mod gpu_strokes;
pub(crate) use gpu_strokes::*;
#[path = "render/render_helpers.rs"]
mod render_helpers;
pub(crate) use render_helpers::{
    draw_text, draw_text_clipped, key_value_row_height, suffix_id, text_row_height_for_size,
    trace_graphic_timing, trace_render_timing,
};
include!("render/test_support.rs");
#[cfg(test)]
mod layout_invariant_tests;
#[cfg(test)]
mod lib_extra_tests;
#[cfg(test)]
mod render_contract_tests;
#[cfg(test)]
mod terminal_dock_contract_tests;

include!("render/tests.rs");
#[cfg(test)]
#[path = "render/board_text_mesh_tests.rs"]
mod board_text_mesh_tests;
#[cfg(test)]
#[path = "render/grid_tests.rs"]
mod grid_tests;
