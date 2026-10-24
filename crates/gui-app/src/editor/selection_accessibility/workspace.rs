//! Small UI metadata configures the existing lazy selection accessibility tree.
use datum_gui_protocol::inspector_context::{
    InspectorContext, InspectorOwner, InspectorSectionId, InspectorValue,
};
use datum_gui_protocol::schematic_documents::SchematicDocumentKey;
use datum_gui_protocol::{ApplicationFocus, PaneContent, ReviewWorkspaceState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Field {
    pub label: String,
    pub text: Option<String>,
    pub native_field: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Section {
    pub id: InspectorSectionId,
    pub fields: Vec<Field>,
    pub collapsed: bool,
    pub focused: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Document {
    pub key: SchematicDocumentKey,
    pub parent: Option<SchematicDocumentKey>,
    pub name: String,
    pub description: String,
    pub expanded: bool,
    pub inspected: bool,
    pub visible: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceInspection {
    pub project: String,
    pub revision: String,
    pub title: String,
    pub sections: Vec<Section>,
    pub documents: Vec<Document>,
    pub navigator_focused: bool,
    pub inspector_focused: bool,
    pub rerun_available: bool,
    pub selection_visible: bool,
}
impl WorkspaceInspection {
    pub fn capture(state: &ReviewWorkspaceState) -> Option<Self> {
        if state.schematic_documents.project.is_none()
            || (matches!(
                state.ui.layout.focused_content(),
                PaneContent::Library | PaneContent::Revision(_)
            ) && state.inspector_context.owner != InspectorOwner::Navigator)
        {
            return None;
        }
        let context = InspectorContext::capture(state);
        let sections = context
            .sections
            .iter()
            .map(|section| Section {
                id: section.id,
                collapsed: state.inspector_context.collapsed.contains(&section.id),
                focused: state.ui.focus == ApplicationFocus::Inspector
                    && state.inspector_context.focused == Some(section.id),
                fields: section
                    .fields
                    .iter()
                    .map(|field| {
                        let (text, native_field) = match field.value {
                            InspectorValue::SelectionField(name) => (None, Some(name.into())),
                            _ => (Some(context.field_text(field, usize::MAX)), None),
                        };
                        Field {
                            label: field.label.to_string(),
                            text,
                            native_field,
                        }
                    })
                    .collect(),
            })
            .collect();
        let visible = state.schematic_documents.visible_rows();
        let documents = state.schematic_documents.rows.iter().map(|row| Document {
            key: row.key.clone(), parent: row.parent.clone(), name: row.name.clone(),
            description: format!("Document {}; source {} ({}); source UUID {}; occurrence {}; path {}; Project {}; source revision {}. Single click inspects; Enter or double-click opens beside the focused pane. Authored selection is unchanged.",
                state.schematic_documents.document_name, row.source_name, row.source_sheet_name, row.key.source_sheet,
                row.breadcrumb.join(" / "), row.key.instance_path.iter().map(ToString::to_string).collect::<Vec<_>>().join(" / "),
                row.key.project, state.schematic_documents.revision.as_ref().map(|r| r.0.as_str()).unwrap_or("Unavailable")),
            expanded: state.schematic_documents.expanded.contains(&row.key),
            inspected: state.schematic_documents.inspected.as_ref() == Some(&row.key),
            visible: visible.iter().any(|visible| visible.key == row.key),
        }).collect();
        Some(Self {
            project: state.scene.project_uuid.clone(),
            revision: state.scene.source_revision.clone(),
            title: context.title,
            sections,
            documents,
            navigator_focused: state.ui.focus == ApplicationFocus::Navigator,
            inspector_focused: state.ui.focus == ApplicationFocus::Inspector,
            rerun_available: context.rerun_available,
            selection_visible: context.view.is_some(),
        })
    }
}
