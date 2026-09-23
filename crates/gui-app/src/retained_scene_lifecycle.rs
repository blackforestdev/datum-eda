use super::*;

impl RetainedSceneHistory {
    fn construct<T>(
        &mut self,
        document: &str,
        mut build: impl FnMut() -> anyhow::Result<T>,
    ) -> Option<T> {
        if self.construction_error.is_some() {
            return None;
        }
        let first = build();
        let result = if first.is_err() {
            let mut retired = false;
            while let Some(index) = self
                .entries
                .iter()
                .position(|entry| entry.key.scene_id == document)
            {
                self.evict(index);
                retired = true;
            }
            if self.entries.is_empty() {
                self.entries = Vec::new();
                self.entry_storage = None;
            }
            if retired { build() } else { first }
        } else {
            first
        };
        match result {
            Ok(scene) => Some(scene),
            Err(error) => {
                self.construction_error = Some(error.to_string());
                None
            }
        }
    }
}

impl Runtime {
    pub(crate) fn ensure_schematic_retained_scene(&mut self) -> bool {
        if self.schematic_scene_accounting.construction_error.is_some() {
            return false;
        }
        if self.schematic_retained_scene.is_none() {
            let state = self.session.workspace();
            let Some(schematic) = &state.schematic_scene else {
                return true;
            };
            let Some(scene) =
                self.schematic_scene_accounting
                    .construct(&schematic.scene_id, || {
                        RetainedScene::try_from_workspace_schematic_for_surface(
                            state,
                            self.config.width,
                            self.config.height,
                            self.scale_factor,
                        )
                    })
            else {
                return false;
            };
            if let Some(scene) = &scene {
                self.schematic_scene_accounting.limit_for_active(scene);
            }
            self.schematic_retained_scene = scene;
        }
        true
    }

    pub(crate) fn clear_schematic_retained_scene(&mut self) {
        self.schematic_retained_scene = None;
        self.schematic_scene_accounting.clear();
    }

    pub(crate) fn retained_scene_cache_key(&self) -> RetainedSceneCacheKey {
        let workspace = self.workspace();
        RetainedSceneCacheKey {
            scene_id: workspace.scene.scene_id.clone(),
            source_revision: workspace.scene.source_revision.clone(),
            width: self.config.width,
            height: self.config.height,
            scale_bits: self.scale_factor.to_bits(),
            dock_height_px: workspace.ui.effective_dock_height_px(),
            show_authored: workspace.ui.filters.show_authored,
            show_proposed: workspace.ui.filters.show_proposed,
            show_unrouted: workspace.ui.filters.show_unrouted,
            dim_unrelated: workspace.ui.filters.dim_unrelated,
            layer_visibility: workspace
                .ui
                .filters
                .layer_visibility
                .iter()
                .map(|(key, value)| (key.clone(), *value))
                .collect(),
            selection: retained_selection_cache_key(workspace, &workspace.selection),
        }
    }

    pub(crate) fn restore_cached_retained_scene(&mut self) -> bool {
        let key = self.retained_scene_cache_key();
        if let Some(retained) = self.retained_scene_cache.take(&key) {
            self.retained_scene = Some(retained);
            return true;
        }
        false
    }

    pub(crate) fn ensure_retained_scene(&mut self) -> bool {
        if self.retained_scene_cache.construction_error.is_some() {
            return false;
        }
        if self.retained_scene.is_none() {
            let state = self.session.workspace();
            let Some(retained) = self
                .retained_scene_cache
                .construct(&state.scene.scene_id, || {
                    RetainedScene::try_from_workspace_for_surface(
                        state,
                        self.config.width,
                        self.config.height,
                        self.scale_factor,
                    )
                })
            else {
                return false;
            };
            self.retained_scene_cache.limit_for_active(&retained);
            self.retained_scene = Some(retained);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_failure_retires_matching_history_and_retries_once() {
        let scene = super::super::tests::scene();
        let mut owner = RetainedSceneHistory::default();
        let mut matching = super::super::tests::key(0);
        matching.scene_id = "construction-matching".into();
        let unrelated = super::super::tests::key(1);
        owner.insert(matching, scene.clone());
        owner.insert(unrelated.clone(), scene.clone());
        let mut attempts = 0;
        let result = owner.construct("construction-matching", || {
            attempts += 1;
            if attempts == 1 {
                anyhow::bail!("vertex expansion refused")
            }
            Ok(scene.clone())
        });
        assert_eq!(attempts, 2);
        assert_eq!(result, Some(scene));
        assert_eq!(owner.entries.len(), 1);
        assert_eq!(owner.entries[0].key, unrelated);
        owner.check_render_budget().unwrap();
    }

    #[test]
    fn construction_failure_is_sticky_for_input_and_explicitly_retryable() {
        let mut owner = RetainedSceneHistory::default();
        assert!(
            owner
                .construct::<()>("construction-failed", || anyhow::bail!(
                    "vertex expansion refused"
                ))
                .is_none()
        );
        assert!(
            owner
                .check_render_budget()
                .unwrap_err()
                .to_string()
                .contains("vertex expansion refused")
        );
        assert!(
            owner
                .construct::<()>("construction-failed", || panic!(
                    "input must not repeatedly rebuild refused scene"
                ))
                .is_none()
        );
        owner.retry_construction();
        assert_eq!(owner.construct("construction-failed", || Ok(7)), Some(7));
        owner.check_render_budget().unwrap();
    }
}
