//! Retained painter range vocabulary shared by scene and GPU owners.
use std::ops::Range;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RetainedDrawCommand {
    Quads {
        layer_id: Option<String>,
        range: Range<u32>,
    },
    Glyphs {
        layer_id: Option<String>,
        range: Range<u32>,
        material: u32,
    },
    Strokes {
        layer_id: Option<String>,
        range: Range<u32>,
    },
}
