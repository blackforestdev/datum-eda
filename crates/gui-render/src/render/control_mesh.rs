//! Renderer-owned CPU control meshes; placement, color and clipping stay live.
use super::*;
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
}

impl ControlMeshCache {
    pub(crate) fn retained_cpu_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.entries.capacity() * std::mem::size_of::<Entry>()
            + self.payload_bytes
    }

    fn with_mesh(
        &mut self,
        key: Key,
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
        let mesh = build();
        let bytes = std::mem::size_of_val(mesh.as_ref());
        // Reject an impossible payload without allocating entry storage. Caps
        // are ceilings, not startup reservation targets (MEM-02).
        if bytes > MAX_CPU_BYTES - std::mem::size_of::<Self>() - std::mem::size_of::<Entry>() {
            consume(&mesh);
            return;
        }
        // Grow only for a new slot, then charge the actual allocation before
        // admitting payload. At the entry limit eviction reuses an old slot.
        if self.entries.len() < MAX_ENTRIES && self.entries.len() == self.entries.capacity() {
            self.entries.reserve(1);
        }
        // Table growth can itself displace previously retained payload, even
        // when the new mesh is subsequently too large to retain.
        while self.retained_cpu_bytes() > MAX_CPU_BYTES {
            let old = self
                .entries
                .pop_back()
                .expect("payload exceeds cache budget");
            self.payload_bytes -= std::mem::size_of_val(old.mesh.as_ref());
        }
        let metadata_bytes = self.retained_cpu_bytes() - self.payload_bytes;
        if bytes > MAX_CPU_BYTES.saturating_sub(metadata_bytes) {
            consume(&mesh);
            return;
        }
        while self.entries.len() >= MAX_ENTRIES || self.retained_cpu_bytes() + bytes > MAX_CPU_BYTES
        {
            let old = self.entries.pop_back().expect("bounded cache has an entry");
            self.payload_bytes -= std::mem::size_of_val(old.mesh.as_ref());
        }
        consume(&mesh);
        self.payload_bytes += bytes;
        self.entries.push_front(Entry { key, mesh });
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
        self.paint_mesh(key, (rect.x, rect.y), color, || {
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
        });
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
        self.paint_mesh(key, (rect.x + rx, rect.y + ry), color, || {
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
        });
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
            (rect.x + rect.width * 0.5, rect.y + rect.height * 0.5),
            color,
            || {
                let points =
                    ellipse_points((0.0, 0.0), rect.width, rect.height, 0.0, segments as usize);
                let mut quads = Vec::new();
                push_convex_polygon_fill(&mut quads, &points, [0.0; 3]);
                quads
                    .into_iter()
                    .map(|quad| quad.points)
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            },
        );
    }

    fn paint_mesh(
        &mut self,
        key: Key,
        origin: (f32, f32),
        color: [f32; 3],
        build: impl FnOnce() -> Mesh,
    ) {
        let quads = &mut self.quads;
        self.cache.with_mesh(key, build, |mesh| {
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
