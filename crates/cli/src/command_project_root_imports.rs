use std::collections::BTreeMap;

use super::*;

pub(super) fn component_graphic_count<T>(
    map: &BTreeMap<String, Vec<T>>,
    component_key: &str,
) -> usize {
    super::command_project_board_component_query::component_graphic_count(map, component_key)
}

pub(super) fn component_has_persisted_silkscreen(
    project: &LoadedNativeProject,
    component_key: &str,
) -> bool {
    super::command_project_board_component_query::component_has_persisted_silkscreen(
        project,
        component_key,
    )
}

pub(super) fn component_has_persisted_mechanical(
    project: &LoadedNativeProject,
    component_key: &str,
) -> bool {
    super::command_project_board_component_query::component_has_persisted_mechanical(
        project,
        component_key,
    )
}

pub(super) fn component_model_count(project: &LoadedNativeProject, component_key: &str) -> usize {
    super::command_project_board_component_query::component_model_count(project, component_key)
}

pub(super) fn component_package_pad_count(
    project: &LoadedNativeProject,
    component_key: &str,
) -> usize {
    super::command_project_board_component_query::component_package_pad_count(project, component_key)
}

pub(super) fn collect_native_project_pool_ref_views(
    project: &LoadedNativeProject,
) -> Vec<NativeProjectInspectPoolRefView> {
    super::command_project_pool_query::collect_native_project_pool_ref_views(project)
}
