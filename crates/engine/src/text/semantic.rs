use super::registry::{FAMILY_NEWSTROKE, STYLE_REGULAR};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextRenderIntent {
    Manufacturing,
    Annotation,
    Branding,
    Documentation,
    UiPreview,
}

impl Default for TextRenderIntent {
    fn default() -> Self {
        Self::Manufacturing
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextFamilyId(pub String);

impl Default for TextFamilyId {
    fn default() -> Self {
        Self(FAMILY_NEWSTROKE.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextFamilySource {
    ImplicitDefault,
    Explicit,
}

impl Default for TextFamilySource {
    fn default() -> Self {
        Self::ImplicitDefault
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextStyleId(pub String);

impl Default for TextStyleId {
    fn default() -> Self {
        Self(STYLE_REGULAR.to_string())
    }
}
