//! Transient feedback for a captured shared Move; never canonical geometry.
use crate::{AuthoringViewBasis, PointNm};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveHandleKind {
    Endpoint,
    Vertex,
    Junction,
    Segment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveHandleFeedback {
    pub basis: AuthoringViewBasis,
    pub kind: MoveHandleKind,
    pub source: PointNm,
    pub destination: PointNm,
    /// An exact eligible pin contact from the shared snap service, not a grid hit.
    pub snap_target: Option<PointNm>,
}
