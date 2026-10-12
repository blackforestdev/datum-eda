//! One captured-scene lookup for text qualification, preserving vector-first aliases.
use super::{LayoutPoint, layout_polygon};
use crate::{BoardTextGeometryPrimitive, BoardTextPrimitive};
use std::collections::BTreeMap;

pub(super) struct TextLayoutIndex<'a> {
    text_objects: BTreeMap<&'a str, &'a str>,
    geometries: BTreeMap<&'a str, &'a BoardTextGeometryPrimitive>,
}
impl<'a> TextLayoutIndex<'a> {
    pub(super) fn new(
        texts: &'a [BoardTextPrimitive],
        layouts: &'a [BoardTextGeometryPrimitive],
    ) -> Self {
        let mut text_objects = BTreeMap::new();
        for text in texts {
            // The old predicate ORed the raw render UUID and source UUID. The
            // first record matching either wins, including cross-alias collisions.
            text_objects
                .entry(text.text_uuid.as_str())
                .or_insert(text.object_id.as_str());
            if let Some(source) = text.source_object_uuid.as_deref() {
                text_objects
                    .entry(source)
                    .or_insert(text.object_id.as_str());
            }
        }
        let mut geometries = BTreeMap::new();
        for geometry in layouts {
            geometries
                .entry(geometry.object_id.as_str())
                .or_insert(geometry);
        }
        Self {
            text_objects,
            geometries,
        }
    }
    pub(super) fn layout(&self, uuid: &str) -> Option<[LayoutPoint; 4]> {
        let object = self.text_objects.get(uuid)?;
        layout_polygon(self.geometries.get(object)?)
    }
}

#[cfg(test)]
mod tests;
