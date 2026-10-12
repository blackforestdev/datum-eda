use super::*;
#[test]
fn exact_instance_stream_preserves_order_material_clip_bits_and_every_mode() {
    let mut strokes = Vec::new();
    for mode in 0..3 {
        for contrast in 0..2 {
            strokes.push(Stroke {
                from: [mode as f32, -0.0],
                to: [7.25, -12.0],
                material_width: [0.125, 0.5, 0.75, 6.0],
                clip: if contrast == 0 {
                    [-0.0, 2.0, 30.0, 40.0]
                } else {
                    [0.0, 2.0, 30.0, 40.0]
                },
                high_contrast: [contrast as f32, mode as f32, 0.0, 0.0],
            });
        }
    }
    let before = bytemuck::cast_slice::<Stroke, u8>(&strokes).to_vec();
    let budget = Budget::new(1024);
    let (bytes, clips) = encode(&strokes, &budget).unwrap();
    assert_eq!(std::mem::size_of::<Instance>(), 40);
    assert_eq!(bytes.len(), 240);
    assert_eq!(
        clips.len(),
        32,
        "signed zero clips remain distinct exact bit patterns"
    );
    for (index, source) in strokes.iter().enumerate() {
        let encoded: Instance = bytemuck::pod_read_unaligned(&bytes[index * 40..index * 40 + 40]);
        assert_eq!(
            bytemuck::bytes_of(&encoded.from),
            bytemuck::bytes_of(&source.from)
        );
        assert_eq!(
            bytemuck::bytes_of(&encoded.to),
            bytemuck::bytes_of(&source.to)
        );
        assert_eq!(
            bytemuck::bytes_of(&encoded.material_width),
            bytemuck::bytes_of(&source.material_width)
        );
        let start = encoded.clip as usize * 16;
        assert_eq!(&clips[start..start + 16], bytemuck::bytes_of(&source.clip));
        assert_eq!(encoded.flags & 1, source.high_contrast[0] as u32);
        assert_eq!(encoded.flags >> 1, source.high_contrast[1] as u32);
    }
    assert_eq!(bytemuck::cast_slice::<Stroke, u8>(&strokes), before);
    assert!(budget.used() >= bytes.allocated_bytes() + clips.allocated_bytes());
    drop((bytes, clips));
    assert_eq!(budget.used(), 0);
}
#[test]
fn refused_encoding_allocates_no_untracked_payload_or_partial_stream() {
    let stroke = Stroke {
        from: [0.0; 2],
        to: [1.0; 2],
        material_width: [1.0; 4],
        clip: [0.0, 0.0, 2.0, 2.0],
        high_contrast: [0.0; 4],
    };
    let budget = Budget::new(39);
    assert!(encode(&[stroke], &budget).is_err());
    assert_eq!(budget.used(), 0);
}
