use super::*;

impl RetainedSceneHistory {
    pub(crate) fn construct<T>(
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
