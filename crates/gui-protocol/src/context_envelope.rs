use serde::{Deserialize, Serialize};

use crate::{SceneBounds, SelectionTarget};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DatumToolSessionLifecycle {
    Running,
    Terminating,
    Exited,
    Restarted,
    Closed,
}

impl DatumToolSessionLifecycle {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumSelectionContext {
    pub kind: String,
    pub id: Option<String>,
}

impl DatumSelectionContext {
    pub fn from_selection(selection: &SelectionTarget) -> Self {
        match selection {
            SelectionTarget::None => Self {
                kind: "none".to_string(),
                id: None,
            },
            SelectionTarget::ReviewAction(id) => Self {
                kind: "review_action".to_string(),
                id: Some(id.clone()),
            },
            SelectionTarget::AuthoredObject(id) => Self {
                kind: "authored_object".to_string(),
                id: Some(id.clone()),
            },
            SelectionTarget::CheckFinding(id) => Self {
                kind: "check_finding".to_string(),
                id: Some(id.clone()),
            },
            SelectionTarget::Finding(id) => Self {
                kind: "finding".to_string(),
                id: Some(id.clone()),
            },
            SelectionTarget::JournalEntry(id) => Self {
                kind: "journal_entry".to_string(),
                id: Some(id.clone()),
            },
            SelectionTarget::Relationship(id) => Self {
                kind: "relationship".to_string(),
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
                kind: "none".to_string(),
                id: None
            }
        );
        assert_eq!(
            DatumSelectionContext::from_selection(&SelectionTarget::ReviewAction(
                "route-action".to_string()
            )),
            DatumSelectionContext {
                kind: "review_action".to_string(),
                id: Some("route-action".to_string())
            }
        );
        assert_eq!(
            DatumSelectionContext::from_selection(&SelectionTarget::AuthoredObject(
                "board-text".to_string()
            )),
            DatumSelectionContext {
                kind: "authored_object".to_string(),
                id: Some("board-text".to_string())
            }
        );
        assert_eq!(
            DatumSelectionContext::from_selection(&SelectionTarget::CheckFinding(
                "sha256:finding-a".to_string()
            )),
            DatumSelectionContext {
                kind: "check_finding".to_string(),
                id: Some("sha256:finding-a".to_string())
            }
        );
    }
}
