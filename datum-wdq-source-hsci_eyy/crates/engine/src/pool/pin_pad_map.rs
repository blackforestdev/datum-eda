use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::PadMapEntry;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinPadMap {
    pub uuid: Uuid,
    pub part: Uuid,
    #[serde(default)]
    pub footprint: Option<Uuid>,
    #[serde(default)]
    pub mappings: HashMap<Uuid, PadMapEntry>,
    #[serde(default)]
    pub tags: HashSet<String>,
}
