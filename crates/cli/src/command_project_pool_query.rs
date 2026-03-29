use std::path::Path;

use anyhow::Result;

use super::{LoadedNativeProject, NativeProjectInspectPoolRefView, load_native_project};

pub(super) fn collect_native_project_pool_ref_views(
    project: &LoadedNativeProject,
) -> Vec<NativeProjectInspectPoolRefView> {
    project
        .manifest
        .pools
        .iter()
        .map(|pool_ref| {
            let resolved_path =
                super::resolve_native_project_pool_path(&project.root, &pool_ref.path);
            NativeProjectInspectPoolRefView {
                manifest_path: pool_ref.path.clone(),
                priority: pool_ref.priority,
                resolved_path: resolved_path.display().to_string(),
                exists: resolved_path.exists(),
            }
        })
        .collect()
}

pub(crate) fn query_native_project_pools(
    root: &Path,
) -> Result<Vec<NativeProjectInspectPoolRefView>> {
    let project = load_native_project(root)?;
    Ok(collect_native_project_pool_ref_views(&project))
}
