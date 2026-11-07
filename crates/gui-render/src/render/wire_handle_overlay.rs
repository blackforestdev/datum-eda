//! Immediate edit grips over a qualified selection; pointer updates reuse this snapshot.
use super::*;
use datum_gui_protocol::{AuthoredSelectionClass as Class, MoveHandleKind as Kind};
use eda_engine::api::native_write::schematic_wire_edit::{
    WireEndpoint, WireHandle, available_handles,
};

#[derive(Debug, Clone, PartialEq)]
struct Handle {
    point: PointNm,
    kind: Kind,
    width: Option<i64>,
    rank: String,
    key: String,
    identity: datum_gui_protocol::AuthoredSelectionIdentity,
    handle: WireHandle,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Catalog {
    panes: Vec<(datum_gui_protocol::PaneId, Vec<Handle>)>,
    context: Option<(String, String, datum_gui_protocol::SelectionSubject)>,
}

impl Catalog {
    pub(crate) fn capture(state: &ReviewWorkspaceState) -> Self {
        if state.authoring.gesture.is_active() {
            return Self::default();
        }
        let panes = state
            .ui
            .layout
            .leaves()
            .into_iter()
            .filter_map(|pane| {
                let scene = datum_gui_protocol::camera_scene_for_pane(state, pane)?;
                let handles = selected_handles(state, scene, pane);
                (!handles.is_empty()).then_some((pane, handles))
            })
            .collect();
        let context = if let SelectionTarget::Native(view) = &state.selection {
            Some((
                state.scene.project_uuid.clone(),
                state.scene.source_revision.clone(),
                view.subject.clone(),
            ))
        } else {
            None
        };
        Self { panes, context }
    }

    pub(crate) fn is_current(&self, state: &ReviewWorkspaceState) -> bool {
        let SelectionTarget::Native(view) = &state.selection else {
            return false;
        };
        let datum_gui_protocol::SelectionSubject::Object(id) = &view.subject else {
            return false;
        };
        !view.stale
            && view.locked.is_empty()
            && !state.authoring.gesture.is_active()
            && !view.hidden.contains(id)
            && !view.unprojected.contains(id)
            && state.ui.filters.show_authored
            && !state
                .ui
                .filters
                .excluded_selection_classes
                .contains(&id.class)
            && matches!(
                state.tool,
                datum_gui_protocol::WorkspaceTool::Select | datum_gui_protocol::WorkspaceTool::Move
            )
            && state.ui.active_menu.is_none()
            && !state.ui.new_project.open
            && !state.ui.project_open.open
            && state.library.acquisition.is_none()
            && self
                .context
                .as_ref()
                .is_some_and(|(project, revision, subject)| {
                    *project == state.scene.project_uuid
                        && *revision == state.scene.source_revision
                        && *subject == view.subject
                })
    }
    pub(crate) fn contains_pane(&self, pane: datum_gui_protocol::PaneId) -> bool {
        self.panes
            .iter()
            .any(|(id, handles)| *id == pane && !handles.is_empty())
    }

    pub(crate) fn hover_kind(
        &self,
        pass: &PreparedSurfacePass,
        cursor: (f32, f32),
    ) -> Option<Kind> {
        self.hovered(pass, cursor).map(|handle| handle.kind)
    }
    pub(crate) fn pick(
        &self,
        pass: &PreparedSurfacePass,
        cursor: (f32, f32),
    ) -> Option<(
        String,
        datum_gui_protocol::AuthoredSelectionIdentity,
        WireHandle,
    )> {
        self.hovered(pass, cursor)
            .map(|h| (h.key.clone(), h.identity.clone(), h.handle))
    }

    fn hovered(&self, pass: &PreparedSurfacePass, cursor: (f32, f32)) -> Option<&Handle> {
        let (_, handles) = self.panes.iter().find(|(pane, _)| *pane == pass.pane_id)?;
        if !pass.scene_viewport.contains(cursor.0, cursor.1) {
            return None;
        }
        let projection = Projection::new(
            inset_rect(pass.scene_viewport, 10., 10., 10., 10.),
            &pass.bounds,
            pass.camera,
        );
        let world = crate::coordinate_hit::editor_viewport(pass.pane_id, pass.surface, &projection)
            .screen_to_world(datum_gui_protocol::ScreenPointPx {
                x: cursor.0,
                y: cursor.1,
            })?;
        // Six pixels enclose the widest ring; keep any larger source-ink envelope.
        let grip_width = (12. / projection.scale).ceil() as i64;
        PreparedScene::nearest_schematic_contact(
            world,
            handles.iter().map(|h| {
                (
                    h.point,
                    Some(h.width.unwrap_or(0).max(grip_width)),
                    h.rank.clone(),
                    h,
                )
            }),
        )
    }

    pub(crate) fn push_hover(
        &self,
        out: &mut Vec<Quad>,
        pass: &PreparedSurfacePass,
        cursor: (f32, f32),
    ) {
        let Some(handle) = self.hovered(pass, cursor) else {
            return;
        };
        let projection = Projection::new(
            inset_rect(pass.scene_viewport, 10., 10., 10., 10.),
            &pass.bounds,
            pass.camera,
        );
        let at = project_point(handle.point, &projection);
        let start = out.len();
        ring(out, at, 7., HOVER_HIGHLIGHT, 1.);
        glyph(out, at, handle.kind, true);
        clip(out, start, pass.scene_viewport);
    }
}

fn selected_handles(
    state: &ReviewWorkspaceState,
    scene: &BoardReviewSceneV1,
    pane: datum_gui_protocol::PaneId,
) -> Vec<Handle> {
    use datum_gui_protocol::{PaneContent, SelectionSubject, WorkspaceTool};
    if !state.ui.filters.show_authored
        || state.ui.active_menu.is_some()
        || state.ui.new_project.open
        || state.ui.project_open.open
        || state.library.acquisition.is_some()
        || !matches!(state.tool, WorkspaceTool::Select | WorkspaceTool::Move)
        || state.schematic_authoring.picker
        || state.authoring.gesture.is_active()
        || pane != state.ui.layout.focused
    {
        return Vec::new();
    }
    let SelectionTarget::Native(view) = &state.selection else {
        return Vec::new();
    };
    // A grip is one captured Object intent. A compound keeps its complete group Move.
    let SelectionSubject::Object(id) = &view.subject else {
        return Vec::new();
    };
    if !matches!(id.class, Class::Wire | Class::Junction)
        || !view.locked.is_empty()
        || view.hidden.contains(id)
        || view.unprojected.contains(id)
        || state
            .ui
            .filters
            .excluded_selection_classes
            .contains(&id.class)
    {
        return Vec::new();
    }
    let Some(keys) = view.projection_keys(scene, PaneContent::Schematic) else {
        return Vec::new();
    };
    let Ok(basis) =
        datum_gui_protocol::AuthoringViewBasis::capture_qualified_schematic(state, pane)
    else {
        return Vec::new();
    };
    let Some(document) = &scene.schematic_document else {
        return Vec::new();
    };
    if basis.document.as_ref() != Some(document) || id.instance_path != document.instance_path {
        return Vec::new();
    }
    let Some(graphic) = scene.board_graphics.iter().find(|graphic| {
        graphic.source_object_uuid == id.id.to_string()
            && keys.contains(&graphic.object_id)
            && layer_visible(state, &graphic.layer_id)
    }) else {
        return Vec::new();
    };
    let Some(sheet) = state
        .schematic_authoring
        .source
        .as_ref()
        .and_then(|source| source.sheets.get(&document.source_sheet))
    else {
        return Vec::new();
    };
    if id.class == Class::Junction {
        return sheet
            .junctions
            .get(&id.id)
            .map(|junction| {
                vec![Handle {
                    point: PointNm {
                        x: junction.position.x,
                        y: junction.position.y,
                    },
                    kind: Kind::Junction,
                    width: graphic.width_nm,
                    rank: "0:junction".into(),
                    key: graphic.object_id.clone(),
                    identity: id.clone(),
                    handle: WireHandle::Vertex {
                        point: junction.position,
                    },
                }]
            })
            .unwrap_or_default();
    }
    let Some(wire) = sheet.wires.get(&id.id) else {
        return Vec::new();
    };
    let Ok(handles) = available_handles(sheet, id.id) else {
        return Vec::new();
    };
    let mut points = std::collections::BTreeMap::new();
    for handle in handles {
        let (point, mut kind, mut priority) = match handle {
            WireHandle::Endpoint { end } => {
                let point = if end == WireEndpoint::From {
                    wire.from
                } else {
                    wire.to
                };
                (point, Kind::Endpoint, 1)
            }
            WireHandle::Vertex { point } => (
                point,
                if sheet.junctions.values().any(|j| j.position == point) {
                    Kind::Junction
                } else {
                    Kind::Vertex
                },
                0,
            ),
            _ => continue, // The span itself is the body handle; no extra marker owns it.
        };
        let mut identity = id.clone();
        let mut key = graphic.object_id.clone();
        let mut picked_handle = handle;
        if let Some(junction) = sheet.junctions.values().find(|j| j.position == point) {
            let junction_graphic = scene.board_graphics.iter().find(|g| {
                g.source_object_uuid == junction.uuid.to_string()
                    && layer_visible(state, &g.layer_id)
            });
            if !state
                .ui
                .filters
                .excluded_selection_classes
                .contains(&Class::Junction)
                && let Some(junction_graphic) = junction_graphic
            {
                kind = Kind::Junction;
                priority = 0;
                identity.id = junction.uuid;
                identity.class = Class::Junction;
                key = junction_graphic.object_id.clone();
                picked_handle = WireHandle::Vertex { point };
            } else if !matches!(handle, WireHandle::Vertex { .. }) {
                continue; // A hidden/ineligible node is not an accidental reconnect grip.
            } else {
                kind = Kind::Vertex;
            }
        }
        let candidate = Handle {
            point: PointNm {
                x: point.x,
                y: point.y,
            },
            kind,
            width: graphic.width_nm,
            rank: format!("{priority}:{handle:?}"),
            key,
            identity,
            handle: picked_handle,
        };
        points
            .entry((point.x, point.y))
            .and_modify(|old: &mut Handle| {
                if candidate.rank < old.rank {
                    *old = candidate.clone();
                }
            })
            .or_insert(candidate);
    }
    points.into_values().collect()
}

pub(crate) fn push_selected(
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
    pane: datum_gui_protocol::PaneId,
    projection: &Projection,
    viewport: RectPx,
) {
    let start = out.len();
    if let Some(feedback) = &state.authoring.handle_feedback {
        if feedback.basis.pane == pane
            && feedback.basis.is_current(state, pane)
            && scene.schematic_document.as_ref() == feedback.basis.document.as_ref()
            && state.authoring.gesture.is_active()
        {
            if feedback.kind == Kind::Endpoint && feedback.source != feedback.destination {
                glyph(
                    out,
                    project_point(feedback.source, projection),
                    Kind::Endpoint,
                    false,
                );
            }
            glyph(
                out,
                project_point(feedback.destination, projection),
                feedback.kind,
                true,
            );
            if let Some(target) = feedback.snap_target {
                ring(
                    out,
                    project_point(target, projection),
                    7.,
                    AUTHOR_SELECTED,
                    1.5,
                );
            }
        }
    } else {
        for handle in selected_handles(state, scene, pane) {
            glyph(
                out,
                project_point(handle.point, projection),
                handle.kind,
                false,
            );
        }
    }
    clip(out, start, viewport);
}

fn glyph(out: &mut Vec<Quad>, at: (f32, f32), kind: Kind, filled: bool) {
    let (x, y) = at;
    match kind {
        Kind::Junction => ring(out, at, 5., AUTHOR_SELECTED, if filled { 1.5 } else { 1. }),
        Kind::Vertex => {
            let points = [(x, y - 4.), (x + 4., y), (x, y + 4.), (x - 4., y)];
            if filled {
                push_projected_quad(out, &points, AUTHOR_SELECTED);
            } else {
                for i in 0..4 {
                    line(out, points[i], points[(i + 1) % 4], AUTHOR_SELECTED, 1.);
                }
            }
        }
        Kind::Endpoint | Kind::Segment => {
            let rect = RectPx {
                x: x - 3.,
                y: y - 3.,
                width: 6.,
                height: 6.,
            };
            if filled {
                out.push(Quad::from_rect(rect, AUTHOR_SELECTED));
            } else {
                push_rect_border(out, rect, AUTHOR_SELECTED, 1.);
            }
        }
    }
}
fn ring(out: &mut Vec<Quad>, at: (f32, f32), radius: f32, color: [f32; 3], width: f32) {
    for i in 0..24 {
        let angle = |n| std::f32::consts::TAU * n as f32 / 24.;
        let point = |a: f32| (at.0 + radius * a.cos(), at.1 + radius * a.sin());
        line(out, point(angle(i)), point(angle(i + 1)), color, width);
    }
}
fn line(out: &mut Vec<Quad>, a: (f32, f32), b: (f32, f32), color: [f32; 3], width: f32) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length = dx.hypot(dy).max(f32::EPSILON);
    let (nx, ny) = (-dy / length * width / 2., dx / length * width / 2.);
    push_projected_quad(
        out,
        &[
            (a.0 + nx, a.1 + ny),
            (b.0 + nx, b.1 + ny),
            (b.0 - nx, b.1 - ny),
            (a.0 - nx, a.1 - ny),
        ],
        color,
    );
}
fn clip(out: &mut Vec<Quad>, start: usize, viewport: RectPx) {
    hit_clipping::clip_content(out, &mut Vec::new(), &mut Vec::new(), start, 0, 0, viewport);
}

#[cfg(test)]
#[path = "wire_handle_overlay_tests.rs"]
mod tests;
