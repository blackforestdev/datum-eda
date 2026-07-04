// commands/pool — pool queries shared across surfaces (pool refs, library
// objects, models) and pool package-graphics materialization consumed by the
// board handoff and board-component mutation paths.
//
// Wave 2 move. materialization/query came from decls in command_project.rs
// with named re-exports in command_project_surface.rs; the re-exports below
// reproduce those exactly (collect_native_project_pool_ref_views was
// pub(super) under command_project and is now pub(crate) because its
// consumers — command_project_root_imports.rs — live outside this family).
//
// validation.rs stays a #[path] child of command_project_validate.rs (its
// only consumer), following the nested-child pattern; it is deliberately
// not declared here.

mod materialization;
mod query;

pub(crate) use self::materialization::{
    materialize_supported_pool_package_graphics, resolve_native_project_pool_path,
};
pub(crate) use self::query::{
    collect_native_project_pool_ref_views, query_native_project_pool_library_objects,
    query_native_project_pool_models, query_native_project_pools,
};
