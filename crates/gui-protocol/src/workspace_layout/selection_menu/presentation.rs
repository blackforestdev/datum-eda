//! Source-bound conflict labels and transient projection keys, never identity.
use super::*;
use crate::{BoardReviewSceneV1, selection_scene::SceneSelectionBindings};
use std::collections::{BTreeMap, BTreeSet};

impl SelectionMenu {
    pub fn bind_presentation(
        &mut self,
        scene: &BoardReviewSceneV1,
        bindings: &SceneSelectionBindings,
        authority: &NativeSelectionResolution,
    ) {
        self.preview_keys.clear();
        if scene.project_uuid != self.project.to_string()
            || scene.source_revision != self.revision
            || scene.scene_id != self.source_scene
            || authority.project() != self.project
            || authority.revision().0 != self.revision
        {
            return;
        }
        let candidates: BTreeSet<_> = self.overlap_candidates().cloned().collect();
        let mut layers: BTreeMap<Id, BTreeSet<&str>> = BTreeMap::new();
        let mut nets: BTreeMap<Id, BTreeSet<String>> = BTreeMap::new();
        for (key, binding) in bindings.iter() {
            if !candidates.contains(&binding.acquired) || !binding.eligible(&self.filters) {
                continue;
            }
            self.preview_keys
                .entry(binding.acquired.clone())
                .or_default()
                .insert(key.into());
            layers
                .entry(binding.acquired.clone())
                .or_default()
                .extend(binding.layers.iter().map(String::as_str));
            let net = scene
                .tracks
                .iter()
                .find(|v| v.object_id == key)
                .and_then(|v| v.net_uuid.as_deref())
                .or_else(|| {
                    scene
                        .pads
                        .iter()
                        .find(|v| v.object_id == key)
                        .and_then(|v| v.net_uuid.as_deref())
                })
                .or_else(|| {
                    scene
                        .vias
                        .iter()
                        .find(|v| v.object_id == key)
                        .and_then(|v| v.net_uuid.as_deref())
                })
                .or_else(|| {
                    scene
                        .zones
                        .iter()
                        .find(|v| v.object_id == key)
                        .and_then(|v| v.net_uuid.as_deref())
                });
            if let Some(net) = net {
                let name = scene
                    .net_display
                    .iter()
                    .find(|v| v.net_uuid == net)
                    .map_or_else(|| net.to_owned(), |v| v.net_name.clone());
                nets.entry(binding.acquired.clone())
                    .or_default()
                    .insert(name);
            }
        }
        for item in &mut self.items {
            let Command::Object(id) = &item.command else {
                continue;
            };
            let uuid = id.id.to_string();
            let reference = scene
                .components
                .iter()
                .find(|v| v.source_object_uuid == uuid)
                .map(|v| v.reference.as_str())
                .or_else(|| {
                    authority
                        .snapshot()
                        .schematic_source()
                        .and_then(|s| {
                            scene
                                .board_uuid
                                .parse()
                                .ok()
                                .and_then(|sheet| s.sheets.get(&sheet))
                        })
                        .and_then(|s| s.symbols.get(&id.id))
                        .map(|v| v.reference.as_str())
                })
                .or_else(|| {
                    scene
                        .board_texts
                        .iter()
                        .find(|v| {
                            v.source_object_uuid.as_deref() == Some(uuid.as_str())
                                || v.text_uuid == uuid
                        })
                        .map(|v| v.text.as_str())
                });
            let mut detail = Vec::new();
            if let Some(reference) = reference {
                detail.push(reference.to_owned());
            }
            if let Some(names) = nets.get(id) {
                detail.extend(names.iter().map(|v| format!("Net {v}")));
            }
            if let Some(layers) = layers.get(id) {
                detail.extend(layers.iter().map(|id| {
                    scene
                        .layers
                        .iter()
                        .find(|v| v.layer_id == *id)
                        .map_or_else(|| (*id).to_owned(), |v| v.name.clone())
                }));
            }
            // Keep a stable discriminator after the useful details; label text
            // never participates in acquisition or preview-key resolution.
            detail.push(uuid[28..].to_owned());
            item.label = format!("Select {} · {}", class_label(id.class), detail.join(" · "));
        }
    }

    pub fn preview_candidate(&self) -> Option<(&Id, &BTreeSet<String>)> {
        let item = self.items.get(self.preview?)?;
        if !self.open || item.unavailable.is_some() {
            return None;
        }
        let Command::Object(id) = &item.command else {
            return None;
        };
        Some((id, self.preview_keys.get(id)?))
    }
}
