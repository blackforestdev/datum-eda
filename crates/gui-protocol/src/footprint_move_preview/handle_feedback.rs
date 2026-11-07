//! The captured edit intent supplies its feedback role without re-acquisition.
use super::*;
use crate::MoveHandleKind;

impl NativeMovePreview {
    pub fn wire_handle_source(&self) -> Option<(MoveHandleKind, PointNm)> {
        let kind = match self {
            Self::Connectivity {
                target: ConnectivityMoveTarget::Wiring { edit, .. },
                authoring,
                base,
                ..
            } => match edit.handle {
                WireHandle::Endpoint { .. } => MoveHandleKind::Endpoint,
                WireHandle::Segment { .. } => MoveHandleKind::Segment,
                WireHandle::InsertVertex { .. } => MoveHandleKind::Vertex,
                WireHandle::Vertex { point } => {
                    let junction = base
                        .schematic_document
                        .as_ref()
                        .and_then(|document| {
                            authoring
                                .source
                                .as_ref()?
                                .sheets
                                .get(&document.source_sheet)
                        })
                        .is_some_and(|sheet| sheet.junctions.values().any(|j| j.position == point));
                    if junction {
                        MoveHandleKind::Junction
                    } else {
                        MoveHandleKind::Vertex
                    }
                }
            },
            Self::Connectivity {
                target: ConnectivityMoveTarget::Junction(_),
                ..
            } => MoveHandleKind::Junction,
            Self::Wiring(_) if self.uses_electrical_pin_snap() => MoveHandleKind::Endpoint,
            _ => return None,
        };
        Some((kind, self.reference_position()))
    }
}
