//! Inspector context borrows the shared projection and references governed native details.
use super::*;
use crate::inspector_context::{
    InspectorContext, InspectorOwner, InspectorSectionId, InspectorValue,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumInspectorContext {
    pub owner: String,
    pub pane: u32,
    pub document: Option<crate::schematic_documents::SchematicDocumentKey>,
    pub sections: Vec<DatumInspectorSection>,
    pub rerun_available: bool,
    pub check_job: crate::CheckProducerState,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumInspectorSection {
    pub id: InspectorSectionId,
    pub collapsed: bool,
    pub fields: Vec<DatumInspectorField>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumInspectorField {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    /// References the existing selection_context.details.inspection entry.
    /// Large membership evidence stays governed by that entry's omission policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_field: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_value_state: Option<crate::selection_view::InspectionState>,
    pub detail_omitted: bool,
}
impl DatumInspectorContext {
    pub(super) fn capture(
        state: &crate::ReviewWorkspaceState,
        selected: &DatumSelectionContext,
    ) -> Self {
        let context = InspectorContext::capture(state);
        let sections = context
            .sections
            .iter()
            .map(|section| DatumInspectorSection {
                id: section.id,
                collapsed: state.inspector_context.collapsed.contains(&section.id),
                fields: section
                    .fields
                    .iter()
                    .map(|field| {
                        let (value, native_field, native_value_state, detail_omitted) =
                            match &field.value {
                                InspectorValue::Text(value) => (
                                    Some(serde_json::Value::String(value.to_string())),
                                    None,
                                    None,
                                    false,
                                ),
                                InspectorValue::Json(value) => {
                                    (Some((*value).clone()), None, None, false)
                                }
                                InspectorValue::SelectionField(name) => (
                                    None,
                                    Some((*name).into()),
                                    context
                                        .view
                                        .and_then(|view| view.field_ref(name))
                                        .map(|value| value.state()),
                                    selected.details.as_ref().is_some_and(|details| {
                                        details
                                            .omitted_inspection_fields
                                            .iter()
                                            .any(|field| field == name)
                                    }),
                                ),
                            };
                        DatumInspectorField {
                            label: field.label.to_string(),
                            value,
                            native_field,
                            native_value_state,
                            detail_omitted,
                        }
                    })
                    .collect(),
            })
            .collect();
        Self {
            owner: match state.inspector_context.owner {
                InspectorOwner::Selection => "selection",
                InspectorOwner::Navigator => "navigator",
            }
            .into(),
            pane: state.ui.layout.focused.0,
            document: context.document.cloned(),
            sections,
            rerun_available: context.rerun_available,
            check_job: state.check_producer.clone(),
        }
    }
}
