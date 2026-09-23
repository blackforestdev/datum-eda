//! Shared control mesh identity, bounds, contour and reuse regressions.
use super::*;

fn key(n: u32) -> Key {
    Key {
        kind: MeshKind::RoundedRect,
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
fn ellipse_meshes_preserve_uncached_vertices_and_reuse_across_placement_and_color() {
    let mut cache = ControlMeshCache::default();
    for (width, height) in [(6.0, 6.0), (10.0, 10.0), (14.0, 14.0), (31.25, 12.75)] {
        let builds = cache.builds;
        for (x, y, color) in [
            (0.0, 0.0, [0.2; 3]),
            (228.25, 54.75, [0.8; 3]),
            (-10.125, 700.5, [0.5; 3]),
        ] {
            let rect = RectPx {
                x,
                y,
                width,
                height,
            };
            let mut expected = Vec::new();
            push_projected_ellipse(&mut expected, rect, color, 16);
            let mut actual = Vec::new();
            ControlPainter::new(&mut actual, &mut cache, 1.5).ellipse_fill(rect, color, 16);
            assert_eq!(actual, expected, "cached contour changes at {rect:?}");
            assert_eq!(cache.builds, builds + 1, "placement/color must stay live");
        }
    }
    let mut output = Vec::new();
    let builds = cache.builds;
    ControlPainter::new(&mut output, &mut cache, 1.5).ellipse_fill(
        RectPx {
            x: 0.0,
            y: 0.0,
            width: 0.5,
            height: 14.0,
        },
        [1.0; 3],
        16,
    );
    assert!(output.is_empty());
    assert_eq!(cache.builds, builds);

    let mut cache = ControlMeshCache::default();
    for x in [20.0, 20.25, 400.0] {
        crate::global_preferences_primitives::draw_search_icon(
            x,
            40.5,
            &mut ControlPainter::new(&mut output, &mut cache, 1.0),
        );
        assert_eq!(cache.builds, 2, "production search rings must remain warm");
    }
}

#[test]
fn convex_ellipse_retains_the_existing_perimeter_fan() {
    let mut cache = ControlMeshCache::default();
    for (x, y) in [(20.0, 30.0), (700.25, 41.75), (-10.5, 12.25)] {
        let rect = RectPx {
            x,
            y,
            width: 7.0,
            height: 7.0,
        };
        let mut expected = Vec::new();
        let points = ellipse_points((x + 3.5, y + 3.5), 7.0, 7.0, 0.0, 20);
        push_convex_polygon_fill(&mut expected, &points, [0.3; 3]);
        let mut actual = Vec::new();
        ControlPainter::new(&mut actual, &mut cache, 1.0).convex_ellipse_fill(rect, [0.3; 3], 24);
        assert_eq!(actual, expected);
        assert_eq!(cache.builds, 1, "translated indicator must stay warm");
    }
}

#[test]
fn control_mesh_keys_bounds_and_lru_are_complete() {
    let mut cache = ControlMeshCache::default();
    let mesh = || vec![[(0.0, 0.0); 4]; 3].into_boxed_slice();
    for field in 0..9 {
        let mut changed = key(0);
        match field {
            0 => changed.dimensions[0] += 1,
            1 => changed.dimensions[1] += 1,
            2 => changed.radius += 1,
            3 => changed.border += 1,
            4 => changed.dpi += 1,
            5 => changed.contour_revision += 1,
            6 => changed.segments += 1,
            7 => changed.style_generation += 1,
            _ => changed.kind = MeshKind::Ellipse,
        }
        cache.with_mesh(changed, mesh, |_| {});
        cache.with_mesh(changed, || panic!("warm mesh rebuilt"), |_| {});
    }
    assert_eq!(cache.builds, 9);
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
        || vec![[(0.0, 0.0); 4]; MAX_CPU_BYTES / 32 + 1].into_boxed_slice(),
        |mesh| assert!(std::mem::size_of_val(mesh) > MAX_CPU_BYTES),
    );
    assert_eq!(cache.payload_bytes, bytes);
    assert!(!cache.entries.iter().any(|entry| entry.key == key(999)));
    // Exercise byte eviction before the entry cap with admitted-sized meshes.
    cache = ControlMeshCache::default();
    for n in 0..5 {
        cache.with_mesh(
            key(n),
            || vec![[(0.0, 0.0); 4]; MAX_CPU_BYTES / 64].into_boxed_slice(),
            |_| {},
        );
        assert!(cache.retained_cpu_bytes() <= MAX_CPU_BYTES);
        assert_eq!(cache.entries.len(), 1);
    }
    assert_eq!(cache.payload_bytes, MAX_CPU_BYTES / 2);
    assert!(cache.retained_cpu_bytes() > cache.payload_bytes);
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

#[test]
fn metadata_is_charged_before_admitting_a_payload_at_the_cpu_limit() {
    let mut cache = ControlMeshCache::default();
    let metadata = cache.retained_cpu_bytes();
    assert!(metadata >= MAX_ENTRIES * std::mem::size_of::<Entry>());
    let capacity = cache.entries.capacity();
    let mut consumed = false;
    cache.with_mesh(
        key(42),
        || vec![[(0.0, 0.0); 4]; MAX_CPU_BYTES / 32].into_boxed_slice(),
        |mesh| {
            consumed = true;
            assert_eq!(std::mem::size_of_val(mesh), MAX_CPU_BYTES);
        },
    );
    assert!(consumed, "uncacheable content must still paint");
    assert!(cache.entries.is_empty(), "payload alone fills the cap");
    assert_eq!(cache.retained_cpu_bytes(), metadata);
    for n in 0..512 {
        cache.with_mesh(key(n), || Box::new([]), |_| {});
        assert_eq!(cache.entries.capacity(), capacity);
        assert_eq!(cache.retained_cpu_bytes(), metadata);
    }
    assert_eq!(cache.entries.len(), MAX_ENTRIES);
}
