//! One read-only Inspector projection for rendered, keyboard, AX and context consumers.
use crate::schematic_documents::SchematicDocumentKey;
use crate::selection_view::SelectionView;
use crate::{PaneContent, ReviewWorkspaceState, SelectionTarget};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeSet;
use uuid::Uuid;
mod checks;
mod components;
mod nets;
mod persistence;
pub use persistence::{InspectorActivity, materialization_state, saved_state};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectorOwner {
    #[default]
    Selection,
    Navigator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectorSectionId {
    Document,
    Source,
    Occurrence,
    Component,
    Placement,
    LibraryInfo,
    Readiness,
    Properties,
    Connectivity,
    Checks,
}
impl InspectorSectionId {
    pub fn label(self) -> &'static str {
        match self {
            Self::Document => "Document",
            Self::Source => "Source",
            Self::Occurrence => "Occurrence",
            Self::Component => "Component",
            Self::Placement => "Placement",
            Self::LibraryInfo => "Library info",
            Self::Readiness => "Command readiness",
            Self::Properties => "Properties",
            Self::Connectivity => "Connectivity",
            Self::Checks => "Checks",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InspectorNavigation {
    pub owner: InspectorOwner,
    pub collapsed: BTreeSet<InspectorSectionId>,
    pub focused: Option<InspectorSectionId>,
    /// Captured from existing Runtime owners, never inferred from shard hashes.
    pub activity: Option<InspectorActivity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorActionBasis {
    pub project: Uuid,
    pub revision: String,
    pub root: std::path::PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorNavigationBasis {
    pub action: Option<InspectorActionBasis>,
    pub owner: InspectorOwner,
    pub pane: crate::PaneId,
    pub document: Option<SchematicDocumentKey>,
    pub collapsed: BTreeSet<InspectorSectionId>,
    pub selection: Option<crate::selection_view::inspector_state::InspectorPresentationBasis>,
}
impl InspectorNavigationBasis {
    pub fn for_context(
        state: &ReviewWorkspaceState,
        context: &InspectorContext<'_>,
        selection: Option<crate::selection_view::inspector_state::InspectorPresentationBasis>,
    ) -> Self {
        Self {
            action: context.action_basis.clone(),
            owner: state.inspector_context.owner,
            pane: state.ui.layout.focused,
            document: context.document.cloned(),
            collapsed: state.inspector_context.collapsed.clone(),
            selection,
        }
    }
    pub fn is_current(&self, state: &ReviewWorkspaceState) -> bool {
        let context = InspectorContext::capture(state);
        let selection = context.view.map(|view| {
            crate::selection_view::inspector_state::InspectorPresentationBasis::capture(
                view,
                &state.ui.selection_inspector,
            )
        });
        *self == Self::for_context(state, &context, selection)
    }
}
impl InspectorActionBasis {
    pub fn capture(state: &ReviewWorkspaceState) -> Option<Self> {
        if !crate::CheckProducerRequest::workspace_available(state) {
            return None;
        }
        let model = state.library.basis.as_ref()?;
        Some(Self {
            project: model.project.project_id,
            revision: model.model_revision.0.clone(),
            root: state.backing.as_ref()?.request.project_root.clone(),
        })
    }
    pub fn is_current(&self, state: &ReviewWorkspaceState) -> bool {
        Self::capture(state).as_ref() == Some(self)
    }
}

pub enum InspectorValue<'a> {
    Text(Cow<'a, str>),
    Json(&'a serde_json::Value),
    /// Exact value stays in the existing complete selection projection; no large
    /// membership array is copied into another Inspector or context store.
    SelectionField(&'a str),
}
pub struct InspectorField<'a> {
    pub label: Cow<'a, str>,
    pub value: InspectorValue<'a>,
}
impl<'a> InspectorField<'a> {
    pub fn text(label: &'static str, value: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: Cow::Borrowed(label),
            value: InspectorValue::Text(value.into()),
        }
    }
}
pub struct InspectorSection<'a> {
    pub id: InspectorSectionId,
    pub fields: Vec<InspectorField<'a>>,
}
pub struct InspectorContext<'a> {
    pub title: String,
    pub view: Option<&'a SelectionView>,
    pub document: Option<&'a SchematicDocumentKey>,
    pub sections: Vec<InspectorSection<'a>>,
    pub action_basis: Option<InspectorActionBasis>,
    pub rerun_available: bool,
}
impl<'a> InspectorContext<'a> {
    pub fn capture(state: &'a ReviewWorkspaceState) -> Self {
        let view = match &state.selection {
            SelectionTarget::Native(view)
                if state.inspector_context.owner == InspectorOwner::Selection =>
            {
                Some(view.as_ref())
            }
            _ => None,
        };
        let document = if state.inspector_context.owner == InspectorOwner::Navigator {
            state.schematic_documents.inspected.as_ref()
        } else if state.ui.layout.focused_content() == PaneContent::Schematic {
            state
                .schematic_documents
                .panes
                .get(&state.ui.layout.focused)
        } else {
            None
        };
        let action_basis = InspectorActionBasis::capture(state);
        let rerun_available = action_basis.is_some()
            && !matches!(
                state.inspector_context.activity,
                Some(
                    InspectorActivity::Pending
                        | InspectorActivity::Preview
                        | InspectorActivity::Unconfirmed
                )
            )
            && !matches!(
                state.check_producer.phase,
                crate::CheckProducerPhase::Starting | crate::CheckProducerPhase::Running
            )
            && !state.schematic_authoring.active()
            && !state.schematic_authoring.picker
            && state.library.draft.is_none();
        let mut context = Self {
            title: view.map_or_else(|| "Document".into(), SelectionView::inspector_title),
            view,
            document,
            sections: Vec::new(),
            action_basis,
            rerun_available,
        };
        if let Some(view) = view {
            for field in view.fields() {
                if field == "library_land_pattern_sources" {
                    continue;
                }
                let section = if field.contains("position")
                    || field.contains("rotation")
                    || field.starts_with("section_")
                {
                    InspectorSectionId::Placement
                } else if field.contains("net")
                    || field.contains("bus")
                    || field == "Related component"
                {
                    InspectorSectionId::Connectivity
                } else if field.starts_with("library_")
                    || field.starts_with("binding_")
                    || field == "Component bindings"
                {
                    InspectorSectionId::LibraryInfo
                } else if matches!(field, "reference" | "value" | "locked" | "board_layer") {
                    InspectorSectionId::Component
                } else {
                    InspectorSectionId::Properties
                };
                context.push(
                    section,
                    InspectorField {
                        label: crate::selection_view::field_label(field).into(),
                        value: InspectorValue::SelectionField(field),
                    },
                );
            }
            components::append(&mut context, state);
            nets::append(&mut context, state);
        } else {
            context.document_fields(state);
        }
        checks::append(&mut context, state);
        context.sections.sort_by_key(|section| section.id);
        context
    }

    fn document_fields(&mut self, state: &'a ReviewWorkspaceState) {
        use InspectorSectionId as S;
        let documents = &state.schematic_documents;
        self.push(
            S::Document,
            InspectorField::text("Name", documents.document_name.as_str()),
        );
        if let Some(project) = documents.project {
            self.push(
                S::Document,
                InspectorField::text("Project", project.to_string()),
            );
        }
        let row = self.document.and_then(|key| documents.row(key));
        if let Some(row) = row {
            self.push(
                S::Source,
                InspectorField::text("Source", row.source_name.as_str()),
            );
            self.push(
                S::Source,
                InspectorField::text("Source sheet", row.source_sheet_name.as_str()),
            );
            self.push(
                S::Source,
                InspectorField::text("Source UUID", row.key.source_sheet.to_string()),
            );
            self.push(
                S::Occurrence,
                InspectorField::text("Occurrence", row.name.as_str()),
            );
            self.push(
                S::Occurrence,
                InspectorField::text("Breadcrumb", row.breadcrumb.join(" / ")),
            );
            self.push(
                S::Occurrence,
                InspectorField::text(
                    "Path",
                    if row.key.instance_path.is_empty() {
                        "Root occurrence".into()
                    } else {
                        row.key
                            .instance_path
                            .iter()
                            .map(Uuid::to_string)
                            .collect::<Vec<_>>()
                            .join(" / ")
                    },
                ),
            );
        } else if state.inspector_context.owner == InspectorOwner::Selection
            && state.ui.layout.focused_content() == PaneContent::Board
        {
            self.push(
                S::Source,
                InspectorField::text("Source", state.scene.board_name.as_str()),
            );
            self.push(
                S::Source,
                InspectorField::text("Source UUID", state.scene.board_uuid.as_str()),
            );
            self.push(
                S::Occurrence,
                InspectorField::text("Occurrence", "Board source"),
            );
        } else {
            self.push(
                S::Source,
                InspectorField::text(
                    "Source",
                    "Unavailable — no qualified schematic source in this view",
                ),
            );
            self.push(
                S::Occurrence,
                InspectorField::text(
                    "Occurrence",
                    "Unavailable — no document occurrence resolved",
                ),
            );
        }
        self.push(
            S::Document,
            InspectorField::text(
                "Snap",
                if state.authoring.snap.enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
            ),
        );
        self.push(S::Document, InspectorField::text("Grid", snap_grid(state)));
        self.push(
            S::Document,
            InspectorField::text("Saved state", saved_state(state)),
        );
        self.push(
            S::Document,
            InspectorField::text("Source materialization", materialization_state(state)),
        );
    }

    pub fn push(&mut self, id: InspectorSectionId, field: InspectorField<'a>) {
        if let Some(section) = self.sections.iter_mut().find(|section| section.id == id) {
            section.fields.push(field);
        } else {
            self.sections.push(InspectorSection {
                id,
                fields: vec![field],
            });
        }
    }
    pub fn field_text(&self, field: &InspectorField<'_>, columns: usize) -> String {
        match &field.value {
            InspectorValue::Text(value) => value.to_string(),
            InspectorValue::Json(value) => value
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| value.to_string()),
            InspectorValue::SelectionField(name) => self.view.map_or_else(
                || "Unavailable".into(),
                |view| view.field_display_preview(name, columns),
            ),
        }
    }
    pub fn visible_row_count(&self, navigation: &InspectorNavigation) -> usize {
        self.sections
            .iter()
            .map(|section| {
                1 + if navigation.collapsed.contains(&section.id) {
                    0
                } else {
                    section.fields.len()
                }
            })
            .sum()
    }
}

pub fn snap_grid(state: &ReviewWorkspaceState) -> String {
    snap_grid_for(state, state.ui.layout.focused_content())
}

pub fn snap_grid_for(state: &ReviewWorkspaceState, content: PaneContent) -> String {
    use eda_engine::ir::units::{FormatLengthRequest, Nanometers, QuantityKind, format_length};
    let result = (|| -> anyhow::Result<String> {
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Project basis unavailable"))?;
        let units = eda_engine::api::native_write::project::project_display_units(model)?
            .resolve()
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let display = if content == PaneContent::Board {
            units.board
        } else {
            units.schematic
        };
        let rendered = format_length(FormatLengthRequest {
            value: Nanometers::new(state.authoring.snap.grid_nm),
            quantity: QuantityKind::Length,
            display_unit: display.unit,
            measurement_system: units.system,
            precision: display.precision,
        })
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(rendered.rendered)
    })();
    result.unwrap_or_else(|error| format!("Unavailable — {error}"))
}

/// Local pane chrome reads the pane's exact document, independently of focus.
/// Full source identities remain available in the shared Inspector projection.
pub fn pane_header(
    state: &ReviewWorkspaceState,
    pane: crate::PaneId,
    content: PaneContent,
) -> Option<(String, String)> {
    state.schematic_documents.project?;
    if !matches!(content, PaneContent::Schematic | PaneContent::Board) {
        return None;
    }
    let title = if content == PaneContent::Schematic {
        let row = state
            .schematic_documents
            .panes
            .get(&pane)
            .and_then(|key| state.schematic_documents.row(key));
        row.map_or_else(
            || "Source unavailable · Occurrence unavailable".into(),
            |row| {
                format!(
                    "Source: {} · Occurrence: {}",
                    row.source_name,
                    row.breadcrumb.join(" / ")
                )
            },
        )
    } else {
        format!(
            "Board: {} · Source: {}",
            state.scene.board_name, state.scene.board_uuid
        )
    };
    let saved = saved_state(state);
    let checks = &state.checks;
    let erc = if matches!(
        state.check_producer.phase,
        crate::CheckProducerPhase::Starting | crate::CheckProducerPhase::Running
    ) {
        format!(
            "ERC {}",
            if state.check_producer.phase == crate::CheckProducerPhase::Running {
                "running"
            } else {
                "starting"
            }
        )
    } else if checks.persisted
        && checks.check_run_id.is_some()
        && checks.profile_id.as_deref() == Some("erc")
    {
        if checks.project_id.as_deref() == Some(state.scene.project_uuid.as_str())
            && checks.model_revision.as_deref() == Some(state.scene.source_revision.as_str())
        {
            format!("ERC {} Project findings", checks.finding_count)
        } else {
            format!("ERC stale · {} historical findings", checks.finding_count)
        }
    } else if state
        .library
        .basis
        .as_ref()
        .is_some_and(|model| model.check_runs.is_empty())
    {
        "ERC not run".into()
    } else {
        "ERC unavailable".into()
    };
    let detail = format!(
        "Doc: {} · Snap {} {} · {saved} · {erc}",
        state.schematic_documents.document_name,
        if state.authoring.snap.enabled {
            "on"
        } else {
            "off"
        },
        snap_grid_for(state, content)
    );
    Some((title, detail))
}

#[cfg(test)]
#[path = "inspector_context/persistence_tests.rs"]
mod persistence_tests;
