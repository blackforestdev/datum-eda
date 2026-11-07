//! Immutable contact-aware wire handles and shared preview/commit geometry.
mod basis;
mod geometry;
mod lifecycle;
mod move_plan;
mod path;
mod types;
pub use basis::available_handles;
pub use path::routed_wire_points;
pub use types::{
    WireCornerOrder, WireEditAction, WireEditBasis, WireEditPlan, WireEndpoint, WireHandle,
    WireRouteConfig, WireRouting,
};
#[cfg(test)]
mod tests;

#[cfg(test)]
mod lifecycle_tests;

#[cfg(test)]
mod path_tests;

mod transform;
pub use transform::WiringTransformBasis;
#[cfg(test)]
mod transform_tests;
