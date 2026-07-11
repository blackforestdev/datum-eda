//! Board review/object fit targets, separated from universal pane camera routing
//! so the high-frequency input module remains within its immutable source budget.

use super::*;

impl Runtime {
    pub(super) fn fit_review_target(&mut self) -> bool {
        let Some(bounds) = self.active_review_bounds() else {
            return false;
        };
        self.camera = CameraState::fit_to_bounds(&bounds);
        self.invalidate_frame();
        true
    }

    pub(super) fn fit_scene_object(&mut self, object_id: &str) -> bool {
        let Some(bounds) = self.scene_object_bounds(object_id) else {
            return false;
        };
        self.camera = CameraState::fit_to_bounds(&bounds);
        self.invalidate_frame();
        true
    }

    pub(super) fn scene_object_bounds(&self, object_id: &str) -> Option<SceneBounds> {
        let scene = &self.workspace().scene;
        if let Some(component) = scene
            .components
            .iter()
            .find(|item| item.object_id == object_id)
        {
            return Some(padded_rect_bounds(component.bounds, 1_500_000));
        }
        if let Some(pad) = scene.pads.iter().find(|item| item.object_id == object_id) {
            return Some(padded_rect_bounds(pad.bounds, 500_000));
        }
        if let Some(track) = scene.tracks.iter().find(|item| item.object_id == object_id) {
            return bounds_from_points(track.path.iter().copied(), 750_000);
        }
        if let Some(via) = scene.vias.iter().find(|item| item.object_id == object_id) {
            let radius = (via.diameter_nm / 2).max(250_000);
            return bounds_from_points([via.position], radius + 500_000);
        }
        if let Some(zone) = scene.zones.iter().find(|item| item.object_id == object_id) {
            return bounds_from_points(zone.polygon.iter().copied(), 750_000);
        }
        if let Some(text) = scene
            .board_texts
            .iter()
            .find(|item| item.object_id == object_id)
        {
            return bounds_from_points([text.position], text.height_nm.max(500_000));
        }
        if let Some(graphic) = scene
            .board_graphics
            .iter()
            .find(|item| item.object_id == object_id)
        {
            return bounds_from_points(graphic.path.iter().copied(), 750_000);
        }
        scene
            .outline
            .iter()
            .find(|item| item.object_id == object_id)
            .and_then(|outline| bounds_from_points(outline.path.iter().copied(), 750_000))
    }

    pub(super) fn active_review_bounds(&self) -> Option<SceneBounds> {
        let action_id = &self.workspace().active_review_target_id;
        let mut points = Vec::<PointNm>::new();
        for overlay in self
            .workspace()
            .scene
            .proposal_overlay_primitives
            .iter()
            .filter(|overlay| &overlay.proposal_action_id == action_id)
        {
            points.extend(overlay.path.iter().copied());
        }
        if let Some(evidence_key) = self
            .workspace()
            .selected_review_action()
            .map(|action| format!("segment:{}", action.selected_path_segment_index))
        {
            for review in self
                .workspace()
                .scene
                .review_primitives
                .iter()
                .filter(|review| review.evidence_key.as_deref() == Some(evidence_key.as_str()))
            {
                points.extend(review.path.iter().copied());
            }
        }
        let action = self.workspace().selected_review_action()?;
        for pad in self.workspace().scene.pads.iter().filter(|pad| {
            pad.pad_uuid == action.from_anchor_pad_uuid || pad.pad_uuid == action.to_anchor_pad_uuid
        }) {
            points.push(pad.center);
        }
        if points.is_empty() {
            return None;
        }
        let (min_x, max_x) = points
            .iter()
            .map(|point| point.x)
            .fold((i64::MAX, i64::MIN), |(min_x, max_x), x| {
                (min_x.min(x), max_x.max(x))
            });
        let (min_y, max_y) = points
            .iter()
            .map(|point| point.y)
            .fold((i64::MAX, i64::MIN), |(min_y, max_y), y| {
                (min_y.min(y), max_y.max(y))
            });
        let padding_nm = 1_500_000_i64;
        Some(SceneBounds {
            min_x: min_x.saturating_sub(padding_nm),
            min_y: min_y.saturating_sub(padding_nm),
            max_x: max_x.saturating_add(padding_nm),
            max_y: max_y.saturating_add(padding_nm),
        })
    }
}
