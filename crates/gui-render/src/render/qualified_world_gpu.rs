//! Existing retained-buffer lifetimes configured for each qualified schematic source.
use super::*;
use crate::text_gpu::staging_vec::StagingVec;

pub(crate) struct Buffers {
    pub(crate) basis: RetainedScene,
    projection: Option<RectPx>,
    pub(crate) vertices: gpu_data::retained_buffer::RetainedBuffer<Vertex>,
    pub(crate) strokes: gpu_data::retained_buffer::RetainedBuffer<WorldStrokeInstance>,
    pub(crate) points: gpu_data::retained_buffer::RetainedBuffer<retained_glyph::Point>,
    pub(crate) materials: gpu_data::retained_buffer::RetainedBuffer<retained_glyph::Material>,
}
impl Buffers {
    pub(crate) fn matches(&self, scene: &RetainedScene) -> bool {
        self.vertices.matches_source(&scene.world_vertices)
            && self.strokes.matches_source(scene.world_strokes())
            && self.points.matches_source(&scene.glyph.points)
            && self.materials.matches_source(&scene.glyph.materials)
    }
    fn same_document(&self, scene: &RetainedScene, viewport: RectPx) -> bool {
        let projection = (!scene.can_reuse_for_surface_resize()).then_some(viewport);
        if self.projection != projection {
            return false;
        }
        self.basis
            .presentation
            .source
            .as_ref()
            .zip(scene.presentation.source.as_ref())
            .is_some_and(|(a, b)| a.project == b.project && a.scene == b.scene)
    }
    pub(crate) fn pending_bytes(&self) -> usize {
        self.vertices.pending_bytes()
            + self.strokes.pending_bytes()
            + self.points.pending_bytes()
            + self.materials.pending_bytes()
    }
    pub(crate) fn refs(&self) -> impl Iterator<Item = crate::text_gpu::lifetime::SubmissionRef> {
        [
            self.vertices.submission_ref(),
            self.strokes.submission_ref(),
            self.points.submission_ref(),
            self.materials.submission_ref(),
        ]
        .into_iter()
        .flatten()
    }
    pub(crate) fn cancel(&mut self) {
        self.vertices.cancel_uploads();
        self.strokes.cancel_uploads();
        self.points.cancel_uploads();
        self.materials.cancel_uploads();
    }
    pub(crate) fn append_chunk<'a>(
        &'a self,
        remaining: &mut usize,
        uploads: &mut StagingVec<crate::text_gpu::upload::BufferUpload<'a>>,
    ) -> [usize; 4] {
        [
            self.vertices.append_chunk(remaining, uploads),
            self.strokes.append_chunk(remaining, uploads),
            self.points.append_chunk(remaining, uploads),
            self.materials.append_chunk(remaining, uploads),
        ]
    }
    pub(crate) fn consume_chunk(&mut self, counts: [usize; 4]) {
        self.vertices.consume_chunk(counts[0]);
        self.strokes.consume_chunk(counts[1]);
        self.points.consume_chunk(counts[2]);
        self.materials.consume_chunk(counts[3]);
    }
    fn sync(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RetainedScene,
    ) -> anyhow::Result<()> {
        self.vertices.sync(
            device,
            queue,
            "datum-document-world-vertices",
            &scene.world_vertices,
        )?;
        self.strokes.sync(
            device,
            queue,
            "datum-document-world-strokes",
            scene.world_strokes(),
        )?;
        self.points.sync(
            device,
            queue,
            "datum-document-glyph-points",
            &scene.glyph.points,
        )?;
        self.materials.sync(
            device,
            queue,
            "datum-document-glyph-materials",
            &scene.glyph.materials,
        )?;
        self.basis = scene.clone();
        Ok(())
    }
    fn new(scene: &RetainedScene, viewport: RectPx) -> Self {
        Self {
            basis: scene.clone(),
            projection: (!scene.can_reuse_for_surface_resize()).then_some(viewport),
            vertices: Default::default(),
            strokes: Default::default(),
            points: Default::default(),
            materials: Default::default(),
        }
    }
    fn replacement(&self) -> Self {
        Self {
            basis: self.basis.clone(),
            projection: self.projection,
            vertices: self.vertices.replacement(),
            strokes: self.strokes.replacement(),
            points: self.points.replacement(),
            materials: self.materials.replacement(),
        }
    }
}
#[derive(Default)]
pub(crate) struct QualifiedWorldGpu {
    pub(crate) entries: StagingVec<Buffers>,
}
impl QualifiedWorldGpu {
    pub(crate) fn get(&self, scene: &RetainedScene) -> Option<&Buffers> {
        self.entries.iter().find(|e| e.matches(scene))
    }
    pub(crate) fn sync(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &PreparedScene,
        staging: &std::sync::Arc<crate::text_gpu::budget::Budget>,
    ) -> anyhow::Result<()> {
        self.entries.retain(|entry| {
            prepared
                .schematic_sources
                .iter()
                .any(|(key, viewport, scene)| key.is_some() && entry.same_document(scene, viewport))
        });
        for (_, viewport, scene) in prepared
            .schematic_sources
            .iter()
            .filter(|(key, _, _)| key.is_some())
        {
            if let Some(entry) = self
                .entries
                .iter_mut()
                .find(|e| e.same_document(scene, viewport))
            {
                entry.sync(device, queue, scene)?;
            } else {
                let mut entry = Buffers::new(scene, viewport);
                entry.sync(device, queue, scene)?;
                self.entries.try_push(entry, staging)?;
            }
        }
        Ok(())
    }
    pub(crate) fn replacement(
        &self,
        staging: &std::sync::Arc<crate::text_gpu::budget::Budget>,
    ) -> anyhow::Result<Self> {
        let mut entries = StagingVec::new(self.entries.len(), staging)?;
        for entry in self.entries.iter() {
            entries.push(entry.replacement());
        }
        Ok(Self { entries })
    }
}
