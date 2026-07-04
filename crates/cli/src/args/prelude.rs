// Scope prelude for the args/ family files: they reach these names through
// `use crate::*` (crate root's `use args::*` re-exports this module).
// main.rs also imports most of these names at crate root for its own use, so
// the compiler credits those bindings and would flag these re-exports as
// unused — but removing them would couple every arg file to main.rs's import
// list (and would drop `Subcommand`, which main.rs does not import).
#[allow(unused_imports)]
pub(crate) use std::path::PathBuf;

#[allow(unused_imports)]
pub(crate) use clap::{Parser, Subcommand};
#[allow(unused_imports)]
pub(crate) use eda_engine::api::ScopedComponentReplacementPlan;
#[allow(unused_imports)]
pub(crate) use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
pub(crate) use uuid::Uuid;
