use super::registry::{FAMILY_NEWSTROKE, STYLE_REGULAR};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum TextRenderIntent {
    #[default]
    Manufacturing,
    Annotation,
    Branding,
    Documentation,
    UiPreview,
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
#[derive(Default)]
pub enum TextFamilySource {
    #[default]
    ImplicitDefault,
    Explicit,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextStyleId(pub String);

impl Default for TextStyleId {
    fn default() -> Self {
        Self(STYLE_REGULAR.to_string())
    }
}
