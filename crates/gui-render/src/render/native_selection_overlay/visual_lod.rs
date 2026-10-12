//! Current prepared presentation summary, never selection or membership authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionVisualLod {
    ExactMask { panes: usize, minimum_cues: usize },
    Detailed { panes: usize, minimum_cues: usize },
    Unavailable,
}
impl std::fmt::Display for SelectionVisualLod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExactMask {
                panes,
                minimum_cues,
            } => write!(f, "Exact mask · {minimum_cues} minimum · {panes} panes"),
            Self::Detailed {
                panes,
                minimum_cues,
            } => write!(f, "Detailed cues · {minimum_cues} minimum · {panes} panes"),
            Self::Unavailable => f.write_str("Visual LOD: unavailable projection"),
        }
    }
}
