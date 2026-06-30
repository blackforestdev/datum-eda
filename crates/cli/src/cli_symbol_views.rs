use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectSymbolMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) symbol_uuid: String,
    pub(crate) reference: String,
    pub(crate) value: String,
    pub(crate) lib_id: Option<String>,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
    pub(crate) rotation_deg: i32,
    pub(crate) mirrored: bool,
    pub(crate) entity_uuid: Option<String>,
    pub(crate) gate_uuid: Option<String>,
    pub(crate) part_uuid: Option<String>,
    pub(crate) component_instance_uuid: Option<String>,
    pub(crate) binding_status: String,
    pub(crate) binding_diagnostics: Vec<String>,
    pub(crate) binding_evidence: Option<NativeProjectPlaceSymbolBindingEvidenceView>,
    pub(crate) unit_selection: Option<String>,
    pub(crate) display_mode: String,
    pub(crate) hidden_power_behavior: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectPlaceSymbolBindingEvidenceView {
    pub(crate) pool_symbol_ref: NativeProjectRevisionedRefView,
    pub(crate) pool_unit_ref: NativeProjectRevisionedRefView,
    pub(crate) entity_ref: NativeProjectRevisionedRefView,
    pub(crate) gate_uuid: String,
    pub(crate) part_ref: Option<NativeProjectRevisionedRefView>,
    pub(crate) placed_symbol_ref: NativeProjectRevisionedRefView,
    pub(crate) component_instance_ref: Option<NativeProjectRevisionedRefView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectRevisionedRefView {
    pub(crate) object_id: String,
    pub(crate) object_revision: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectSymbolFieldMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) symbol_uuid: String,
    pub(crate) field_uuid: String,
    pub(crate) key: String,
    pub(crate) value: String,
    pub(crate) visible: bool,
    pub(crate) x_nm: Option<i64>,
    pub(crate) y_nm: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectSymbolPinInfoView {
    pub(crate) symbol_uuid: String,
    pub(crate) pin_uuid: String,
    pub(crate) number: String,
    pub(crate) name: String,
    pub(crate) electrical_type: String,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
    pub(crate) anchor_orientation: Option<String>,
    pub(crate) anchor_length_nm: Option<i64>,
    pub(crate) anchor_decoration: Option<String>,
    pub(crate) visible_override: Option<bool>,
    pub(crate) override_x_nm: Option<i64>,
    pub(crate) override_y_nm: Option<i64>,
}
