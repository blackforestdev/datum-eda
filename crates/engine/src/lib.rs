// eda-engine: headless PCB design engine
//
// This crate has no GUI, rendering, or windowing dependencies.
// It compiles as a library consumed by: cli, engine-daemon, test-harness.
//
// Module organization follows specs/ENGINE_SPEC.md.
// Public API surface is in api::Engine.

pub mod api;
pub mod board;
pub mod connectivity;
pub mod drc;
pub mod erc;
pub mod error;
pub mod export;
pub mod import;
pub mod ir;
pub mod ops;
pub mod pool;
pub mod rules;
pub mod schematic;
pub mod session;
