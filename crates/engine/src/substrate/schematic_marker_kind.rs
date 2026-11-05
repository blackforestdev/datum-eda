//! Canonical schematic marker source classes.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchematicMarkerKind {
    Junction,
    NoConnect,
}

impl SchematicMarkerKind {
    pub fn map_name(&self) -> &'static str {
        match self {
            SchematicMarkerKind::Junction => "junctions",
            SchematicMarkerKind::NoConnect => "noconnects",
        }
    }
}
