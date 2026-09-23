//! Immutable text mesh lookup with one admitted index and borrowed assets.
use super::*;

pub(crate) trait GlyphMeshLookup {
    fn mesh(&self, key: &GlyphMeshHandlePrimitive) -> Option<&GlyphMeshAssetPrimitive>;
}

pub(super) struct MeshIndex<'a>(Vec<(usize, &'a GlyphMeshAssetPrimitive)>);
impl<'a> MeshIndex<'a> {
    pub(super) fn new(
        assets: &'a [GlyphMeshAssetPrimitive],
        output: &mut impl Output<Quad>,
    ) -> Option<Self> {
        let mut entries = output.scratch(assets.len())?;
        entries.extend(assets.iter().enumerate());
        entries.sort_unstable_by_key(|(index, asset)| (asset.handle, *index));
        Some(Self(entries))
    }
}
impl GlyphMeshLookup for MeshIndex<'_> {
    fn mesh(&self, key: &GlyphMeshHandlePrimitive) -> Option<&GlyphMeshAssetPrimitive> {
        let end = self.0.partition_point(|(_, asset)| asset.handle <= *key);
        let (_, asset) = self.0.get(end.checked_sub(1)?)?;
        (asset.handle == *key).then_some(*asset)
    }
}

// Keep the preceding map available as an independent test/reference adapter.
#[cfg(test)]
impl GlyphMeshLookup for BTreeMap<GlyphMeshHandlePrimitive, &GlyphMeshAssetPrimitive> {
    fn mesh(&self, key: &GlyphMeshHandlePrimitive) -> Option<&GlyphMeshAssetPrimitive> {
        self.get(key).copied()
    }
}
