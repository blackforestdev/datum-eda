//! One source-validated scene index; display keys never mint identities.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneSelectionBinding {
    pub projected: Id,
    pub acquired: Id,
    pub layers: Vec<String>,
}
impl SceneSelectionBinding {
    /// Ordinary acquisition gates the source origin, never derived membership.
    /// Ctrl+A deliberately bypasses this predicate at its shared acquisition entry.
    pub fn eligible(&self, filters: &crate::WorkspaceFilterState) -> bool {
        acquisition_eligible(
            self.acquired.class,
            self.layers.iter().map(String::as_str),
            filters,
        )
    }
    pub fn visible(&self, visible: impl Fn(&str) -> bool) -> bool {
        self.layers.iter().any(|layer| visible(layer))
    }
}
/// Shared ordinary origin/candidate eligibility for pointer, menu and regions.
/// Complete derived membership and explicit Ctrl+A do not consume this gate.
pub fn acquisition_eligible<'a>(
    class: Class,
    layers: impl IntoIterator<Item = &'a str>,
    filters: &crate::WorkspaceFilterState,
) -> bool {
    filters.show_authored
        && !filters.excluded_selection_classes.contains(&class)
        && layers
            .into_iter()
            .any(|layer| filters.layer_visibility.get(layer).copied().unwrap_or(true))
}
#[derive(Debug, Default)]
pub struct SceneSelectionBindings {
    bindings: BTreeMap<String, SceneSelectionBinding>,
    ambiguous: BTreeSet<String>,
}
impl SceneSelectionBindings {
    pub fn capture(scene: &BoardReviewSceneV1, authority: &NativeSelectionResolution) -> Self {
        let mut result = Self::default();
        let sources: BTreeMap<_, _> = authority
            .authored_identities()
            .iter()
            .filter(|id| id.instance_path.is_empty())
            .map(|id| (id.id, id))
            .collect();
        let parents: BTreeMap<_, _> = authority
            .snapshot()
            .schematic_source()
            .and_then(|s| {
                scene
                    .board_uuid
                    .parse()
                    .ok()
                    .and_then(|uuid| s.sheets.get(&uuid))
            })
            .into_iter()
            .flat_map(|sheet| sheet.symbols.values())
            .flat_map(|symbol| symbol.pins.iter().map(move |pin| (pin.uuid, symbol.uuid)))
            .collect();
        let mut bind = |key: &str,
                        class: Class,
                        uuid: &str,
                        parent: Option<(Class, String)>,
                        layers: Vec<String>| {
            let Ok(uuid) = uuid.parse() else {
                return;
            };
            let projected = Id {
                class,
                id: uuid,
                instance_path: vec![],
            };
            if !authority.authored_contains(&projected) {
                return;
            }
            let acquired = if let Some((class, uuid)) = parent {
                let Ok(uuid) = uuid.parse() else {
                    return;
                };
                Id {
                    class,
                    id: uuid,
                    instance_path: vec![],
                }
            } else if class == Class::Pin {
                let Some(parent) = parents.get(&uuid) else {
                    return;
                };
                Id {
                    class: Class::Symbol,
                    id: *parent,
                    instance_path: vec![],
                }
            } else {
                projected.clone()
            };
            if !authority.authored_contains(&acquired) {
                return;
            }
            result.insert(
                key,
                SceneSelectionBinding {
                    projected,
                    acquired,
                    layers,
                },
            );
        };
        for v in &scene.components {
            bind(
                &v.object_id,
                Class::Footprint,
                &v.source_object_uuid,
                None,
                vec![v.placement_layer.clone()],
            );
        }
        for v in &scene.pads {
            let layers = if v.copper_layer_ids.is_empty() {
                vec![v.layer_id.clone()]
            } else {
                v.copper_layer_ids.clone()
            };
            bind(
                &v.object_id,
                Class::Pad,
                &v.pad_uuid,
                Some((Class::Footprint, v.component_uuid.clone())),
                layers,
            );
        }
        for v in &scene.tracks {
            bind(
                &v.object_id,
                Class::Track,
                &v.source_object_uuid,
                None,
                vec![v.layer_id.clone()],
            );
        }
        for v in &scene.vias {
            bind(
                &v.object_id,
                Class::Via,
                &v.source_object_uuid,
                None,
                vec![v.start_layer_id.clone(), v.end_layer_id.clone()],
            );
        }
        for v in &scene.zones {
            bind(
                &v.object_id,
                Class::Zone,
                &v.source_object_uuid,
                None,
                vec![v.layer_id.clone()],
            );
        }
        for v in &scene.outline {
            bind(
                &v.object_id,
                Class::BoardOutline,
                &v.source_object_uuid,
                None,
                vec![v.layer_id.clone()],
            );
        }
        for v in &scene.board_texts {
            let uuid = v.source_object_uuid.as_deref().unwrap_or(&v.text_uuid);
            let class = uuid
                .parse()
                .ok()
                .and_then(|uuid| sources.get(&uuid))
                .map_or(Class::BoardText, |id| id.class);
            bind(&v.object_id, class, uuid, None, vec![v.layer_id.clone()]);
        }
        for v in &scene.board_graphics {
            let class = if v.object_kind == "board_outline" {
                Class::BoardOutline
            } else {
                match v.schematic_hit_kind() {
                    Some(crate::SchematicHitKind::Symbol) => Class::Symbol,
                    Some(crate::SchematicHitKind::Wire) => Class::Wire,
                    Some(crate::SchematicHitKind::Bus) => Class::BusSection,
                    Some(crate::SchematicHitKind::Label) => Class::Label,
                    Some(crate::SchematicHitKind::Port) => Class::Port,
                    Some(crate::SchematicHitKind::Junction) => Class::Junction,
                    Some(crate::SchematicHitKind::NoConnect) => Class::NoConnect,
                    Some(crate::SchematicHitKind::Pin) => Class::Pin,
                    Some(crate::SchematicHitKind::Drawing) => Class::SchematicGraphic,
                    None => continue,
                }
            };
            bind(
                &v.object_id,
                class,
                &v.source_object_uuid,
                None,
                vec![v.layer_id.clone()],
            );
        }
        result
    }
    fn insert(&mut self, key: &str, binding: SceneSelectionBinding) {
        if self.ambiguous.contains(key) {
            return;
        }
        if let Some(previous) = self.bindings.get(key) {
            if previous != &binding {
                self.bindings.remove(key);
                self.ambiguous.insert(key.to_owned());
            }
        } else {
            self.bindings.insert(key.to_owned(), binding);
        }
    }
    pub fn get(&self, key: &str) -> Option<&SceneSelectionBinding> {
        self.bindings.get(key)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&str, &SceneSelectionBinding)> {
        self.bindings
            .iter()
            .map(|(key, binding)| (key.as_str(), binding))
    }
}
