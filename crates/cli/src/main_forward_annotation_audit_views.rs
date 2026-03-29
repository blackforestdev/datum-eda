use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationAuditView {
    pub(crate) domain: &'static str,
    pub(crate) schematic_symbol_count: usize,
    pub(crate) board_component_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) unresolved_symbol_count: usize,
    pub(crate) missing_on_board: Vec<NativeProjectForwardAnnotationMissingView>,
    pub(crate) orphaned_on_board: Vec<NativeProjectForwardAnnotationOrphanView>,
    pub(crate) value_mismatches: Vec<NativeProjectForwardAnnotationValueMismatchView>,
    pub(crate) part_mismatches: Vec<NativeProjectForwardAnnotationPartMismatchView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationMissingView {
    pub(crate) symbol_uuid: String,
    pub(crate) sheet_uuid: String,
    pub(crate) reference: String,
    pub(crate) value: String,
    pub(crate) part_uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationOrphanView {
    pub(crate) component_uuid: String,
    pub(crate) reference: String,
    pub(crate) value: String,
    pub(crate) part_uuid: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationValueMismatchView {
    pub(crate) reference: String,
    pub(crate) symbol_uuid: String,
    pub(crate) component_uuid: String,
    pub(crate) schematic_value: String,
    pub(crate) board_value: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectForwardAnnotationPartMismatchView {
    pub(crate) reference: String,
    pub(crate) symbol_uuid: String,
    pub(crate) component_uuid: String,
    pub(crate) schematic_part_uuid: String,
    pub(crate) board_part_uuid: String,
}
