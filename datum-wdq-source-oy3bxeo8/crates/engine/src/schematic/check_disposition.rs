use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{CheckDomain, WaiverTarget};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckDeviation {
    pub uuid: Uuid,
    pub domain: CheckDomain,
    pub target: WaiverTarget,
    pub rationale: String,
    pub accepted_by: Option<String>,
    pub approval_status: DeviationApprovalStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviationApprovalStatus {
    Accepted,
}
