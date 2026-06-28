use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ir::geometry::LayerId;
use crate::rules::ast::RuleType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrcSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrcLocation {
    pub x_nm: i64,
    pub y_nm: i64,
    pub layer: Option<LayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrcViolation {
    pub id: Uuid,
    pub code: String,
    pub rule_type: RuleType,
    pub severity: DrcSeverity,
    pub message: String,
    pub location: Option<DrcLocation>,
    pub objects: Vec<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standards_basis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_key: Option<String>,
    #[serde(default)]
    pub waived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrcSummary {
    pub errors: usize,
    pub warnings: usize,
    #[serde(default)]
    pub waived: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrcReport {
    pub passed: bool,
    pub violations: Vec<DrcViolation>,
    pub summary: DrcSummary,
}
