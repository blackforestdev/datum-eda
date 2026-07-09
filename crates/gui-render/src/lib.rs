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
include!("render/scene.rs");
include!("render/retained.rs");
include!("render/overlay.rs");
mod dim_policy;
pub(crate) use dim_policy::*;

include!("render/draw_primitives.rs");
include!("render/pads_and_layers.rs");
include!("render/geometry.rs");
include!("render/gpu.rs");
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
