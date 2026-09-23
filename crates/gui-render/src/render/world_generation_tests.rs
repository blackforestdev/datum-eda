//! Retained world admission includes resources held by encodings/submissions.
use super::*;

#[test]
#[ignore = "requires local GPU; run serially with visual feature"]
fn two_live_allocations_survive_clear_and_document_switch_until_retirement() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let first = SharedGeometry::for_document(vec![1_u32; 4], "generation-first");
    let second = SharedGeometry::for_document(vec![2_u32; 8], "generation-first");
    let third = SharedGeometry::for_document(vec![3_u32; 16], "generation-other");
    let budget = first.document_budget().unwrap().clone();
    let other = third.document_budget().unwrap().clone();
    let mut stream = RetainedBuffer::default();
    let generations = stream.generation_budget.clone();
    stream.sync(&device, &queue, "first", &first).unwrap();
    stream.flush_uploads(&device, &queue);
    let held_first = stream.submission_ref().unwrap();
    stream.sync(&device, &queue, "second", &second).unwrap();
    stream.flush_uploads(&device, &queue);
    let held_second = stream.submission_ref().unwrap();
    assert_eq!(generations.used(), 2);
    assert_eq!(budget.used(), 48);
    let current = stream.buffer().unwrap().clone();
    let error = stream.sync(&device, &queue, "third", &third).unwrap_err();
    assert!(error.to_string().contains("two live GPU allocations"));
    assert_eq!(stream.buffer(), Some(&current));
    assert!(stream.matches_source(&second));
    assert_eq!(stream.pending_bytes(), 0);
    assert_eq!(budget.used(), 48);
    assert_eq!(other.used(), 0, "refusal must not leak document admission");
    drop(current);

    // Equal-sized updates reuse current storage even while both slots are live.
    let warm = SharedGeometry::for_document(vec![4_u32; 8], "generation-first");
    stream.sync(&device, &queue, "reuse", &warm).unwrap();
    assert_eq!(generations.used(), 2);
    stream.flush_uploads(&device, &queue);
    stream.clear();
    assert!(stream.buffer().is_none());
    assert_eq!(
        generations.used(),
        2,
        "clear does not retire external holds"
    );
    assert!(stream.sync(&device, &queue, "still held", &third).is_err());
    stream = stream.replacement();
    assert!(
        stream
            .sync(&device, &queue, "replacement still held", &third)
            .is_err()
    );
    drop(held_first);
    assert_eq!(generations.used(), 1);
    stream.sync(&device, &queue, "retry", &third).unwrap();
    assert!(stream.matches_source(&third));
    assert_eq!(generations.used(), 2);
    assert_eq!(budget.used(), 32);
    assert_eq!(other.used(), 64);
    stream.flush_uploads(&device, &queue);
    drop(held_second);
    assert_eq!(budget.used(), 0);
    assert_eq!(generations.used(), 1);

    // The same hold path used by production submissions outlives the CPU owner.
    let hold = stream.submission_ref().unwrap();
    let _submission = queue.submit([]);
    crate::text_gpu::hold_until_done(&queue, vec![hold]);
    drop(stream);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert_eq!(generations.used(), 0);
    assert_eq!(other.used(), 0);
}

#[test]
#[ignore = "requires local GPU; renderer recovery allocation continuity"]
fn renderer_recreation_preserves_all_four_world_stream_limits() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut first = crate::Renderer::new(&device, &queue, format, 4).unwrap();
    let mut second = first
        .recreate_for_device(&device, &queue, format, 4)
        .unwrap();
    let mut third = second
        .recreate_for_device(&device, &queue, format, 4)
        .unwrap();
    macro_rules! exercise {
        ($field:ident) => {{
            let source = SharedGeometry::from(vec![bytemuck::Zeroable::zeroed(); 3]);
            first
                .$field
                .sync(&device, &queue, "first", &source)
                .unwrap();
            second
                .$field
                .sync(&device, &queue, "second", &source)
                .unwrap();
            assert!(
                third
                    .$field
                    .sync(&device, &queue, "third", &source)
                    .is_err()
            );
            assert!(third.$field.buffer().is_none());
            first.$field.clear();
            third
                .$field
                .sync(&device, &queue, "retry", &source)
                .unwrap();
            assert_eq!(third.$field.generation_budget.used(), 2);
        }};
    }
    exercise!(world_vertices_gpu);
    exercise!(world_strokes_gpu);
    exercise!(schematic_world_vertices_gpu);
    exercise!(schematic_world_strokes_gpu);
}
