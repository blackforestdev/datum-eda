use crate::board::RouteCorridorSpan;
use crate::ir::geometry::Point;
use uuid::Uuid;

pub(super) const ROUTE_PATH_CANDIDATE_SELECTION_RULE: &str = "select the first unblocked matching corridor span in corridor report order (sorted by candidate copper layer order, then pair index)";

pub(super) fn matching_corridor_spans<'a>(
    corridor_spans: &'a [RouteCorridorSpan],
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Vec<&'a RouteCorridorSpan> {
    corridor_spans
        .iter()
        .filter(|span| {
            (span.from_anchor_pad_uuid == from_anchor_pad_uuid
                && span.to_anchor_pad_uuid == to_anchor_pad_uuid)
                || (span.from_anchor_pad_uuid == to_anchor_pad_uuid
                    && span.to_anchor_pad_uuid == from_anchor_pad_uuid)
        })
        .collect()
}

pub(super) fn selected_matching_span<'a>(
    matching_spans: &[&'a RouteCorridorSpan],
) -> Option<&'a RouteCorridorSpan> {
    matching_spans.iter().copied().find(|span| !span.blocked)
}

pub(super) fn oriented_span_points(
    span: &RouteCorridorSpan,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Vec<Point> {
    if span.from_anchor_pad_uuid == from_anchor_pad_uuid
        && span.to_anchor_pad_uuid == to_anchor_pad_uuid
    {
        vec![span.from, span.to]
    } else {
        debug_assert!(
            span.from_anchor_pad_uuid == to_anchor_pad_uuid
                && span.to_anchor_pad_uuid == from_anchor_pad_uuid
        );
        vec![span.to, span.from]
    }
}
