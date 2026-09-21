//! Renderer-owned CPU control meshes; placement, color and clipping stay live.
use super::*;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

type Mesh = Box<[[(f32, f32); 4]]>;
const MAX_ENTRIES: usize = 256;
const MAX_PAYLOAD_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Key {
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
    #[cfg(test)]
    pub(crate) builds: usize,
}

impl ControlMeshCache {
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
        #[cfg(test)]
        {
            self.builds += 1;
        }
        let mesh = build();
        let bytes = std::mem::size_of_val(mesh.as_ref());
        if bytes > MAX_PAYLOAD_BYTES {
            consume(&mesh);
            return;
        }
        while self.entries.len() >= MAX_ENTRIES || self.payload_bytes + bytes > MAX_PAYLOAD_BYTES {
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

    pub(crate) fn rounded_fill(&mut self, rect: RectPx, color: [f32; 3], radius: f32, border: f32) {
        let radius = radius.max(0.0).min(rect.width * 0.5).min(rect.height * 0.5);
        let key = Key {
            dimensions: [rect.width.to_bits(), rect.height.to_bits()],
            radius: radius.to_bits(),
            border: border.to_bits(),
            dpi: self.dpi.to_bits(),
            contour_revision: 1,
            segments: ROUNDED_RECT_CORNER_SEGMENTS as u32,
            style_generation: 0,
        };
        let quads = &mut self.quads;
        self.cache.with_mesh(
            key,
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
            |mesh| {
                quads.extend(mesh.iter().map(|points| Quad {
                    points: points.map(|(x, y)| (rect.x + x, rect.y + y)),
                    color,
                }));
            },
        );
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
mod tests {
    use super::*;

    fn key(n: u32) -> Key {
        Key {
            dimensions: [n, 1],
            radius: 2,
            border: 3,
            dpi: 4,
            contour_revision: 1,
            segments: 4,
            style_generation: 0,
        }
    }

    #[test]
    fn control_mesh_keys_bounds_and_lru_are_complete() {
        let mut cache = ControlMeshCache::default();
        let mesh = || vec![[(0.0, 0.0); 4]; 3].into_boxed_slice();
        for field in 0..8 {
            let mut changed = key(0);
            match field {
                0 => changed.dimensions[0] += 1,
                1 => changed.dimensions[1] += 1,
                2 => changed.radius += 1,
                3 => changed.border += 1,
                4 => changed.dpi += 1,
                5 => changed.contour_revision += 1,
                6 => changed.segments += 1,
                _ => changed.style_generation += 1,
            }
            cache.with_mesh(changed, mesh, |_| {});
            cache.with_mesh(changed, || panic!("warm mesh rebuilt"), |_| {});
        }
        assert_eq!(cache.builds, 8);
        cache = ControlMeshCache::default();
        for n in 0..MAX_ENTRIES as u32 {
            cache.with_mesh(key(n), mesh, |_| {});
        }
        cache.with_mesh(key(0), || panic!("retained mesh absent"), |_| {});
        cache.with_mesh(key(MAX_ENTRIES as u32), mesh, |_| {});
        assert_eq!(cache.entries.len(), MAX_ENTRIES);
        assert!(cache.entries.iter().any(|entry| entry.key == key(0)));
        assert!(!cache.entries.iter().any(|entry| entry.key == key(1)));
        assert_eq!(
            cache.payload_bytes,
            MAX_ENTRIES * 3 * std::mem::size_of::<[(f32, f32); 4]>()
        );
        eprintln!(
            "control cache entries={} payload_bytes={} key_bytes={} entry_storage_bytes={}",
            cache.entries.len(),
            cache.payload_bytes,
            cache.entries.len() * std::mem::size_of::<Key>(),
            cache.entries.capacity() * std::mem::size_of::<Entry>()
        );
        let bytes = cache.payload_bytes;
        cache.with_mesh(
            key(999),
            || vec![[(0.0, 0.0); 4]; MAX_PAYLOAD_BYTES / 32 + 1].into_boxed_slice(),
            |mesh| assert!(std::mem::size_of_val(mesh) > MAX_PAYLOAD_BYTES),
        );
        assert_eq!(cache.payload_bytes, bytes);
        assert!(!cache.entries.iter().any(|entry| entry.key == key(999)));
        // Exercise byte eviction before the entry cap with admitted-sized meshes.
        cache = ControlMeshCache::default();
        for n in 0..5 {
            cache.with_mesh(
                key(n),
                || vec![[(0.0, 0.0); 4]; MAX_PAYLOAD_BYTES / 64].into_boxed_slice(),
                |_| {},
            );
            assert!(cache.payload_bytes <= MAX_PAYLOAD_BYTES);
            assert!(cache.entries.len() <= 2);
        }
        assert_eq!(cache.payload_bytes, MAX_PAYLOAD_BYTES);
    }

    #[test]
    fn placement_and_color_reuse_control_mesh_but_geometry_and_dpi_miss() {
        let mut cache = ControlMeshCache::default();
        let mut output = Vec::new();
        let rect = RectPx {
            x: 20.25,
            y: 30.5,
            width: 140.0,
            height: 28.0,
        };
        ControlPainter::new(&mut output, &mut cache, 1.0).rounded_fill(rect, [0.2; 3], 4.0, 1.0);
        let original = output.clone();
        output.clear();
        ControlPainter::new(&mut output, &mut cache, 1.0).rounded_fill(
            RectPx {
                x: rect.x + 10.0,
                y: rect.y + 20.0,
                ..rect
            },
            [0.7; 3],
            4.0,
            1.0,
        );
        assert_eq!(cache.builds, 1);
        assert_eq!(output.len(), original.len());
        assert!(output.iter().all(|quad| quad.color == [0.7; 3]));
        for (width, height, radius, border, dpi) in [
            (141.0, 28.0, 4.0, 1.0, 1.0),
            (140.0, 29.0, 4.0, 1.0, 1.0),
            (140.0, 28.0, 5.0, 1.0, 1.0),
            (140.0, 28.0, 4.0, 2.0, 1.0),
            (140.0, 28.0, 4.0, 1.0, 1.5),
        ] {
            ControlPainter::new(&mut output, &mut cache, dpi).rounded_fill(
                RectPx {
                    width,
                    height,
                    ..rect
                },
                [0.2; 3],
                radius,
                border,
            );
        }
        assert_eq!(cache.builds, 6);
    }
}
