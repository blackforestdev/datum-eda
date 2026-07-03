use super::command_project_schematic_symbol_library_materialization::PoolSymbolComponentBinding;
use super::*;
use crate::{NativeProjectPlaceSymbolBindingEvidenceView, NativeProjectRevisionedRefView};

pub(crate) fn symbol_mutation_report(
    action: &str,
    project: &LoadedNativeProject,
    sheet_uuid: Uuid,
    sheet_path: &Path,
    symbol: &PlacedSymbol,
) -> NativeProjectSymbolMutationReportView {
    symbol_mutation_report_with_binding(
        action,
        project,
        sheet_uuid,
        sheet_path,
        symbol,
        "not_applicable".to_string(),
        Vec::new(),
        None,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn symbol_mutation_report_with_binding(
    action: &str,
    project: &LoadedNativeProject,
    sheet_uuid: Uuid,
    sheet_path: &Path,
    symbol: &PlacedSymbol,
    binding_status: String,
    binding_diagnostics: Vec<String>,
    binding_evidence: Option<NativeProjectPlaceSymbolBindingEvidenceView>,
    component_instance_uuid: Option<Uuid>,
) -> NativeProjectSymbolMutationReportView {
    NativeProjectSymbolMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference.clone(),
        value: symbol.value.clone(),
        lib_id: symbol.lib_id.clone(),
        x_nm: symbol.position.x,
        y_nm: symbol.position.y,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
        entity_uuid: symbol.entity.map(|uuid| uuid.to_string()),
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        part_uuid: symbol.part.map(|uuid| uuid.to_string()),
        component_instance_uuid: component_instance_uuid.map(|uuid| uuid.to_string()),
        binding_status,
        binding_diagnostics,
        binding_evidence,
        unit_selection: symbol.unit_selection.clone(),
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    }
}

pub(crate) fn component_instance_uuid_for_pool_symbol(
    project: &LoadedNativeProject,
    symbol_uuid: Uuid,
    binding: &PoolSymbolComponentBinding,
) -> Uuid {
    eda_engine::api::native_write::schematic_symbols::placed_symbol_component_instance_id(
        &project.manifest.uuid,
        binding.symbol_id,
        symbol_uuid,
    )
}

pub(crate) fn binding_evidence_for_pool_symbol(
    binding: &PoolSymbolComponentBinding,
    symbol_uuid: Uuid,
    component_instance_uuid: Option<Uuid>,
) -> NativeProjectPlaceSymbolBindingEvidenceView {
    NativeProjectPlaceSymbolBindingEvidenceView {
        pool_symbol_ref: revisioned_ref_view(binding.symbol_id, binding.symbol_revision),
        pool_unit_ref: revisioned_ref_view(binding.unit_id, binding.unit_revision),
        entity_ref: revisioned_ref_view(binding.entity_id, binding.entity_revision),
        gate_uuid: binding.gate_id.to_string(),
        part_ref: binding
            .part
            .as_ref()
            .map(|part| revisioned_ref_view(part.part_id, part.part_revision)),
        placed_symbol_ref: revisioned_ref_view(symbol_uuid, 0),
        component_instance_ref: component_instance_uuid.map(|uuid| revisioned_ref_view(uuid, 0)),
    }
}

fn revisioned_ref_view(object_id: Uuid, object_revision: u64) -> NativeProjectRevisionedRefView {
    NativeProjectRevisionedRefView {
        object_id: object_id.to_string(),
        object_revision,
    }
}
