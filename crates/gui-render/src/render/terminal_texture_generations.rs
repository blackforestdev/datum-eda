//! Preserve immutable terminal texture identity through close and device recovery.
use super::TerminalGraphicTextureKey;
use crate::text_gpu::budget::Budget;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

#[derive(Clone, Default)]
pub(super) struct TextureGenerations(Arc<Mutex<BTreeMap<TerminalGraphicTextureKey, Weak<Budget>>>>);

impl TextureGenerations {
    pub(super) fn for_key(&self, key: TerminalGraphicTextureKey) -> Arc<Budget> {
        let mut entries = self.0.lock().unwrap_or_else(|e| e.into_inner());
        entries.retain(|_, owner| owner.strong_count() != 0);
        if let Some(owner) = entries.get(&key).and_then(Weak::upgrade) {
            return owner;
        }
        let owner = Budget::new(2);
        entries.insert(key, Arc::downgrade(&owner));
        owner
    }
}
