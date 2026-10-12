//! One admitted borrowed first-match layout index shared by both hit visits.
use datum_gui_protocol::BoardTextGeometryPrimitive;
use std::alloc::Layout;

pub(super) struct Lookup<'a> {
    geometries: &'a [BoardTextGeometryPrimitive],
    ordinals: Vec<usize>,
}
impl<'a> Lookup<'a> {
    pub(super) fn new(
        geometries: &'a [BoardTextGeometryPrimitive],
        admit: &mut impl FnMut(usize) -> anyhow::Result<()>,
    ) -> anyhow::Result<Self> {
        let mut ordinals = Vec::new();
        if !geometries.is_empty() {
            // The retained constructor's active allocation scope owns this
            // transient storage. It stays live through hit-output admission.
            admit(crate::cpu_alloc::heap::allocation_bytes(Layout::array::<
                usize,
            >(
                geometries.len(),
            )?))?;
            ordinals.try_reserve_exact(geometries.len())?;
            anyhow::ensure!(
                ordinals.capacity() == geometries.len(),
                "layout index capacity differs from admission"
            );
            ordinals.extend(0..geometries.len());
            ordinals.sort_unstable_by(|&a, &b| {
                geometries[a]
                    .object_id
                    .cmp(&geometries[b].object_id)
                    .then(a.cmp(&b))
            });
        }
        Ok(Self {
            geometries,
            ordinals,
        })
    }
    pub(super) fn get(&self, object_id: &str) -> Option<&'a BoardTextGeometryPrimitive> {
        let position = self
            .ordinals
            .partition_point(|&i| self.geometries[i].object_id.as_str() < object_id);
        let ordinal = *self.ordinals.get(position)?;
        let geometry = &self.geometries[ordinal];
        (geometry.object_id == object_id).then_some(geometry)
    }
}

#[cfg(test)]
#[path = "text_layouts/tests.rs"]
mod tests;
