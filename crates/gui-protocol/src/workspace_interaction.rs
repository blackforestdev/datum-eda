//! Shared transient pointer and crosshair state for every workspace surface.

use crate::PaneContent;

/// The user-selected cursor-crosshair presentation for every drawing surface.
/// Consumer state only; never journaled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrosshairStyle {
    #[default]
    FullViewport,
    Local,
    None,
}

/// Device-pixel screen position, deliberately distinct from authored nm space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScreenPointPx {
    pub x: f32,
    pub y: f32,
}

/// Typed ownership of the object currently under the pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverTarget {
    pub object_id: String,
    pub surface: PaneContent,
}

/// Complete transient pointer state for one shared viewport interaction path.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ViewportInteraction {
    pub cursor: Option<ScreenPointPx>,
    pub hover: Option<HoverTarget>,
}
