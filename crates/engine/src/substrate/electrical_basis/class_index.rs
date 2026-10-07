//! Source-derived membership acceleration; resolver kind strings never qualify it.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;
// These are the source maps queried by existing electrical/selection owners.
// Unknown classes retain the original recursive lookup.
const CLASSES: &[&str] = &[
    "wires",
    "labels",
    "ports",
    "junctions",
    "pins",
    "buses",
    "bus_entries",
    "symbols",
    "texts",
    "drawings",
    "packages",
    "pads",
    "tracks",
    "vias",
    "zones",
    "graphics",
    "nets",
    "instances",
    "pin_pad_maps",
];
pub(super) struct Index(BTreeMap<(Uuid, usize), BTreeSet<Uuid>>);
impl Index {
    pub(super) fn capture(values: &BTreeMap<Uuid, Value>) -> Self {
        let mut index = Self(BTreeMap::new());
        for (&shard, value) in values {
            index.walk(shard, value, 0);
        }
        index
    }
    fn walk(&mut self, shard: Uuid, value: &Value, classes: u32) {
        if classes != 0 && let Some(text) = value.get("uuid").and_then(Value::as_str)
            && let Ok(id) = text.parse::<Uuid>()
            // Preserve find_class's exact canonical UUID spelling test.
            && text == id.to_string()
        {
            for class in 0..CLASSES.len() {
                if classes & (1 << class) != 0 {
                    self.0.entry((shard, class)).or_default().insert(id);
                }
            }
        }
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    let child_classes = CLASSES
                        .iter()
                        .position(|class| *class == key)
                        .map_or(classes, |index| classes | (1 << index));
                    if key == "component_pads"
                        && let Some(groups) = child.as_object()
                    {
                        for pad in groups
                            .values()
                            .filter_map(Value::as_array)
                            .flatten()
                            .filter(|p| p.get("source_binding").is_some_and(|v| !v.is_null()))
                        {
                            if let Some(id) = pad
                                .get("uuid")
                                .and_then(Value::as_str)
                                .and_then(|s| s.parse::<Uuid>().ok())
                            {
                                self.0
                                    .entry((
                                        shard,
                                        CLASSES.iter().position(|c| *c == "pads").unwrap(),
                                    ))
                                    .or_default()
                                    .insert(id);
                            }
                        }
                    }
                    self.walk(shard, child, child_classes);
                }
            }
            Value::Array(array) => {
                for child in array {
                    self.walk(shard, child, classes);
                }
            }
            _ => {}
        }
    }
    pub(super) fn contains(&self, shard: Uuid, class: &str, id: Uuid) -> Option<bool> {
        let class = CLASSES.iter().position(|candidate| *candidate == class)?;
        Some(
            self.0
                .get(&(shard, class))
                .is_some_and(|ids| ids.contains(&id)),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn index_matches_recursive_source_class_membership_in_maps_arrays_and_nested_sources() {
        let shard = Uuid::new_v4();
        let other = Uuid::new_v4();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let uppercase = Uuid::from_u128(0xabcdefabcdef40008000abcdefabcdef);
        let value = serde_json::json!({
            "symbols": { "noncanonical-map-key": {"uuid":a, "pins":[{"uuid":b}]}},
            "nested": [{"texts": {"another-key": {"uuid":c}}}],
            "tracks": [{"uuid": uppercase.to_string().to_uppercase()}],
            "unclassified": {"uuid":uppercase}
        });
        let index = Index::capture(&BTreeMap::from([(shard, value.clone())]));
        for class in CLASSES {
            for id in [a, b, c, uppercase, Uuid::nil()] {
                assert_eq!(
                    index.contains(shard, class, id),
                    Some(super::super::find_class(&value, class, id).is_some()),
                    "{class}/{id}"
                );
                assert_eq!(index.contains(other, class, id), Some(false));
            }
        }
        assert_eq!(index.contains(shard, "future_class", a), None);
    }
}
