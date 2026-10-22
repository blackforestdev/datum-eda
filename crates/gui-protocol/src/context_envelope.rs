use serde::{Deserialize, Serialize};

use crate::{SceneBounds, SelectionTarget};
mod selection_inspection;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DatumToolSessionLifecycle {
    Running,
    Attached,
    Detached,
    Terminating,
    Exited,
    Restarted,
    Closed,
}

impl DatumToolSessionLifecycle {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Attached => "attached",
            Self::Detached => "detached",
            Self::Terminating => "terminating",
            Self::Exited => "exited",
            Self::Restarted => "restarted",
            Self::Closed => "closed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumToolSessionMetadata {
    pub session_id: String,
    pub context_id: String,
    pub actor_type: String,
    pub capabilities: Vec<String>,
    pub created_model_revision: Option<String>,
    pub lifecycle: DatumToolSessionLifecycle,
    pub created_unix_ms: u128,
    pub updated_unix_ms: u128,
    pub expires_unix_ms: Option<u128>,
    pub process_group_id: Option<i32>,
    pub process_exit_code: Option<i32>,
    #[serde(default)]
    pub process_exit_signal: Option<i32>,
    #[serde(default)]
    pub process_core_dumped: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumSelectionContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<NativeSelectionContext>,
    pub kind: String,
    pub id: Option<String>,
}

impl DatumSelectionContext {
    pub fn from_selection(selection: &SelectionTarget) -> Self {
        match selection {
            SelectionTarget::Native(view) => Self {
                kind: view.subject.kind().into(),
                id: None,
                details: Some(NativeSelectionContext::from_view(view)),
            },
            SelectionTarget::None => Self {
                details: None,
                kind: "none".to_string(),
                id: None,
            },
            SelectionTarget::ReviewAction(id) => Self {
                details: None,
                kind: "review_action".to_string(),
                id: Some(id.clone()),
            },
            SelectionTarget::AuthoredObject(id) => Self {
                details: None,
                kind: "authored_object".to_string(),
                id: Some(id.clone()),
            },
            SelectionTarget::CheckFinding(id) => Self {
                details: None,
                kind: "check_finding".to_string(),
                id: Some(id.clone()),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumCursorContext {
    pub screen_px: Option<[i32; 2]>,
    pub hovered_object_id: Option<String>,
    pub active_dock_tab: Option<String>,
    pub active_tool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumProjectionContext {
    pub scene_id: String,
    pub board_id: Option<String>,
    pub board_name: Option<String>,
    pub scene_bounds_nm: Option<DatumSceneBoundsContext>,
    pub active_projection_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumSceneBoundsContext {
    pub min: [i64; 2],
    pub max: [i64; 2],
}

impl DatumSceneBoundsContext {
    pub fn from_bounds(bounds: &SceneBounds) -> Self {
        Self {
            min: [bounds.min_x, bounds.min_y],
            max: [bounds.max_x, bounds.max_y],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datum_selection_context_derives_stable_context_kinds() {
        assert_eq!(
            DatumSelectionContext::from_selection(&SelectionTarget::None),
            DatumSelectionContext {
                details: None,
                kind: "none".to_string(),
                id: None
            }
        );
        assert_eq!(
            DatumSelectionContext::from_selection(&SelectionTarget::ReviewAction(
                "route-action".to_string()
            )),
            DatumSelectionContext {
                details: None,
                kind: "review_action".to_string(),
                id: Some("route-action".to_string())
            }
        );
        assert_eq!(
            DatumSelectionContext::from_selection(&SelectionTarget::AuthoredObject(
                "board-text".to_string()
            )),
            DatumSelectionContext {
                details: None,
                kind: "authored_object".to_string(),
                id: Some("board-text".to_string())
            }
        );
        assert_eq!(
            DatumSelectionContext::from_selection(&SelectionTarget::CheckFinding(
                "sha256:finding-a".to_string()
            )),
            DatumSelectionContext {
                details: None,
                kind: "check_finding".to_string(),
                id: Some("sha256:finding-a".to_string())
            }
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeSelectionContext {
    pub project: crate::SelectionProjectId,
    pub revision: crate::selection_resolution::SelectionModelRevision,
    pub subject: Option<crate::SelectionSubject>,
    pub focus: Option<crate::AuthoredSelectionIdentity>,
    pub total: usize,
    pub scope: Option<crate::AuthoredSelectionClass>,
    pub per_type: std::collections::BTreeMap<crate::AuthoredSelectionClass, usize>,
    pub inspection: std::collections::BTreeMap<String, crate::selection_view::InspectionValue>,
    pub members: Option<Vec<crate::AuthoredSelectionIdentity>>,
    /// The governed member list is omitted above 256; counts/identity stay whole.
    #[serde(default)]
    pub truncated: bool,
    /// Complete per-member evidence is queried explicitly above the member cap.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub omitted_inspection_fields: Vec<String>,
    /// Canonical value state remains available when only its detail is omitted.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub omitted_inspection_field_states:
        std::collections::BTreeMap<String, crate::selection_view::InspectionState>,
    pub hidden: usize,
    pub locked: usize,
    pub unprojected: usize,
    pub notice: Option<String>,
    pub stale: bool,
}
impl NativeSelectionContext {
    fn from_view(view: &crate::selection_view::SelectionView) -> Self {
        // Compound identity itself also must not leak its uncapped list into transport.
        let subject = match &view.subject {
            crate::SelectionSubject::Compound(_) => None,
            other => Some(other.clone()),
        };
        let truncated = view.members.len() > 256;
        let (inspection, omitted_inspection_fields, omitted_inspection_field_states) =
            selection_inspection::capture(view, truncated);
        Self {
            project: view.project,
            revision: view.revision.clone(),
            subject,
            focus: view.focus().cloned(),
            total: view.members.len(),
            scope: view.scope,
            per_type: view.members.iter().fold(
                std::collections::BTreeMap::new(),
                |mut counts, id| {
                    *counts.entry(id.class).or_insert(0) += 1;
                    counts
                },
            ),
            inspection,
            members: (view.members.len() <= 256).then(|| view.members.iter().cloned().collect()),
            truncated,
            omitted_inspection_fields,
            omitted_inspection_field_states,
            hidden: view.hidden.len(),
            locked: view.locked.len(),
            unprojected: view.unprojected.len(),
            notice: view.notice.clone(),
            stale: view.stale,
        }
    }
}
