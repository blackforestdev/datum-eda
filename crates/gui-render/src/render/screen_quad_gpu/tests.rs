use super::*;
#[test]
fn exact_corners_colors_topology_and_admission_preserve_the_expanded_source() {
    let quad = Quad {
        points: [(1.25, -0.0), (3.75, 2.0), (7.5, 9.25), (-2.0, 4.5)],
        color: [0.07, 0.5, 0.99],
    };
    let vertices = crate::gpu_data::quad_vertices(quad);
    let budget = Budget::new(1024);
    let bytes = encode(&vertices, &budget).unwrap();
    assert_eq!(bytes.len(), 44);
    let x: [f32; 4] = bytemuck::pod_read_unaligned(&bytes[..16]);
    let y: [f32; 4] = bytemuck::pod_read_unaligned(&bytes[16..32]);
    let color: [f32; 3] = bytemuck::pod_read_unaligned(&bytes[32..]);
    let indices = [0, 1, 2, 0, 2, 3];
    for (vertex, corner) in vertices.iter().zip(indices) {
        assert_eq!(
            bytemuck::bytes_of(&vertex.pos),
            bytemuck::bytes_of(&[x[corner], y[corner]])
        );
        assert_eq!(
            bytemuck::bytes_of(&vertex.color),
            bytemuck::bytes_of(&color)
        );
    }
    let before = budget.available();
    assert!(encode(&vertices[..5], &budget).is_err());
    let mut malformed = vertices;
    malformed[4].pos[0] += 1.0;
    assert!(encode(&malformed, &budget).is_err());
    malformed = vertices;
    malformed[5].color[0] += 0.1;
    assert!(encode(&malformed, &budget).is_err());
    assert_eq!(budget.available(), before);
    assert!(encode(&vertices, &Budget::new(43)).is_err());
    assert_eq!(
        bytemuck::bytes_of(&vertices),
        bytemuck::bytes_of(&crate::gpu_data::quad_vertices(quad))
    );
}

#[test]
fn separate_coordinate_plane_preserves_order_and_exact_changed_words() {
    let budget = Budget::new(100_000);
    let quads: Vec<_> = (0..100)
        .map(|i| Quad {
            points: [
                (i as f32, 1.),
                (i as f32 + 3., 4.),
                (i as f32 + 5., 9.),
                (i as f32 - 2., 7.),
            ],
            color: [i as f32 / 100., 0.3, 0.8],
        })
        .collect();
    let original = quads_to_vertices(&quads);
    let bytes = encode(&original, &budget).unwrap();
    assert_eq!(bytes.len(), 100 * 44);
    for (i, quad) in quads.iter().enumerate() {
        let x = bytemuck::pod_read_unaligned::<[f32; 4]>(&bytes[i * 16..(i + 1) * 16]);
        let y = bytemuck::pod_read_unaligned::<[f32; 4]>(
            &bytes[100 * 16 + i * 16..100 * 16 + (i + 1) * 16],
        );
        let coordinates = std::array::from_fn::<_, 4, _>(|corner| [x[corner], y[corner]]);
        let color = bytemuck::pod_read_unaligned::<[f32; 3]>(
            &bytes[100 * 32 + i * 12..100 * 32 + (i + 1) * 12],
        );
        assert_eq!(
            bytemuck::bytes_of(&coordinates),
            bytemuck::bytes_of(&quad.points.map(|(x, y)| [x, y]))
        );
        assert_eq!(bytemuck::bytes_of(&color), bytemuck::bytes_of(&quad.color));
    }
    for axis in 0..2 {
        let moved: Vec<_> = quads
            .iter()
            .map(|q| Quad {
                points: q.points.map(|(x, y)| {
                    if axis == 0 {
                        (x + 32., y)
                    } else {
                        (x, y + 32.)
                    }
                }),
                color: q.color,
            })
            .collect();
        let changed = encode(&quads_to_vertices(&moved), &budget).unwrap();
        let ranges: Vec<_> =
            crate::text_gpu::upload::plan::Dirty::new(&bytes, &changed, 44).collect();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].0, axis * 100 * 16);
        assert_eq!(ranges[0].1.len(), 100 * 16);
        assert_eq!(&bytes[100 * 32..], &changed[100 * 32..]);
    }
}
