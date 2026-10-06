//! Governed `selection-lock-anchor` screen-space cue (Rendering Book §2.5).
//!
//! The glyph is the committed 24px keyline asset, drawn at the approved Selection
//! Study canvas size. It is presentation only: it never changes membership,
//! eligibility or authored state, and schematic symbols carry no S5A lock.
use super::*;
use datum_gui_protocol::{SelectionSubject, selection_view::SelectionView};

/// Asset of record; `asset_matches_code_geometry` binds the drawing below to it.
#[cfg(test)]
const ASSET: &str = include_str!("../../../engine/assets/icons/eda/selection-lock-anchor.svg");
const COLOR: [f32; 3] = crate::design_tokens::content::LOCK_ANCHOR;
/// Selection Study panel 3 draws a 12px-wide lock body; the keyline body is 14
/// of 24 units, so one unit is 12/14 px and the 1.7 stroke stays proportional.
const PX_PER_UNIT: f32 = 12.0 / 14.0;
const STROKE_UNITS: f32 = 1.7;
/// Keyline glyph centre (the 24-unit box centre) sits on the authored anchor.
const CENTRE: (f32, f32) = (12.0, 12.0);

/// Package-owned hit targets of locked placements, keyed to the authored anchor.
/// Pad hits collapse to their parent Footprint, as selection does.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct HoverAnchors(Vec<(String, PointNm)>);

impl HoverAnchors {
    pub(crate) fn build(scene: &BoardReviewSceneV1) -> Self {
        let locked: std::collections::BTreeMap<&str, (&str, PointNm)> = scene
            .components
            .iter()
            .filter(|c| c.locked)
            .map(|c| {
                (
                    c.component_uuid.as_str(),
                    (c.object_id.as_str(), c.position),
                )
            })
            .collect();
        if locked.is_empty() {
            return Self::default();
        }
        let mut anchors: Vec<_> = locked
            .values()
            .map(|(id, at)| (id.to_string(), *at))
            .chain(scene.pads.iter().filter_map(|pad| {
                locked
                    .get(pad.component_uuid.as_str())
                    .map(|(_, at)| (pad.object_id.clone(), *at))
            }))
            .collect();
        anchors.sort_by(|a, b| a.0.cmp(&b.0));
        anchors.dedup_by(|a, b| a.0 == b.0);
        Self(anchors)
    }
    pub(crate) fn get(&self, hit_target: &str) -> Option<PointNm> {
        self.0
            .binary_search_by(|(id, _)| id.as_str().cmp(hit_target))
            .ok()
            .map(|i| self.0[i].1)
    }
}

/// Selected locked members. A single object shows its padlock; a compound shows
/// only its explicit focus member, so repeated glyphs never form a field while
/// the Inspector carries the exact locked count. Hidden members emit no cue.
pub(crate) fn push_selected(
    out: &mut Vec<Quad>,
    view: &SelectionView,
    projection: &Projection,
    viewport: RectPx,
) {
    let candidates: Vec<_> = match &view.subject {
        SelectionSubject::Object(id) => vec![id],
        SelectionSubject::Compound(_) => view.focus().into_iter().collect(),
        _ => Vec::new(),
    };
    for id in candidates {
        if view.locked.contains(id)
            && !view.hidden.contains(id)
            && let Some(anchor) = view.authored_anchor(id)
        {
            push_at(out, projection.project_point(anchor), viewport);
        }
    }
}

/// Hovered locked placement, whether or not it is selected.
pub(crate) fn push_hover(
    out: &mut Vec<Quad>,
    anchor: Option<PointNm>,
    projection: &Projection,
    viewport: RectPx,
) {
    if let Some(anchor) = anchor {
        push_at(out, projection.project_point(anchor), viewport);
    }
}

fn push_at(out: &mut Vec<Quad>, anchor: (f32, f32), viewport: RectPx) {
    // Whole glyph or nothing: a pane edge never shows a partial padlock.
    let half = 12.0 * PX_PER_UNIT;
    if anchor.0 - half < viewport.x
        || anchor.1 - half < viewport.y
        || anchor.0 + half > viewport.x + viewport.width
        || anchor.1 + half > viewport.y + viewport.height
    {
        return;
    }
    let at = |x: f32, y: f32| {
        (
            anchor.0 + (x - CENTRE.0) * PX_PER_UNIT,
            anchor.1 + (y - CENTRE.1) * PX_PER_UNIT,
        )
    };
    let width = STROKE_UNITS * PX_PER_UNIT;
    let mut path = |points: &[(f32, f32)], closed: bool| {
        let mapped: Vec<_> = points.iter().map(|p| at(p.0, p.1)).collect();
        let count = if closed {
            mapped.len()
        } else {
            mapped.len() - 1
        };
        for i in 0..count {
            push_segment(out, mapped[i], mapped[(i + 1) % mapped.len()], width);
        }
    };
    // <rect x="5" y="11" width="14" height="9" rx="1.5"/>
    path(&rounded_rect(5.0, 11.0, 14.0, 9.0, 1.5), true);
    // <path d="M8 11V8a4 4 0 0 1 8 0v3"/>: the shackle arc passes over (12, 4).
    let mut shackle = vec![(8.0, 11.0)];
    shackle.extend(arc((12.0, 8.0), 4.0, std::f32::consts::PI, 0.0, 12));
    shackle.push((16.0, 11.0));
    path(&shackle, false);
    // <circle cx="12" cy="15" r="1.2"/>
    path(
        &arc((12.0, 15.0), 1.2, 0.0, std::f32::consts::TAU, 12),
        true,
    );
}

/// Screen-space arc from `from` to `to` radians (y down, so decreasing angle
/// sweeps over the top), inclusive of both endpoints.
fn arc(c: (f32, f32), r: f32, from: f32, to: f32, steps: usize) -> Vec<(f32, f32)> {
    let sweep = if to < from { -(from - to) } else { to - from };
    (0..=steps)
        .map(|i| {
            let a = from + sweep * i as f32 / steps as f32;
            (c.0 + r * a.cos(), c.1 - r * a.sin())
        })
        .collect()
}

fn rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Vec<(f32, f32)> {
    use std::f32::consts::{FRAC_PI_2, PI};
    let mut points = Vec::new();
    for (c, start) in [
        ((x + w - r, y + h - r), -FRAC_PI_2),
        ((x + w - r, y + r), 0.0),
        ((x + r, y + r), FRAC_PI_2),
        ((x + r, y + h - r), PI),
    ] {
        points.extend(arc(c, r, start, start + FRAC_PI_2, 3));
    }
    points
}

fn push_segment(out: &mut Vec<Quad>, a: (f32, f32), b: (f32, f32), width: f32) {
    // Extend each end by half the stroke so joins close like round caps.
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length = dx.hypot(dy).max(f32::EPSILON);
    let (ux, uy) = (dx / length * width * 0.5, dy / length * width * 0.5);
    let (a, b) = ((a.0 - ux, a.1 - uy), (b.0 + ux, b.1 + uy));
    out.push(Quad {
        points: [
            (a.0 - uy, a.1 + ux),
            (b.0 - uy, b.1 + ux),
            (b.0 + uy, b.1 - ux),
            (a.0 + uy, a.1 - ux),
        ],
        color: COLOR,
    });
}

#[cfg(test)]
#[path = "lock_cue_tests.rs"]
mod tests;
