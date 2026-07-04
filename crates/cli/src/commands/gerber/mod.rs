// commands/gerber — Gerber export planning, layer rendering, semantics,
// panelization, evidence, and the CLI view types for gerber commands.
//
// `use super::*;` anchors the family scope on commands/, which in turn
// anchors on the crate root, keeping crate-root names visible to member
// files exactly as the old command_project chain did.

#[allow(unused_imports)]
use super::*;

mod inspect;
mod layers;
mod mechanical;
pub(crate) mod plan;
mod semantics;
mod semantics_utils;
mod silkscreen;
mod views;
mod views_inspect;
mod views_mechanical;
mod views_render;
mod views_set;
mod views_silkscreen;

pub(crate) use self::inspect::inspect_gerber;
pub(crate) use self::layers::{
    compare_native_project_gerber_copper_layer, compare_native_project_gerber_outline,
    compare_native_project_gerber_paste_layer, compare_native_project_gerber_silkscreen_layer,
    compare_native_project_gerber_soldermask_layer, export_native_project_gerber_copper_layer,
    export_native_project_gerber_outline, export_native_project_gerber_paste_layer,
    export_native_project_gerber_silkscreen_layer, export_native_project_gerber_soldermask_layer,
    validate_native_project_gerber_copper_layer, validate_native_project_gerber_outline,
    validate_native_project_gerber_paste_layer, validate_native_project_gerber_silkscreen_layer,
    validate_native_project_gerber_soldermask_layer,
};
pub(crate) use self::mechanical::{
    NativeComponentMechanicalArc, NativeComponentMechanicalCircle, NativeComponentMechanicalLine,
    NativeComponentMechanicalPolygon, NativeComponentMechanicalPolyline,
    NativeComponentMechanicalText, compare_native_project_gerber_mechanical_layer,
    export_native_project_gerber_mechanical_layer, validate_native_project_gerber_mechanical_layer,
};
pub(crate) use self::plan::{
    compare_native_project_gerber_export_plan, compare_native_project_gerber_set,
    export_native_project_gerber_set, export_native_project_gerber_set_from_plan,
    export_native_project_gerber_set_without_output_run,
    export_native_project_gerber_set_without_output_run_for_output_job, panelize_rs274x_gerber,
    plan_native_project_gerber_export, validate_native_project_gerber_set,
};
pub(crate) use self::semantics::{
    DEFAULT_GERBER_OUTLINE_APERTURE_NM, ParsedGerber, ParsedGerberAperture, ParsedGerberGeometry,
    classify_via_hole_class, compare_entry_views, gerber_copper_actual_entries,
    gerber_copper_expected_entries, gerber_outline_actual_entries, gerber_outline_expected_entries,
    gerber_silkscreen_expected_entries, gerber_soldermask_actual_entries,
    gerber_soldermask_expected_entries, parse_rs274x_subset, render_circular_flash_geometry,
    render_mm_6, render_pad_flash_geometry, render_parsed_flash_geometry, render_region_geometry,
    render_stroke_geometry, resolve_native_project_soldermask_context,
};
pub(crate) use self::silkscreen::{
    NativeComponentSilkscreenArc, NativeComponentSilkscreenCircle, NativeComponentSilkscreenLine,
    NativeComponentSilkscreenPolygon, NativeComponentSilkscreenPolyline,
    NativeComponentSilkscreenText,
};
pub(crate) use self::silkscreen::{
    count_native_component_silkscreen_arcs, count_native_component_silkscreen_circles,
    count_native_component_silkscreen_lines, count_native_component_silkscreen_polygons,
    count_native_component_silkscreen_polylines, count_native_component_silkscreen_texts,
    resolve_native_project_silkscreen_context,
};
pub(crate) use self::views::*;
pub(crate) use self::views_inspect::*;
pub(crate) use self::views_mechanical::*;
pub(crate) use self::views_render::*;
pub(crate) use self::views_set::*;
pub(crate) use self::views_silkscreen::*;
