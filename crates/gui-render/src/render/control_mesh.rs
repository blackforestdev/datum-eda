//! Renderer-owned CPU control meshes; placement, color and clipping stay live.
use super::*;
use crate::cpu_alloc::heap::capacity_bytes;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

type Mesh = Box<[[(f32, f32); 4]]>;
const MAX_ENTRIES: usize = 256;
const MAX_CPU_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MeshKind {
    RoundedRect,
    Ellipse,
    ConvexEllipse,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Key {
    kind: MeshKind,
    dimensions: [u32; 2],
    radius: u32,
    border: u32,
    dpi: u32,
    contour_revision: u32,
    segments: u32,
    style_generation: u32,
}

struct Entry {
    key: Key,
    mesh: Mesh,
}

#[derive(Default)]
pub(crate) struct ControlMeshCache {
    entries: VecDeque<Entry>,
    payload_bytes: usize,
    pub(crate) builds: usize,
    #[cfg(test)]
    build_live_bytes: usize,
}

impl ControlMeshCache {
    pub(crate) fn usage(&self) -> crate::ControlMeshUsage {
        crate::ControlMeshUsage {
            entries: self.entries.len(),
            entry_capacity: self.entries.capacity(),
            key_capacity_bytes: self.entries.capacity() * std::mem::size_of::<Key>(),
            entry_storage_bytes: capacity_bytes::<Entry>(self.entries.capacity()),
            mesh_storage_bytes: self.payload_bytes,
            total_bytes: self.retained_cpu_bytes(),
        }
    }

    pub(crate) fn retained_cpu_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + capacity_bytes::<Entry>(self.entries.capacity())
            + self.payload_bytes
    }

    fn with_mesh(
        &mut self,
        key: Key,
        quad_count: usize,
        build: impl FnOnce() -> Mesh,
        consume: impl FnOnce(&[[(f32, f32); 4]]),
    ) {
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key) {
            let entry = self.entries.remove(index).expect("matched control mesh");
            self.entries.push_front(entry);
            consume(&self.entries.front().expect("retained hit").mesh);
            return;
        }
        self.builds = self.builds.saturating_add(1);
        let bytes = capacity_bytes::<[(f32, f32); 4]>(quad_count);
        let retain = self.reserve_mesh(bytes);
        // Eviction precedes allocation: cacheable replacement payload must not
        // coexist with history that its admission is about to discard.
        let mesh = build();
        assert_eq!(mesh.len(), quad_count);
        #[cfg(test)]
        {
            self.build_live_bytes = self.retained_cpu_bytes() + bytes;
        }
        consume(&mesh);
        if retain {
            self.payload_bytes += bytes;
            self.entries.push_front(Entry { key, mesh });
        }
    }

    fn reserve_mesh(&mut self, bytes: usize) -> bool {
        // Oversized required content still paints through the uncached path.
        if bytes > MAX_CPU_BYTES - std::mem::size_of::<Self>() - capacity_bytes::<Entry>(1) {
            return false;
        }
        if self.entries.len() < MAX_ENTRIES && self.entries.len() == self.entries.capacity() {
            let capacity = (self.entries.capacity() * 2).clamp(4, MAX_ENTRIES);
            // A growing ring may allocate the replacement before releasing its
            // old storage. Evict payload first, charging both tables during that
            // transition, without preallocating the maximum entry allowance.
            let replacement = capacity_bytes::<Entry>(capacity);
            while self.retained_cpu_bytes() + replacement > MAX_CPU_BYTES {
                let old = self
                    .entries
                    .pop_back()
                    .expect("growth needs payload eviction");
                self.payload_bytes -= capacity_bytes::<[(f32, f32); 4]>(old.mesh.len());
            }
            self.entries.reserve_exact(capacity - self.entries.len());
        }
        while self.retained_cpu_bytes() > MAX_CPU_BYTES {
            let old = self
                .entries
                .pop_back()
                .expect("payload exceeds cache budget");
            self.payload_bytes -= capacity_bytes::<[(f32, f32); 4]>(old.mesh.len());
        }
        let metadata_bytes = self.retained_cpu_bytes() - self.payload_bytes;
        if bytes > MAX_CPU_BYTES.saturating_sub(metadata_bytes) {
            return false;
        }
        while self.entries.len() >= MAX_ENTRIES || self.retained_cpu_bytes() + bytes > MAX_CPU_BYTES
        {
            let old = self.entries.pop_back().expect("bounded cache has an entry");
            self.payload_bytes -= capacity_bytes::<[(f32, f32); 4]>(old.mesh.len());
        }
        true
    }
}

pub(crate) struct ControlPainter<'a> {
    quads: &'a mut Vec<Quad>,
    cache: &'a mut ControlMeshCache,
    dpi: f32,
}

impl<'a> ControlPainter<'a> {
    pub(crate) fn new(quads: &'a mut Vec<Quad>, cache: &'a mut ControlMeshCache, dpi: f32) -> Self {
        Self { quads, cache, dpi }
    }

    pub(crate) fn scale_factor(&self) -> f32 {
        self.dpi
    }

    pub(crate) fn rounded_fill(&mut self, rect: RectPx, color: [f32; 3], radius: f32, border: f32) {
        let radius = radius.max(0.0).min(rect.width * 0.5).min(rect.height * 0.5);
        let key = Key {
            kind: MeshKind::RoundedRect,
            dimensions: [rect.width.to_bits(), rect.height.to_bits()],
            radius: radius.to_bits(),
            border: border.to_bits(),
            dpi: self.dpi.to_bits(),
            contour_revision: 1,
            segments: ROUNDED_RECT_CORNER_SEGMENTS as u32,
            style_generation: 0,
        };
        self.paint_mesh(
            key,
            if radius == 0.0 {
                1
            } else {
                (ROUNDED_RECT_CORNER_SEGMENTS + 1) * 2 - 1
            },
            (rect.x, rect.y),
            color,
            || {
                let points = rounded_rect_points(
                    RectPx {
                        x: 0.0,
                        y: 0.0,
                        ..rect
                    },
                    radius,
                );
                (1..points.len() - 1)
                    .step_by(2)
                    .map(|index| {
                        [
                            points[0],
                            points[index],
                            points[index + 1],
                            points[(index + 2).min(points.len() - 1)],
                        ]
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            },
        );
    }

    pub(crate) fn ellipse_fill(&mut self, rect: RectPx, color: [f32; 3], segments: u32) {
        if rect.width <= 0.5 || rect.height <= 0.5 || segments < 3 {
            return;
        }
        let (rx, ry) = (rect.width * 0.5, rect.height * 0.5);
        let key = Key {
            kind: MeshKind::Ellipse,
            dimensions: [rect.width.to_bits(), rect.height.to_bits()],
            radius: 0,
            border: 0,
            dpi: self.dpi.to_bits(),
            contour_revision: 1,
            segments,
            style_generation: 0,
        };
        // Keep offsets relative to the center so translation performs exactly
        // the same floating-point additions as the original ellipse painter.
        self.paint_mesh(
            key,
            segments as usize,
            (rect.x + rx, rect.y + ry),
            color,
            || {
                let step = std::f32::consts::TAU / segments as f32;
                let mut previous = (rx, 0.0);
                (1..=segments)
                    .map(|i| {
                        let angle = step * i as f32;
                        let next = (rx * angle.cos(), ry * angle.sin());
                        let points = [(0.0, 0.0), previous, next, next];
                        previous = next;
                        points
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            },
        );
    }

    /// Preserve the perimeter-anchored fan used by existing pane indicators.
    pub(crate) fn convex_ellipse_fill(&mut self, rect: RectPx, color: [f32; 3], segments: u32) {
        let segments = segments.max(24);
        let key = Key {
            kind: MeshKind::ConvexEllipse,
            dimensions: [rect.width.to_bits(), rect.height.to_bits()],
            radius: 0,
            border: 0,
            dpi: self.dpi.to_bits(),
            contour_revision: 1,
            segments,
            style_generation: 0,
        };
        self.paint_mesh(
            key,
            segments as usize - 2,
            (rect.x + rect.width * 0.5, rect.y + rect.height * 0.5),
            color,
            || {
                let points =
                    ellipse_points((0.0, 0.0), rect.width, rect.height, 0.0, segments as usize);
                points[1..]
                    .windows(2)
                    .map(|edge| [points[0], edge[0], edge[1], edge[1]])
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            },
        );
    }

    fn paint_mesh(
        &mut self,
        key: Key,
        quad_count: usize,
        origin: (f32, f32),
        color: [f32; 3],
        build: impl FnOnce() -> Mesh,
    ) {
        let quads = &mut self.quads;
        self.cache.with_mesh(key, quad_count, build, |mesh| {
            quads.extend(mesh.iter().map(|points| Quad {
                points: points.map(|(x, y)| (origin.0 + x, origin.1 + y)),
                color,
            }));
        });
    }
}

impl Deref for ControlPainter<'_> {
    type Target = Vec<Quad>;
    fn deref(&self) -> &Self::Target {
        self.quads
    }
}
impl DerefMut for ControlPainter<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.quads
    }
}

#[cfg(test)]
#[path = "control_mesh_tests.rs"]
mod tests;
