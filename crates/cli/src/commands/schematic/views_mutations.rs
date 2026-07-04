// commands/schematic/views_mutations.rs — CLI view structs and text renderers
// for schematic mutation reports (moved from main.rs in the Wave 2 schematic
// lane). Symbol/field renderers render the views_symbol.rs types.

use crate::{NativeProjectSymbolFieldMutationReportView, NativeProjectSymbolMutationReportView};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectLabelMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) label_uuid: String,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectWireMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) wire_uuid: String,
    pub(crate) from_x_nm: i64,
    pub(crate) from_y_nm: i64,
    pub(crate) to_x_nm: i64,
    pub(crate) to_y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectJunctionMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) junction_uuid: String,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPortMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) port_uuid: String,
    pub(crate) name: String,
    pub(crate) direction: String,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBusMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) bus_uuid: String,
    pub(crate) name: String,
    pub(crate) members: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBusEntryMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) bus_entry_uuid: String,
    pub(crate) bus_uuid: String,
    pub(crate) wire_uuid: Option<String>,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectNoConnectMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) noconnect_uuid: String,
    pub(crate) symbol_uuid: String,
    pub(crate) pin_uuid: String,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectSymbolSemanticsView {
    pub(crate) symbol_uuid: String,
    pub(crate) gate_uuid: Option<String>,
    pub(crate) unit_selection: Option<String>,
    pub(crate) hidden_power_behavior: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPinOverrideMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) symbol_uuid: String,
    pub(crate) pin_uuid: String,
    pub(crate) visible: Option<bool>,
    pub(crate) x_nm: Option<i64>,
    pub(crate) y_nm: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectTextMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) text_uuid: String,
    pub(crate) text: String,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
    pub(crate) rotation_deg: i32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectDrawingMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) drawing_uuid: String,
    pub(crate) kind: String,
    pub(crate) from_x_nm: i64,
    pub(crate) from_y_nm: i64,
    pub(crate) to_x_nm: i64,
    pub(crate) to_y_nm: i64,
}

pub(crate) fn render_native_project_label_mutation_text(
    report: &NativeProjectLabelMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("label_uuid: {}", report.label_uuid),
        format!("name: {}", report.name),
        format!("kind: {}", report.kind),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_wire_mutation_text(
    report: &NativeProjectWireMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("wire_uuid: {}", report.wire_uuid),
        format!("from_x_nm: {}", report.from_x_nm),
        format!("from_y_nm: {}", report.from_y_nm),
        format!("to_x_nm: {}", report.to_x_nm),
        format!("to_y_nm: {}", report.to_y_nm),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_junction_mutation_text(
    report: &NativeProjectJunctionMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("junction_uuid: {}", report.junction_uuid),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_port_mutation_text(
    report: &NativeProjectPortMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("port_uuid: {}", report.port_uuid),
        format!("name: {}", report.name),
        format!("direction: {}", report.direction),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_bus_mutation_text(
    report: &NativeProjectBusMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("bus_uuid: {}", report.bus_uuid),
        format!("name: {}", report.name),
    ];
    if !report.members.is_empty() {
        lines.push("members:".to_string());
        for member in &report.members {
            lines.push(format!("  {member}"));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_bus_entry_mutation_text(
    report: &NativeProjectBusEntryMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("bus_entry_uuid: {}", report.bus_entry_uuid),
        format!("bus_uuid: {}", report.bus_uuid),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
    ];
    if let Some(wire_uuid) = &report.wire_uuid {
        lines.push(format!("wire_uuid: {}", wire_uuid));
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_noconnect_mutation_text(
    report: &NativeProjectNoConnectMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("noconnect_uuid: {}", report.noconnect_uuid),
        format!("symbol_uuid: {}", report.symbol_uuid),
        format!("pin_uuid: {}", report.pin_uuid),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_symbol_mutation_text(
    report: &NativeProjectSymbolMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("symbol_uuid: {}", report.symbol_uuid),
        format!("reference: {}", report.reference),
        format!("value: {}", report.value),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
        format!("rotation_deg: {}", report.rotation_deg),
        format!("mirrored: {}", report.mirrored),
    ];
    if let Some(lib_id) = &report.lib_id {
        lines.push(format!("lib_id: {}", lib_id));
    }
    if let Some(entity_uuid) = &report.entity_uuid {
        lines.push(format!("entity_uuid: {}", entity_uuid));
    }
    if let Some(gate_uuid) = &report.gate_uuid {
        lines.push(format!("gate_uuid: {}", gate_uuid));
    }
    if let Some(part_uuid) = &report.part_uuid {
        lines.push(format!("part_uuid: {}", part_uuid));
    }
    if let Some(component_instance_uuid) = &report.component_instance_uuid {
        lines.push(format!(
            "component_instance_uuid: {}",
            component_instance_uuid
        ));
    }
    lines.push(format!("binding_status: {}", report.binding_status));
    for diagnostic in &report.binding_diagnostics {
        lines.push(format!("binding_diagnostic: {}", diagnostic));
    }
    if let Some(unit_selection) = &report.unit_selection {
        lines.push(format!("unit_selection: {}", unit_selection));
    }
    lines.push(format!("display_mode: {}", report.display_mode));
    lines.push(format!(
        "hidden_power_behavior: {}",
        report.hidden_power_behavior
    ));
    lines.join("\n")
}

pub(crate) fn render_native_project_pin_override_mutation_text(
    report: &NativeProjectPinOverrideMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("symbol_uuid: {}", report.symbol_uuid),
        format!("pin_uuid: {}", report.pin_uuid),
    ];
    if let Some(visible) = report.visible {
        lines.push(format!("visible: {}", visible));
    }
    if let Some(x_nm) = report.x_nm {
        lines.push(format!("x_nm: {}", x_nm));
    }
    if let Some(y_nm) = report.y_nm {
        lines.push(format!("y_nm: {}", y_nm));
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_symbol_field_mutation_text(
    report: &NativeProjectSymbolFieldMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("symbol_uuid: {}", report.symbol_uuid),
        format!("field_uuid: {}", report.field_uuid),
        format!("key: {}", report.key),
        format!("value: {}", report.value),
        format!("visible: {}", report.visible),
    ];
    if let Some(x_nm) = report.x_nm {
        lines.push(format!("x_nm: {}", x_nm));
    }
    if let Some(y_nm) = report.y_nm {
        lines.push(format!("y_nm: {}", y_nm));
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_text_mutation_text(
    report: &NativeProjectTextMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("text_uuid: {}", report.text_uuid),
        format!("text: {}", report.text),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
        format!("rotation_deg: {}", report.rotation_deg),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_drawing_mutation_text(
    report: &NativeProjectDrawingMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("drawing_uuid: {}", report.drawing_uuid),
        format!("kind: {}", report.kind),
        format!("from_x_nm: {}", report.from_x_nm),
        format!("from_y_nm: {}", report.from_y_nm),
        format!("to_x_nm: {}", report.to_x_nm),
        format!("to_y_nm: {}", report.to_y_nm),
    ]
    .join("\n")
}
