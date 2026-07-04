// commands/manufacturing — manufacturing set export/validation, plans,
// plan proposals, and the CLI view types for manufacturing commands.

#[allow(unused_imports)]
use super::*;

pub(crate) mod manufacturing;
mod plan_proposals;
mod plans;
mod views;

pub(crate) use self::manufacturing::{
    compare_native_project_manufacturing_set, export_native_project_manufacturing_set,
    export_native_project_manufacturing_set_without_output_run,
    inspect_native_project_manufacturing_set, manifest_native_project_manufacturing_set,
    report_native_project_manufacturing, validate_native_project_manufacturing_set,
};
pub(crate) use self::plan_proposals::{
    propose_create_native_project_manufacturing_plan,
    propose_create_native_project_panel_projection,
    propose_delete_native_project_manufacturing_plan,
    propose_delete_native_project_panel_projection,
    propose_update_native_project_manufacturing_plan,
    propose_update_native_project_panel_projection,
};
pub(crate) use self::plans::{
    create_native_project_manufacturing_plan, create_native_project_panel_projection,
    delete_native_project_manufacturing_plan, delete_native_project_panel_projection,
    query_native_project_manufacturing_plans, query_native_project_panel_projections,
    update_native_project_manufacturing_plan, update_native_project_panel_projection,
};
pub(crate) use self::views::*;
