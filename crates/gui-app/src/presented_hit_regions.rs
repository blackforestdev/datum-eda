//! Hit geometry belongs to the last successfully presented native frame.
use datum_gui_render::{HitRegion, HitTarget, PreparedScene};

#[derive(Default)]
pub(crate) struct PresentedHitRegions {
    regions: Vec<HitRegion>,
    pending: bool,
}

impl PresentedHitRegions {
    pub(crate) fn mark_pending(&mut self) {
        self.pending = true;
    }

    pub(crate) fn present(&mut self, prepared: &mut PreparedScene) {
        if self.pending {
            self.regions = std::mem::take(&mut prepared.hit_regions);
            self.pending = false;
        }
    }

    pub(crate) fn clear(&mut self) {
        self.regions.clear();
    }

    pub(crate) fn regions(&self) -> &[HitRegion] {
        &self.regions
    }

    pub(crate) fn hit_test(&self, x: f32, y: f32) -> Option<&HitTarget> {
        self.regions
            .iter()
            .rev()
            .find(|region| region.rect.contains(x, y))
            .map(|region| &region.target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_render::{CameraState, RetainedScene};

    #[test]
    fn presented_menu_survives_dirty_and_cached_frames_until_replacement() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.active_menu = Some("File".to_owned());
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let camera = CameraState::fit_to_bounds(&state.scene.bounds);
        let mut prepared =
            PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
        let regions = prepared.hit_regions.clone();
        let samples: Vec<_> = regions
            .iter()
            .map(|region| {
                let x = region.rect.x + region.rect.width * 0.5;
                let y = region.rect.y + region.rect.height * 0.5;
                (x, y, prepared.hit_test(x, y).cloned())
            })
            .collect();
        assert!(
            samples
                .iter()
                .any(|(_, _, target)| matches!(target, Some(HitTarget::MenuItem { .. })))
        );
        let allocation = prepared.hit_regions.as_ptr();
        let mut hits = PresentedHitRegions::default();
        hits.mark_pending();
        hits.present(&mut prepared);
        assert_eq!(hits.regions().as_ptr(), allocation);
        assert!(prepared.hit_regions.is_empty());
        // Rendering the cached frame again must not replace the published map
        // with the now-empty preparation vector.
        hits.present(&mut prepared);
        hits.mark_pending();
        state.ui.active_menu = None;
        let mut replacement =
            PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
        // Preparation or a failed frame cannot publish a menu dismissal.
        for (x, y, expected) in samples {
            assert_eq!(hits.hit_test(x, y).cloned(), expected);
        }
        hits.present(&mut replacement);
        assert!(
            !hits
                .regions()
                .iter()
                .any(|r| matches!(r.target, HitTarget::MenuItem { .. }))
        );
        hits.clear();
        assert!(hits.regions().is_empty());
    }
}
